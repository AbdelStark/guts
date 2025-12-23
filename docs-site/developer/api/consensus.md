# Consensus API

Query the Simplex BFT consensus engine status and blocks.

## Overview

Guts uses Simplex BFT (Byzantine Fault Tolerant) consensus for total ordering of all state changes. This API provides insight into the consensus state.

::: tip
These endpoints are read-only and provide network-wide consensus information.
:::

## Get Consensus Status

```http
GET /api/consensus/status
```

Returns the current consensus engine status.

### Example

```bash
curl https://api.guts.network/api/consensus/status \
  -H "Authorization: Bearer guts_xxx"
```

### Response

```json
{
  "enabled": true,
  "use_simplex_bft": true,
  "block_height": 12345,
  "block_time_ms": 2000,
  "last_block_hash": "abc123...",
  "last_block_time": "2025-01-20T12:00:00Z",
  "validator_count": 4,
  "is_validator": false,
  "sync_status": "synced",
  "mempool_size": 5,
  "finalized_height": 12343
}
```

### Status Fields

| Field | Description |
|-------|-------------|
| `enabled` | Consensus engine is enabled |
| `use_simplex_bft` | Using Simplex BFT (vs. simple consensus) |
| `block_height` | Current block height |
| `block_time_ms` | Target block time in milliseconds |
| `last_block_hash` | Hash of the last block |
| `last_block_time` | Timestamp of last block |
| `validator_count` | Number of validators |
| `is_validator` | This node is a validator |
| `sync_status` | `synced`, `syncing`, `stalled` |
| `mempool_size` | Pending transactions |
| `finalized_height` | Last finalized block |

## List Recent Blocks

```http
GET /api/consensus/blocks
```

### Parameters

| Name | Type | Description |
|------|------|-------------|
| `page` | integer | Page number |
| `per_page` | integer | Items per page (max: 100) |

### Example

```bash
curl "https://api.guts.network/api/consensus/blocks?per_page=10" \
  -H "Authorization: Bearer guts_xxx"
```

### Response

```json
{
  "items": [
    {
      "height": 12345,
      "hash": "abc123...",
      "parent_hash": "xyz789...",
      "timestamp": "2025-01-20T12:00:00Z",
      "proposer": "validator1_pubkey...",
      "transactions_count": 5,
      "size_bytes": 1024
    },
    {
      "height": 12344,
      "hash": "xyz789...",
      "parent_hash": "def456...",
      "timestamp": "2025-01-20T11:59:58Z",
      "proposer": "validator2_pubkey...",
      "transactions_count": 3,
      "size_bytes": 768
    }
  ],
  "total_count": 12345
}
```

## Get Block by Height

```http
GET /api/consensus/blocks/{height}
```

### Example

```bash
curl https://api.guts.network/api/consensus/blocks/12345 \
  -H "Authorization: Bearer guts_xxx"
```

### Response

```json
{
  "height": 12345,
  "hash": "abc123...",
  "parent_hash": "xyz789...",
  "timestamp": "2025-01-20T12:00:00Z",
  "proposer": "validator1_pubkey...",
  "transactions_count": 5,
  "size_bytes": 1024,
  "transactions": [
    {
      "id": "tx_abc123",
      "type": "ref_update",
      "repository": "alice/my-project",
      "data": {
        "ref": "refs/heads/main",
        "old_oid": "aaa111...",
        "new_oid": "bbb222..."
      }
    }
  ]
}
```

## List Validators

```http
GET /api/consensus/validators
```

Returns the current validator set.

### Example

```bash
curl https://api.guts.network/api/consensus/validators \
  -H "Authorization: Bearer guts_xxx"
```

### Response

```json
{
  "validators": [
    {
      "public_key": "abc123...",
      "name": "validator-1",
      "power": 1,
      "address": "/ip4/1.2.3.4/tcp/9000",
      "status": "active",
      "last_block_proposed": 12340,
      "blocks_proposed": 3000,
      "uptime_percent": 99.9
    },
    {
      "public_key": "def456...",
      "name": "validator-2",
      "power": 1,
      "address": "/ip4/5.6.7.8/tcp/9000",
      "status": "active",
      "last_block_proposed": 12343,
      "blocks_proposed": 3100,
      "uptime_percent": 99.8
    }
  ],
  "total": 4,
  "quorum_threshold": 3
}
```

### Validator Status

| Status | Description |
|--------|-------------|
| `active` | Participating in consensus |
| `inactive` | Not responding |
| `syncing` | Catching up with the network |
| `jailed` | Temporarily excluded |

## Get Mempool Statistics

```http
GET /api/consensus/mempool
```

Returns mempool statistics.

### Example

```bash
curl https://api.guts.network/api/consensus/mempool \
  -H "Authorization: Bearer guts_xxx"
```

### Response

```json
{
  "size": 5,
  "max_size": 10000,
  "bytes": 2048,
  "max_bytes": 104857600,
  "oldest_tx_age_seconds": 2.5
}
```

## Submit a Transaction

```http
POST /api/consensus/transactions
```

Submit a transaction to the mempool. Most operations go through higher-level APIs (repos, PRs, etc.), but this endpoint allows direct transaction submission.

### Example

```bash
curl -X POST https://api.guts.network/api/consensus/transactions \
  -H "Authorization: Bearer guts_xxx" \
  -H "Content-Type: application/json" \
  -d '{
    "type": "ref_update",
    "data": {
      "repository": "alice/my-project",
      "ref": "refs/heads/main",
      "old_oid": "aaa111...",
      "new_oid": "bbb222..."
    }
  }'
```

### Response

```json
{
  "id": "tx_xyz789",
  "status": "pending",
  "submitted_at": "2025-01-20T12:00:01Z"
}
```

## Byzantine Fault Tolerance

The Simplex BFT consensus provides:

| Property | Value |
|----------|-------|
| **Fault Tolerance** | f < n/3 Byzantine validators |
| **Finality** | 3 network hops |
| **Block Time** | ~2 seconds (configurable) |

For a 4-validator network:
- **Tolerates**: 1 malicious or offline validator
- **Requires**: 3 validators for consensus

```
Validators: 4
Byzantine tolerance (f): 1
Quorum: 2f + 1 = 3
```

## Health Indicators

Use these metrics to monitor consensus health:

| Metric | Healthy | Warning |
|--------|---------|---------|
| Block height | Increasing | Stalled > 1 min |
| Validator count | Expected count | Missing validators |
| Sync status | `synced` | `syncing` or `stalled` |
| Mempool size | < 1000 | > 5000 |
