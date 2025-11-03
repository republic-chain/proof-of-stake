# Consensus Mechanism

## Overview

Production PoS implements a modern proof-of-stake consensus mechanism based on proven designs from Ethereum 2.0 and other successful PoS networks. The consensus ensures network security through economic incentives and provides fast finality.

## Core Concepts

### Slots and Epochs

**Slots**
- Fixed time intervals (12 seconds by default)
- Each slot can contain at most one block
- Proposer is deterministically selected for each slot

**Epochs**
- Groups of 32 slots (6.4 minutes by default)
- Used for validator set updates and checkpointing
- Finality decisions occur at epoch boundaries

### Validators

**Validator Requirements**
- Minimum stake requirement (32 ETH equivalent by default)
- Valid cryptographic keypair
- Network connectivity and uptime

**Validator Lifecycle**
1. **Registration**: Submit stake and validator metadata
2. **Activation**: Wait for inclusion in validator set
3. **Active**: Participate in block proposal and attestation
4. **Exit**: Voluntary or involuntary removal from set
5. **Withdrawal**: Stake becomes available after delay

### Staking and Delegation

**Self-Staking**
- Validators must stake their own tokens
- Self-stake demonstrates commitment to network

**Delegation**
- Token holders can delegate to validators
- Delegated stake increases validator's voting power
- Delegators share in rewards and slashing risks

## Consensus Algorithm

### Block Proposal

1. **Proposer Selection**
   - Weighted random selection based on stake
   - Deterministic using slot number and randomness
   - Higher stake increases selection probability

2. **Block Assembly**
   - Collect pending transactions from mempool
   - Validate transaction eligibility and ordering
   - Calculate state root after transaction execution
   - Include attestations from previous slots

3. **Block Signing**
   - Proposer signs block with private key
   - Signature proves proposer authorization
   - Block broadcast to network peers

### Attestation Process

**Attestation Data**
- Source checkpoint (previous justified)
- Target checkpoint (current epoch)
- Beacon block root (head of chain)
- Slot number and committee index

**Committee Assignment**
- Validators divided into committees per slot
- Committee size based on total validator count
- Random but deterministic assignment

**Voting Process**
1. Validator reviews proposed block
2. Creates attestation with vote data
3. Signs attestation with private key
4. Broadcasts to network

### Fork Choice Rule

**LMD-GHOST Algorithm**
- Latest Message Driven Greedy Heaviest Observed SubTree
- Follows chain with most validator support
- Provides canonical chain in case of forks

**Implementation Steps**
1. Start from finalized checkpoint
2. At each fork, choose child with highest weight
3. Weight = sum of validator stakes voting for subtree
4. Continue until reaching leaf block

### Finality Mechanism

**Checkpoints**
- First slot of each epoch becomes checkpoint
- Validators vote on checkpoint pairs (source, target)
- Justification requires 2/3 supermajority

**Finalization**
- Justified checkpoint with consecutive justification
- Two consecutive justified checkpoints create finality
- Finalized checkpoints cannot be reverted

## Economic Incentives

### Rewards

**Block Proposal Rewards**
- Fixed reward for proposing valid block
- Additional fees from transactions included
- Proportional to validator's total stake

**Attestation Rewards**
- Reward for timely, correct attestations
- Proportional to stake and attestation accuracy
- Bonus for inclusion in canonical chain

**Commission System**
- Validators charge commission on delegated stake
- Commission rate set during registration
- Delegators receive remaining rewards

### Penalties

**Attestation Penalties**
- Small penalty for missing attestations
- Encourages consistent participation
- Reduces rewards rather than slashing stake

**Inactivity Penalties**
- Larger penalties during network outages
- Penalizes validators who don't participate
- Helps network recover from extended downtime

### Slashing Conditions

**Proposer Slashing**
- Validator proposes two blocks for same slot
- Proves malicious behavior with evidence
- Results in stake slashing and ejection

**Attester Slashing**
- Double voting: two attestations for same target
- Surround voting: attestation surrounds another
- Both provable with cryptographic evidence

**Slashing Penalties**
- Minimum slashing amount plus additional penalties
- Penalty increases with number of slashed validators
- Whistleblower rewards for reporting slashing

## Security Properties

### Byzantine Fault Tolerance
- Tolerates up to 1/3 Byzantine validators
- Safety maintained with honest supermajority
- Liveness requires 2/3 honest and online

### Economic Security
- Cost of attack proportional to stake
- Slashing creates economic disincentive
- Long-term stake commitment reduces mobility

### Finality Guarantees
- Finalized blocks cannot be reverted
- Provides strong consistency guarantees
- Enables confident transaction confirmation

## Implementation Details

### Proposer Selection Algorithm

```rust
fn select_proposer(slot: Slot, validators: &ValidatorSet) -> Address {
    let randomness = generate_slot_randomness(slot);
    let total_stake = validators.total_stake();
    let threshold = random_from_bytes(randomness) % total_stake;

    let mut cumulative = 0;
    for validator in validators.active_validators() {
        cumulative += validator.effective_balance();
        if threshold < cumulative {
            return validator.address;
        }
    }
}
```

### Fork Choice Implementation

```rust
fn get_head(&self) -> Hash {
    let mut current = self.finalized_checkpoint.root;

    loop {
        let children = self.get_children(current);
        if children.is_empty() { break; }

        current = children.into_iter()
            .max_by_key(|child| self.get_weight(*child))
            .unwrap();
    }

    current
}
```

### Attestation Validation

```rust
fn validate_attestation(&self, attestation: &Attestation) -> Result<()> {
    // Check validator is in committee
    let committee = self.get_committee(attestation.slot)?;
    if !committee.contains(&attestation.validator_index) {
        return Err("Validator not in committee");
    }

    // Verify signature
    let message = attestation.signing_root();
    let pubkey = self.get_validator_pubkey(attestation.validator_index)?;
    verify_signature(&pubkey, &message, &attestation.signature)?;

    // Check slashing conditions
    self.check_slashing_conditions(attestation)?;

    Ok(())
}
```

## Performance Characteristics

### Throughput
- Block time: 12 seconds
- Transaction throughput: Limited by gas limit
- Attestation aggregation reduces bandwidth

### Latency
- Block confirmation: 1 slot (12 seconds)
- Weak finality: 2 epochs (~13 minutes)
- Strong finality: 2 consecutive justified epochs

### Scalability
- Validator set size: Up to 1M+ validators
- Committee size: Automatically adjusted
- Signature aggregation reduces overhead

## Network Conditions

### Normal Operation
- 2/3+ validators online and honest
- Regular block production every slot
- Fast finality within 2 epochs

### Adversarial Conditions
- Up to 1/3 Byzantine validators tolerated
- Network partitions handled gracefully
- Inactivity leak ensures eventual recovery

### Recovery Mechanisms
- Weak subjectivity checkpoints
- Long-range attack protection
- Social consensus for extreme scenarios