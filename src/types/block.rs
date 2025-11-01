use super::{Hash, Signature, Address, Slot, Epoch, PublicKey};
use crate::types::transaction::Transaction;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockHeader {
    pub height: u64,
    pub previous_hash: Hash,
    pub merkle_root: Hash,
    pub state_root: Hash,
    pub timestamp: DateTime<Utc>,
    pub slot: Slot,
    pub epoch: Epoch,
    pub proposer: Address,
    pub proposer_signature: Signature,
    pub randao_reveal: Hash,
    pub gas_limit: u64,
    pub gas_used: u64,
}

impl Block {
    pub fn new(
        height: u64,
        previous_hash: Hash,
        state_root: Hash,
        slot: Slot,
        epoch: Epoch,
        proposer: Address,
        transactions: Vec<Transaction>,
        randao_reveal: Hash,
        gas_limit: u64,
    ) -> Self {
        let merkle_root = Self::calculate_merkle_root(&transactions);
        let gas_used = transactions.iter().map(|tx| tx.gas_limit).sum();

        let header = BlockHeader {
            height,
            previous_hash,
            merkle_root,
            state_root,
            timestamp: Utc::now(),
            slot,
            epoch,
            proposer,
            proposer_signature: Signature([0u8; 64]), // Will be set during signing
            randao_reveal,
            gas_limit,
            gas_used,
        };

        Block {
            header,
            transactions,
        }
    }

    pub fn hash(&self) -> Hash {
        let mut hasher = Sha256::new();
        let serialized = serde_json::to_vec(&self.header).expect("Failed to serialize block header");
        hasher.update(serialized);
        hasher.finalize().into()
    }

    pub fn sign(&mut self, private_key: &ed25519_dalek::SigningKey) {
        use ed25519_dalek::Signer;
        let hash = self.hash_for_signature();
        let signature = private_key.sign(&hash);
        self.header.proposer_signature = Signature(signature.to_bytes());
    }

    pub fn verify_signature(&self, public_key: &PublicKey) -> Result<(), ed25519_dalek::SignatureError> {
        use ed25519_dalek::Verifier;
        let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(public_key)?;
        let signature = ed25519_dalek::Signature::from_bytes(&self.header.proposer_signature.0);
        let hash = self.hash_for_signature();
        verifying_key.verify(&hash, &signature)
    }

    pub fn is_valid(&self) -> bool {
        // Basic validation checks
        if self.header.gas_used > self.header.gas_limit {
            return false;
        }

        if self.header.merkle_root != Self::calculate_merkle_root(&self.transactions) {
            return false;
        }

        // Validate all transactions
        for transaction in &self.transactions {
            if !transaction.is_valid() {
                return false;
            }
        }

        true
    }

    fn hash_for_signature(&self) -> Hash {
        let mut hasher = Sha256::new();

        // Hash all header fields except the signature
        hasher.update(self.header.height.to_le_bytes());
        hasher.update(self.header.previous_hash);
        hasher.update(self.header.merkle_root);
        hasher.update(self.header.state_root);
        hasher.update(self.header.timestamp.timestamp().to_le_bytes());
        hasher.update(self.header.slot.to_le_bytes());
        hasher.update(self.header.epoch.to_le_bytes());
        hasher.update(self.header.proposer.0);
        hasher.update(self.header.randao_reveal);
        hasher.update(self.header.gas_limit.to_le_bytes());
        hasher.update(self.header.gas_used.to_le_bytes());

        hasher.finalize().into()
    }

    fn calculate_merkle_root(transactions: &[Transaction]) -> Hash {
        if transactions.is_empty() {
            return [0u8; 32];
        }

        let mut hashes: Vec<Hash> = transactions
            .iter()
            .map(|tx| tx.hash())
            .collect();

        while hashes.len() > 1 {
            let mut next_level = Vec::new();

            for chunk in hashes.chunks(2) {
                let mut hasher = Sha256::new();
                hasher.update(chunk[0]);
                if chunk.len() > 1 {
                    hasher.update(chunk[1]);
                } else {
                    hasher.update(chunk[0]); // Duplicate if odd number
                }
                next_level.push(hasher.finalize().into());
            }

            hashes = next_level;
        }

        hashes[0]
    }
}

impl Default for Block {
    fn default() -> Self {
        Block {
            header: BlockHeader {
                height: 0,
                previous_hash: [0u8; 32],
                merkle_root: [0u8; 32],
                state_root: [0u8; 32],
                timestamp: Utc::now(),
                slot: 0,
                epoch: 0,
                proposer: Address([0u8; 32]),
                proposer_signature: Signature([0u8; 64]),
                randao_reveal: [0u8; 32],
                gas_limit: 1_000_000,
                gas_used: 0,
            },
            transactions: Vec::new(),
        }
    }
}