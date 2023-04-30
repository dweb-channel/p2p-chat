//! ChatBehaviour 将作为整个应用的网络层
use libp2p::{
    gossipsub::{self, ConfigBuilder, MessageId, PublishError},
    mdns,
};
use libp2p_swarm::NetworkBehaviourEventProcess;
use libp2p_swarm_derive::NetworkBehaviour;
use pchat_account::Account;
use pchat_utils::message_id_generator::MessageIdGenerator;
use std::{
    collections::{hash_map::DefaultHasher},
    hash::{Hash, Hasher},
    time::Duration,
};

// const GOSSIPSUB_PROTOCOL_ID_PREFIX: Cow<'static, str> =Cow::Borrowed("meshsub");
#[derive(NetworkBehaviour)]
// #[behaviour(out_event = "OutEvent")]
pub struct ChatBehaviour {
    pub gossipsub: gossipsub::Behaviour, // gossipsub 行为用于点对点广播消息， https://github.com/libp2p/specs/tree/master/pubsub/gossipsub
    pub mdns: mdns::async_io::Behaviour, // Mdns 行为用于发现局域网中的其他节点。
}

impl NetworkBehaviourEventProcess<mdns::Event> for ChatBehaviour {
    // 处理mdns事件
    fn inject_event(&mut self, event: mdns::Event) {
        match event {
            // 新对等点
            mdns::Event::Discovered(list) => {
                for (peer, _) in list {
                    self.gossipsub.add_explicit_peer(&peer);
                }
            }
            // 移除对等点
            mdns::Event::Expired(expired_list) => {
                for (peer, _addr) in expired_list {
                    if !self.mdns.has_node(&peer) {
                        self.gossipsub.remove_explicit_peer(&peer);
                    }
                }
            }
            _ => {}
        }
    }
}

impl NetworkBehaviourEventProcess<gossipsub::Event> for ChatBehaviour {
    fn inject_event(&mut self, event: gossipsub::Event) {
        match event {
            gossipsub::Event::Message { message, .. } => {
                if let Ok(message_string) = String::from_utf8(message.data.clone()) {
                    // self.events.push_back(message_string);
                    println!("NetworkBehaviourEventProcess gossipsub: {message_string}");
                }
            },
            // 订阅该主题
            gossipsub::Event::Subscribed { .. } => {
                println!("Subscribed to topic");
            },
            // 退出该主题
            gossipsub::Event::Unsubscribed { .. } => {
                println!("Unsubscribed from topic");
            },
            _ => {}
        }
    }
}

impl ChatBehaviour {
    pub fn new(user: Account) -> Self {
        // 对于内容地址消息，我们可以获取消息并且加上发送时间的哈希值并将其用作 ID。
        let message_id_fn = |_message: &gossipsub::Message| {
            let mut s = DefaultHasher::new();
            // message.data.hash(&mut s);
            MessageIdGenerator::next_id().hash(&mut s);
            gossipsub::MessageId::from(s.finish().to_string())
        };

        let gossipsub_config = ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10)) // 设置为通过不使日志混乱来帮助调试
            .validation_mode(gossipsub::ValidationMode::Strict) // 这设置了消息验证的类型。 默认值为 Strict（强制消息签名）
            .max_transmit_size(1_048_576) // 1MB
            .message_id_fn(message_id_fn) // 内容地址消息。 不会传播相同内容的两条消息。
            .build()
            .expect("Valid config");

        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(user.id_keys),
            gossipsub_config,
        )
        .expect("Correct configuration");

        let mdns = mdns::async_io::Behaviour::new(mdns::Config::default(), user.peer_id).unwrap();

        Self { gossipsub, mdns }
    }
    /// 将消息作为String类型广播到所有已知的对等节点
    pub fn broadcast_message(&mut self, message: &[u8]) -> Result<MessageId, PublishError> {
        let topic = gossipsub::IdentTopic::new("test-net");
        return self.gossipsub.publish(topic, message);
    }
    /// 该方法允许我们将消息直接发送到指定的对等节点，而不是广播到所有对等节点。
    pub fn send_direct_message(
        &mut self,
        peer_id: libp2p::PeerId,
        message: &[u8],
    ) -> Result<(), String> {
        // 看看有没有被列入黑名单
        // let peer_score =self.gossipsub.blacklist_peer(&peer_id);
        // if peer_score {
            self.gossipsub
                .publish(gossipsub::IdentTopic::new("direct-messages"), message)
                .map_err(|e| e.to_string())?;
            Ok(())
        // } else {
        //     Err(format!("Peer {:?} not found in peer list", peer_id))
        // }
    }
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

// #[allow(clippy::large_enum_variant)]
// #[derive(Debug)]
// enum OutEvent {
//     Gossipsub(gossipsub::Event),
//     Mdns(mdns::Event),
// }

// impl From<mdns::Event> for OutEvent {
//     fn from(v: mdns::Event) -> Self {
//         Self::Mdns(v)
//     }
// }

// impl From<gossipsub::Event> for OutEvent {
//     fn from(v: gossipsub::Event) -> Self {
//         Self::Gossipsub(v)
//     }
// }
