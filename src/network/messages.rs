use serde::{Deserialize, Serialize};

/// Types of messages that can be sent over the network
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageType {
    /// Block message
    Block,
    /// Transaction message
    Transaction,
    /// Ping message for connectivity testing
    Ping,
}

/// Network message wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMessage {
    /// Type of the message
    pub msg_type: MessageType,
    /// Serialized message data
    pub data: Vec<u8>,
    /// Timestamp when the message was created
    pub timestamp: u64,
}

impl NetworkMessage {
    /// Create a new network message
    pub fn new(msg_type: MessageType, data: Vec<u8>) -> Self {
        Self {
            msg_type,
            data,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
        }
    }

    /// Create a block message
    pub fn block(block: &crate::types::Block) -> Result<Self, serde_json::Error> {
        let data = serde_json::to_vec(block)?;
        Ok(Self::new(MessageType::Block, data))
    }

    /// Create a transaction message
    pub fn transaction(transaction: &crate::types::Transaction) -> Result<Self, serde_json::Error> {
        let data = serde_json::to_vec(transaction)?;
        Ok(Self::new(MessageType::Transaction, data))
    }

    /// Create a ping message
    pub fn ping() -> Self {
        Self::new(MessageType::Ping, vec![])
    }

    /// Get the size of the message in bytes
    pub fn size(&self) -> usize {
        self.data.len() + std::mem::size_of::<MessageType>() + std::mem::size_of::<u64>()
    }

    /// Check if the message is recent (within the last 5 minutes)
    pub fn is_recent(&self) -> bool {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        let five_minutes = 5 * 60 * 1000; // 5 minutes in milliseconds
        now.saturating_sub(self.timestamp) < five_minutes
    }

    /// Validate the message
    pub fn validate(&self) -> Result<(), String> {
        // Check if message is too old
        if !self.is_recent() {
            return Err("Message is too old".to_string());
        }

        // Check message size
        const MAX_MESSAGE_SIZE: usize = 10 * 1024 * 1024; // 10MB
        if self.size() > MAX_MESSAGE_SIZE {
            return Err("Message is too large".to_string());
        }

        // Validate based on message type
        match self.msg_type {
            MessageType::Block => {
                if self.data.is_empty() {
                    return Err("Block message cannot be empty".to_string());
                }
                // Try to deserialize to validate structure
                serde_json::from_slice::<crate::types::Block>(&self.data)
                    .map_err(|e| format!("Invalid block data: {}", e))?;
            }
            MessageType::Transaction => {
                if self.data.is_empty() {
                    return Err("Transaction message cannot be empty".to_string());
                }
                // Try to deserialize to validate structure
                serde_json::from_slice::<crate::types::Transaction>(&self.data)
                    .map_err(|e| format!("Invalid transaction data: {}", e))?;
            }
            MessageType::Ping => {
                // Ping messages should be empty
                if !self.data.is_empty() {
                    return Err("Ping message should be empty".to_string());
                }
            }
        }

        Ok(())
    }
}

/// Message validation result
#[derive(Debug, Clone)]
pub enum ValidationResult {
    /// Message is valid
    Valid,
    /// Message is invalid
    Invalid,
}

impl From<Result<(), String>> for ValidationResult {
    fn from(result: Result<(), String>) -> Self {
        match result {
            Ok(()) => ValidationResult::Valid,
            Err(_) => ValidationResult::Invalid,
        }
    }
}