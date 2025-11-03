# API Reference

## Overview

The Production PoS node exposes a RESTful JSON API for interacting with the blockchain. This API provides access to chain data, validator operations, and network information.

## Base URL

```
http://localhost:8080/api/v1
```

## Authentication

Currently, the API does not require authentication for read operations. Write operations (validator registration, staking) require valid signatures.

## Response Format

All responses follow a consistent JSON format:

```json
{
  "success": true,
  "data": {
    // Response data here
  },
  "error": null,
  "timestamp": "2024-01-01T00:00:00Z"
}
```

Error responses:

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "INVALID_PARAMETER",
    "message": "Block hash must be 64 hex characters"
  },
  "timestamp": "2024-01-01T00:00:00Z"
}
```

## Chain Endpoints

### Get Chain Head

Returns information about the current chain head.

```http
GET /chain/head
```

**Response:**
```json
{
  "success": true,
  "data": {
    "hash": "0x1234567890abcdef...",
    "height": 12345,
    "slot": 98765,
    "epoch": 3086,
    "proposer": "0xabcdef1234567890...",
    "timestamp": "2024-01-01T12:00:00Z",
    "transaction_count": 150,
    "gas_used": 8500000,
    "gas_limit": 10000000
  }
}
```

### Get Block by Hash

Retrieves a specific block by its hash.

```http
GET /chain/block/{hash}
```

**Parameters:**
- `hash` (string): Block hash (64 hex characters)

**Response:**
```json
{
  "success": true,
  "data": {
    "hash": "0x1234567890abcdef...",
    "header": {
      "height": 12345,
      "previous_hash": "0x0987654321fedcba...",
      "merkle_root": "0xabcdef1234567890...",
      "state_root": "0x1111222233334444...",
      "timestamp": "2024-01-01T12:00:00Z",
      "slot": 98765,
      "epoch": 3086,
      "proposer": "0xvalidator123...",
      "gas_limit": 10000000,
      "gas_used": 8500000
    },
    "transactions": [
      {
        "hash": "0xtx1234567890...",
        "from": "0xsender123...",
        "to": "0xrecipient456...",
        "amount": "1000000000000000000",
        "gas_limit": 21000,
        "gas_price": "1000000000",
        "nonce": 5
      }
    ]
  }
}
```

### Get Block by Height

Retrieves a specific block by its height.

```http
GET /chain/block/height/{height}
```

**Parameters:**
- `height` (integer): Block height

### Get Transaction

Retrieves a specific transaction by its hash.

```http
GET /chain/transaction/{hash}
```

**Parameters:**
- `hash` (string): Transaction hash

**Response:**
```json
{
  "success": true,
  "data": {
    "hash": "0xtx1234567890...",
    "block_hash": "0xblock123...",
    "block_height": 12345,
    "index": 0,
    "from": "0xsender123...",
    "to": "0xrecipient456...",
    "amount": "1000000000000000000",
    "gas_limit": 21000,
    "gas_price": "1000000000",
    "gas_used": 21000,
    "nonce": 5,
    "status": "success",
    "timestamp": "2024-01-01T12:00:00Z"
  }
}
```

## Account Endpoints

### Get Account Balance

Returns the balance of a specific account.

```http
GET /account/{address}/balance
```

**Parameters:**
- `address` (string): Account address

**Response:**
```json
{
  "success": true,
  "data": {
    "address": "0xaccount123...",
    "balance": "5000000000000000000",
    "nonce": 10,
    "is_contract": false
  }
}
```

### Get Account Transactions

Returns transaction history for an account.

```http
GET /account/{address}/transactions?limit=20&offset=0
```

**Parameters:**
- `address` (string): Account address
- `limit` (integer, optional): Number of transactions to return (default: 20, max: 100)
- `offset` (integer, optional): Number of transactions to skip (default: 0)

**Response:**
```json
{
  "success": true,
  "data": {
    "transactions": [
      {
        "hash": "0xtx1234567890...",
        "from": "0xsender123...",
        "to": "0xrecipient456...",
        "amount": "1000000000000000000",
        "timestamp": "2024-01-01T12:00:00Z",
        "status": "success"
      }
    ],
    "total": 150,
    "limit": 20,
    "offset": 0
  }
}
```

## Validator Endpoints

### Get Validator Info

Returns information about a specific validator.

```http
GET /validator/{address}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "address": "0xvalidator123...",
    "public_key": "0xpubkey456...",
    "stake": "32000000000000000000",
    "delegated_stake": "128000000000000000000",
    "commission_rate": 500,
    "status": "Active",
    "registration_epoch": 100,
    "last_active_epoch": 3086,
    "metadata": {
      "name": "My Validator",
      "website": "https://myvalidator.com",
      "description": "Professional validator service"
    },
    "performance": {
      "blocks_proposed": 245,
      "blocks_missed": 2,
      "attestations_made": 98450,
      "attestations_missed": 50,
      "uptime_ratio": 0.9918
    }
  }
}
```

### List All Validators

Returns a list of all validators.

```http
GET /validators?status=active&limit=50&offset=0
```

**Parameters:**
- `status` (string, optional): Filter by status (active, inactive, jailed, exiting, exited)
- `limit` (integer, optional): Number of validators to return (default: 50, max: 500)
- `offset` (integer, optional): Number of validators to skip (default: 0)

### Register Validator

Registers a new validator.

```http
POST /validator/register
```

**Request Body:**
```json
{
  "validator_key": "0xpubkey123...",
  "commission_rate": 500,
  "minimum_stake": "32000000000000000000",
  "metadata": {
    "name": "My Validator",
    "website": "https://myvalidator.com",
    "description": "Professional validator service",
    "contact": "validator@example.com"
  },
  "signature": "0xsignature456..."
}
```

### Stake Tokens

Stakes tokens with a validator.

```http
POST /validator/stake
```

**Request Body:**
```json
{
  "validator": "0xvalidator123...",
  "amount": "1000000000000000000",
  "from": "0xdelegator456...",
  "signature": "0xsignature789..."
}
```

### Unstake Tokens

Initiates unstaking process.

```http
POST /validator/unstake
```

**Request Body:**
```json
{
  "validator": "0xvalidator123...",
  "amount": "1000000000000000000",
  "from": "0xdelegator456...",
  "signature": "0xsignature789..."
}
```

## Network Endpoints

### Get Network Status

Returns overall network status and statistics.

```http
GET /network/status
```

**Response:**
```json
{
  "success": true,
  "data": {
    "network_id": "Mainnet",
    "chain_head": {
      "height": 12345,
      "hash": "0x1234567890abcdef..."
    },
    "sync_status": {
      "is_syncing": false,
      "sync_progress": 1.0,
      "peers_count": 45
    },
    "validator_stats": {
      "total_validators": 1500,
      "active_validators": 1450,
      "total_stake": "50000000000000000000000"
    },
    "epoch_info": {
      "current_epoch": 3086,
      "current_slot": 98765,
      "slots_per_epoch": 32,
      "slot_duration": 12
    }
  }
}
```

### Get Peer Information

Returns information about connected peers.

```http
GET /network/peers
```

**Response:**
```json
{
  "success": true,
  "data": {
    "peers": [
      {
        "id": "12D3KooWABC123...",
        "address": "/ip4/192.168.1.100/tcp/9000",
        "direction": "outbound",
        "connected_at": "2024-01-01T10:30:00Z",
        "last_seen": "2024-01-01T12:00:00Z"
      }
    ],
    "total_peers": 45,
    "max_peers": 50
  }
}
```

## Consensus Endpoints

### Get Consensus State

Returns current consensus state information.

```http
GET /consensus/state
```

**Response:**
```json
{
  "success": true,
  "data": {
    "current_epoch": 3086,
    "current_slot": 98765,
    "justified_checkpoint": {
      "epoch": 3085,
      "root": "0xjustified123..."
    },
    "finalized_checkpoint": {
      "epoch": 3084,
      "root": "0xfinalized456..."
    },
    "fork_choice_head": "0x1234567890abcdef..."
  }
}
```

### Get Epoch Information

Returns detailed information about a specific epoch.

```http
GET /consensus/epoch/{epoch}
```

**Parameters:**
- `epoch` (integer): Epoch number

## WebSocket API

The node also provides a WebSocket endpoint for real-time updates:

```
ws://localhost:8080/ws
```

### Subscription Messages

**Subscribe to new blocks:**
```json
{
  "method": "subscribe",
  "params": {
    "type": "blocks"
  }
}
```

**Subscribe to new transactions:**
```json
{
  "method": "subscribe",
  "params": {
    "type": "transactions",
    "filter": {
      "address": "0xaccount123..."
    }
  }
}
```

### Event Messages

**New block event:**
```json
{
  "type": "block",
  "data": {
    "hash": "0x1234567890abcdef...",
    "height": 12346,
    "timestamp": "2024-01-01T12:01:00Z"
  }
}
```

## Error Codes

| Code | Description |
|------|-------------|
| `INVALID_PARAMETER` | Invalid or missing parameter |
| `NOT_FOUND` | Requested resource not found |
| `INVALID_SIGNATURE` | Cryptographic signature verification failed |
| `INSUFFICIENT_BALANCE` | Account has insufficient balance |
| `VALIDATOR_EXISTS` | Validator already registered |
| `VALIDATOR_NOT_FOUND` | Validator not found |
| `NETWORK_ERROR` | Network or connectivity error |
| `INTERNAL_ERROR` | Internal server error |

## Rate Limiting

The API implements rate limiting to prevent abuse:

- **Read operations**: 100 requests per minute per IP
- **Write operations**: 10 requests per minute per IP
- **WebSocket connections**: 5 connections per IP

Rate limit headers are included in responses:

```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 99
X-RateLimit-Reset: 1640995200
```

## SDK Examples

### JavaScript/TypeScript

```typescript
import { ProductionPosClient } from 'production-pos-sdk';

const client = new ProductionPosClient('http://localhost:8080');

// Get chain head
const head = await client.chain.getHead();
console.log('Current height:', head.height);

// Get account balance
const balance = await client.account.getBalance('0x123...');
console.log('Balance:', balance.balance);

// Subscribe to new blocks
client.ws.subscribe('blocks', (block) => {
  console.log('New block:', block.height);
});
```

### Python

```python
from production_pos import Client

client = Client('http://localhost:8080')

# Get chain head
head = client.chain.get_head()
print(f'Current height: {head.height}')

# Get account balance
balance = client.account.get_balance('0x123...')
print(f'Balance: {balance.balance}')
```

### Rust

```rust
use production_pos_client::Client;

let client = Client::new("http://localhost:8080")?;

// Get chain head
let head = client.chain().get_head().await?;
println!("Current height: {}", head.height);

// Get account balance
let balance = client.account().get_balance("0x123...").await?;
println!("Balance: {}", balance.balance);
```