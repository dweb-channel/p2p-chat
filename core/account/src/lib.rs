//! 每个节点可以对应多个账号
//!

use libp2p::{PeerId, identity::{self, Keypair}, Multiaddr};

pub struct Account {
    pub id_keys: Keypair,
    // pub username: String, // todo  应用层逻辑
    pub peer_id: PeerId,
    pub address:Multiaddr,
}

impl Account {
    pub fn new() -> Self {
           // Create a random PeerId
    let id_keys = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(id_keys.public());
    let listen_address = "/ip4/0.0.0.0/tcp/0".parse::<Multiaddr>().unwrap();
        Self {
            id_keys,
            peer_id:local_peer_id,
            address:listen_address
        }
    }
}
