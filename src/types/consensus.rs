use super::{Hash, Signature, Slot, Epoch, PublicKey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attestation {
    pub slot: Slot,
    pub beacon_block_root: Hash,
    pub source_epoch: Epoch,
    pub source_root: Hash,
    pub target_epoch: Epoch,
    pub target_root: Hash,
    pub validator_index: u64,
    pub signature: Signature,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttestationData {
    pub slot: Slot,
    pub beacon_block_root: Hash,
    pub source: Checkpoint,
    pub target: Checkpoint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Checkpoint {
    pub epoch: Epoch,
    pub root: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposerSlashing {
    pub signed_header_1: SignedBlockHeader,
    pub signed_header_2: SignedBlockHeader,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttesterSlashing {
    pub attestation_1: IndexedAttestation,
    pub attestation_2: IndexedAttestation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedBlockHeader {
    pub header: BlockHeaderCore,
    pub signature: Signature,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockHeaderCore {
    pub slot: Slot,
    pub proposer_index: u64,
    pub parent_root: Hash,
    pub state_root: Hash,
    pub body_root: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexedAttestation {
    pub attesting_indices: Vec<u64>,
    pub data: AttestationData,
    pub signature: Signature,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitteeAssignment {
    pub slot: Slot,
    pub committee_index: u64,
    pub validators: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fork {
    pub previous_version: [u8; 4],
    pub current_version: [u8; 4],
    pub epoch: Epoch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeaconState {
    pub genesis_time: u64,
    pub genesis_validators_root: Hash,
    pub slot: Slot,
    pub fork: Fork,
    pub latest_block_header: BlockHeaderCore,
    pub block_roots: Vec<Hash>, // Historical block roots
    pub state_roots: Vec<Hash>, // Historical state roots
    pub historical_roots: Vec<Hash>,
    pub eth1_data: Eth1Data,
    pub validators: Vec<ValidatorInfo>,
    pub balances: Vec<u64>,
    pub randao_mixes: Vec<Hash>,
    pub slashings: Vec<u64>, // Per-epoch slashing amounts
    pub previous_epoch_attestations: Vec<PendingAttestation>,
    pub current_epoch_attestations: Vec<PendingAttestation>,
    pub justification_bits: [bool; 4],
    pub previous_justified_checkpoint: Checkpoint,
    pub current_justified_checkpoint: Checkpoint,
    pub finalized_checkpoint: Checkpoint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatorInfo {
    pub pubkey: PublicKey,
    pub withdrawal_credentials: Hash,
    pub effective_balance: u64,
    pub slashed: bool,
    pub activation_eligibility_epoch: Epoch,
    pub activation_epoch: Epoch,
    pub exit_epoch: Epoch,
    pub withdrawable_epoch: Epoch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Eth1Data {
    pub deposit_root: Hash,
    pub deposit_count: u64,
    pub block_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingAttestation {
    pub aggregation_bits: Vec<bool>,
    pub data: AttestationData,
    pub inclusion_delay: u64,
    pub proposer_index: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsensusConfig {
    pub slots_per_epoch: u64,
    pub min_genesis_delay: u64,
    pub genesis_delay: u64,
    pub min_validator_withdrawability_delay: Epoch,
    pub shard_committee_period: Epoch,
    pub min_epochs_to_inactivity_penalty: Epoch,
    pub epochs_per_eth1_voting_period: Epoch,
    pub slots_per_historical_root: u64,
    pub min_deposit_amount: u64,
    pub max_effective_balance: u64,
    pub ejection_balance: u64,
    pub effective_balance_increment: u64,
    pub hysteresis_quotient: u64,
    pub hysteresis_downward_multiplier: u64,
    pub hysteresis_upward_multiplier: u64,
    pub proportional_slashing_multiplier: u64,
    pub min_slashing_penalty_quotient: u64,
    pub whistleblower_reward_quotient: u64,
    pub proposer_reward_quotient: u64,
    pub inactivity_penalty_quotient: u64,
    pub min_slashing_penalty_quotient_altair: u64,
    pub proportional_slashing_multiplier_altair: u64,
    pub inactivity_penalty_quotient_altair: u64,
    pub min_slashing_penalty_quotient_bellatrix: u64,
    pub proportional_slashing_multiplier_bellatrix: u64,
    pub inactivity_penalty_quotient_bellatrix: u64,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        ConsensusConfig {
            slots_per_epoch: 32,
            min_genesis_delay: 86400, // 1 day
            genesis_delay: 604800, // 1 week
            min_validator_withdrawability_delay: 256,
            shard_committee_period: 256,
            min_epochs_to_inactivity_penalty: 4,
            epochs_per_eth1_voting_period: 64,
            slots_per_historical_root: 8192,
            min_deposit_amount: 1_000_000_000, // 1 ETH equivalent
            max_effective_balance: 32_000_000_000, // 32 ETH equivalent
            ejection_balance: 16_000_000_000, // 16 ETH equivalent
            effective_balance_increment: 1_000_000_000, // 1 ETH equivalent
            hysteresis_quotient: 4,
            hysteresis_downward_multiplier: 1,
            hysteresis_upward_multiplier: 5,
            proportional_slashing_multiplier: 1,
            min_slashing_penalty_quotient: 128,
            whistleblower_reward_quotient: 512,
            proposer_reward_quotient: 8,
            inactivity_penalty_quotient: 67_108_864,
            min_slashing_penalty_quotient_altair: 64,
            proportional_slashing_multiplier_altair: 2,
            inactivity_penalty_quotient_altair: 50_331_648,
            min_slashing_penalty_quotient_bellatrix: 32,
            proportional_slashing_multiplier_bellatrix: 3,
            inactivity_penalty_quotient_bellatrix: 16_777_216,
        }
    }
}