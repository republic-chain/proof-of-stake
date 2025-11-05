# P2P Networking Implementation

This document describes the comprehensive P2P networking implementation for the Republic Chain proof-of-stake blockchain.

## Overview

The networking module provides a fully-featured P2P networking solution built on top of `libp2p`, designed specifically for running locally or in controlled network environments. It supports peer discovery, message propagation, and secure communication between blockchain nodes.

## Features

- **libp2p-based architecture** - Uses the robust libp2p networking stack
- **Local network support** - Optimized for local testing and development
- **Automatic peer discovery** - mDNS for local network discovery
- **Gossip protocol** - Efficient message propagation using GossipSub
- **Multiple transport protocols** - TCP with Noise encryption and Yamux multiplexing
- **DHT support** - Kademlia DHT for peer routing
- **Message validation** - Built-in message validation and duplicate detection
- **Peer reputation system** - Tracks peer reliability and performance

## Architecture

### Core Components

1. **NetworkService** - Main service that manages the P2P network
2. **NetworkHandle** - Client interface for interacting with the network
3. **P2PBehaviour** - libp2p behaviour combining all protocols
4. **NetworkConfig** - Configuration for network parameters
5. **NetworkEvent** - Events emitted by the network service

### Protocols Used

- **TCP Transport** - Reliable connection-oriented transport
- **Noise Protocol** - Secure encryption and authentication
- **Yamux** - Stream multiplexing over single connections
- **GossipSub** - Publish-subscribe messaging for block/transaction propagation
- **Identify** - Peer information exchange
- **Kademlia DHT** - Distributed hash table for peer discovery
- **mDNS** - Local network peer discovery
- **Ping** - Connection health monitoring

## Usage

### Basic Setup

```rust
use proof_of_stake::network::{NetworkConfig, NetworkService};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a network configuration
    let config = NetworkConfig::local_node(0); // Node 0 on port 9000

    // Create the network service
    let (service, mut handle) = NetworkService::new(config)?;

    // Start the service
    let service_task = tokio::spawn(async move {
        service.run().await
    });

    // Wait for startup
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Use the handle to interact with the network
    let peers = handle.get_peers().await?;
    println!("Connected peers: {}", peers.len());

    Ok(())
}
```

### Local Multi-Node Setup

```rust
// Create multiple nodes for local testing
let node_configs = vec![
    NetworkConfig::local_node(0), // Port 9000
    NetworkConfig::local_node(1), // Port 9001
    NetworkConfig::local_node(2), // Port 9002
];

// Each node will automatically try to connect to the others
for (i, config) in node_configs.into_iter().enumerate() {
    let (service, handle) = NetworkService::new(config)?;

    tokio::spawn(async move {
        service.run().await
    });
}
```

### Broadcasting Messages

```rust
// Broadcast a block
let block = create_block();
handle.broadcast_block(block).await?;

// Broadcast a transaction
let transaction = create_transaction();
handle.broadcast_transaction(transaction).await?;
```

### Handling Network Events

```rust
while let Some(event) = handle.next_event().await {
    match event {
        NetworkEvent::BlockReceived { block, from } => {
            println!("Received block {} from {}", block.header.height, from);
            // Process the block
        }
        NetworkEvent::TransactionReceived { transaction, from } => {
            println!("Received transaction from {}", from);
            // Process the transaction
        }
        NetworkEvent::PeerConnected { peer_id } => {
            println!("New peer connected: {}", peer_id);
        }
        _ => {}
    }
}
```

## Configuration

### NetworkConfig Options

```rust
let mut config = NetworkConfig::default();

// Set specific port
config.port = 8080;

// Add bootstrap peers
config.add_bootstrap_peer("/ip4/192.168.1.100/tcp/9000".parse()?);

// Configure connection limits
config.max_connections = 100;
config.connection_timeout = Duration::from_secs(30);

// Enable/disable mDNS
config.set_mdns(true);

// Add custom topics
config.add_topic("custom-topic".to_string());
```

### Local Network Configuration

```rust
let config = NetworkConfig::local_node(0);

// This automatically:
// - Sets port to 9000 + node_id
// - Adds bootstrap peers for other local nodes (9001, 9002, etc.)
// - Enables mDNS for local discovery
// - Binds to localhost
```

## Router Configuration

Since the implementation is designed for local networks, it works well behind routers with no special configuration needed. For local development:

1. **No port forwarding required** - All communication happens on the local network
2. **mDNS discovery** - Nodes automatically find each other on the local network
3. **Configurable ports** - Use different ports for each node to avoid conflicts

For production deployment across different networks, you would need to:

1. Configure bootstrap peers with public IP addresses
2. Set up port forwarding on routers if needed
3. Disable mDNS and rely on DHT discovery

## Message Types

The network supports three main message types:

1. **Block** - Blockchain blocks
2. **Transaction** - Individual transactions
3. **Ping** - Connectivity testing

All messages are automatically:
- Serialized using JSON
- Validated for structure and timestamps
- Deduplicated to prevent spam
- Propagated using gossip protocol

## Peer Management

The implementation includes a sophisticated peer management system:

### Peer Status Tracking
- Connected/Disconnected/Connecting/Failed/Banned states
- Connection success/failure rates
- Round-trip time measurements

### Reputation System
- Peers start with neutral reputation (50/100)
- Successful connections increase reputation
- Failed connections decrease reputation
- Peers with very low reputation are automatically banned

### Connection Management
- Automatic reconnection to important peers
- Prioritization of reliable, low-latency peers
- Configurable connection limits

## Security

- **Noise Protocol** - All connections are encrypted and authenticated
- **Peer authentication** - Each peer has a unique cryptographic identity
- **Message validation** - All messages are validated before processing
- **Reputation system** - Protects against misbehaving peers

## Testing

Run the networking tests:

```bash
# Run all tests including networking
cargo test

# Run only network tests
cargo test test_network

# Run the networking example
cargo run --example network_example
```

## Troubleshooting

### Common Issues

1. **Port conflicts** - Make sure each node uses a different port
2. **Firewall blocking** - Ensure firewall allows the configured ports
3. **mDNS not working** - Some networks disable mDNS, use explicit peer addresses
4. **Connection timeouts** - Increase `connection_timeout` in config for slow networks

### Debug Logging

Enable debug logging to see networking activity:

```rust
tracing_subscriber::fmt()
    .with_max_level(Level::DEBUG)
    .init();
```

### Network Diagnostics

```rust
// Check peer information
let peers = handle.get_peers().await?;
for peer in peers {
    println!("Peer: {} - Status: {:?} - RTT: {:?}",
        peer.peer_id, peer.status, peer.avg_rtt);
}
```

## Future Enhancements

- WebRTC transport for browser compatibility
- Advanced peer scoring and selection algorithms
- Bandwidth monitoring and throttling
- Network topology optimization
- Support for private/public key node identification
- Integration with consensus mechanism for validator discovery