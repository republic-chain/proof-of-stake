use super::{Address, Amount, PublicKey, Epoch};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Validator {
    pub address: Address,
    pub public_key: PublicKey,
    pub stake: Amount,
    pub delegated_stake: Amount,
    pub commission_rate: u16, // Basis points (e.g., 500 = 5%)
    pub status: ValidatorStatus,
    pub registration_epoch: Epoch,
    pub last_active_epoch: Epoch,
    pub metadata: ValidatorMetadata,
    pub performance: ValidatorPerformance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidatorStatus {
    Active,
    Inactive,
    Jailed,
    Exiting,
    Exited,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatorMetadata {
    pub name: String,
    pub website: Option<String>,
    pub description: Option<String>,
    pub contact: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatorPerformance {
    pub blocks_proposed: u64,
    pub blocks_missed: u64,
    pub attestations_made: u64,
    pub attestations_missed: u64,
    pub slash_count: u64,
    pub last_slash_epoch: Option<Epoch>,
}

impl Validator {
    pub fn new(
        address: Address,
        public_key: PublicKey,
        stake: Amount,
        commission_rate: u16,
        registration_epoch: Epoch,
        metadata: ValidatorMetadata,
    ) -> Self {
        Validator {
            address,
            public_key,
            stake,
            delegated_stake: 0,
            commission_rate,
            status: ValidatorStatus::Active,
            registration_epoch,
            last_active_epoch: registration_epoch,
            metadata,
            performance: ValidatorPerformance::default(),
        }
    }

    pub fn total_stake(&self) -> Amount {
        self.stake + self.delegated_stake
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, ValidatorStatus::Active)
    }

    pub fn is_eligible(&self, min_stake: Amount) -> bool {
        self.is_active() && self.total_stake() >= min_stake
    }

    pub fn update_performance(&mut self, proposed: bool, attested: bool, epoch: Epoch) {
        if proposed {
            self.performance.blocks_proposed += 1;
        } else {
            self.performance.blocks_missed += 1;
        }

        if attested {
            self.performance.attestations_made += 1;
        } else {
            self.performance.attestations_missed += 1;
        }

        self.last_active_epoch = epoch;
    }

    pub fn slash(&mut self, amount: Amount, epoch: Epoch) {
        self.stake = self.stake.saturating_sub(amount);
        self.performance.slash_count += 1;
        self.performance.last_slash_epoch = Some(epoch);

        // Jail validator if slashed amount is significant
        if amount > self.total_stake() / 10 {
            self.status = ValidatorStatus::Jailed;
        }
    }

    pub fn uptime_ratio(&self) -> f64 {
        let total_proposals = self.performance.blocks_proposed + self.performance.blocks_missed;
        if total_proposals == 0 {
            return 1.0;
        }
        self.performance.blocks_proposed as f64 / total_proposals as f64
    }

    pub fn attestation_ratio(&self) -> f64 {
        let total_attestations = self.performance.attestations_made + self.performance.attestations_missed;
        if total_attestations == 0 {
            return 1.0;
        }
        self.performance.attestations_made as f64 / total_attestations as f64
    }
}

impl Default for ValidatorPerformance {
    fn default() -> Self {
        ValidatorPerformance {
            blocks_proposed: 0,
            blocks_missed: 0,
            attestations_made: 0,
            attestations_missed: 0,
            slash_count: 0,
            last_slash_epoch: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatorSet {
    pub validators: HashMap<Address, Validator>,
    pub total_stake: Amount,
    pub min_stake: Amount,
    pub max_validators: usize,
    pub epoch: Epoch,
}

impl ValidatorSet {
    pub fn new(min_stake: Amount, max_validators: usize, epoch: Epoch) -> Self {
        ValidatorSet {
            validators: HashMap::new(),
            total_stake: 0,
            min_stake,
            max_validators,
            epoch,
        }
    }

    pub fn add_validator(&mut self, validator: Validator) -> Result<(), String> {
        if self.validators.len() >= self.max_validators {
            return Err("Maximum number of validators reached".to_string());
        }

        if validator.total_stake() < self.min_stake {
            return Err("Insufficient stake".to_string());
        }

        self.total_stake += validator.total_stake();
        self.validators.insert(validator.address, validator);
        Ok(())
    }

    pub fn remove_validator(&mut self, address: &Address) -> Result<Validator, String> {
        match self.validators.remove(address) {
            Some(validator) => {
                self.total_stake -= validator.total_stake();
                Ok(validator)
            }
            None => Err("Validator not found".to_string()),
        }
    }

    pub fn get_active_validators(&self) -> Vec<&Validator> {
        self.validators
            .values()
            .filter(|v| v.is_eligible(self.min_stake))
            .collect()
    }

    pub fn select_proposer(&self, slot: u64, randomness: &[u8; 32]) -> Option<Address> {
        let active_validators = self.get_active_validators();
        if active_validators.is_empty() {
            return None;
        }

        // Weighted random selection based on stake
        let mut cumulative_weights = Vec::new();
        let mut total_weight = 0u128;

        for validator in &active_validators {
            total_weight += validator.total_stake() as u128;
            cumulative_weights.push(total_weight);
        }

        // Use slot and randomness to generate deterministic random number
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(slot.to_le_bytes());
        hasher.update(randomness);
        let hash = hasher.finalize();

        let random_value = u128::from_le_bytes([
            hash[0], hash[1], hash[2], hash[3], hash[4], hash[5], hash[6], hash[7],
            hash[8], hash[9], hash[10], hash[11], hash[12], hash[13], hash[14], hash[15],
        ]) % total_weight;

        for (i, &cumulative_weight) in cumulative_weights.iter().enumerate() {
            if random_value < cumulative_weight {
                return Some(active_validators[i].address);
            }
        }

        // Fallback (should not happen)
        Some(active_validators[0].address)
    }
}