# Production PoS

A production-grade Proof of Stake blockchain implementation written in Rust.

## Features

- **Secure Consensus**: Implementation of modern PoS consensus with fork choice rules
- **Cryptographic Security**: Ed25519 signatures, SHA-256 hashing, and Merkle trees
- **Validator Management**: Comprehensive staking, delegation, and slashing mechanisms
- **Network Layer**: P2P networking with libp2p for robust peer communication
- **Storage**: Persistent blockchain state with SQLite backend
- **Configuration**: Flexible configuration system for different network environments
- **Monitoring**: Built-in metrics and observability features

## Quick Start

### Prerequisites

- Rust 1.70 or higher
- SQLite development libraries

### Installation

```bash
git clone https://github.com/republic-chain/proof-of-stake
cd proof-of-stake
cargo build --release
```

### Running a Node

```bash
# Start a development node
cargo run --bin node

# Start with custom configuration
cargo run --bin node -- --config custom.toml

# Enable validator mode
cargo run --bin node -- --validator
```

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
- **P2P Network**: Gossip protocol for block and attestation propagation

### Key Concepts

- **Slots & Epochs**: Time is divided into slots (12 seconds) and epochs (32 slots)
- **Validators**: Entities that propose blocks and attest to the chain state
- **Staking**: Economic security through validator deposits and delegations
- **Slashing**: Penalties for provable misbehavior to ensure network security

## Configuration

Create a `config.toml` file for custom settings:

```toml
[network]
network_id = "Devnet"
listen_address = "/ip4/0.0.0.0/tcp/9000"
max_peers = 50

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
# Run unit tests
cargo test

# Run integration tests
cargo test --test integration_tests

# Run with verbose output
cargo test -- --nocapture
```

## Security Considerations

- Keep validator private keys secure and use hardware security modules for production
- Regularly update the software to receive security patches
- Monitor validator performance to avoid slashing penalties
- Use firewall rules to protect the P2P and API endpoints

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