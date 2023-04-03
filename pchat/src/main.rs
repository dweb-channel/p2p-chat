// mod cli; // 命令行支持

use async_std::io;
use futures::{future::Either, prelude::*, select};
use libp2p::{
    core::{muxing::StreamMuxerBox, transport::OrTransport, upgrade},
    gossipsub, mdns, noise,
    swarm::{SwarmBuilder, SwarmEvent},
    tcp, yamux, Transport,
};
use libp2p_quic as quic;
use libp2p_swarm_derive::NetworkBehaviour;
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use pchat_utils::message_id_generator::MessageIdGenerator;
use pchat_account::Account;


#[derive(NetworkBehaviour)]
struct ChatBehaviour {
    gossipsub: gossipsub::Behaviour, // gossipsub 行为用于点对点广播消息， https://github.com/libp2p/specs/tree/master/pubsub/gossipsub
    mdns: mdns::async_io::Behaviour, // Mdns 行为用于发现局域网中的其他节点。
}

impl From<gossipsub::Event> for ChatBehaviourEvent {
    fn from(event: gossipsub::Event) -> Self {
        ChatBehaviourEvent::Gossipsub(event)
    }
}

impl From<mdns::Event> for ChatBehaviourEvent {
    fn from(event: mdns::Event) -> Self {
        ChatBehaviourEvent::Mdns(event)
    }
}

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
            noise::NoiseAuthenticated::xx(&user.id_keys).expect("signing libp2p-noise static keypair"),
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

    // 对于内容地址消息，我们可以获取消息并且加上发送时间的哈希值并将其用作 ID。
    let message_id_fn = |_message: &gossipsub::Message| {
        let mut s = DefaultHasher::new();
        // message.data.hash(&mut s);
        MessageIdGenerator::next_id().hash(&mut s);
        gossipsub::MessageId::from(s.finish().to_string())
    };

    // 设置自定义 gossipsub 配置
    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(10)) // 设置为通过不使日志混乱来帮助调试
        .validation_mode(gossipsub::ValidationMode::Strict) // 这设置了消息验证的类型。 默认值为 Strict（强制消息签名）
        .message_id_fn(message_id_fn) // 内容地址消息。 不会传播相同内容的两条消息。
        .build()
        .expect("Valid config");

    //建立订阅网络行为
    let mut gossipsub = gossipsub::Behaviour::new(
        gossipsub::MessageAuthenticity::Signed(user.id_keys),
        gossipsub_config,
    )
    .expect("Correct configuration");
    // Create a Gossipsub topic
    let topic = gossipsub::IdentTopic::new("test-net");
    // subscribes to our topic
    gossipsub.subscribe(&topic)?;

    // Create a Swarm to manage peers and events
    let mut swarm = {
        let mdns = mdns::async_io::Behaviour::new(mdns::Config::default(), user.peer_id)?;
        let behaviour = ChatBehaviour { gossipsub, mdns };
        SwarmBuilder::with_async_std_executor(transport, behaviour, user.peer_id).build()
    };

    // 从 stdin 读取整行
    let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();

    //监听所有接口和操作系统分配的任何端口
    swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    println!("Enter messages via STDIN and they will be sent to connected peers using Gossipsub");

    // Kick it off
    loop {
        select! {
            line = stdin.select_next_some() => {
                if let Err(e) = swarm
                    .behaviour_mut().gossipsub
                    .publish(topic.clone(), line.expect("Stdin not to close").as_bytes()) {
                    println!("Publish error: {e:?}");
                }
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
                }
                _ => {}
            }
        }
    }
}
