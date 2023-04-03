use libp2p::{gossipsub, mdns};
use libp2p_swarm_derive::NetworkBehaviour;

#[derive(NetworkBehaviour)]
pub struct ChatBehaviour {
    pub gossipsub: gossipsub::Behaviour, // gossipsub 行为用于点对点广播消息， https://github.com/libp2p/specs/tree/master/pubsub/gossipsub
    pub mdns: mdns::async_io::Behaviour, // Mdns 行为用于发现局域网中的其他节点。
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
