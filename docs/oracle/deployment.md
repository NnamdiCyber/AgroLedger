# Warehouse Oracle Deployment Guide

## Overview

The `oracle/` service is a standalone Rust binary that runs alongside the deployed AgroLedger contracts. It is not a Soroban contract — it is an off-chain service that:

1. Polls external price APIs (AFEX, GCX, CME) at configurable intervals
2. Signs lot attestation payloads with the oracle secret key
3. Submits signed prices and attestations to the `WarehouseOracle` contract on-chain

## Architecture

```
┌─────────────┐    ┌──────────────┐    ┌────────────────┐
│   AFEX API  │───▶│              │    │                │
├─────────────┤    │    Oracle    │───▶│ WarehouseOracle │
│   GCX API   │───▶│   Sidecar    │    │   (Soroban)    │
├─────────────┤    │              │    └────────────────┘
│   CME API   │───▶│              │
└─────────────┘    └──────────────┘
```

## Prerequisites

- Rust stable (matching `rust-toolchain.toml`)
- Access to the Stellar network (testnet or mainnet)
- Funded Stellar account with XLM for transaction fees
- API keys for AFEX (and optionally GCX, CME)
- Warehouse inspector keypairs for multi-sig attestation

## Installation

### Build the Binary

```bash
cargo build -p oracle --release
```

The binary will be at `target/release/oracle`.

### Configuration

Copy and edit the configuration file:

```bash
cp oracle/config.toml.example oracle/config.toml
```

### Config Reference

```toml
[stellar]
rpc_url = "https://soroban-testnet.stellar.org"
network_passphrase = "Test SDF Network ; September 2015"
# Dedicated oracle keypair — NOT the deployer/admin key
signing_secret = "S..."

[contracts]
warehouse_oracle = "C..."  # Deployed WarehouseOracle contract ID
crop_token = "C..."        # Deployed CropToken contract ID

[price_feeds]
interval_seconds = 300

[price_feeds.afex]
url = "https://api.afexnigeria.com/v1"
api_key = "..."  # AFEX API key
commodities = ["MAIZE", "COCOA", "SOY", "PALM_OIL"]

[price_feeds.gcx]
url = "https://api.gcx.com.gh/v1"
api_key = "..."
commodities = ["COCOA", "MAIZE"]

[price_feeds.cme]
url = "https://datamine.cmegroup.com/cme/api/v1"
api_key = "..."
commodities = ["MAIZE", "SOY"]

[attestation]
multisig_threshold = 3
inspector_public_keys = [
  "G...",  # Inspector 1
  "G...",  # Inspector 2
  "G...",  # Inspector 3
]

[logging]
level = "info"
```

## Running

### Standalone

```bash
ORACLE_SIGNING_SECRET=S... ./target/release/oracle --config oracle/config.toml
```

### Docker

```bash
docker build -t agroledger-oracle ./oracle
docker run \
  --env-file .env \
  -v $(pwd)/oracle/config.toml:/app/config.toml \
  agroledger-oracle
```

### Docker Compose

```yaml
services:
  oracle:
    build: ./oracle
    env_file: .env
    volumes:
      - ./oracle/config.toml:/app/config.toml
    restart: unless-stopped
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
```

## Key Management

### Oracle Key

- The oracle key is registered in the `WarehouseOracle` contract at initialization
- Only the `push_price()` function requires this key's authentication
- Rotate the oracle key by calling `initialize()` again with a new pubkey (admin-only)
- Never use the deployer/admin key as the oracle key

### Inspector Keys

- Inspector keys are set in the `InspectorSet` at contract initialization
- Each inspector authenticates individually via `require_auth()` when submitting lots
- For lots > 50 MT, at least 3 inspector signatures are required
- Inspectors should use separate keypairs from the oracle key

## Monitoring

### Health Check Endpoint

The oracle exposes a health check HTTP endpoint:

```
GET /health
{
  "status": "ok",
  "last_price_push": 1700000000,
  "last_lot_submission": 1699999999,
  "price_feeds_running": true,
  "stellar_rpc_connected": true
}
```

### Metrics

Key metrics to monitor:

| Metric | Description | Warning Threshold |
|---|---|---|
| `last_price_push` | Timestamp of last on-chain price | > 10 minutes stale |
| `price_feed_errors` | Consecutive API failures | > 5 |
| `stellar_tx_failures` | Consecutive transaction failures | > 3 |
| `balance_xlm` | Oracle account balance | < 10 XLM |

### Logging

Logs are structured JSON. Use the `LOG_LEVEL` env var to control verbosity:

```json
{"level":"info","msg":"Price pushed","commodity":"MAIZE","price":200000000,"timestamp":1700000000}
{"level":"error","msg":"AFEX API request failed","retry_count":3,"error":"timeout"}
```

## High Availability

For production deployments, run at least 2 oracle instances behind a load balancer. Only one instance should submit prices at a time (leader election via Redis or PostgreSQL advisory lock). The `WarehouseOracle` contract overwrites prices on each push, so duplicate submissions are idempotent.

## Disaster Recovery

1. **Key compromise**: Call `initialize()` with a new oracle pubkey, then restart the oracle with the new key
2. **Contract upgrade**: Deploy new contract, transfer admin, update oracle config
3. **Network outage**: Oracle queues prices and submits once connection is restored
4. **Data loss**: Re-sync from on-chain event history (see indexer documentation)

## Hardware Requirements

| Environment | CPU | RAM | Storage | Network |
|---|---|---|---|---|
| Development | 1 core | 512 MB | 10 GB | Any |
| Production | 2 cores | 2 GB | 50 GB SSD | Low-latency (<50ms to Stellar RPC) |
