//! 在轮询 Swarm 期间，会处理各种事件，
//! 包括新的监听地址、连接建立、连接关闭、传入连接、传入连接错误、不可达报告、未知对等点不可达、Mdns 和 Gossipsub 事件等

// use std::collections::HashSet;
// use libp2p::gossipsub::{Topic};

// struct SwarmEvent {
//   // listening_gossip_topics: HashSet<Topic<>>
// }

// impl SwarmEvent {
//   /// 发现新的监听地址
// fn new_listen_addr() {
    
// }

// /// 连接建立
// fn connection_established() {}

// /// 连接关闭   
// fn connection_closed() {}

// /// 不可达报告
// fn unreachability_report() {}

// }

// loop {
//   futures::select! {
//       swarm_event = swarm_fuse.next() => {
//           if let Some(event) = swarm_event {
//               match event {
//                   libp2p::SwarmEvent::Behaviour(GossipsubEvent::Message { id, message, .. }) => {
//                       println!("Received message from {:?} with id {:?}: {:?}", message.source, id, String::from_utf8_lossy(&message.data));
//                   }
//                   _ => {}
//               }
//           }
//       }
//       line = stdin.next_line() => {
//           let line = line.expect("stdin closed").expect("stdin error");
//           let mut parts = line.split_whitespace();

//           match parts.next() {
//               Some("send") => {
//                   // 发送点对点消息
//                   if let (Some(peer), Some(msg)) = (parts.next(), parts.next()) {
//                       let peer_id = match peer.parse() {
//                           Ok(peer_id) => peer_id,
//                           Err(err) => {
//                               eprintln!("Invalid peer id: {:?}", err);
//                               continue;
//                           }
//                       };
//                       let msg_bytes = msg.as_bytes();
//                       swarm.send_message(&peer_id, Arc::from(msg_bytes), &topic);
//                   }
//               }
//               Some("broadcast") => {
//                   // 发送群发消息
//                   if let Some(msg) = parts.next() {
//                       let msg_bytes = msg.as_bytes();
//                       swarm.publish(&topic, Arc::from(msg_bytes));
//                   }
//               }
//               Some("connect") => {
//                   // 连接到其他节点
//                   if let Some(addr) = parts.next() {
//                       let addr = match addr.parse() {
//                           Ok(addr) => addr,
//                           Err(err) => {
//                               eprintln!("Invalid multiaddr: {:?}", err);
//                               continue;
//                           }
//                       };
//                       swarm.dial_addr(addr).expect("Failed to dial address");
//                   }
//               }
//               Some("help") | Some(_) | None => {
//                   eprintln!(
//                       "Available commands:\n\
//                        send PEER_ID MESSAGE\n\
//                        broadcast MESSAGE\n\
//                        connect MULTIADDR\n\
//                        help\n\
//                        exit"
//                   );
//               }
//               Some("exit") => break,
//           }
//       }
//  }
// }