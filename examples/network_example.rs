use proof_of_stake::network::{NetworkConfig, NetworkService};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting network example");

    // Create three nodes for local testing
    let node_configs = vec![
        NetworkConfig::local_node(0), // Port 9000
        NetworkConfig::local_node(1), // Port 9001
        NetworkConfig::local_node(2), // Port 9002
    ];

    let mut handles = Vec::new();

    // Start the first node
    let (service1, mut handle1) = NetworkService::new(node_configs[0].clone()).unwrap();
    let handle1_clone = tokio::spawn(async move {
        if let Err(e) = service1.run().await {
            eprintln!("Node 1 error: {}", e);
        }
    });
    handles.push(handle1_clone);

    // Wait a bit for the first node to start
    sleep(Duration::from_secs(2)).await;

    // Start the second node
    let (service2, mut handle2) = NetworkService::new(node_configs[1].clone()).unwrap();
    let handle2_clone = tokio::spawn(async move {
        if let Err(e) = service2.run().await {
            eprintln!("Node 2 error: {}", e);
        }
    });
    handles.push(handle2_clone);

    // Wait a bit for the second node to start
    sleep(Duration::from_secs(2)).await;

    // Start the third node
    let (service3, mut handle3) = NetworkService::new(node_configs[2].clone()).unwrap();
    let handle3_clone = tokio::spawn(async move {
        if let Err(e) = service3.run().await {
            eprintln!("Node 3 error: {}", e);
        }
    });
    handles.push(handle3_clone);

    // Wait for nodes to discover each other
    sleep(Duration::from_secs(5)).await;

    // Check peer counts
    info!("Node 1 peers: {:?}", handle1.get_peers().await.unwrap_or_default());
    info!("Node 2 peers: {:?}", handle2.get_peers().await.unwrap_or_default());
    info!("Node 3 peers: {:?}", handle3.get_peers().await.unwrap_or_default());

    // Create and broadcast a test block from node 1
    let test_block = proof_of_stake::types::Block::new(
        1,                    // height
        [0u8; 32],           // previous_hash
        [1u8; 32],           // state_root
        1,                   // slot
        0,                   // epoch
        proof_of_stake::types::Address([0u8; 32]), // proposer
        Vec::new(),          // transactions
        [0u8; 32],           // randao_reveal
        1000000,             // gas_limit
    );

    info!("Broadcasting test block from node 1");
    if let Err(e) = handle1.broadcast_block(test_block).await {
        eprintln!("Failed to broadcast block: {}", e);
    }

    // Listen for events on other nodes
    let event_listener = tokio::spawn(async move {
        for i in 0..10 {
            if let Some(event) = handle2.next_event().await {
                info!("Node 2 received event {}: {}", i, event.description());
                if matches!(event, proof_of_stake::network::NetworkEvent::BlockReceived { .. }) {
                    info!("Successfully received block broadcast!");
                    break;
                }
            }
        }
    });

    let event_listener2 = tokio::spawn(async move {
        for i in 0..10 {
            if let Some(event) = handle3.next_event().await {
                info!("Node 3 received event {}: {}", i, event.description());
                if matches!(event, proof_of_stake::network::NetworkEvent::BlockReceived { .. }) {
                    info!("Successfully received block broadcast!");
                    break;
                }
            }
        }
    });

    // Wait for events
    let _ = tokio::join!(event_listener, event_listener2);

    // Let the network run for a bit
    sleep(Duration::from_secs(10)).await;

    info!("Network example completed successfully!");

    // Clean shutdown
    for handle in handles {
        handle.abort();
    }

    Ok(())
}