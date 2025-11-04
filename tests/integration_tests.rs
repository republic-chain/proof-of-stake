use production_pos::{
    types::*,
    crypto::*,
    consensus::*,
    config::*,
};
// use tempfile::TempDir; // Not needed for basic tests

#[tokio::test]
async fn test_basic_block_creation_and_validation() {
    let config = ConsensusConfig::default();
    let genesis_validators = create_test_validators(3);

    let mut consensus = ConsensusEngine::new(config, genesis_validators).unwrap();

    // Create a test block
    let proposer = consensus.validator_set.get_active_validators()[0].address;
    let block = create_test_block(1, [0u8; 32], proposer);

    // Process the block
    assert!(consensus.process_block(&block).is_ok());
}

#[tokio::test]
async fn test_fork_choice_basic() {
    let mut fork_choice = ForkChoice::new();

    // Add genesis block
    let genesis = create_test_block(0, [0u8; 32], Address([0u8; 32]));
    let genesis_hash = genesis.hash();
    fork_choice.add_block(genesis);

    // Add block on top
    let block1 = create_test_block(1, genesis_hash, Address([1u8; 32]));
    fork_choice.add_block(block1.clone());

    // Should select the longest chain
    let head = fork_choice.get_head().unwrap();
    assert_eq!(head, block1.hash());
}

#[tokio::test]
async fn test_proposer_selection() {
    let config = ConsensusConfig::default();
    let selector = ProposerSelector::new(config);

    let mut validator_set = ValidatorSet::new(1000, 100, 0);
    for validator in create_test_validators(5) {
        validator_set.add_validator(validator).unwrap();
    }

    // Test deterministic proposer selection
    let proposer1 = selector.select_proposer(1, &validator_set).unwrap();
    let proposer2 = selector.select_proposer(1, &validator_set).unwrap();
    assert_eq!(proposer1, proposer2);

    // Different slots should potentially have different proposers
    let proposer_slot_2 = selector.select_proposer(2, &validator_set).unwrap();
    // Note: due to randomness, they might be the same, but selection should work
    assert!(validator_set.validators.contains_key(&proposer_slot_2));
}

#[tokio::test]
async fn test_crypto_operations() {
    // Test key generation
    let keypair = KeyPair::generate();
    assert_eq!(keypair.private_key.len(), 32);
    assert_eq!(keypair.public_key.len(), 32);

    // Test signing and verification
    let message = b"test message";
    let signature = SignatureUtils::sign(&keypair.signing_key(), message);

    assert!(SignatureUtils::verify(&keypair.public_key, message, &signature).is_ok());

    // Test with wrong message
    let wrong_message = b"wrong message";
    assert!(SignatureUtils::verify(&keypair.public_key, wrong_message, &signature).is_err());
}

#[tokio::test]
async fn test_merkle_tree() {
    let data = vec![b"a", b"b", b"c", b"d"];
    let tree = MerkleTree::from_data(&data);

    // Test proof generation and verification
    let proof = tree.get_proof(1).unwrap();
    assert!(proof.verify());
    assert!(proof.verify_with_root(&tree.root));

    // Test with wrong root
    let wrong_root = [1u8; 32];
    assert!(!proof.verify_with_root(&wrong_root));
}

#[tokio::test]
async fn test_transaction_creation_and_validation() {
    let keypair = KeyPair::generate();
    let from = keypair.address;
    let to = Address([1u8; 32]);

    let mut transaction = Transaction::new(
        from,
        to,
        1000,     // amount
        21000,    // gas_limit
        1000000,  // gas_price
        1,        // nonce
        Vec::new(), // data
    );

    transaction.sign(&keypair.signing_key());

    assert!(transaction.is_valid());
    assert!(transaction.verify_signature(&keypair.public_key).is_ok());
}

#[tokio::test]
async fn test_validator_set_operations() {
    let mut validator_set = ValidatorSet::new(1000, 10, 0);

    // Add validators
    for validator in create_test_validators(3) {
        validator_set.add_validator(validator).unwrap();
    }

    assert_eq!(validator_set.validators.len(), 3);
    assert_eq!(validator_set.get_active_validators().len(), 3);

    // Test proposer selection
    let randomness = [42u8; 32];
    let proposer = validator_set.select_proposer(1, &randomness);
    assert!(proposer.is_some());

    let selected_address = proposer.unwrap();
    assert!(validator_set.validators.contains_key(&selected_address));
}

#[tokio::test]
async fn test_block_signing_and_verification() {
    let keypair = KeyPair::generate();
    let proposer = keypair.address;

    let mut block = create_test_block(1, [0u8; 32], proposer);
    block.sign(&keypair.signing_key());

    assert!(block.verify_signature(&keypair.public_key).is_ok());

    // Test with wrong key
    let wrong_keypair = KeyPair::generate();
    assert!(block.verify_signature(&wrong_keypair.public_key).is_err());
}

// Helper functions

fn create_test_validators(count: usize) -> Vec<Validator> {
    let mut validators = Vec::new();

    for i in 0..count {
        let keypair = KeyPair::generate();
        let metadata = ValidatorMetadata {
            name: format!("validator_{}", i),
            website: None,
            description: None,
            contact: None,
        };

        let validator = Validator::new(
            keypair.address,
            keypair.public_key,
            10000, // stake
            500,   // commission rate (5%)
            0,     // registration epoch
            metadata,
        );

        validators.push(validator);
    }

    validators
}

fn create_test_block(height: u64, previous_hash: Hash, proposer: Address) -> Block {
    Block::new(
        height,
        previous_hash,
        [0u8; 32], // state_root
        height,    // slot
        height / 32, // epoch
        proposer,
        Vec::new(), // transactions
        [0u8; 32], // randao_reveal
        1000000,   // gas_limit
    )
}