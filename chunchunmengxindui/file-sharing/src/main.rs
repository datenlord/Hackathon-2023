mod network;
use std::{
    collections::BTreeMap,
    error::Error,
    fs::File,
    io::{Read, Write},
    option,
};

use async_std::task::spawn;
use clap::Parser;
use libp2p::{futures::future::ok, multiaddr::Protocol, Multiaddr};
#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    //存储在节点中的 待获取文件的信息
    let mut file_metadata_hashmap = BTreeMap::new();
    file_metadata_hashmap.insert("file1", ("file1"));
    file_metadata_hashmap.insert("file2", ("file2"));

    //获取控制台参数
    let options = Options::parse();
    //构建network节点
    let (mut node_client, file_event_receiver, event_loop) = network::new().await?;
    //运行第二层管道处理
    spawn(event_loop.run());
    //运行第三层管道处理
    spawn( async{
        loop {
            
        }
    });
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
                    .start_listening(addr)
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
            let mut node_client_clone=node_client.clone();
            spawn(async move{
                for (file_name,value) in file_metadata_hashmap {
                     //从DHT网络中获取当前所需文件的提供者
                let providers=node_client_clone.get_providers(file_name.to_owned().to_owned()).await;
                
                }
            });
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

async fn get_file_content_by_s3_cache(file_name: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut f = File::open("../target/fileStorage/ans.txt")?;
    let mut buffer = Vec::new();
    // read the whole file
    f.read_to_end(&mut buffer)?;
    Ok(buffer)
}
async fn write_one_file(file_name: &str, file_content_vec: Vec<u8>) -> Result<(), Box<dyn Error>> {
    let mut file: File = File::create(format!(
        "../target/fileStorage/{}.txt",
        file_name
    ))?;

    file.write_all(&file_content_vec)?;
    file.flush()?;
    Ok(())
}
