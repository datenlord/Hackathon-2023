mod network;
use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
    fs::File,
    io::{Read, Write},
    option, vec,
};

use async_std::task::spawn;
use clap::Parser;
use futures::{channel::mpsc, FutureExt, SinkExt, Stream, StreamExt};
use libp2p::{build_multiaddr, futures::future::ok, multiaddr::Protocol, Multiaddr};
use network::{Command, Event};
use tracing::{info, level_filters::LevelFilter, span, warn, Level};
use tracing_subscriber::EnvFilter;
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    //用于帮助测试的参数，为true会使节点下载ans_4KB.txt，目的是为了验证文件传输的正确性。为false会下载ans_1MB.txt文件，目的是测试传输速度
    let test_content_flag = false;
    //注册tracing_subscriber
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::WARN.into())
                .from_env_lossy(),
        )
        .try_init();

    //存储在节点中的 待获取文件的信息 ans_1GB.txt文件假设是由1024个file block组成，一个file block用ans_1MB.txt文件模拟；ans_4KB.txt则是用于测试传输准确性，其只由ans_4KB.txt文件组成
    let mut file_metadata_hashmap = HashMap::new();
    file_metadata_hashmap.insert("ans_1GB.txt".to_string(), ("ans_1MB.txt".to_string(), 1024));
    file_metadata_hashmap.insert("ans_4KB.txt".to_string(), ("ans_4KB.txt".to_string(), 1));

    //需要获取的文件名称  修改这里以进行测试
    let need_get_file_names;
    if test_content_flag {
         need_get_file_names = vec!["ans_4KB.txt"];
    }else {
         need_get_file_names = vec!["ans_1GB.txt"];
    }
    

    //获取控制台参数
    let options = Options::parse();
    //构建network节点
    let (mut node_client, file_event_receiver, event_loop) = network::new(options.clone()).await?;

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
            info!("引导节点创建成功");
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
                let node_client = node_client_clone;
                for file_name in need_get_file_names {
                    info!("文件{}对应任务：开始获取文件", file_name);
                    let file_child_blocks =
                        FileManage::get_file_block_by_metadata(&file_metadata_hashmap, file_name);
                    let len = file_child_blocks.len();
                    warn!(
                        ">>>>>>>>>>>>启动文件:{}获取，该文件由{}个子文件组成",
                        &file_name, len
                    );

                    for (file_name, index) in file_child_blocks {
                        let mut node_client = node_client.clone();
                        //从DHT网络中获取当前所需文件的提供者
                        let providers = node_client
                            .get_providers(file_name.to_owned().to_owned())
                            .await;
                        //如果不存在提供者，就向filemanager发送通过s3下载的请求
                        let file_content: Vec<u8>;
                        if providers.is_empty() {
                            info!("文件{}对应任务：当前网络无提供者，从S3中下载", file_name);
                            file_content = node_client
                                .get_file_content_by_s3_cache(file_name.to_string())
                                .await
                                .unwrap();
                            info!(
                                "文件{}对应任务：当前网络无提供者，从S3中下载完成",
                                file_name
                            );
                        } else {
                            info!("文件{}对应任务：从网络中的提供者处下载文件", file_name);
                            //创建一些从Peer节点获取文件内容的future
                            let requests = providers.into_iter().map(|p| {
                                let mut node_client = node_client.clone();
                                let file_name = file_name.clone();
                                async move {
                                    // node_client.get_closest_peers(p).await;
                                    node_client.request_file(p.0, p.1, file_name).await
                                }
                                .boxed()
                            });
                            //选择其中一个进行执行
                            let get_file_result = futures::future::select_ok(requests).await;
                            match get_file_result {
                                Ok(((fille_content_result, ..), ..)) => {
                                    file_content = fille_content_result;
                                    // info!("读取文件转化为字符串{:?}",String::from_utf8(file_content.clone()).unwrap());
                                    info!(
                                        "文件{}对应任务：从网络中的提供者处下载文件完成",
                                        file_name
                                    );
                                }
                                Err(e) => {
                                    info!("文件{}对应任务：当前网络中提供者无法提供该文件，转移成使用S3下载",file_name);
                                    file_content = node_client
                                        .get_file_content_by_s3_cache(file_name.to_string())
                                        .await
                                        .unwrap();
                                    info!(
                                        "文件{}对应任务：当前网络提供者无法正常提供，成功从S3下载",
                                        file_name
                                    );
                                }
                            }
                        }
                        if test_content_flag {
                            warn!(
                                "查看文件内容：{:?}",
                                String::from_utf8(file_content.clone()).unwrap()
                            );
                        }
                        //本地存储文件缓存
                        let _ = node_client
                            .set_file_cache(file_content, file_name.to_string())
                            .await;
                        //当前节点开始提供此文件
                        node_client.start_providing(file_name.to_string()).await;
                        info!("文件{}对应任务：当前节点开始提供该文件", file_name);
                    }
                    warn!(
                        "<<<<<<<<<<<<文件:{}获取完成，该文件可能由{}个子文件组成",
                        &file_name, len
                    );
                }
            });
            loop {}
        }
    }

    Ok(())
}

///
/// ```
/// //启动引导节点
/// cargo run -- --listen-address-with-port  /ip4/192.168.6.182/tcp/13000 \
/// common-node
///
/// //启动普通节点
/// cargo run -- --listen-address-with-port  /ip4/192.168.6.182/tcp/14000 \
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
#[derive(Parser, Debug, Clone)]
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
                    info!("当前节点处理入站请求：获取文件{}", &request);
                    let _ = self
                        .command_sender
                        .send(Command::RespondFile {
                            file: self.get_file_content_by_self_cache(&request).unwrap(),
                            file_name: request,
                            channel,
                        })
                        .await
                        .expect("Event receiver not to be dropped.");
                }
                Some(network::Event::SetFileCache {
                    file_name,
                    file_content,
                    sender,
                }) => {
                    info!("处理SetFileCache,将文件:{}存入文件管理器缓存", file_name);
                    self.set_file_cache(&file_name, file_content);
                    sender.send(Ok(()));
                }
                Some(network::Event::GetFileFromS3 { file_name, sender }) => {
                    info!("处理GetFileFromS3,从s3中获取文件：{}", file_name);
                    sender.send(Ok(
                        FileManage::get_file_content_by_s3_cache(&file_name).unwrap()
                    ));
                }
                e => todo!("{:?}", e),
            }
        }
    }

    /// 模拟从文件metadata中取出文件信息，目前假设一个block是1MB，所以一个1GB文件需要1024个1MB的block来组成.这里为了方便，改为获取存储的1MB文件 1024次
    /// 此方法返回的结果就是表示一串block，实际上会是(已有文件名，index)的向量
    fn get_file_block_by_metadata(
        file_metadata_hashmap: &HashMap<String, (String, u16)>,
        file_name: &str,
    ) -> Vec<(String, u16)> {
        let vecs = file_metadata_hashmap.get(file_name).unwrap();
        let mut result = Vec::new();
        for i in 0..vecs.1 {
            result.push((vecs.0.to_string(), i));
        }
        result
    }
    fn set_file_cache(&mut self, file_name: &str, file_content_vec: Vec<u8>) -> Option<Vec<u8>> {
        self.file_cache
            .insert(file_name.to_owned(), file_content_vec)
    }

    fn get_file_content_by_self_cache(&self, file_name: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        let content = self.file_cache.get(file_name);
        match content {
            Some(file_vec) => Ok(file_vec.to_owned().clone()),
            None => {
                //正常来说调用此方法是因为本身节点宣布提供该文件，此处却无法提供，所以panic
                panic!("异常：当前节点无法提供{}文件", file_name);
            }
        }
    }

    fn get_file_content_by_s3_cache(file_name: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        let current_dir = std::env::current_dir().expect("Failed to get current directory");
        let file_path = current_dir.join(format!("fileStorage/{}", file_name));
        let mut f = File::open(file_path)?;
        let mut buffer = Vec::new();
        // read the whole file
        f.read_to_end(&mut buffer)?;
        Ok(buffer)
    }
    fn write_one_file(file_name: &str, file_content_vec: Vec<u8>) -> Result<(), Box<dyn Error>> {
        let current_dir = std::env::current_dir().expect("Failed to get current directory");
        let file_path = current_dir.join(format!("fileStorage/{}", file_name));
        let mut file: File = File::create(file_path)?;
        file.write_all(&file_content_vec)?;
        file.flush()?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {

    use std::{
        fs::File,
        io::{Read, Write},
    };

    #[test]
    fn create_file_by_size() {
        let file_name = "ans_1MB.txt";
        let file_size = 1 * 1024 * 1024; //
        let current_dir = std::env::current_dir().expect("Failed to get current directory");
        println!("{:?}", current_dir);
        let file_path = current_dir.join(format!("fileStorage/{}", file_name));
        let mut file = File::create(file_path).unwrap();
        // 写入数据，将文件大小增加到指定大小
        for _ in 0..file_size {
            file.write_all(&[0u8]).unwrap();
        }
    }
}
