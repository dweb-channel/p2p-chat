# p2p chat protocal

## 介绍

本文档是对pchat协议的描述。规定了每个帐户就是一个节点，每个节点可以连接到多个设备上。

## IPFS

### 特性
[IPFS](https://ipfs.tech/)是一种点对点网络，用于在分布式文件系统中存储和共享数据。这是对 IPFS 工作原理的非常简洁的
描述：

- 每个对等方都有一个身份密钥对
- 每个对等点使用 peerID（基本上是其身份公钥的散列）来标识自己。
- 每个内容都使用 contentID（基本上是内容的哈希值）来标识。
- 对等点可以使本地存储的内容在网络上可用，宣布给定的 contentID 由其 peerID 提供，因此其他对等点可以连接到它并获取内容。
- 一旦对等点通过 IPFS 获取内容，它也可以成为该内容的提供者，从而提高其可用性。
- 为了使其全部正常工作，IPFS 使用libp2p 网络堆栈 ，它提供了创建 p2p 应用程序所需的一切：DHT、PubSub、Nat Traversal、传输集合等……


## 帐户

### 帐户创建
为了使用 Berty 协议，用户必须创建一个帐户。创建帐户不需要个人数据。
请注意，在整个 pchat Protocol 中，所有的密钥对都是 X25519 用于加密，Ed25519 用于签名。

### 账户创建步骤：

1. 生成帐户 ID 密钥对。此操作不再重复。此密钥对是帐户的身份，因此无法更改。
2. 生成别名密钥对。不再重复创建。
3. 在用于帐户创建的设备上生成设备 ID 密钥对。将在每个新设备上重复此操作。