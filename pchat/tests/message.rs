use futures::future::Either;
use libp2p::{
    core::{muxing::StreamMuxerBox, transport::OrTransport, upgrade},
    gossipsub::{self},
    mdns, noise,
    swarm::SwarmBuilder,
    tcp, yamux, Swarm, Transport,
};
use libp2p_quic as quic;
use pchat::behaviour::*;
use pchat_account::Account;
use pchat_utils::message_id_generator::MessageIdGenerator;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;

#[tokio::test]
async fn test_message() {
    // 节点 1
    let node1 = Account::new();

    let mut swarm1 = create_chat(node1);
    // 节点 2
    let node2 = Account::new();
    let mut swarm2 = create_chat(node2);

    // 发送消息
    if let Err(e) = swarm1
        .behaviour_mut()
        .gossipsub
        .publish(gossipsub::IdentTopic::new("test-net"), "Hello, node1!".as_bytes().to_vec())
    {
        println!("Publish error: {e:?}");
    }

    if let Err(e) = swarm2
        .behaviour_mut()
        .gossipsub
        .publish(gossipsub::IdentTopic::new("test-net"), "Hello, node2!".as_bytes().to_vec())
    {
        println!("Publish error: {e:?}");
    }
}

fn create_chat(node: Account) -> Swarm<pchat::behaviour::ChatBehaviour> {
    // 通过 Mplex 协议设置加密的启用 DNS 的 TCP 传输。
    let tcp_transport = tcp::async_io::Transport::new(tcp::Config::default().nodelay(true))
        .upgrade(upgrade::Version::V1)
        .authenticate(
            noise::NoiseAuthenticated::xx(&node.id_keys)
                .expect("signing libp2p-noise static keypair"),
        )
        .multiplex(yamux::YamuxConfig::default())
        .timeout(std::time::Duration::from_secs(20))
        .boxed();
    let quic_transport = quic::tokio::Transport::new(quic::Config::new(&node.id_keys));
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
    let gossipsub = gossipsub::Behaviour::new(
        gossipsub::MessageAuthenticity::Signed(node.id_keys),
        gossipsub_config,
    )
    .expect("Correct configuration");

    // Create a Swarm to manage peers and events
    let mut swarm = {
        let mdns = mdns::async_io::Behaviour::new(mdns::Config::default(), node.peer_id).unwrap();
        let behaviour = ChatBehaviour { gossipsub, mdns };
        SwarmBuilder::with_async_std_executor(transport, behaviour, node.peer_id).build()
    };

    //监听所有接口和操作系统分配的任何端口
    swarm.listen_on(node.address).unwrap();
    return swarm;
}
