use libp2p::{Multiaddr, PeerId};
use crate::types::{Block, Transaction};

/// Network events that can be emitted by the network service
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    /// Network service started listening on an address
    ListeningStarted {
        address: Multiaddr,
    },

    /// A peer has connected
    PeerConnected {
        peer_id: PeerId,
    },

    /// A peer has disconnected
    PeerDisconnected {
        peer_id: PeerId,
    },

    /// A new block was received from a peer
    BlockReceived {
        block: Block,
        from: PeerId,
    },

    /// A new transaction was received from a peer
    TransactionReceived {
        transaction: Transaction,
        from: PeerId,
    },

    /// A ping was received from a peer
    PingReceived {
        from: PeerId,
    },

    /// Failed to connect to a peer
    ConnectionFailed {
        peer_id: Option<PeerId>,
        error: String,
    },

    /// A peer was discovered via mDNS
    PeerDiscovered {
        peer_id: PeerId,
        addresses: Vec<Multiaddr>,
    },

    /// Network error occurred
    NetworkError {
        error: String,
    },

    /// Gossipsub message validation failed
    MessageValidationFailed {
        from: PeerId,
        reason: String,
    },

    /// Successfully subscribed to a topic
    TopicSubscribed {
        topic: String,
    },

    /// Successfully unsubscribed from a topic
    TopicUnsubscribed {
        topic: String,
    },
}

impl NetworkEvent {
    /// Check if this is a critical event that should be handled immediately
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            NetworkEvent::NetworkError { .. } | NetworkEvent::ConnectionFailed { .. }
        )
    }

    /// Get the peer ID associated with this event, if any
    pub fn peer_id(&self) -> Option<PeerId> {
        match self {
            NetworkEvent::PeerConnected { peer_id } => Some(*peer_id),
            NetworkEvent::PeerDisconnected { peer_id } => Some(*peer_id),
            NetworkEvent::BlockReceived { from, .. } => Some(*from),
            NetworkEvent::TransactionReceived { from, .. } => Some(*from),
            NetworkEvent::PingReceived { from } => Some(*from),
            NetworkEvent::ConnectionFailed { peer_id, .. } => *peer_id,
            NetworkEvent::PeerDiscovered { peer_id, .. } => Some(*peer_id),
            NetworkEvent::MessageValidationFailed { from, .. } => Some(*from),
            _ => None,
        }
    }

    /// Get a human-readable description of the event
    pub fn description(&self) -> String {
        match self {
            NetworkEvent::ListeningStarted { address } => {
                format!("Started listening on {}", address)
            }
            NetworkEvent::PeerConnected { peer_id } => {
                format!("Connected to peer {}", peer_id)
            }
            NetworkEvent::PeerDisconnected { peer_id } => {
                format!("Disconnected from peer {}", peer_id)
            }
            NetworkEvent::BlockReceived { block, from } => {
                format!("Received block #{} from {}", block.header.height, from)
            }
            NetworkEvent::TransactionReceived { transaction, from } => {
                format!("Received transaction {:?} from {}", transaction.hash(), from)
            }
            NetworkEvent::PingReceived { from } => {
                format!("Received ping from {}", from)
            }
            NetworkEvent::ConnectionFailed { peer_id, error } => {
                if let Some(peer_id) = peer_id {
                    format!("Failed to connect to {}: {}", peer_id, error)
                } else {
                    format!("Connection failed: {}", error)
                }
            }
            NetworkEvent::PeerDiscovered { peer_id, addresses } => {
                format!("Discovered peer {} at {:?}", peer_id, addresses)
            }
            NetworkEvent::NetworkError { error } => {
                format!("Network error: {}", error)
            }
            NetworkEvent::MessageValidationFailed { from, reason } => {
                format!("Message validation failed from {}: {}", from, reason)
            }
            NetworkEvent::TopicSubscribed { topic } => {
                format!("Subscribed to topic: {}", topic)
            }
            NetworkEvent::TopicUnsubscribed { topic } => {
                format!("Unsubscribed from topic: {}", topic)
            }
        }
    }
}