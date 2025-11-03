# Validator Guide

## Overview

Validators are the backbone of the Production PoS network. They propose blocks, attest to the chain state, and secure the network through their stake. This guide covers everything needed to run a validator successfully.

## Prerequisites

### Hardware Requirements

**Minimum Specifications**
- CPU: 4 cores, 2.5+ GHz
- RAM: 8 GB
- Storage: 100 GB SSD
- Network: Stable internet, 10+ Mbps

**Recommended Specifications**
- CPU: 8 cores, 3.0+ GHz
- RAM: 16 GB
- Storage: 500 GB NVMe SSD
- Network: Dedicated connection, 100+ Mbps

### Software Requirements
- Linux (Ubuntu 20.04+ recommended)
- Rust 1.70+
- SQLite3
- Reliable backup solution

### Staking Requirements
- Minimum stake: 32,000,000,000 tokens (32 ETH equivalent)
- Additional stake for better selection probability
- Access to tokens for initial deposit

## Setup Process

### 1. Install Node Software

```bash
# Clone repository
git clone https://github.com/republic-chain/proof-of-stake
cd production-pos

# Build release version
cargo build --release

# Verify installation
./target/release/node --version
```

### 2. Generate Validator Keys

```bash
# Generate new keypair
./target/release/validator generate-keys \
  --output validator_key.json

# Backup the key file securely
cp validator_key.json /secure/backup/location/

# Show validator address
./target/release/validator show-address \
  --keyfile validator_key.json
```

**⚠️ CRITICAL: Secure Your Keys**
- Store private keys in multiple secure locations
- Never share private keys with anyone
- Consider hardware security modules for production
- Test recovery procedures regularly

### 3. Configure Node

Create `validator.toml` configuration:

```toml
[network]
network_id = "Mainnet"  # or "Testnet"
listen_address = "/ip4/0.0.0.0/tcp/9000"
max_peers = 100

[storage]
data_dir = "/opt/production-pos/data"
cache_size = 268435456  # 256MB

[validator]
enabled = true
keystore_path = "/secure/path/validator_key.json"
graffiti = "My Validator"
fee_recipient = "0x1234...abcd"  # Your fee address

[api]
enabled = false  # Disable for security

[logging]
level = "info"
file = "/var/log/production-pos/validator.log"
```

### 4. Start Node

```bash
# Start validator node
./target/release/node \
  --config validator.toml \
  --validator

# Or use systemd service (recommended)
sudo systemctl start production-pos-validator
```

## Validator Registration

### On-Chain Registration

```bash
# Register as validator
./target/release/validator register \
  --keyfile validator_key.json \
  --stake 32000000000 \
  --name "My Validator" \
  --commission 500 \
  --website "https://myvalidator.com" \
  --description "Professional validator service"
```

### Registration Parameters

**Required Fields**
- `stake`: Initial stake amount (minimum 32 ETH equivalent)
- `name`: Human-readable validator name
- `commission`: Commission rate in basis points (500 = 5%)

**Optional Fields**
- `website`: Validator website URL
- `description`: Description of validator service
- `contact`: Contact information

### Activation Process

1. **Submit Registration**: Transaction is included in blockchain
2. **Stake Verification**: Network verifies minimum stake
3. **Queue Position**: Added to activation queue
4. **Activation**: Becomes active validator after queue processing
5. **First Duties**: Begins proposing blocks and attesting

## Validator Operations

### Daily Operations

**Monitoring Checklist**
- [ ] Node is running and synchronized
- [ ] No missed attestations in last 24h
- [ ] Block proposals completed successfully
- [ ] No slashing warnings or alerts
- [ ] System resources within normal ranges

**Performance Metrics**
- Attestation success rate: >99%
- Block proposal success rate: 100%
- Network connectivity: Stable
- Sync status: Up to date

### Maintenance Tasks

**Weekly Tasks**
- Review validator performance metrics
- Check for software updates
- Verify backup systems
- Monitor system logs for errors

**Monthly Tasks**
- Update software if new versions available
- Review and optimize system configuration
- Test disaster recovery procedures
- Analyze validator economics and rewards

### Updates and Upgrades

**Software Updates**
```bash
# Check current version
./target/release/node --version

# Download new version
git pull origin main
cargo build --release

# Stop validator gracefully
sudo systemctl stop production-pos-validator

# Update binary
sudo cp target/release/node /usr/local/bin/

# Restart validator
sudo systemctl start production-pos-validator
```

## Rewards and Economics

### Reward Sources

**Block Proposal Rewards**
- Base reward: Fixed amount per successful proposal
- Transaction fees: Variable based on block contents
- MEV rewards: Additional value from transaction ordering

**Attestation Rewards**
- Timely attestations: Reward for correct, on-time votes
- Source/target accuracy: Bonus for voting on correct checkpoints
- Inclusion delay: Higher rewards for faster inclusion

### Commission Structure

**Self-Stake Rewards**
- Validator keeps 100% of rewards on self-staked amount
- No commission charged on own stake

**Delegated Stake Rewards**
- Commission percentage applied to delegated stake rewards
- Remaining rewards distributed to delegators
- Commission rate set during registration

### Reward Calculation

```
Total Rewards = Base Rewards + Performance Bonus + Commission

Base Rewards = (Validator Stake / Total Network Stake) × Epoch Rewards
Performance Bonus = Base Rewards × (Actual Performance / Expected Performance)
Commission = Delegated Stake Rewards × Commission Rate
```

## Slashing and Penalties

### Slashable Offenses

**Proposer Slashing**
- Double block proposal: Proposing two blocks for same slot
- Detection: Automatic by network protocol
- Penalty: Minimum 1/32 of stake + additional penalties

**Attester Slashing**
- Double voting: Two conflicting attestations
- Surround voting: Attestation that surrounds another
- Detection: Automatic slashing detection system

### Penalty Structure

**Minor Penalties**
- Missed attestations: Small ongoing penalty
- Late attestations: Reduced rewards
- Offline time: Proportional to downtime

**Major Penalties (Slashing)**
- Minimum penalty: 1/32 of effective balance
- Correlation penalty: Increases with other slashed validators
- Maximum penalty: Entire stake in extreme cases

### Avoiding Slashing

**Best Practices**
- Run only one validator instance per key
- Ensure system clock synchronization
- Maintain stable network connectivity
- Monitor for software bugs and updates
- Use proper key management practices

**Infrastructure Recommendations**
- Redundant internet connections
- Uninterruptible power supply (UPS)
- Monitoring and alerting systems
- Automated failover procedures
- Regular security audits

## Monitoring and Alerting

### Key Metrics

**Node Health**
- Sync status and block height
- Peer count and connectivity
- Resource usage (CPU, RAM, disk)
- Network latency and bandwidth

**Validator Performance**
- Attestation success rate
- Block proposal frequency
- Inclusion delay statistics
- Reward accumulation rate

### Monitoring Tools

**Built-in Metrics**
```bash
# Check node status
curl http://localhost:8080/status

# View validator performance
curl http://localhost:8080/validator/performance

# Monitor network peers
curl http://localhost:8080/network/peers
```

**External Monitoring**
- Prometheus metrics export
- Grafana dashboards
- Custom alerting rules
- Third-party validator monitoring services

### Alert Configuration

**Critical Alerts**
- Node offline or not syncing
- Missed block proposals
- Slashing conditions detected
- System resource exhaustion

**Warning Alerts**
- High attestation miss rate
- Network connectivity issues
- Software update available
- Performance degradation

## Security Best Practices

### Key Management

**Storage Security**
- Use hardware security modules (HSMs)
- Encrypt keystore files
- Multiple secure backup locations
- Regular backup verification

**Access Control**
- Limit system access to essential personnel
- Use strong authentication (2FA)
- Regular security audits
- Network segmentation

### Operational Security

**System Hardening**
- Keep operating system updated
- Disable unnecessary services
- Configure firewall rules
- Use fail2ban for intrusion prevention

**Network Security**
- VPN access for remote management
- Monitor network traffic
- Use secure communication channels
- Regular penetration testing

### Incident Response

**Preparation**
- Document incident response procedures
- Test recovery scenarios regularly
- Maintain emergency contact list
- Keep backup systems ready

**Response Process**
1. Detect and assess incident
2. Contain and isolate affected systems
3. Investigate root cause
4. Implement fixes and recovery
5. Document lessons learned

## Troubleshooting

### Common Issues

**Sync Problems**
```bash
# Check sync status
curl http://localhost:8080/sync/status

# Reset chain data (last resort)
rm -rf data/chain/
./target/release/node --config validator.toml
```

**Performance Issues**
```bash
# Check resource usage
htop
iostat -x 1

# Optimize database
sqlite3 data/blocks.db "VACUUM;"
```

**Network Connectivity**
```bash
# Test P2P connectivity
netstat -tlnp | grep 9000

# Check peer connections
curl http://localhost:8080/network/peers
```

### Support Resources

**Documentation**
- [API Reference](./api.md)
- [Development Guide](./development.md)
- [Consensus Mechanism](./consensus.md)

**Community**
- Discord support channel
- GitHub issues and discussions
- Validator forums and mailing lists
- Regular community calls

**Professional Support**
- Validator service providers
- Infrastructure consultants
- Security audit services
- 24/7 monitoring solutions