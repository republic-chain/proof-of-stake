use libp2p::{Multiaddr, PeerId};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Status of a peer connection
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeerStatus {
    /// Peer is connected and active
    Connected,
    /// Peer is disconnected
    Disconnected,
    /// Peer is connecting
    Connecting,
    /// Peer connection failed
    Failed,
    /// Peer is banned (misbehaving)
    Banned,
}

/// Information about a peer
#[derive(Debug, Clone)]
pub struct PeerInfo {
    /// Peer ID
    pub peer_id: PeerId,
    /// Current status
    pub status: PeerStatus,
    /// Known addresses for this peer
    pub addresses: Vec<Multiaddr>,
    /// When the peer was first seen
    pub first_seen: Option<u64>,
    /// When the peer was last seen
    pub last_seen: Option<u64>,
    /// Number of successful connections
    pub connection_count: u32,
    /// Number of failed connection attempts
    pub failed_connections: u32,
    /// Average round-trip time
    pub avg_rtt: Option<Duration>,
    /// Latest round-trip time
    pub latest_rtt: Option<Duration>,
    /// Protocol version
    pub protocol_version: Option<String>,
    /// Agent version
    pub agent_version: Option<String>,
    /// Reputation score (0-100)
    pub reputation: u8,
    /// Last time reputation was updated
    pub last_reputation_update: Option<Instant>,
}

impl PeerInfo {
    /// Create new peer info
    pub fn new(peer_id: PeerId, status: PeerStatus) -> Self {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        Self {
            peer_id,
            status,
            addresses: Vec::new(),
            first_seen: Some(now),
            last_seen: Some(now),
            connection_count: 0,
            failed_connections: 0,
            avg_rtt: None,
            latest_rtt: None,
            protocol_version: None,
            agent_version: None,
            reputation: 50, // Start with neutral reputation
            last_reputation_update: Some(Instant::now()),
        }
    }

    /// Add an address for this peer
    pub fn add_address(&mut self, addr: Multiaddr) {
        if !self.addresses.contains(&addr) {
            self.addresses.push(addr);
        }
    }

    /// Update connection status
    pub fn set_status(&mut self, status: PeerStatus) {
        self.last_seen = Some(chrono::Utc::now().timestamp_millis() as u64);

        match &status {
            PeerStatus::Connected => {
                self.connection_count += 1;
                self.increase_reputation(5);
            }
            PeerStatus::Failed => {
                self.failed_connections += 1;
                self.decrease_reputation(10);
            }
            PeerStatus::Banned => {
                self.reputation = 0;
            }
            _ => {}
        }

        self.status = status;
    }

    /// Update round-trip time
    pub fn update_rtt(&mut self, rtt: Duration) {
        self.latest_rtt = Some(rtt);

        match self.avg_rtt {
            Some(avg) => {
                // Simple exponential moving average
                let alpha = 0.125; // 1/8
                let new_avg = Duration::from_nanos(
                    (avg.as_nanos() as f64 * (1.0 - alpha) + rtt.as_nanos() as f64 * alpha) as u64
                );
                self.avg_rtt = Some(new_avg);
            }
            None => {
                self.avg_rtt = Some(rtt);
            }
        }
    }

    /// Update protocol information
    pub fn update_protocol_info(&mut self, protocol_version: String, agent_version: String) {
        self.protocol_version = Some(protocol_version);
        self.agent_version = Some(agent_version);
    }

    /// Increase reputation (max 100)
    pub fn increase_reputation(&mut self, amount: u8) {
        self.reputation = (self.reputation.saturating_add(amount)).min(100);
        self.last_reputation_update = Some(Instant::now());
    }

    /// Decrease reputation (min 0)
    pub fn decrease_reputation(&mut self, amount: u8) {
        self.reputation = self.reputation.saturating_sub(amount);
        self.last_reputation_update = Some(Instant::now());

        // Ban peer if reputation drops too low
        if self.reputation < 10 {
            self.status = PeerStatus::Banned;
        }
    }

    /// Check if peer is reliable (good reputation and connection history)
    pub fn is_reliable(&self) -> bool {
        self.reputation >= 70 &&
        self.connection_count > 0 &&
        (self.failed_connections as f64 / (self.connection_count + self.failed_connections) as f64) < 0.3
    }

    /// Check if peer should be prioritized for connections
    pub fn is_priority(&self) -> bool {
        self.is_reliable() && self.avg_rtt.map_or(false, |rtt| rtt < Duration::from_millis(100))
    }

    /// Get connection success rate
    pub fn success_rate(&self) -> f64 {
        let total = self.connection_count + self.failed_connections;
        if total == 0 {
            0.0
        } else {
            self.connection_count as f64 / total as f64
        }
    }

    /// Check if peer has been seen recently (within last hour)
    pub fn is_recent(&self) -> bool {
        if let Some(last_seen) = self.last_seen {
            let now = chrono::Utc::now().timestamp_millis() as u64;
            let one_hour = 60 * 60 * 1000; // 1 hour in milliseconds
            now.saturating_sub(last_seen) < one_hour
        } else {
            false
        }
    }

    /// Get a score for this peer (higher is better)
    pub fn score(&self) -> u32 {
        let mut score = self.reputation as u32 * 10;

        // Bonus for successful connections
        score += self.connection_count * 5;

        // Penalty for failed connections
        score = score.saturating_sub(self.failed_connections * 10);

        // Bonus for low latency
        if let Some(rtt) = self.avg_rtt {
            if rtt < Duration::from_millis(50) {
                score += 100;
            } else if rtt < Duration::from_millis(100) {
                score += 50;
            } else if rtt < Duration::from_millis(200) {
                score += 25;
            }
        }

        // Bonus for being recent
        if self.is_recent() {
            score += 50;
        }

        score
    }
}