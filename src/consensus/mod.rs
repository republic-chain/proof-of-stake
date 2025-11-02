pub mod engine;
pub mod fork_choice;
pub mod proposer_selection;
pub mod attestation;
pub mod slashing;

pub use engine::*;
pub use fork_choice::*;
pub use proposer_selection::*;
pub use attestation::*;
pub use slashing::*;

use crate::types::*;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct ConsensusEngine {
    pub config: ConsensusConfig,
    pub fork_choice: ForkChoice,
    pub validator_set: ValidatorSet,
    pub current_epoch: Epoch,
    pub current_slot: Slot,
    pub proposer_selector: ProposerSelector,
}

impl ConsensusEngine {
    pub fn new(config: ConsensusConfig, genesis_validators: Vec<Validator>) -> Result<Self> {
        let mut validator_set = ValidatorSet::new(
            config.min_deposit_amount,
            1000, // max validators
            0,    // genesis epoch
        );

        for validator in genesis_validators {
            validator_set.add_validator(validator).map_err(|e| anyhow::anyhow!(e))?;
        }

        let fork_choice = ForkChoice::new();
        let proposer_selector = ProposerSelector::new(config.clone());

        Ok(ConsensusEngine {
            config,
            fork_choice,
            validator_set,
            current_epoch: 0,
            current_slot: 0,
            proposer_selector,
        })
    }

    pub fn process_block(&mut self, block: &Block) -> Result<()> {
        // Validate block
        self.validate_block(block)?;

        // Update fork choice
        self.fork_choice.add_block(block.clone());

        // Update current slot/epoch
        self.current_slot = block.header.slot;
        self.current_epoch = block.header.epoch;

        // Process validator updates
        self.process_validator_updates(block)?;

        Ok(())
    }

    pub fn validate_block(&self, block: &Block) -> Result<()> {
        // Basic block validation
        if !block.is_valid() {
            return Err(anyhow::anyhow!("Invalid block"));
        }

        // Check proposer
        let expected_proposer = self.get_proposer_for_slot(block.header.slot)?;
        if block.header.proposer != expected_proposer {
            return Err(anyhow::anyhow!("Invalid proposer"));
        }

        // Verify proposer signature
        let validator = self.validator_set.validators
            .get(&block.header.proposer)
            .ok_or_else(|| anyhow::anyhow!("Proposer not found"))?;

        block.verify_signature(&validator.public_key)?;

        // Check slot is valid
        if block.header.slot <= self.current_slot {
            return Err(anyhow::anyhow!("Block slot is not in the future"));
        }

        // Validate epoch
        let expected_epoch = self.slot_to_epoch(block.header.slot);
        if block.header.epoch != expected_epoch {
            return Err(anyhow::anyhow!("Invalid epoch"));
        }

        Ok(())
    }

    pub fn get_proposer_for_slot(&self, slot: Slot) -> Result<Address> {
        self.proposer_selector.select_proposer(slot, &self.validator_set)
    }

    pub fn slot_to_epoch(&self, slot: Slot) -> Epoch {
        slot / self.config.slots_per_epoch
    }

    pub fn epoch_to_slot(&self, epoch: Epoch) -> Slot {
        epoch * self.config.slots_per_epoch
    }

    pub fn is_epoch_boundary(&self, slot: Slot) -> bool {
        slot % self.config.slots_per_epoch == 0
    }

    pub fn get_head(&self) -> Option<Hash> {
        self.fork_choice.get_head()
    }

    pub fn process_attestation(&mut self, attestation: &Attestation) -> Result<()> {
        // Validate attestation
        self.validate_attestation(attestation)?;

        // Add to fork choice
        self.fork_choice.add_attestation(attestation.clone());

        Ok(())
    }

    pub fn validate_attestation(&self, attestation: &Attestation) -> Result<()> {
        // Check if validator exists and is active
        let validator_index = attestation.validator_index;
        if validator_index as usize >= self.validator_set.validators.len() {
            return Err(anyhow::anyhow!("Invalid validator index"));
        }

        // Additional attestation validation logic would go here
        // - Check attestation data
        // - Verify signature
        // - Check slashing conditions

        Ok(())
    }

    pub fn process_validator_updates(&mut self, block: &Block) -> Result<()> {
        for transaction in &block.transactions {
            // Simplified - would deserialize and process staking/registration transactions
            // In production, this would parse transaction.data to determine transaction type
            self.process_generic_transaction(transaction)?;
        }

        Ok(())
    }

    fn process_generic_transaction(&mut self, _transaction: &Transaction) -> Result<()> {
        // Simplified transaction processing
        // In production, this would parse transaction data and route to appropriate handlers
        Ok(())
    }

    pub fn finalize_epoch(&mut self, epoch: Epoch) -> Result<()> {
        // Process epoch finalization
        // - Calculate rewards
        // - Process slashings
        // - Update validator set

        self.calculate_rewards(epoch)?;
        self.process_slashings(epoch)?;
        self.update_validator_set(epoch)?;

        Ok(())
    }

    fn calculate_rewards(&mut self, epoch: Epoch) -> Result<()> {
        // Calculate and distribute rewards for the epoch
        let total_rewards = self.calculate_total_rewards(epoch);
        let total_stake = self.validator_set.total_stake; // Copy the value

        for validator in self.validator_set.validators.values_mut() {
            if validator.is_active() {
                let validator_reward = Self::calculate_validator_reward_static(validator, total_rewards, total_stake);
                validator.stake += validator_reward;
            }
        }

        Ok(())
    }

    fn calculate_total_rewards(&self, _epoch: Epoch) -> u64 {
        // Simplified reward calculation
        // In practice, this would consider attestation performance, block proposals, etc.
        1000000 // 1M units per epoch
    }

    fn calculate_validator_reward_static(validator: &Validator, total_rewards: u64, total_stake: u64) -> u64 {
        // Proportional to stake with performance adjustments
        let base_reward = (validator.total_stake() * total_rewards) / total_stake;

        // Apply performance multiplier
        let uptime_multiplier = validator.uptime_ratio();
        let attestation_multiplier = validator.attestation_ratio();

        (base_reward as f64 * uptime_multiplier * attestation_multiplier) as u64
    }

    fn process_slashings(&mut self, _epoch: Epoch) -> Result<()> {
        // Process any pending slashings
        // This would involve checking for slashable offenses and applying penalties
        Ok(())
    }

    fn update_validator_set(&mut self, epoch: Epoch) -> Result<()> {
        // Update validator set for the new epoch
        // - Activate new validators
        // - Deactivate validators with insufficient stake
        // - Process exits

        let mut validators_to_remove = Vec::new();

        for (address, validator) in &mut self.validator_set.validators {
            // Check if validator should be ejected
            if validator.total_stake() < self.config.ejection_balance {
                validator.status = ValidatorStatus::Exiting;
            }

            // Process exiting validators
            if matches!(validator.status, ValidatorStatus::Exiting) {
                if epoch >= validator.last_active_epoch + self.config.min_validator_withdrawability_delay {
                    validators_to_remove.push(*address);
                }
            }
        }

        // Remove exited validators
        for address in validators_to_remove {
            self.validator_set.remove_validator(&address).map_err(|e| anyhow::anyhow!(e))?;
        }

        self.validator_set.epoch = epoch;
        Ok(())
    }
}