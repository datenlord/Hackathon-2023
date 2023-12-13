mod network;
use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
    fs::File,
    io::{Read, Write},
    option,
};

use async_std::task::spawn;
use clap::Parser;
use futures::{channel::mpsc, FutureExt, SinkExt, Stream, StreamExt};
use libp2p::{build_multiaddr, futures::future::ok, multiaddr::Protocol, Multiaddr};
use network::{Command, Event};
use tracing::{span, Level, info};
use tracing_subscriber::EnvFilter;
#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    //注册tracing_subscriber
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();
    let span = span!(Level::TRACE, "my_span");
    let _guard = span.enter();

    //存储在节点中的 待获取文件的信息
    let mut file_metadata_hashmap = BTreeMap::new();
    file_metadata_hashmap.insert("file1", ("file1"));
    file_metadata_hashmap.insert("file2", ("file2"));

    //获取控制台参数
    // let options = Options::parse();
    let options = Options {
        listen_address_with_port: None,
        bootstrap_peer: Some(
            "/ip4/192.168.3.21/tcp/53527/p2p/12D3KooW9szUy2HyAGDMPp17w1J3stoMwQ6djSz7fa9DpiGL4csX"
                .parse::<Multiaddr>()
                .unwrap(),
        ),
        node_type: NodeTypes::CommonNode,
    };
    //构建network节点
    let (mut node_client, file_event_receiver, event_loop) = network::new().await?;

    //运行第二层管道处理
    spawn(event_loop.run());

    //文件管理器初始化
    let file_manger = FileManage {
        file_event_receiver: file_event_receiver,
        command_sender: node_client.get_sender_clone(),
        file_cache: HashMap::new(),
    };
    //运行第三层管道处理
    spawn(file_manger.run());

    //判断节点是什么类型
    match options.node_type {
        NodeTypes::Bootstrap => {
            //引导节点初始化只需要监听端口事件，后续不再进行
            //监听指定地址端口
            match options.listen_address_with_port {
                Some(addr) => node_client
                    .start_listening(addr)
                    .await
                    .expect("Listening not to fail."),
                None => node_client
                    .start_listening("/ip4/0.0.0.0/tcp/0".parse()?)
                    .await
                    .expect("Listening not to fail."),
            };
            loop {
                //作为引导节点存在，不处理实际业务
            }
        }
        NodeTypes::CommonNode => {
            //普通节点初始化需要监听端口事件、dialing引导节点
            match options.listen_address_with_port {
                Some(addr) => node_client
                    .start_listening(addr.clone())
                    .await
                    .expect(format!("监听地址：{}失败", addr).as_str()),
                None => node_client
                    .start_listening("/ip4/0.0.0.0/tcp/0".parse()?)
                    .await
                    .expect(format!("监听地址：/ip4/0.0.0.0/tcp/0 失败").as_str()),
            };
            if let Some(dialing_multiAddr) = options.bootstrap_peer {
                let Some(Protocol::P2p(peer_id)) = dialing_multiAddr.iter().last() else {
                    return Err("引导节点地址出错：缺失引导节点的PeerId".into());
                };
                node_client
                    .dial(peer_id, dialing_multiAddr)
                    .await
                    .expect("Dialing失败");
            }

            //按照自身的hashmap获取对应的文件,后续也可以改成根据控制台来获取对应文件
            let node_client_clone = node_client.clone();
            spawn(async move {
                //move进来，避免生命周期低于spawn里的异步任务
                let mut node_client = node_client_clone;
                for (file_name, value) in file_metadata_hashmap {
                    //从DHT网络中获取当前所需文件的提供者
                    let providers = node_client
                        .get_providers(file_name.to_owned().to_owned())
                        .await;
                    //如果不存在提供者，就向filemanager发送通过s3下载的请求
                    let file_content: Vec<u8>;
                    if providers.is_empty() {
                        file_content = node_client
                            .get_file_content_by_s3_cache(file_name.to_string())
                            .await
                            .unwrap();
                    } else {
                        //创建一些从Peer节点获取文件内容的future
                        let requests = providers.into_iter().map(|p| {
                            let mut node_client = node_client.clone();
                            async move { node_client.request_file(p, file_name.to_owned()).await }
                                .boxed()
                        });
                        //选择其中一个进行执行
                        (file_content, ..) = futures::future::select_ok(requests)
                            .await
                            .map_err(|_| "None of the providers returned file.")
                            .unwrap()
                            .0;
                    }
                    //本地存储文件缓存
                    let _ = node_client
                        .set_file_cache(file_content, file_name.to_string())
                        .await;
                    //当前节点开始提供此文件
                    node_client.start_providing(file_name.to_string()).await;
                }
            });
            loop {}
        }
    }

    Ok(())
}

///
/// ```
/// cargo run -- --listen-address-with-port  /ip4/192.168.6.182/tcp/12000 \
/// --bootstrap-peer /ip4/192.168.6.55/tcp/12001/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X  \
/// common-node
/// cargo run -- --listen-address-with-port  /ip4/192.168.6.182/tcp/12000 \
/// --bootstrap-peer /ip4/192.168.6.55/tcp/12001/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X  \
/// bootstrap
/// ```
/// 测试代码：写在main中用来测试控制台传参是否能正确的获取到
//   let options=Options::parse();
// if let Some(listen_address_with_port)=options.listen_address_with_port{
//     println!("{}",listen_address_with_port);
//    }
//    if let Some(bootstrap_peer)=options.bootstrap_peer{
//     println!("{}",bootstrap_peer);
//    }
//    match options.node_type {
//        NodeTypes::Bootstrap=>{
//         print!("bootstrap");
//        },
//        NodeTypes::CommonNode=>{
//         print!("commonNode") ;
//        }
//    }
#[derive(Parser, Debug)]
#[clap(name = "file_sharing options")]
struct Options {
    //当前节点初始化时需要监听的地址端口
    #[clap(long)]
    listen_address_with_port: Option<Multiaddr>,

    //需要dialing的引导节点的MultiAddr
    #[clap(long)]
    bootstrap_peer: Option<Multiaddr>,

    //需要dialing的引导节点的MultiAddr
    #[clap(subcommand)]
    node_type: NodeTypes,
}
#[derive(Debug, Parser, Clone)]
enum NodeTypes {
    Bootstrap,
    CommonNode,
}

struct FileManage {
    file_event_receiver: mpsc::Receiver<Event>,
    command_sender: mpsc::Sender<Command>,
    file_cache: HashMap<String, Vec<u8>>,
}
impl FileManage {
    async fn run(mut self) {
        loop {
            match self.file_event_receiver.next().await {
                Some(network::Event::InboundRequest { request, channel }) => {
                    let _ = self.command_sender.send(Command::RespondFile {
                        file: self.get_file_content_by_self_cache(&request).unwrap(),
                        file_name: request,
                        channel,
                    });
                }
                Some(network::Event::SetFileCache {
                    file_name,
                    file_content,
                    sender,
                }) => {
                    self.set_file_cache(&file_name, file_content);
                    sender.send(Ok(()));
                }
                Some(network::Event::GetFileFromS3 { file_name, sender }) => {
                    sender.send(Ok(self
                        .get_file_content_by_s3_cache(&file_name)
                        .await
                        .unwrap()));
                }
                e => todo!("{:?}", e),
            }
        }
    }

    fn set_file_cache(&mut self, file_name: &str, file_content_vec: Vec<u8>) -> Option<Vec<u8>> {
        self.file_cache
            .insert(file_name.to_owned(), file_content_vec)
    }

    fn get_file_content_by_self_cache(&self, file_name: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        let content = self.file_cache.get(file_name);
        match content {
            Some(file_vec) => Ok(file_vec.to_owned()),
            None => {
                //正常来说调用此方法是因为本身节点宣布提供该文件，此处却无法提供，所以panic
                panic!("异常：当前节点无法提供{}文件", file_name);
            }
        }
    }

    async fn get_file_content_by_s3_cache(
        &self,
        file_name: &str,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut f = File::open("/Users/wangshuai/Documents/rustWork/Hackathon-2023/chunchunmengxindui/file-sharing/target/fileStorage/ans.txt")?;
        let mut buffer = Vec::new();
        // read the whole file
        f.read_to_end(&mut buffer)?;
        Ok(buffer)
    }
    async fn write_one_file(
        &self,
        file_name: &str,
        file_content_vec: Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        let mut file: File = File::create(format!("../../target/fileStorage/{}.txt", file_name))?;

        file.write_all(&file_content_vec)?;
        file.flush()?;
        Ok(())
    }
}
