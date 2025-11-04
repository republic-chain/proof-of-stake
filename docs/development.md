# Development Guide

## Getting Started

### Development Environment Setup

**Prerequisites:**
- Rust 1.70 or higher
- Git
- SQLite development libraries
- Node.js 16+ (for documentation tools)

**Installation:**
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install additional tools
cargo install cargo-watch cargo-audit cargo-deny

# Clone repository
git clone https://github.com/republic-chain/proof-of-stake
cd production-pos

# Install dependencies
cargo build
```

### Project Structure

```
production-pos/
├── src/
│   ├── types/           # Core data structures
│   ├── crypto/          # Cryptographic utilities
│   ├── consensus/       # Consensus implementation
│   ├── network/         # P2P networking (TODO)
│   ├── storage/         # Database and persistence (TODO)
│   ├── validator/       # Validator operations (TODO)
│   ├── config/          # Configuration management
│   ├── bin/             # Binary executables
│   └── lib.rs           # Main library
├── tests/               # Integration tests
├── docs/                # Documentation
├── examples/            # Example code
└── Cargo.toml          # Project configuration
```

## Development Workflow

### Local Development

**Start development environment:**
```bash
# Run in watch mode for auto-recompilation
cargo watch -x 'run --bin node'

# Run tests in watch mode
cargo watch -x test

# Run with debug logging
RUST_LOG=debug cargo run --bin node
```

**Code formatting and linting:**
```bash
# Format code
cargo fmt

# Run clippy for lints
cargo clippy

# Run all checks
cargo check --all-targets --all-features
```

### Testing

**Run all tests:**
```bash
cargo test
```

**Run specific test suites:**
```bash
# Unit tests only
cargo test --lib

# Integration tests
cargo test --test integration_tests

# Documentation tests
cargo test --doc

# Run with output
cargo test -- --nocapture
```

**Test coverage:**
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out html
```

### Benchmarking

```bash
# Install criterion
cargo install cargo-criterion

# Run benchmarks
cargo bench

# Generate benchmark reports
cargo criterion --message-format=json > bench.json
```

## Architecture Guidelines

### Code Organization

**Module Structure:**
- Each major component gets its own module
- Public APIs defined in `mod.rs` files
- Internal implementations in separate files
- Clear separation between interfaces and implementations

**Error Handling:**
```rust
use anyhow::{Result, Context};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConsensusError {
    #[error("Invalid block: {0}")]
    InvalidBlock(String),
    #[error("Validator not found: {address}")]
    ValidatorNotFound { address: String },
}

pub fn process_block(block: &Block) -> Result<()> {
    validate_block(block)
        .context("Failed to validate block")?;

    apply_block(block)
        .context("Failed to apply block to state")?;

    Ok(())
}
```

**Async Programming:**
```rust
use tokio::sync::{RwLock, mpsc};
use futures::future::join_all;

// Use async/await throughout
pub async fn sync_with_peers(&self) -> Result<()> {
    let futures = self.peers.iter()
        .map(|peer| self.sync_with_peer(peer));

    join_all(futures).await;
    Ok(())
}

// Use channels for component communication
let (tx, mut rx) = mpsc::channel(100);

tokio::spawn(async move {
    while let Some(event) = rx.recv().await {
        handle_event(event).await;
    }
});
```

### Design Patterns

**Builder Pattern for Complex Types:**
```rust
impl BlockBuilder {
    pub fn new() -> Self { /* ... */ }

    pub fn with_transactions(mut self, txs: Vec<Transaction>) -> Self {
        self.transactions = txs;
        self
    }

    pub fn with_proposer(mut self, proposer: Address) -> Self {
        self.proposer = proposer;
        self
    }

    pub fn build(self) -> Result<Block> { /* ... */ }
}
```

**Trait-based Abstractions:**
```rust
#[async_trait]
pub trait Storage: Send + Sync {
    async fn get_block(&self, hash: &Hash) -> Result<Option<Block>>;
    async fn store_block(&self, block: Block) -> Result<()>;
    async fn get_latest_height(&self) -> Result<u64>;
}

#[async_trait]
pub trait NetworkService: Send + Sync {
    async fn broadcast_block(&self, block: Block) -> Result<()>;
    async fn request_blocks(&self, from: u64, to: u64) -> Result<Vec<Block>>;
}
```

## Adding New Features

### 1. Define Types

Start by defining the data structures in the `types/` module:

```rust
// src/types/my_feature.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MyFeature {
    pub id: u64,
    pub data: Vec<u8>,
    pub timestamp: DateTime<Utc>,
}

impl MyFeature {
    pub fn new(data: Vec<u8>) -> Self {
        MyFeature {
            id: generate_id(),
            data,
            timestamp: Utc::now(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        // Validation logic
        Ok(())
    }
}
```

### 2. Add Tests

Write tests before implementing functionality:

```rust
// src/types/my_feature.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_feature_creation() {
        let data = vec![1, 2, 3, 4];
        let feature = MyFeature::new(data.clone());

        assert_eq!(feature.data, data);
        assert!(feature.validate().is_ok());
    }

    #[test]
    fn test_my_feature_validation() {
        let feature = MyFeature {
            id: 0,
            data: vec![],
            timestamp: Utc::now(),
        };

        // Should fail validation for empty data
        assert!(feature.validate().is_err());
    }
}
```

### 3. Implement Core Logic

Add the business logic in appropriate modules:

```rust
// src/consensus/my_feature.rs
use crate::types::MyFeature;

pub struct MyFeatureProcessor {
    // State and dependencies
}

impl MyFeatureProcessor {
    pub fn new() -> Self {
        MyFeatureProcessor {}
    }

    pub async fn process(&mut self, feature: MyFeature) -> Result<()> {
        // Validate
        feature.validate()?;

        // Process
        self.apply_feature(feature).await?;

        Ok(())
    }

    async fn apply_feature(&mut self, feature: MyFeature) -> Result<()> {
        // Implementation
        Ok(())
    }
}
```

### 4. Add Integration Points

Update the main components to use the new feature:

```rust
// src/consensus/mod.rs
pub mod my_feature;
pub use my_feature::*;

// In ConsensusEngine
impl ConsensusEngine {
    pub async fn process_my_feature(&mut self, feature: MyFeature) -> Result<()> {
        self.my_feature_processor.process(feature).await
    }
}
```

### 5. Add API Endpoints

Expose functionality through the API:

```rust
// src/api/my_feature.rs
use axum::{Json, extract::Path};
use crate::types::MyFeature;

pub async fn create_my_feature(
    Json(payload): Json<CreateMyFeatureRequest>
) -> Result<Json<MyFeature>, ApiError> {
    let feature = MyFeature::new(payload.data);

    // Process through consensus
    consensus.process_my_feature(feature.clone()).await?;

    Ok(Json(feature))
}

pub async fn get_my_feature(
    Path(id): Path<u64>
) -> Result<Json<MyFeature>, ApiError> {
    let feature = storage.get_my_feature(id).await?;
    Ok(Json(feature))
}
```

### 6. Update Documentation

Add documentation for the new feature:

```rust
/// My Feature provides functionality for...
///
/// # Examples
///
/// ```rust
/// let feature = MyFeature::new(vec![1, 2, 3]);
/// assert!(feature.validate().is_ok());
/// ```
pub struct MyFeature {
    /// Unique identifier for this feature
    pub id: u64,
    /// Binary data associated with the feature
    pub data: Vec<u8>,
}
```

## Performance Optimization

### Profiling

**CPU Profiling:**
```bash
# Install flamegraph
cargo install flamegraph

# Generate flame graph
cargo flamegraph --bin node -- --config test.toml

# Profile specific functions
perf record -g cargo run --bin node
perf report
```

**Memory Profiling:**
```bash
# Install valgrind
sudo apt install valgrind

# Run with memory checking
valgrind --tool=memcheck cargo run --bin node

# Memory profiling with massif
valgrind --tool=massif cargo run --bin node
```

### Optimization Techniques

**Async Performance:**
```rust
// Batch operations when possible
async fn process_transactions(txs: Vec<Transaction>) -> Result<()> {
    // Instead of processing one by one
    let futures: Vec<_> = txs.into_iter()
        .map(|tx| process_transaction(tx))
        .collect();

    // Process all concurrently
    try_join_all(futures).await?;
    Ok(())
}

// Use channels for backpressure
let (tx, rx) = mpsc::channel(1000); // Bounded channel

// Buffer operations
let mut buffer = Vec::with_capacity(100);
for item in items {
    buffer.push(item);
    if buffer.len() >= 100 {
        process_batch(&buffer).await?;
        buffer.clear();
    }
}
```

**Memory Optimization:**
```rust
// Use Cow for optional copying
use std::borrow::Cow;

fn process_data(data: Cow<[u8]>) -> Result<()> {
    // Only clone if modification needed
    let owned_data = if needs_modification {
        data.into_owned()
    } else {
        data.as_ref().to_vec()
    };
    Ok(())
}

// Pre-allocate collections
let mut validators = Vec::with_capacity(expected_count);

// Use appropriate data structures
use std::collections::HashMap;
use indexmap::IndexMap; // Preserves insertion order
use ahash::AHashMap;    // Faster hashing
```

## Security Considerations

### Input Validation

```rust
pub fn validate_address(address: &str) -> Result<Address> {
    if address.len() != 42 {
        return Err(anyhow!("Address must be 42 characters"));
    }

    if !address.starts_with("0x") {
        return Err(anyhow!("Address must start with 0x"));
    }

    let bytes = hex::decode(&address[2..])
        .map_err(|_| anyhow!("Invalid hex in address"))?;

    if bytes.len() != 20 {
        return Err(anyhow!("Address must be 20 bytes"));
    }

    Ok(Address::from_slice(&bytes))
}
```

### Cryptographic Security

```rust
// Always verify signatures
pub fn verify_transaction(tx: &Transaction, public_key: &PublicKey) -> Result<()> {
    let message = tx.signing_hash();
    CryptoProvider::verify(public_key, &message, &tx.signature)
        .map_err(|_| anyhow!("Invalid signature"))?;
    Ok(())
}

// Use constant-time comparisons
use subtle::ConstantTimeEq;

pub fn verify_hash(expected: &Hash, actual: &Hash) -> bool {
    expected.ct_eq(actual).into()
}

// Sanitize sensitive data
impl Drop for PrivateKey {
    fn drop(&mut self) {
        // Zero out memory
        self.0.fill(0);
    }
}
```

## Contributing Guidelines

### Pull Request Process

1. **Fork and Branch:**
   ```bash
   git checkout -b feature/my-feature
   ```

2. **Make Changes:**
   - Follow existing code style
   - Add tests for new functionality
   - Update documentation as needed

3. **Test Thoroughly:**
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```

4. **Commit with Clear Messages:**
   ```bash
   git commit -m "feat: add new consensus feature

   - Implement weighted validator selection
   - Add comprehensive tests
   - Update API documentation"
   ```

5. **Submit Pull Request:**
   - Provide clear description
   - Reference related issues
   - Include test results

### Code Review Checklist

**Functionality:**
- [ ] Does the code solve the intended problem?
- [ ] Are edge cases handled properly?
- [ ] Is error handling comprehensive?

**Security:**
- [ ] Are inputs properly validated?
- [ ] Are cryptographic operations secure?
- [ ] Are there any potential vulnerabilities?

**Performance:**
- [ ] Is the code efficient?
- [ ] Are there any unnecessary allocations?
- [ ] Are async operations used appropriately?

**Maintainability:**
- [ ] Is the code readable and well-documented?
- [ ] Are naming conventions followed?
- [ ] Is the design extensible?

**Testing:**
- [ ] Are there sufficient unit tests?
- [ ] Are edge cases tested?
- [ ] Do all tests pass?

### Release Process

1. **Version Bump:**
   ```bash
   cargo set-version 0.2.0
   ```

2. **Update Changelog:**
   Document all changes since last release

3. **Tag Release:**
   ```bash
   git tag -a v0.2.0 -m "Release version 0.2.0"
   git push origin v0.2.0
   ```

4. **Build and Test:**
   ```bash
   cargo build --release
   cargo test --release
   ```

5. **Publish:**
   ```bash
   cargo publish
   ```