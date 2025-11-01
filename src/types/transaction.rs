use super::{Hash, Signature, Address, Amount, Nonce, PublicKey};
use crate::types::validator::ValidatorMetadata;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    pub from: Address,
    pub to: Address,
    pub amount: Amount,
    pub gas_limit: u64,
    pub gas_price: u64,
    pub nonce: Nonce,
    pub data: Vec<u8>,
    pub timestamp: DateTime<Utc>,
    pub signature: Signature,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    Transfer,
    Stake,
    Unstake,
    Delegate,
    Undelegate,
    ValidatorRegistration,
    ValidatorUpdate,
    Contract,
}

impl Transaction {
    pub fn new(
        from: Address,
        to: Address,
        amount: Amount,
        gas_limit: u64,
        gas_price: u64,
        nonce: Nonce,
        data: Vec<u8>,
    ) -> Self {
        Transaction {
            from,
            to,
            amount,
            gas_limit,
            gas_price,
            nonce,
            data,
            timestamp: Utc::now(),
            signature: Signature([0u8; 64]),
        }
    }

    pub fn hash(&self) -> Hash {
        let mut hasher = Sha256::new();
        let serialized = serde_json::to_vec(self).expect("Failed to serialize transaction");
        hasher.update(serialized);
        hasher.finalize().into()
    }

    pub fn sign(&mut self, private_key: &ed25519_dalek::SigningKey) {
        use ed25519_dalek::Signer;
        let hash = self.hash_for_signature();
        let signature = private_key.sign(&hash);
        self.signature = Signature(signature.to_bytes());
    }

    pub fn verify_signature(&self, public_key: &PublicKey) -> Result<(), ed25519_dalek::SignatureError> {
        use ed25519_dalek::Verifier;
        let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(public_key)?;
        let signature = ed25519_dalek::Signature::from_bytes(&self.signature.0);
        let hash = self.hash_for_signature();
        verifying_key.verify(&hash, &signature)
    }

    pub fn is_valid(&self) -> bool {
        // Basic validation
        if self.amount == 0 && self.data.is_empty() {
            return false; // No-op transaction
        }

        if self.gas_limit == 0 || self.gas_price == 0 {
            return false; // Invalid gas parameters
        }

        // Check if transaction is not too old (24 hours)
        let now = Utc::now();
        let age = now.signed_duration_since(self.timestamp);
        if age.num_hours() > 24 {
            return false;
        }

        true
    }

    pub fn fee(&self) -> u64 {
        self.gas_limit * self.gas_price
    }

    pub fn total_cost(&self) -> u64 {
        self.amount + self.fee()
    }

    fn hash_for_signature(&self) -> Hash {
        let mut hasher = Sha256::new();

        // Hash all fields except signature
        hasher.update(self.from.0);
        hasher.update(self.to.0);
        hasher.update(self.amount.to_le_bytes());
        hasher.update(self.gas_limit.to_le_bytes());
        hasher.update(self.gas_price.to_le_bytes());
        hasher.update(self.nonce.to_le_bytes());
        hasher.update(&self.data);
        hasher.update(self.timestamp.timestamp().to_le_bytes());

        hasher.finalize().into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StakeTransaction {
    pub validator: Address,
    pub amount: Amount,
    pub delegator: Option<Address>, // None for self-stake
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatorRegistrationTransaction {
    pub validator_key: PublicKey,
    pub commission_rate: u16, // Basis points (e.g., 500 = 5%)
    pub minimum_stake: Amount,
    pub metadata: ValidatorMetadata,
}

