// mod cli; // 命令行支持

use async_std::io;
use futures::{future::Either, prelude::*, select};
use libp2p::{
    core::{muxing::StreamMuxerBox, transport::OrTransport, upgrade},
    gossipsub::{self, Message},
    noise,
    swarm::{SwarmBuilder, dial_opts::DialOpts, SwarmEvent},
    tcp, yamux, Transport, PeerId, mdns,
};
use libp2p_quic as quic;
use pchat::behaviour::*;
use pchat_account::Account;
use pchat_utils::message_id_generator::MessageIdGenerator;
use std::{error::Error};

// 创建了一个结合了 Gossipsub 和 Mdns 的自定义网络行为。

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("start p2p-chat");
    // 创建账号
    let user = Account::new();

    println!("Local peer id: {}", user.peer_id);

    // 通过 Mplex 协议设置加密的启用 DNS 的 TCP 传输。
    let tcp_transport = tcp::async_io::Transport::new(tcp::Config::default().nodelay(true))
        .upgrade(upgrade::Version::V1)
        .authenticate(
            noise::NoiseAuthenticated::xx(&user.id_keys)
                .expect("signing libp2p-noise static keypair"),
        )
        .multiplex(yamux::YamuxConfig::default())
        .timeout(std::time::Duration::from_secs(20))
        .boxed();
    let quic_transport = quic::tokio::Transport::new(quic::Config::new(&user.id_keys));
    let transport = OrTransport::new(quic_transport, tcp_transport)
        .map(|either_output, _| match either_output {
            Either::Left((peer_id, muxer)) => (peer_id, StreamMuxerBox::new(muxer)),
            Either::Right((peer_id, muxer)) => (peer_id, StreamMuxerBox::new(muxer)),
        })
        .boxed();

    // init message id options
    MessageIdGenerator::init();
    // 创建主题
    let topic = gossipsub::IdentTopic::new("test-net");

    // Create a Swarm to manage peers and events
    let mut swarm = {
        let mut behaviour = ChatBehaviour::new(user.clone());
        // 订阅主题
        behaviour.gossipsub.subscribe(&topic)?;
        SwarmBuilder::with_async_std_executor(transport, behaviour, user.peer_id).build()
    };

    //监听所有接口和操作系统分配的任何端口
    swarm.listen_on(user.address)?;

    // 打印监听地址
    for addr in swarm.listeners() {
        println!("Listening on: {:?}", addr);
    }

    // 从 stdin 读取整行
    let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();

    // 异步处理事件
    loop {
        select! {
            line = stdin.select_next_some() => {
                    let line = line.expect("stdin closed");
                    let mut parts = line.split_whitespace();
          
                    match parts.next() {
                        Some("send") => {
                            // 发送点对点消息
                            if let (Some(peer), Some(msg)) = (parts.next(), parts.next()) {
                                let peer_id:PeerId = match peer.parse() {
                                    Ok(peer_id) => peer_id,
                                    Err(err) => {
                                        eprintln!("Invalid peer id: {:?}", err);
                                        continue;
                                    }
                                };
                                let msg_bytes = msg.as_bytes().to_vec();
                                let message = Message {
                                    source:Some(peer_id.clone()),
                                    data: msg_bytes.clone(),
                                    sequence_number: None,
                                    topic:topic.hash()
                                };
                                // swarm.behaviour_mut().send_direct_message(peer_id, message)
                                // swarm.behaviour_mut().gossipsub.publish(topic.clone(),message)?;
                            };
                        }
                        Some("broadcast") => {
                          // 发送群发消息
                            if let Some(msg) = parts.next() {
                                let msg_bytes = msg.as_bytes();
                                if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic.clone(), msg_bytes) {
                                    println!("broadcast error: {:?}",e);
                                }
                            };
                        }
                        Some("connect") => {
                            // 连接到其他节点
                            if let Some(addr) = parts.next() {
                                let addr = match addr.parse() {
                                    Ok(addr) => addr,
                                    Err(err) => {
                                        eprintln!("Invalid multiaddr: {:?}", err);
                                        continue;
                                    }
                                };
                                swarm.dial(DialOpts::unknown_peer_id().address(addr).build()).expect("Failed to dial address");
                            };
                        }
                        Some("help") | Some(_) | None => {
                            eprintln!(
                                "Available commands:\n\
                                 send PEER_ID MESSAGE\n\
                                 broadcast MESSAGE\n\
                                 connect MULTIADDR\n\
                                 help\n\
                                 exit"
                            );
                        }
                        Some("exit") => {
                            break;
                        },
                    };
            },
            event = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(ChatBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, _multiaddr) in list {
                        println!("mDNS discovered a new peer: {peer_id}");
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(ChatBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    for (peer_id, _multiaddr) in list {
                        println!("mDNS discover peer has expired: {peer_id}");
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(ChatBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                    propagation_source: peer_id,
                    message_id: id,
                    message,
                })) => println!(
                        "Got message: '{}' with id: {id} from peer: {peer_id}",
                        String::from_utf8_lossy(&message.data),
                    ),
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Local node is listening on {address}");
                },
                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    println!("Connected to {}", peer_id);
                },
                SwarmEvent::ConnectionClosed { peer_id, .. } => {
                    println!("Disconnected from {}", peer_id);
                },
                SwarmEvent::IncomingConnection { .. } => {
                    println!("Incoming connection");
                },
                SwarmEvent::IncomingConnectionError { .. } => {
                    println!("Incoming connection error");
                },
                _ => {}
            }
        }
    }
    Ok(())
}
