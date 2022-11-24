use libp2p::futures::StreamExt;
use libp2p::swarm::keep_alive;
use libp2p::swarm::{Swarm, SwarmEvent};
use libp2p::{identity, mdns, ping, Multiaddr, NetworkBehaviour, PeerId};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let new_key = identity::Keypair::generate_ed25519();
    let new_peer_id = PeerId::from(new_key.public());

    println!("new Peer Id is{:?}", new_peer_id);
    // ping
    // let behaviour = PingBehaviour::default();
    // Create an MDNS network behaviour.
    let behaviour = mdns::Mdns::new(mdns::MdnsConfig::default())?;

    let transport = libp2p::development_transport(new_key).await?;
    let mut swarm = Swarm::new(transport, behaviour, new_peer_id);
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    // 本地节点向远程节点发送消息
    if let Some(remote_peer) = std::env::args().nth(1) {
        let remote_peer_multiaddr: Multiaddr = remote_peer.parse()?;
        swarm.dial(remote_peer_multiaddr)?;
        println!("Dialed remote peer:{:?}", remote_peer);
    }

    loop {
        match swarm.select_next_some().await {
            // SwarmEvent::NewListenAddr { address, .. } => {
            //     println!("Listening on Local Address {:?}", address)
            // }
            // SwarmEvent::Behaviour(event) => {
            //     println!("Event received from peer is {:?}", event);
            // }
            SwarmEvent::Behaviour(mdns::MdnsEvent::Discovered(peers)) => {
                for (peer, addr) in peers {
                    println!("discovered{} {}", peer, addr);
                }
            }
            SwarmEvent::Behaviour(mdns::MdnsEvent::Expired(expired)) => {
                for(peer,addr) in expired {
                    println!("expired {} {}",peer,addr);
                }
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
