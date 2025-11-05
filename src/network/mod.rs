use futures::prelude::*;
use libp2p::{
    core::upgrade,
    dns,
    gossipsub::{self, MessageId, ValidationMode},
    identify,
    kad::{self, store::MemoryStore},
    mdns,
    noise,
    ping,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp,
    yamux,
    Multiaddr, PeerId, Swarm, Transport,
};
use std::{
    collections::HashMap,
    error::Error,
    time::Duration,
};
use tokio::{
    select,
    sync::{mpsc, oneshot},
};
use tracing::{debug, error, info, warn};

use crate::types::{Block, Transaction};

mod config;
mod events;
mod messages;
mod peer;

pub use config::NetworkConfig;
pub use events::NetworkEvent;
pub use messages::{NetworkMessage, MessageType};
pub use peer::{PeerInfo, PeerStatus};

#[derive(NetworkBehaviour)]
pub struct P2PBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub kademlia: kad::Behaviour<MemoryStore>,
    pub identify: identify::Behaviour,
    pub ping: ping::Behaviour,
}

pub struct NetworkService {
    swarm: Swarm<P2PBehaviour>,
    command_receiver: mpsc::UnboundedReceiver<NetworkCommand>,
    command_sender: mpsc::UnboundedSender<NetworkCommand>,
    event_sender: mpsc::UnboundedSender<NetworkEvent>,
    peers: HashMap<PeerId, PeerInfo>,
    config: NetworkConfig,
    local_peer_id: PeerId,
    topics: HashMap<String, gossipsub::IdentTopic>,
}

#[derive(Debug)]
pub enum NetworkCommand {
    StartListening {
        addr: Multiaddr,
        response: oneshot::Sender<Result<(), Box<dyn Error + Send + Sync>>>,
    },
    Dial {
        addr: Multiaddr,
        response: oneshot::Sender<Result<(), Box<dyn Error + Send + Sync>>>,
    },
    BroadcastBlock {
        block: Block,
        response: oneshot::Sender<Result<(), Box<dyn Error + Send + Sync>>>,
    },
    BroadcastTransaction {
        transaction: Transaction,
        response: oneshot::Sender<Result<(), Box<dyn Error + Send + Sync>>>,
    },
    GetPeers {
        response: oneshot::Sender<Vec<PeerInfo>>,
    },
    Subscribe {
        topic: String,
        response: oneshot::Sender<Result<(), Box<dyn Error + Send + Sync>>>,
    },
}

pub struct NetworkHandle {
    command_sender: mpsc::UnboundedSender<NetworkCommand>,
    event_receiver: mpsc::UnboundedReceiver<NetworkEvent>,
}

impl NetworkService {
    pub fn new(config: NetworkConfig) -> Result<(Self, NetworkHandle), Box<dyn Error + Send + Sync>> {
        // Create identity keypair
        let local_key = libp2p::identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());

        info!("Local peer id: {}", local_peer_id);

        // Set up transport
        let transport = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
            .upgrade(upgrade::Version::V1Lazy)
            .authenticate(noise::Config::new(&local_key)?)
            .multiplex(yamux::Config::default())
            .timeout(Duration::from_secs(20))
            .boxed();

        // Create DNS transport
        let dns_transport = dns::tokio::Transport::system(transport)?;

        // Set up gossipsub
        let message_id_fn = |message: &gossipsub::Message| {
            use std::hash::{Hash, Hasher};
            let mut s = std::collections::hash_map::DefaultHasher::new();
            message.data.hash(&mut s);
            if let Some(source) = &message.source {
                source.to_bytes().hash(&mut s);
            }
            MessageId::from(s.finish().to_string())
        };

        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10))
            .validation_mode(ValidationMode::Strict)
            .message_id_fn(message_id_fn)
            .build()
            .map_err(|e| format!("Invalid gossipsub config: {}", e))?;

        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_config,
        ).map_err(|e| format!("Failed to create gossipsub behaviour: {}", e))?;

        // Set up mDNS for local peer discovery
        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer_id)?;

        // Set up Kademlia DHT
        let store = MemoryStore::new(local_peer_id);
        let kademlia = kad::Behaviour::new(local_peer_id, store);

        // Set up identify protocol
        let identify = identify::Behaviour::new(identify::Config::new(
            "/republic-chain/1.0.0".to_string(),
            local_key.public(),
        ));

        // Set up ping protocol
        let ping = ping::Behaviour::new(ping::Config::new().with_interval(Duration::from_secs(15)));

        let behaviour = P2PBehaviour {
            gossipsub,
            mdns,
            kademlia,
            identify,
            ping,
        };

        let swarm = Swarm::new(dns_transport.boxed(), behaviour, local_peer_id, libp2p::swarm::Config::with_tokio_executor());

        let (command_sender, command_receiver) = mpsc::unbounded_channel();
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        let handle = NetworkHandle {
            command_sender: command_sender.clone(),
            event_receiver,
        };

        let service = NetworkService {
            swarm,
            command_receiver,
            command_sender,
            event_sender,
            peers: HashMap::new(),
            config,
            local_peer_id,
            topics: HashMap::new(),
        };

        Ok((service, handle))
    }

    /// Get the local peer ID
    pub fn local_peer_id(&self) -> PeerId {
        self.local_peer_id
    }

    /// Get a command sender for this service
    pub fn command_sender(&self) -> mpsc::UnboundedSender<NetworkCommand> {
        self.command_sender.clone()
    }

    pub async fn run(mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Subscribe to default topics
        self.subscribe_to_topic("blocks").await?;
        self.subscribe_to_topic("transactions").await?;

        // Start listening on default address
        let listen_addr: Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", self.config.port)
            .parse()
            .map_err(|e| format!("Invalid listen address: {}", e))?;

        self.swarm.listen_on(listen_addr.clone())?;
        info!("Started listening on {}", listen_addr);

        // Connect to bootstrap peers
        for peer_addr in &self.config.bootstrap_peers {
            if let Err(e) = self.swarm.dial(peer_addr.clone()) {
                warn!("Failed to dial bootstrap peer {}: {}", peer_addr, e);
            }
        }

        loop {
            select! {
                event = self.swarm.select_next_some() => {
                    if let Err(e) = self.handle_swarm_event(event).await {
                        error!("Error handling swarm event: {}", e);
                    }
                }
                command = self.command_receiver.recv() => {
                    match command {
                        Some(cmd) => {
                            if let Err(e) = self.handle_command(cmd).await {
                                error!("Error handling command: {}", e);
                            }
                        }
                        None => {
                            info!("Command channel closed, shutting down network service");
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_swarm_event(
        &mut self,
        event: SwarmEvent<P2PBehaviourEvent>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                info!("Listening on {}", address);
                let _ = self.event_sender.send(NetworkEvent::ListeningStarted { address });
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                info!("Connected to peer: {}", peer_id);
                self.peers.insert(peer_id, PeerInfo::new(peer_id, PeerStatus::Connected));
                let _ = self.event_sender.send(NetworkEvent::PeerConnected { peer_id });
            }
            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                info!("Disconnected from peer: {}", peer_id);
                self.peers.remove(&peer_id);
                let _ = self.event_sender.send(NetworkEvent::PeerDisconnected { peer_id });
            }
            SwarmEvent::Behaviour(P2PBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                propagation_source: peer_id,
                message_id: id,
                message,
            })) => {
                debug!("Received gossipsub message from {}: {:?}", peer_id, id);
                self.handle_gossip_message(peer_id, message).await?;
            }
            SwarmEvent::Behaviour(P2PBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                for (peer_id, multiaddr) in list {
                    info!("Discovered peer via mDNS: {} at {}", peer_id, multiaddr);
                    self.swarm.behaviour_mut().kademlia.add_address(&peer_id, multiaddr);
                }
            }
            SwarmEvent::Behaviour(P2PBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                for (peer_id, _multiaddr) in list {
                    debug!("mDNS peer expired: {}", peer_id);
                }
            }
            SwarmEvent::Behaviour(P2PBehaviourEvent::Identify(identify::Event::Received {
                peer_id,
                info,
                ..
            })) => {
                debug!("Received identify info from {}: {:?}", peer_id, info);
                for addr in info.listen_addrs {
                    self.swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                }
            }
            SwarmEvent::Behaviour(P2PBehaviourEvent::Ping(ping::Event {
                peer,
                result: Ok(rtt),
                ..
            })) => {
                debug!("Ping to {} succeeded with RTT: {:?}", peer, rtt);
                if let Some(peer_info) = self.peers.get_mut(&peer) {
                    peer_info.update_rtt(rtt);
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_command(
        &mut self,
        command: NetworkCommand,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        match command {
            NetworkCommand::StartListening { addr, response } => {
                let result = self.swarm.listen_on(addr).map(|_| ()).map_err(|e| e.into());
                let _ = response.send(result);
            }
            NetworkCommand::Dial { addr, response } => {
                let result = self.swarm.dial(addr).map_err(|e| e.into());
                let _ = response.send(result);
            }
            NetworkCommand::BroadcastBlock { block, response } => {
                let result = self.broadcast_block(&block).await;
                let _ = response.send(result);
            }
            NetworkCommand::BroadcastTransaction { transaction, response } => {
                let result = self.broadcast_transaction(&transaction).await;
                let _ = response.send(result);
            }
            NetworkCommand::GetPeers { response } => {
                let peers = self.peers.values().cloned().collect();
                let _ = response.send(peers);
            }
            NetworkCommand::Subscribe { topic, response } => {
                let result = self.subscribe_to_topic(&topic).await;
                let _ = response.send(result);
            }
        }
        Ok(())
    }

    async fn handle_gossip_message(
        &mut self,
        peer_id: PeerId,
        message: gossipsub::Message,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let _topic_str = message.topic.to_string();

        match serde_json::from_slice::<NetworkMessage>(&message.data) {
            Ok(network_msg) => {
                debug!("Received valid network message from {}: {:?}", peer_id, network_msg.msg_type);

                let event = match network_msg.msg_type {
                    MessageType::Block => {
                        if let Ok(block) = serde_json::from_slice::<Block>(&network_msg.data) {
                            NetworkEvent::BlockReceived { block, from: peer_id }
                        } else {
                            warn!("Failed to deserialize block from {}", peer_id);
                            return Ok(());
                        }
                    }
                    MessageType::Transaction => {
                        if let Ok(transaction) = serde_json::from_slice::<Transaction>(&network_msg.data) {
                            NetworkEvent::TransactionReceived { transaction, from: peer_id }
                        } else {
                            warn!("Failed to deserialize transaction from {}", peer_id);
                            return Ok(());
                        }
                    }
                    MessageType::Ping => {
                        NetworkEvent::PingReceived { from: peer_id }
                    }
                };

                let _ = self.event_sender.send(event);
            }
            Err(e) => {
                warn!("Failed to deserialize network message from {}: {}", peer_id, e);
            }
        }

        Ok(())
    }

    async fn subscribe_to_topic(&mut self, topic_name: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let topic = gossipsub::IdentTopic::new(topic_name);
        self.swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
        self.topics.insert(topic_name.to_string(), topic);
        info!("Subscribed to topic: {}", topic_name);
        Ok(())
    }

    async fn broadcast_block(&mut self, block: &Block) -> Result<(), Box<dyn Error + Send + Sync>> {
        let message = NetworkMessage {
            msg_type: MessageType::Block,
            data: serde_json::to_vec(block)?,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
        };

        let serialized = serde_json::to_vec(&message)?;

        if let Some(topic) = self.topics.get("blocks") {
            self.swarm.behaviour_mut().gossipsub.publish(topic.clone(), serialized)?;
            info!("Broadcasted block with height: {}", block.header.height);
        }

        Ok(())
    }

    async fn broadcast_transaction(&mut self, transaction: &Transaction) -> Result<(), Box<dyn Error + Send + Sync>> {
        let message = NetworkMessage {
            msg_type: MessageType::Transaction,
            data: serde_json::to_vec(transaction)?,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
        };

        let serialized = serde_json::to_vec(&message)?;

        if let Some(topic) = self.topics.get("transactions") {
            self.swarm.behaviour_mut().gossipsub.publish(topic.clone(), serialized)?;
            info!("Broadcasted transaction: {:?}", transaction.hash());
        }

        Ok(())
    }
}

impl NetworkHandle {
    pub async fn start_listening(&self, addr: Multiaddr) -> Result<(), Box<dyn Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        self.command_sender.send(NetworkCommand::StartListening {
            addr,
            response: tx,
        })?;
        rx.await?
    }

    pub async fn dial_peer(&self, addr: Multiaddr) -> Result<(), Box<dyn Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        self.command_sender.send(NetworkCommand::Dial {
            addr,
            response: tx,
        })?;
        rx.await?
    }

    pub async fn broadcast_block(&self, block: Block) -> Result<(), Box<dyn Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        self.command_sender.send(NetworkCommand::BroadcastBlock {
            block,
            response: tx,
        })?;
        rx.await?
    }

    pub async fn broadcast_transaction(&self, transaction: Transaction) -> Result<(), Box<dyn Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        self.command_sender.send(NetworkCommand::BroadcastTransaction {
            transaction,
            response: tx,
        })?;
        rx.await?
    }

    pub async fn get_peers(&self) -> Result<Vec<PeerInfo>, Box<dyn Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        self.command_sender.send(NetworkCommand::GetPeers {
            response: tx,
        })?;
        Ok(rx.await?)
    }

    pub async fn subscribe_to_topic(&self, topic: String) -> Result<(), Box<dyn Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        self.command_sender.send(NetworkCommand::Subscribe {
            topic,
            response: tx,
        })?;
        rx.await?
    }

    pub async fn next_event(&mut self) -> Option<NetworkEvent> {
        self.event_receiver.recv().await
    }
}

impl Default for NetworkService {
    fn default() -> Self {
        let config = NetworkConfig::default();
        let (service, _) = Self::new(config).expect("Failed to create default NetworkService");
        service
    }
}