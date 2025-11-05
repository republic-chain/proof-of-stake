use libp2p::Multiaddr;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Port to listen on
    pub port: u16,

    /// Maximum number of connections
    pub max_connections: u32,

    /// Connection timeout
    pub connection_timeout: Duration,

    /// Heartbeat interval for gossipsub
    pub heartbeat_interval: Duration,

    /// Bootstrap peers to connect to on startup
    pub bootstrap_peers: Vec<Multiaddr>,

    /// Enable mDNS for local peer discovery
    pub enable_mdns: bool,

    /// Topics to subscribe to
    pub default_topics: Vec<String>,

    /// Maximum message size
    pub max_message_size: usize,

    /// Local network configuration for testing
    pub local_network: LocalNetworkConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalNetworkConfig {
    /// Enable local network mode (useful for testing)
    pub enabled: bool,

    /// Base port for local nodes (each node gets port + node_id)
    pub base_port: u16,

    /// Number of local nodes to support
    pub max_local_nodes: u8,

    /// Local IP address to bind to
    pub bind_address: String,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            port: 0, // Let the OS choose a port
            max_connections: 50,
            connection_timeout: Duration::from_secs(30),
            heartbeat_interval: Duration::from_secs(10),
            bootstrap_peers: vec![],
            enable_mdns: true,
            default_topics: vec![
                "blocks".to_string(),
                "transactions".to_string(),
                "consensus".to_string(),
            ],
            max_message_size: 1024 * 1024, // 1MB
            local_network: LocalNetworkConfig::default(),
        }
    }
}

impl Default for LocalNetworkConfig {
    fn default() -> Self {
        Self {
            enabled: true, // Enable by default for local development
            base_port: 9000,
            max_local_nodes: 10,
            bind_address: "127.0.0.1".to_string(),
        }
    }
}

impl NetworkConfig {
    /// Create a config for local testing with specific node ID
    pub fn local_node(node_id: u8) -> Self {
        let mut config = Self::default();
        config.local_network.enabled = true;
        config.port = config.local_network.base_port + node_id as u16;

        // Add bootstrap peers (other local nodes)
        for i in 0..config.local_network.max_local_nodes {
            if i != node_id {
                let peer_port = config.local_network.base_port + i as u16;
                let addr = format!("/ip4/{}/tcp/{}", config.local_network.bind_address, peer_port);
                if let Ok(multiaddr) = addr.parse() {
                    config.bootstrap_peers.push(multiaddr);
                }
            }
        }

        config
    }

    /// Create config for a specific port
    pub fn with_port(port: u16) -> Self {
        let mut config = Self::default();
        config.port = port;
        config
    }

    /// Add a bootstrap peer
    pub fn add_bootstrap_peer(&mut self, addr: Multiaddr) {
        self.bootstrap_peers.push(addr);
    }

    /// Enable or disable mDNS
    pub fn set_mdns(&mut self, enabled: bool) {
        self.enable_mdns = enabled;
    }

    /// Add a topic to subscribe to
    pub fn add_topic(&mut self, topic: String) {
        if !self.default_topics.contains(&topic) {
            self.default_topics.push(topic);
        }
    }
}