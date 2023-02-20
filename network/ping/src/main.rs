use libp2p::futures::StreamExt;
use libp2p::swarm:: {keep_alive,SwarmEvent,Swarm};
use libp2p::{identity, ping, Multiaddr, NetworkBehaviour, PeerId};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let new_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(new_key.public());
    let behaviour = PingBehaviour::default();
    println!("new Peer Id is{:?}", local_peer_id);
    // ping
    let transport = libp2p::development_transport(new_key).await?;
    let mut swarm = Swarm::new(transport, behaviour, local_peer_id);
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    // 本地节点向远程节点发送消息
    if let Some(remote_peer) = std::env::args().nth(1) {
        let remote_peer_multiaddr: Multiaddr = remote_peer.parse()?;
        swarm.dial(remote_peer_multiaddr)?;
        println!("Dialed remote peer:{:?}", remote_peer);
    }

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on Local Address {:?}", address)
            }
            SwarmEvent::Behaviour(event) => {
                println!("Event received from peer is {:?}", event);
            }
            _ => {}
        }
    }
}



#[derive(NetworkBehaviour, Default)]
struct PingBehaviour {
    keep_alive: keep_alive::Behaviour,
    ping: ping::Behaviour,
}
