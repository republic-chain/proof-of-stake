# Production PoS

A production-grade Proof of Stake blockchain implementation written in Rust.

## Features

- **Secure Consensus**: Implementation of modern PoS consensus with fork choice rules
- **Cryptographic Security**: Ed25519 signatures, SHA-256 hashing, and Merkle trees
- **Validator Management**: Comprehensive staking, delegation, and slashing mechanisms
- **Advanced P2P Networking**: Full libp2p implementation with automatic peer discovery
  - TCP transport with Noise encryption and Yamux multiplexing
  - GossipSub for efficient message propagation
  - mDNS for local network discovery
  - Kademlia DHT for peer routing
  - Comprehensive peer management with reputation system
- **Local Network Optimized**: Perfect for local development and testing
- **Configuration**: Flexible configuration system for different network environments
- **Monitoring**: Built-in metrics and observability features

## Quick Start

### Prerequisites

- Rust 1.70 or higher
- SQLite development libraries (optional for storage)
- Network connectivity for P2P communication

### Installation

```bash
git clone https://github.com/republic-chain/proof-of-stake
cd proof-of-stake
cargo build --release
```

### Running a Node

```bash
# Start a single development node
cargo run --bin node

# Start with custom configuration
cargo run --bin node -- --config custom.toml

# Enable validator mode
cargo run --bin node -- --validator
```

### Running Multiple Local Nodes

```bash
# Terminal 1: Start first node (port 9000)
cargo run --example network_example

# Or manually start multiple nodes
# Terminal 1: Node 0 on port 9000
RUST_LOG=info cargo run --bin node -- --port 9000 --node-id 0

# Terminal 2: Node 1 on port 9001
RUST_LOG=info cargo run --bin node -- --port 9001 --node-id 1

# Terminal 3: Node 2 on port 9002
RUST_LOG=info cargo run --bin node -- --port 9002 --node-id 2
```

The nodes will automatically discover each other via mDNS and form a local blockchain network.

### Validator Setup

```bash
# Generate validator keys
cargo run --bin validator generate-keys --output validator_key.json

# Show validator address
cargo run --bin validator show-address --keyfile validator_key.json

# Register as validator
cargo run --bin validator register \
  --keyfile validator_key.json \
  --stake 32000000000 \
  --name "Rohan" \
  --commission 500
```

## Architecture

### Core Components

- **Consensus Engine**: Handles block validation, proposer selection, and finality
- **Fork Choice**: LMD-GHOST implementation for canonical chain selection
- **Validator Set**: Manages active validators, stakes, and performance tracking
- **Cryptography**: Secure key management and signature verification
- **P2P Network**: Full libp2p stack with advanced networking features
  - **Transport Layer**: TCP with Noise encryption and Yamux multiplexing
  - **Discovery**: mDNS for local peers, Kademlia DHT for routing
  - **Messaging**: GossipSub for efficient block/transaction propagation
  - **Peer Management**: Reputation tracking and connection health monitoring

### Key Concepts

- **Slots & Epochs**: Time is divided into slots (12 seconds) and epochs (32 slots)
- **Validators**: Entities that propose blocks and attest to the chain state
- **Staking**: Economic security through validator deposits and delegations
- **Slashing**: Penalties for provable misbehavior to ensure network security

## Configuration

Create a `config.toml` file for custom settings:

```toml
[network]
network_id = "testnet"
port = 9000
max_connections = 50
enable_mdns = true
bootstrap_peers = [
    "/ip4/192.168.1.100/tcp/9000",
    "/ip4/192.168.1.101/tcp/9000"
]

[network.local_network]
enabled = true
base_port = 9000
max_local_nodes = 10
bind_address = "127.0.0.1"

[storage]
data_dir = "./data"
cache_size = 104857600  # 100MB

[validator]
enabled = false
keystore_path = "./validator_key.json"

[api]
enabled = true
listen_address = "127.0.0.1:8080"

[logging]
level = "info"
format = "Pretty"
```

## API

The node exposes a JSON-RPC API for interaction:

- **Chain queries**: Block, transaction, and state information
- **Validator operations**: Registration, staking, and delegation
- **Network status**: Peer count, sync status, and chain head

## Testing

```bash
# Run all tests (including networking)
cargo test

# Run unit tests only
cargo test --lib

# Run integration tests
cargo test --test integration_tests

# Run network-specific tests
cargo test test_network

# Run with verbose output
cargo test -- --nocapture

# Test networking example
cargo run --example network_example
```

## Security Considerations

- Keep validator private keys secure and use hardware security modules for production
- Regularly update the software to receive security patches
- Monitor validator performance to avoid slashing penalties
- **Network Security**:
  - All P2P connections use Noise protocol encryption
  - Peer authentication through cryptographic identities
  - Built-in protection against misbehaving peers via reputation system
  - Message validation and deduplication
- Use firewall rules to protect the P2P and API endpoints appropriately

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Submit a pull request

## License

This project is licensed under the MIT OR Apache-2.0 license.

## Documentation

See the [docs/](./docs/) directory for detailed documentation:

- [Architecture Overview](./docs/architecture.md)
- [Consensus Mechanism](./docs/consensus.md)
- [Validator Guide](./docs/validator.md)
- [API Reference](./docs/api.md)
- [Development Guide](./docs/development.md)
- [**P2P Networking Guide**](./docs/NETWORKING.md) - Comprehensive networking documentation
- [Compile Notes](./docs/COMPILE_NOTES.md)