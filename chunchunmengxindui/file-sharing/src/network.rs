use futures::channel::mpsc::Receiver;
use futures::channel::{mpsc, oneshot};
use futures::prelude::*;

use libp2p::{
    core::Multiaddr,
    identity, kad,
    multiaddr::Protocol,
    noise,
    request_response::{self, OutboundRequestId, ProtocolSupport, ResponseChannel},
    swarm::{NetworkBehaviour, Swarm, SwarmEvent},
    tcp, yamux, PeerId,
};

use libp2p::{build_multiaddr, identify, ping, StreamProtocol};
use serde::{Deserialize, Serialize};
use std::clone;
use std::collections::{hash_map, HashMap, HashSet};
use std::error::Error;
use std::time::Duration;
use tracing::{debug, info};

use crate::{NodeTypes, Options};

pub(crate) async fn new(
    options: Options,
) -> Result<(Client, Receiver<Event>, EventLoop), Box<dyn Error>> {
    //创建一个秘钥对
    let id_keys = match options.node_type {
        NodeTypes::Bootstrap => {
            //通过这种方式让引导节点产生一个固定的peerID
            let mut bytes = [0u8; 32];
            bytes[0] = 1;
            identity::Keypair::ed25519_from_bytes(bytes).unwrap()
        }
        NodeTypes::CommonNode => identity::Keypair::generate_ed25519(),
    };

    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(id_keys)
        .with_async_std()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| Behaviour {
            kademlia: {
                let mut cfg = kad::Config::default();
                cfg.set_query_timeout(Duration::from_secs(5 * 60));
                let store = kad::store::MemoryStore::new(key.public().to_peer_id());
                kad::Behaviour::with_config(key.public().to_peer_id(), store, cfg)
            },
            request_response: request_response::cbor::Behaviour::new(
                [(
                    StreamProtocol::new("/file-exchange/1.0.0"),
                    ProtocolSupport::Full,
                )],
                request_response::Config::default(),
            ),
            identify: identify::Behaviour::new(identify::Config::new(
                "/identify/1.0.0".to_string(),
                key.public(),
            )),
            ping: ping::Behaviour::default(),
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60 * 10)))
        .build();

    swarm
        .behaviour_mut()
        .kademlia
        .set_mode(Some(kad::Mode::Server));
    let _ = swarm.behaviour_mut().kademlia.bootstrap();

    let (command_sender, command_receiver) = mpsc::channel(0);
    let (event_sender, event_receiver) = mpsc::channel(0);

    Ok((
        Client {
            command_sender,
            event_sender: event_sender.clone(),
        },
        event_receiver,
        EventLoop::new(swarm, command_receiver, event_sender, options),
    ))
}

#[derive(Clone)]
pub(crate) struct Client {
    command_sender: mpsc::Sender<Command>,
    event_sender: mpsc::Sender<Event>,
}

impl Client {
    pub fn get_sender_clone(&self) -> mpsc::Sender<Command> {
        self.command_sender.clone()
    }

    /// 监听指定地址端口
    pub(crate) async fn start_listening(
        &mut self,
        addr: Multiaddr,
    ) -> Result<(), Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();
        self.command_sender
            .send(Command::StartListening { addr, sender })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    /// Dial the given peer at the given address.
    pub(crate) async fn dial(
        &mut self,
        peer_id: PeerId,
        peer_addr: Multiaddr,
    ) -> Result<(), Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();
        self.command_sender
            .send(Command::Dial {
                peer_id,
                peer_addr,
                sender,
            })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }
    /// Advertise the local node as the provider of the given file on the DHT.
    pub(crate) async fn start_providing(&mut self, file_name: String) {
        let (sender, receiver) = oneshot::channel();
        self.command_sender
            .send(Command::StartProviding { file_name, sender })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.");
    }

    /// Find the providers for the given file on the DHT.
    pub(crate) async fn get_providers(
        &mut self,
        file_name: String,
    ) -> HashSet<(PeerId, Multiaddr)> {
        let (sender, receiver) = oneshot::channel();
        self.command_sender
            .send(Command::GetProviders { file_name, sender })
            .await
            .expect("Command receiver not to be dropped.");
        let provides = receiver.await.expect("Sender not to be dropped.");

        let (sender, receiver) = oneshot::channel();
        self.command_sender
            .send(Command::GetAddrByProvides {
                provides: provides.clone(),
                sender,
            })
            .await
            .expect("Command receiver not to be dropped.");
        let result: Result<Vec<String>, Box<dyn Error + Send>> =
            receiver.await.expect("Sender not to be dropped.");
        // 记录到已知节点
        info!("当前获取的引导节点DHT记录：{:?}", result);
        let mut hash_set = HashSet::new();
        result.unwrap().iter().for_each(|addr| {
            let ma: Multiaddr = addr.parse().unwrap();
            if let Some(Protocol::P2p(peer_id)) = ma.iter().last() {
                if provides.contains(&peer_id) {
                    hash_set.insert((peer_id, ma));
                }
            }
        });
        info!("该文件有如下提供者:{:?}", provides.clone());
        hash_set
    }
    pub(crate) async fn get_closest_peers(&mut self, peer: PeerId) {
        let (sender, receiver) = oneshot::channel();
        self.command_sender
            .send(Command::GetClosetPeers { peer, sender })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not be dropped.");
    }
    /// Request the content of the given file from the given peer.
    pub(crate) async fn request_file(
        &mut self,
        peer: PeerId,
        multiaddr: Multiaddr,
        file_name: String,
    ) -> Result<(Vec<u8>, String), Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();
        self.command_sender
            .send(Command::RequestFile {
                file_name,
                peer,
                multiaddr,
                sender,
            })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not be dropped.")
    }

    pub(crate) async fn set_file_cache(
        &mut self,
        file_content: Vec<u8>,
        file_name: String,
    ) -> Result<(), Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();
        self.event_sender
            .send(Event::SetFileCache {
                file_name,
                file_content,
                sender,
            })
            .await
            .expect("将文件内容存入文件管理器缓存失败");
        receiver.await.expect("Sender not be dropped.")
    }

    pub(crate) async fn get_file_content_by_s3_cache(
        &mut self,
        file_name: String,
    ) -> Result<Vec<u8>, Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();
        self.event_sender
            .send(Event::GetFileFromS3 { file_name, sender })
            .await
            .expect("从S3中获取文件失败");
        receiver.await.expect("Sender not be dropped.")
    }
}

pub(crate) struct EventLoop {
    swarm: Swarm<Behaviour>,
    command_receiver: mpsc::Receiver<Command>,
    event_sender: mpsc::Sender<Event>,
    pending_dial: HashMap<PeerId, oneshot::Sender<Result<(), Box<dyn Error + Send>>>>,
    pending_start_providing: HashMap<kad::QueryId, oneshot::Sender<()>>,
    pending_get_providers: HashMap<kad::QueryId, oneshot::Sender<HashSet<PeerId>>>,
    pending_request_file: HashMap<
        OutboundRequestId,
        oneshot::Sender<Result<(Vec<u8>, String), Box<dyn Error + Send>>>,
    >,
    pending_request_query_provides:
        HashMap<OutboundRequestId, oneshot::Sender<Result<Vec<String>, Box<dyn Error + Send>>>>,
    options: Options,
}

impl EventLoop {
    fn new(
        swarm: Swarm<Behaviour>,
        command_receiver: mpsc::Receiver<Command>,
        event_sender: mpsc::Sender<Event>,
        options: Options,
    ) -> Self {
        Self {
            swarm,
            command_receiver,
            event_sender,
            pending_dial: Default::default(),
            pending_start_providing: Default::default(),
            pending_get_providers: Default::default(),
            pending_request_file: Default::default(),
            pending_request_query_provides: Default::default(),
            options,
        }
    }

    pub(crate) async fn run(mut self) {
        loop {
            futures::select! {
                event = self.swarm.next() => self.handle_event(event.expect("Swarm stream to be infinite.")).await  ,
                command = self.command_receiver.next() => match command {
                    Some(c) => self.handle_command(c).await,
                    // Command channel closed, thus shutting down the network event loop.
                    None=>  return,
                },
            }
        }
    }

    async fn handle_event(&mut self, event: SwarmEvent<BehaviourEvent>) {
        match event {
            //https://docs.rs/libp2p/latest/libp2p/kad/index.html#important-discrepancies 必须通过Identify才能节点发现
            SwarmEvent::Behaviour(BehaviourEvent::Identify(identify::Event::Received {
                peer_id,
                info: identify::Info { listen_addrs, .. },
            })) => {
                for addr in listen_addrs {
                    self.swarm
                        .behaviour_mut()
                        .kademlia
                        .add_address(&peer_id, addr.clone());
                    debug!("Identify后将{}节点-地址{} 添加到DHT", &peer_id, addr);
                }
            }
            SwarmEvent::Behaviour(BehaviourEvent::Ping(ping::Event {
                peer,
                result: Err(_),
                ..
            })) => {
                //ping失败去除节点
                info!("将{}节点从DHT中去除", &peer);
                self.swarm.behaviour_mut().kademlia.remove_peer(&peer);
            }
            SwarmEvent::Behaviour(BehaviourEvent::Identify(identify::Event::Sent { peer_id })) => {}
            SwarmEvent::Behaviour(BehaviourEvent::Kademlia(
                kad::Event::OutboundQueryProgressed {
                    id,
                    result: kad::QueryResult::StartProviding(_),
                    ..
                },
            )) => {
                let sender: oneshot::Sender<()> = self
                    .pending_start_providing
                    .remove(&id)
                    .expect("Completed query to be previously pending.");
                let _ = sender.send(());
            }
            SwarmEvent::Behaviour(BehaviourEvent::Kademlia(
                kad::Event::OutboundQueryProgressed {
                    id,
                    result:
                        kad::QueryResult::GetProviders(Ok(kad::GetProvidersOk::FoundProviders {
                            providers,
                            ..
                        })),
                    ..
                },
            )) => {
                if let Some(sender) = self.pending_get_providers.remove(&id) {
                    // self.swarm.behaviour_mut().kademlia.kbuckets();
                    info!("swarm中收到如下提供者：{:?}", providers.clone());
                    sender.send(providers).expect("Receiver not to be dropped");

                    // Finish the query. We are only interested in the first result.
                    self.swarm
                        .behaviour_mut()
                        .kademlia
                        .query_mut(&id)
                        .unwrap()
                        .finish();
                }
            }
            SwarmEvent::Behaviour(BehaviourEvent::RequestResponse(
                request_response::Event::Message { message, .. },
            )) => match message {
                request_response::Message::Request {
                    request, channel, ..
                } => {
                    if let MyRequest(message_body) = request {
                        match message_body {
                            MessageRequestBody::FileContentMessageRequestBody { file_name } => {
                                info!("收到入站文件请求 file_name:{:?}", file_name);
                                self.event_sender
                                    .send(Event::InboundRequest {
                                        request: file_name,
                                        channel,
                                    }).await
                                    .expect("Event receiver not to be dropped.");
                            }
                            MessageRequestBody::MultiaddrsMessageRequestBody { provides } => {
                                info!("收到入站DHT记录请求 provides:{:?}", provides);
                                //获取引导节点存储的有关提供者的MultiAddr
                                let know_peers = self.swarm.behaviour_mut().known_peers();
                                //TODO 暂时获取全量记录，不通过provides过滤
                                let multiaddrs: Vec<String> = know_peers
                                    .into_values()
                                    .flatten()
                                    .map(|addr| addr.to_string())
                                    .collect();
                                self.swarm
                                    .behaviour_mut()
                                    .request_response
                                    .send_response(
                                        channel,
                                        MyResponse(MessageRespBody::MultiaddrsMessageRespBody {
                                            multiaddrs,
                                        }),
                                    )
                                    .expect("Connection to peer to be still open.");
                            }
                        }
                    }
                }
                request_response::Message::Response {
                    request_id,
                    response,
                } => match response {
                    MyResponse(MessageRespBody::FileContentMessageRespBody {
                        file_name,
                        file_content,
                    }) => {
                        info!("收到文件内容响应");
                        let _ = self
                            .pending_request_file
                            .remove(&request_id)
                            .expect("Request to still be pending.")
                            .send(Ok((file_content, file_name)));
                    }
                    MyResponse(MessageRespBody::MultiaddrsMessageRespBody { multiaddrs }) => {
                        info!("收到DHT记录响应");
                        let _ = self
                            .pending_request_query_provides
                            .remove(&request_id)
                            .expect("Request to still be pending.")
                            .send(Ok(multiaddrs));
                    }
                },
            },
            SwarmEvent::Behaviour(BehaviourEvent::RequestResponse(
                request_response::Event::OutboundFailure {
                    request_id, error, ..
                },
            )) => {
                info!("OutboundFailure:{:?}", error);
                let _ = self
                    .pending_request_file
                    .remove(&request_id)
                    .expect("Request to still be pending.")
                    .send(Err(Box::new(error)));
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                let local_peer_id = *self.swarm.local_peer_id();
                eprintln!(
                    "Local node is listening on {:?}",
                    address.with(Protocol::P2p(local_peer_id))
                );
            }
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                if endpoint.is_dialer() {
                    if let Some(sender) = self.pending_dial.remove(&peer_id) {
                        let _ = sender.send(Ok(()));
                    }
                }
            }
            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                //链接不到节点就将其从DHT中去除
                if let Some(peer_id) = peer_id {
                    info!("将{}节点从DHT中去除", &peer_id);
                    self.swarm.behaviour_mut().kademlia.remove_peer(&peer_id);
                    if let Some(sender) = self.pending_dial.remove(&peer_id) {
                        let _ = sender.send(Err(Box::new(error)));
                    }
                }
            }
            SwarmEvent::Dialing {
                peer_id: Some(peer_id),
                ..
            } => debug!("Dialing {peer_id}"),
            SwarmEvent::Behaviour(BehaviourEvent::Ping { .. }) => {
                //ping事件是这种形式：Behaviour(BehaviourEvent: Event { peer: PeerId("12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X"), connection: ConnectionId(1), result: Ok(23.470875ms) })
            }
            _e => {
                // println!(
                //     "未处理Swarm事件{:?},当前K桶:{:?}",
                //     e,
                //     self.swarm.behaviour_mut().known_peers()
                // );
            }
        }
    }

    async fn handle_command(&mut self, command: Command) {
        // info!("处理命令：{:?}", command);
        match command {
            Command::StartListening { addr, sender } => {
                let _ = match self.swarm.listen_on(addr) {
                    Ok(_) => sender.send(Ok(())),
                    Err(e) => sender.send(Err(Box::new(e))),
                };
            }
            Command::Dial {
                peer_id,
                peer_addr,
                sender,
            } => {
                if let hash_map::Entry::Vacant(e) = self.pending_dial.entry(peer_id) {
                    self.swarm
                        .behaviour_mut()
                        .kademlia
                        .add_address(&peer_id, peer_addr.clone());
                    match self.swarm.dial(peer_addr.with(Protocol::P2p(peer_id))) {
                        Ok(()) => {
                            e.insert(sender);
                        }
                        Err(e) => {
                            let _ = sender.send(Err(Box::new(e)));
                        }
                    }
                } else {
                    todo!("Already dialing peer.");
                }
            }
            Command::StartProviding { file_name, sender } => {
                let query_id = self
                    .swarm
                    .behaviour_mut()
                    .kademlia
                    .start_providing(file_name.into_bytes().into())
                    .expect("No store error.");
                self.pending_start_providing.insert(query_id, sender);
            }
            Command::GetProviders { file_name, sender } => {
                let query_id = self
                    .swarm
                    .behaviour_mut()
                    .kademlia
                    .get_providers(file_name.into_bytes().into());
                self.pending_get_providers.insert(query_id, sender);
            }
            Command::GetClosetPeers { peer, sender } => {
                async {
                    info!("执行对peerID为{}的get_closest_peers", peer);
                    self.swarm.behaviour_mut().kademlia.get_closest_peers(peer);
                }
                .await;
                sender.send(Ok(()));
            }
            Command::GetAddrByProvides { provides, sender } => {
                //向引导节点发请求，查询provide对应的MultiAddr
                if let Some(Protocol::P2p(peer_id)) =
                    self.options.bootstrap_peer.clone().unwrap().iter().last()
                {
                    let request_id = self.swarm.behaviour_mut().request_response.send_request(
                        &peer_id,
                        MyRequest(MessageRequestBody::MultiaddrsMessageRequestBody {
                            provides: provides.into_iter().map(|p| p.to_base58()).collect(),
                        }),
                    );
                    self.pending_request_query_provides
                        .insert(request_id, sender);
                }
            }
            Command::RequestFile {
                file_name,
                peer,
                multiaddr,
                sender,
            } => {
                self.swarm
                    .behaviour_mut()
                    .kademlia
                    .add_address(&peer, multiaddr);
                info!(
                    "处理文件请求命令:{}",
                    file_name
                );
                let request_id = self.swarm.behaviour_mut().request_response.send_request(
                    &peer,
                    MyRequest(MessageRequestBody::FileContentMessageRequestBody { file_name }),
                );
                self.pending_request_file.insert(request_id, sender);
            }
            Command::RespondFile {
                file,
                file_name,
                channel,
            } => {
                // info!("filecontent:{:?}",file);
                // info!(
                //     "处理RespondFile命令,文件：{},当前k桶已知{:?}",
                //     file_name,
                //     self.swarm.behaviour_mut().known_peers()
                // );
                self.swarm
                    .behaviour_mut()
                    .request_response
                    .send_response(
                        channel,
                        MyResponse(MessageRespBody::FileContentMessageRespBody {
                            file_name,
                            file_content: file,
                        }),
                    )
                    .expect("Connection to peer to be still open.");
            }
        }
    }
}

#[derive(NetworkBehaviour)]
struct Behaviour {
    request_response: request_response::cbor::Behaviour<MyRequest, MyResponse>,
    kademlia: kad::Behaviour<kad::store::MemoryStore>,
    identify: identify::Behaviour,
    ping: ping::Behaviour,
}
impl Behaviour {
    pub fn known_peers(&mut self) -> HashMap<PeerId, Vec<Multiaddr>> {
        let mut peers = HashMap::new();
        for b in self.kademlia.kbuckets() {
            for e in b.iter() {
                peers.insert(*e.node.key.preimage(), e.node.value.clone().into_vec());
            }
        }

        peers
    }
}

#[derive(Debug)]
pub(crate) enum Command {
    StartListening {
        addr: Multiaddr,
        sender: oneshot::Sender<Result<(), Box<dyn Error + Send>>>,
    },
    Dial {
        peer_id: PeerId,
        peer_addr: Multiaddr,
        sender: oneshot::Sender<Result<(), Box<dyn Error + Send>>>,
    },
    StartProviding {
        file_name: String,
        sender: oneshot::Sender<()>,
    },
    GetProviders {
        file_name: String,
        sender: oneshot::Sender<HashSet<PeerId>>,
    },
    RequestFile {
        file_name: String,
        peer: PeerId,
        multiaddr: Multiaddr,
        sender: oneshot::Sender<Result<(Vec<u8>, String), Box<dyn Error + Send>>>,
    },
    RespondFile {
        file: Vec<u8>,
        file_name: String,
        channel: ResponseChannel<MyResponse>,
    },
    GetClosetPeers {
        peer: PeerId,
        sender: oneshot::Sender<Result<(), Box<dyn Error + Send>>>,
    },
    GetAddrByProvides {
        provides: HashSet<PeerId>,
        sender: oneshot::Sender<Result<Vec<String>, Box<dyn Error + Send>>>,
    },
}

#[derive(Debug)]
pub(crate) enum Event {
    InboundRequest {
        request: String,
        channel: ResponseChannel<MyResponse>,
    },
    SetFileCache {
        file_name: String,
        file_content: Vec<u8>,
        sender: oneshot::Sender<Result<(), Box<dyn Error + Send>>>,
    },
    GetFileFromS3 {
        file_name: String,
        sender: oneshot::Sender<Result<Vec<u8>, Box<dyn Error + Send>>>,
    },
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(crate) enum MessageRequestBody {
    FileContentMessageRequestBody { file_name: String },
    MultiaddrsMessageRequestBody { provides: Vec<String> },
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(crate) enum MessageRespBody {
    FileContentMessageRespBody {
        file_content: Vec<u8>,
        file_name: String,
    },
    MultiaddrsMessageRespBody {
        multiaddrs: Vec<String>,
    },
}
//req-res的传输需要有EOF
pub(crate) struct TransportBody(Vec<u8>);
// Simple file exchange protocol
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct MyRequest(MessageRequestBody);
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MyResponse(MessageRespBody);

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{MessageRespBody, MyResponse};

    #[test]
    fn serialize_myresponse() {
        let current_dir = std::env::current_dir().expect("Failed to get current directory");
        let file_path = current_dir.join("fileStorage/ans.txt");
        println!("{:?}", file_path);
        if let Ok(content) = fs::read_to_string(file_path) {
            // println!("{}", &content);
            let a = MyResponse(MessageRespBody::FileContentMessageRespBody {
                file_name: "文件名称".to_string(),
                file_content: content.into_bytes(),
            });
            println!("{}", serde_json::to_string(&a).unwrap());
        }
    }
}
