use crate::types::*;
use crate::crypto::Hasher;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct ProposerSelector {
    _config: ConsensusConfig,
}

impl ProposerSelector {
    pub fn new(config: ConsensusConfig) -> Self {
        ProposerSelector { _config: config }
    }

    pub fn select_proposer(&self, slot: Slot, validator_set: &ValidatorSet) -> Result<Address> {
        let active_validators = validator_set.get_active_validators();
        if active_validators.is_empty() {
            return Err(anyhow::anyhow!("No active validators"));
        }

        // Generate deterministic randomness based on slot
        let randomness = self.get_slot_randomness(slot);

        // Weighted random selection based on stake
        let total_stake: u128 = active_validators.iter().map(|v| v.total_stake() as u128).sum();
        let random_threshold = self.bytes_to_u128(&randomness) % total_stake;

        let mut cumulative_stake = 0u128;
        for validator in &active_validators {
            cumulative_stake += validator.total_stake() as u128;
            if random_threshold < cumulative_stake {
                return Ok(validator.address);
            }
        }

        // Fallback (should not happen with proper implementation)
        Ok(active_validators[0].address)
    }

    fn get_slot_randomness(&self, slot: Slot) -> Hash {
        // In a real implementation, this would use RANDAO or similar
        // For now, use slot number as seed
        Hasher::hash(&slot.to_le_bytes())
    }

    fn bytes_to_u128(&self, bytes: &Hash) -> u128 {
        u128::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
            bytes[8], bytes[9], bytes[10], bytes[11],
            bytes[12], bytes[13], bytes[14], bytes[15],
        ])
    }

    pub fn get_committee(&self, slot: Slot, committee_index: u64, validator_set: &ValidatorSet) -> Vec<u64> {
        let active_validators = validator_set.get_active_validators();
        if active_validators.is_empty() {
            return Vec::new();
        }

        let committee_size = active_validators.len().min(128); // Max committee size
        let seed = self.get_committee_seed(slot, committee_index);

        // Shuffle validators deterministically
        let mut indices: Vec<u64> = (0..active_validators.len() as u64).collect();
        self.shuffle(&mut indices, &seed);

        indices.into_iter().take(committee_size).collect()
    }

    fn get_committee_seed(&self, slot: Slot, committee_index: u64) -> Hash {
        let mut data = Vec::new();
        data.extend_from_slice(&slot.to_le_bytes());
        data.extend_from_slice(&committee_index.to_le_bytes());
        Hasher::hash(&data)
    }

    fn shuffle(&self, list: &mut [u64], seed: &Hash) {
        // Fisher-Yates shuffle with deterministic randomness
        for i in (1..list.len()).rev() {
            let mut hash_input = seed.to_vec();
            hash_input.extend_from_slice(&(i as u64).to_le_bytes());
            let hash = Hasher::hash(&hash_input);
            let j = self.bytes_to_u128(&hash) as usize % (i + 1);
            list.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Validator, ValidatorMetadata, ValidatorPerformance, ValidatorStatus};

    fn create_test_validator(address: Address, stake: u64) -> Validator {
        Validator {
            address,
            public_key: [0u8; 32],
            stake,
            delegated_stake: 0,
            commission_rate: 500,
            status: ValidatorStatus::Active,
            registration_epoch: 0,
            last_active_epoch: 0,
            metadata: ValidatorMetadata {
                name: "test".to_string(),
                website: None,
                description: None,
                contact: None,
            },
            performance: ValidatorPerformance::default(),
        }
    }

    #[test]
    fn test_proposer_selection() {
        let config = ConsensusConfig::default();
        let selector = ProposerSelector::new(config);

        let mut validator_set = ValidatorSet::new(1000, 100, 0);

        // Add validators with different stakes
        let addr1 = Address([1u8; 32]);
        let addr2 = Address([2u8; 32]);
        let validator1 = create_test_validator(addr1, 5000);
        let validator2 = create_test_validator(addr2, 10000);

        validator_set.add_validator(validator1).unwrap();
        validator_set.add_validator(validator2).unwrap();

        // Test proposer selection
        let proposer = selector.select_proposer(1, &validator_set).unwrap();
        assert!(proposer == addr1 || proposer == addr2);
    }

    #[test]
    fn test_committee_generation() {
        let config = ConsensusConfig::default();
        let selector = ProposerSelector::new(config);

        let mut validator_set = ValidatorSet::new(1000, 100, 0);

        // Add multiple validators
        for i in 0..10 {
            let mut addr = [0u8; 32];
            addr[0] = i as u8;
            let validator = create_test_validator(Address(addr), 1000);
            validator_set.add_validator(validator).unwrap();
        }

        let committee = selector.get_committee(1, 0, &validator_set);
        assert!(!committee.is_empty());
        assert!(committee.len() <= 10);
    }

    #[test]
    fn test_deterministic_selection() {
        let config = ConsensusConfig::default();
        let selector = ProposerSelector::new(config);

        let mut validator_set = ValidatorSet::new(1000, 100, 0);
        let validator = create_test_validator(Address([1u8; 32]), 5000);
        validator_set.add_validator(validator).unwrap();

        // Same slot should give same proposer
        let proposer1 = selector.select_proposer(100, &validator_set).unwrap();
        let proposer2 = selector.select_proposer(100, &validator_set).unwrap();
        assert_eq!(proposer1, proposer2);
    }
}