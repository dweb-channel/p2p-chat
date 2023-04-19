//! 在轮询 Swarm 期间，会处理各种事件，
//! 包括新的监听地址、连接建立、连接关闭、传入连接、传入连接错误、不可达报告、未知对等点不可达、Mdns 和 Gossipsub 事件等

use std::collections::HashSet;
use libp2p::gossipsub::{Topic};

struct SwarmEvent {
  // listening_gossip_topics: HashSet<Topic<>>
}

impl SwarmEvent {
  /// 发现新的监听地址
fn new_listen_addr() {
    
}

/// 连接建立
fn connection_established() {}

/// 连接关闭   
fn connection_closed() {}

/// 不可达报告
fn unreachability_report() {}

}
