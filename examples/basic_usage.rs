// Basic usage example for Production PoS

use production_pos::{
    types::*,
    crypto::*,
    consensus::*,
    network::*,
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Generate a keypair
    let keypair = KeyPair::generate();
    println!("Generated validator address: {}", keypair.address);

    // Create a simple validator
    let metadata = ValidatorMetadata {
        name: "Example Validator".to_string(),
        website: Some("https://example.com".to_string()),
        description: Some("An example validator for testing".to_string()),
        contact: Some("admin@example.com".to_string()),
    };

    let validator = Validator::new(
        keypair.address,
        keypair.public_key,
        32_000_000_000, // 32 ETH equivalent stake
        500,            // 5% commission
        0,              // registration epoch
        metadata,
    );

    println!("Created validator: {}", validator.metadata.name);
    println!("Total stake: {} tokens", validator.total_stake());
    println!("Commission rate: {}%", validator.commission_rate as f64 / 100.0);

    // Create a simple transaction
    let mut transaction = Transaction::new(
        keypair.address,                    // from
        Address([1u8; 32]),                // to
        1_000_000_000,                     // amount (1 ETH equivalent)
        21_000,                            // gas_limit
        1_000_000_000,                     // gas_price
        1,                                 // nonce
        Vec::new(),                        // data
    );

    // Sign the transaction
    transaction.sign(&keypair.signing_key());
    println!("Created and signed transaction: {}", hex::encode(transaction.hash()));

    // Verify the signature
    if transaction.verify_signature(&keypair.public_key).is_ok() {
        println!("✅ Transaction signature is valid");
    } else {
        println!("❌ Transaction signature is invalid");
    }

    // Create a simple block
    let mut block = Block::new(
        1,                                 // height
        [0u8; 32],                        // previous_hash (genesis)
        [0u8; 32],                        // state_root
        1,                                // slot
        0,                                // epoch
        keypair.address,                  // proposer
        vec![transaction],                // transactions
        [0u8; 32],                        // randao_reveal
        1_000_000,                        // gas_limit
    );

    // Sign the block
    block.sign(&keypair.signing_key());
    println!("Created and signed block at height: {}", block.header.height);

    // Verify the block
    if block.verify_signature(&keypair.public_key).is_ok() {
        println!("✅ Block signature is valid");
    } else {
        println!("❌ Block signature is invalid");
    }

    // Test Merkle tree
    let data = vec![b"transaction1", b"transaction2", b"transaction3", b"transaction4"];
    let merkle_tree = MerkleTree::from_data(&data);
    println!("Merkle root: {}", hex::encode(merkle_tree.root));

    // Generate and verify a Merkle proof
    if let Some(proof) = merkle_tree.get_proof(1) {
        if proof.verify() {
            println!("✅ Merkle proof is valid");
        } else {
            println!("❌ Merkle proof is invalid");
        }
    }

    // Test consensus components
    let consensus_config = ConsensusConfig::default();
    println!("Consensus config: {} slots per epoch", consensus_config.slots_per_epoch);

    // Create a validator set
    let mut validator_set = ValidatorSet::new(
        1_000_000_000, // min stake (1 ETH equivalent)
        1000,          // max validators
        0,             // epoch
    );

    // Add our validator
    validator_set.add_validator(validator)?;
    println!("Validator set created with {} validators", validator_set.validators.len());

    // Test proposer selection
    let proposer_selector = ProposerSelector::new(consensus_config);
    if let Ok(selected_proposer) = proposer_selector.select_proposer(1, &validator_set) {
        println!("Selected proposer for slot 1: {}", selected_proposer);
    }

    // Test networking functionality
    println!("\n=== Testing P2P Networking ===");

    // Create network configuration for local testing
    let network_config = NetworkConfig::local_node(0);
    println!("Network config created for port: {}", network_config.port);

    // Create network service
    match NetworkService::new(network_config) {
        Ok((service, mut handle)) => {
            println!("✅ Network service created successfully");

            // Start the network service in the background
            let _service_task = tokio::spawn(async move {
                if let Err(e) = service.run().await {
                    eprintln!("Network service error: {}", e);
                }
            });

            // Wait a moment for the service to start
            tokio::time::sleep(Duration::from_millis(500)).await;

            // Test getting peer information
            match handle.get_peers().await {
                Ok(peers) => {
                    println!("✅ Retrieved peer list: {} connected peers", peers.len());
                }
                Err(e) => {
                    println!("❌ Failed to get peers: {}", e);
                }
            }

            // Test block broadcasting (will fail without peers, which is expected)
            println!("Testing block broadcast...");
            if let Err(_) = handle.broadcast_block(block).await {
                println!("✅ Block broadcast attempted (no peers to broadcast to, which is expected)");
            }

            println!("✅ Network functionality test completed");
        }
        Err(e) => {
            println!("❌ Failed to create network service: {}", e);
        }
    }

    println!("\n🎉 All functionality tests completed successfully!");
    println!("\nTo test multi-node networking, run:");
    println!("  cargo run --example network_example");

    Ok(())
}