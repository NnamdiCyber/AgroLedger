# AgroLedger Protocol

> **On-chain trade finance for African agricultural commodities — built on Stellar/Soroban.**

AgroLedger turns certified warehouse receipts into liquid, yield-bearing, cross-border-transferable instruments. Smallholder farmers in Nigeria, Ghana, Kenya, and Ethiopia can unlock instant credit against stored crops; diaspora remittances route directly into productive harvest vaults; commodity buyers hedge forward purchases against tokenized real-world inventory — all with built-in privacy-preserving KYC/AML compliance enforced at the contract level.

---

## Table of Contents

- [Why AgroLedger](#why-agroledger)
- [Architecture Overview](#architecture-overview)
- [Core Contracts](#core-contracts)
- [Protocol Layers](#protocol-layers)
  - [Real-World Assets](#real-world-assets)
  - [DeFi & Trading](#defi--trading)
  - [Payments & Remittances](#payments--remittances)
  - [Privacy & Compliance](#privacy--compliance)
  - [Developer Tooling](#developer-tooling)
  - [End-User Applications](#end-user-applications)
- [Project Structure](#project-structure)
- [Scaffold Reference](#scaffold-reference)
  - [Workspace Cargo.toml](#workspace-cargotoml)
  - [Contract Cargo.toml](#contract-cargotoml-per-crate)
  - [Root package.json](#root-packagejson)
  - [.env.example](#envexample)
  - [Inter-Contract Address Wiring](#inter-contract-address-wiring)
  - [Test Scaffolding](#test-scaffolding)
  - [Oracle Sidecar Config](#oracle-sidecar-config)
- [Getting Started](#getting-started)
- [Contract Reference](#contract-reference)
- [SDK](#sdk)
- [Tokenomics](#tokenomics)
- [Supported Networks](#supported-networks)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)

---

## Why AgroLedger

Africa's smallholder farmers collectively hold hundreds of billions of dollars in stored commodity value — yet most cannot access credit against it. Warehouse receipt financing exists in theory; in practice, it is slow, paper-based, and inaccessible to anyone without a bank account and a lawyer.

At the same time, the Africa-remittance corridor moves roughly $95 billion per year, most of it idling in mobile wallets or low-yield savings. Commodity buyers — flour mills, food processors, exporters — routinely overpay for forward inventory because price discovery across West and East Africa is fragmented and opaque.

AgroLedger resolves all three problems with a single composable protocol:

- A farmer deposits a crop lot at a certified warehouse and receives `CropToken` within minutes.
- That token is immediately usable as collateral to borrow USDC, transferable to a family member's remittance vault, or sold forward to a flour mill through the on-chain AMM.
- Every transfer is compliance-gated via a zero-knowledge `PrivacyPassport`, satisfying FATF Travel Rule requirements without exposing user identity on-chain.

The six protocol pillars — RWA tokenization, DeFi, payments, privacy/compliance, dev tooling, and end-user apps — are not additive features. Each one is structurally necessary for the others to work at scale.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                        END-USER APPLICATIONS                        │
│   USSD (2G/feature phone)  ·  Warehouse Portal  ·  Buyer B2B App   │
│            AgroLedger SDK  ·  On-chain Indexer                      │
└────────────────────────────┬────────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────────┐
│                      PAYMENTS & SETTLEMENT                          │
│   Diaspora Remittance Vault  ·  Instant Credit Disbursement         │
│   Harvest Auto-Repayment  ·  Multi-stablecoin (USDC · cNGN · cXOF) │
└────────────────────────────┬────────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────────┐
│                         DeFi LAYER                                  │
│   Commodity AMM (custom bonding curve)  ·  Harvest Yield Vault      │
│   Forward Hedge Contracts  ·  USDC/XLM/cXOF Liquidity Pools        │
└────────────────────────────┬────────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────────┐
│                     SOROBAN CONTRACT CORE                           │
│   CropToken (SEP-0041)  ·  CollateralVault  ·  ComplianceRegistry  │
│   CrossBorderRouter  ·  WarehouseOracle  ·  PrivacyPassport         │
└──────────┬──────────────────────────────────────────┬───────────────┘
           │                                          │
┌──────────▼──────────┐                  ┌────────────▼───────────────┐
│   REAL-WORLD ASSETS │                  │   PRIVACY & COMPLIANCE     │
│  Certified Warehouse│                  │  ZK Credential  ·  KYC     │
│  Price Oracle (AFEX)│                  │  FATF Travel Rule          │
│  Lot Certification  │                  │  SEC Sandbox (NG/GH/KE)    │
└─────────────────────┘                  └────────────────────────────┘
```

All arrows are bidirectional. The compliance layer is not a gate at the edge — it is called atomically inside every `CollateralVault` and `CrossBorderRouter` execution.

---

## Core Contracts

| Contract | Purpose |
|---|---|
| `CropToken` | SEP-0041 fungible token representing one metric ton of a certified warehouse lot |
| `CollateralVault` | Lock `CropTokens`, draw USDC credit; auto-repay on lot sale |
| `ComplianceRegistry` | Maintain a transfer allow-list keyed to `PrivacyPassport` attestations |
| `CrossBorderRouter` | Stellar path-payment routing with FATF Travel Rule memo injection |
| `WarehouseOracle` | Push certified lot data and AFEX/CME price feeds on-chain |
| `HarvestVault` | Auto-compound warehouse storage income + AMM LP fees |
| `ForwardHedge` | Enable commodity buyers to lock future purchase price against on-chain inventory |
| `PrivacyPassport` | Zero-knowledge credential verifier (Reclaim Protocol / Verite compatible) |

---

## Protocol Layers

### Real-World Assets

`CropToken` is a SEP-0041 compliant token issued 1:1 with a certified warehouse lot. Each token carries:

- **Commodity type** — maize, cocoa, soy, palm oil, cassava (expandable via governance)
- **Lot ID** — traceable to a physical warehouse receipt and inspection certificate
- **Valuation** — live price feed from AFEX (Nigeria), Ghana Commodity Exchange, or CME
- **Expiry** — mirrors the warehouse storage agreement; tokens auto-flag at lot expiry

Warehouse operators run a lightweight `WarehouseOracle` sidecar that signs lot attestations before pushing them to the contract. A multi-sig of three independent inspectors is required for lots above 50 MT.

```rust
// CropToken issuance (simplified)
pub fn issue_crop_token(
    env: Env,
    warehouse_id: Address,
    lot_id: BytesN<32>,
    commodity: Symbol,
    quantity_kg: u64,
    oracle_sig: BytesN<64>,
) -> Address
```

### DeFi & Trading

**Commodity AMM** — A Soroban-native AMM with commodity-specific bonding curves. Unlike standard constant-product AMMs, the curve incorporates seasonal price variance (e.g. pre-harvest discount, post-harvest glut) derived from three years of AFEX historical data. This narrows impermanent loss for LPs providing liquidity to thinly-traded crops.

**Harvest Vault** — A yield-bearing vault that auto-compounds two income streams: (1) warehouse storage fees accrued daily in USDC and (2) AMM LP trading fees. Users deposit `CropTokens` and receive yield-bearing `hCT` receipt tokens. The vault re-balances automatically when a lot is sold.

**Forward Hedge** — Commodity buyers (flour mills, food processors, exporters) lock a purchase price today against a specific lot, settled physically or in USDC at expiry. The contract uses a sealed-bid mechanism to prevent front-running.

**Liquidity Pools** — Base pairs are `CropToken/USDC`, `CropToken/XLM`, and `CropToken/cXOF` (CFA franc stablecoin). Stellar's SDEX provides backstop liquidity for the XLM pair without a cold-start problem.

### Payments & Remittances

AgroLedger is built on Stellar's path payment primitive, which finds optimal cross-currency routes atomically. The `CrossBorderRouter` wraps path payments with:

- **FATF Travel Rule data** — originator/beneficiary encrypted in the transaction memo using the recipient's public key
- **Multi-stablecoin support** — USDC, cNGN (naira), cXOF (CFA franc), cGHS (cedi), cKES (shilling)
- **Diaspora vault routing** — a remittance sender specifies a farmer's `HarvestVault` address; the payment auto-converts to the farmer's preferred stablecoin and earns storage yield until withdrawal

Fees are minimal — Stellar base fees apply (~0.00001 XLM per operation). AgroLedger charges a 0.15% routing fee on cross-border paths, split between the protocol treasury and liquidity providers.

```
NGN (Lagos) ──path payment──▶ XLM ──▶ USDC ──▶ HarvestVault (Kano)
                                                      │
                                              earns ~8–14% APY
                                              (warehouse yield + LP fees)
```

### Privacy & Compliance

The `PrivacyPassport` contract is the compliance backbone. It stores no PII on-chain. Instead, it stores a nullifier hash derived from a verified KYC credential (issued by a licensed identity provider) and a proof of compliance status.

**Supported credential standards:**
- Verite (Circle/Centre)
- Reclaim Protocol
- Self Protocol (for African passport holders)

**What is enforced on-chain:**
- Transfer amounts above $10,000 equivalent trigger a FATF Travel Rule memo requirement
- Transfers to or from jurisdictions outside the `ComplianceRegistry` allow-list are blocked
- Nigerian SEC, Ghana SEC, and Kenya CMA sandbox classifications are embedded in token transfer rules

**What stays off-chain:**
- Name, date of birth, document number
- KYC provider identity
- Full transaction history (only a rolling 90-day compliance attestation is referenced)

Compliance checks run synchronously inside `CollateralVault.borrow()` and `CrossBorderRouter.execute()`. A failed check reverts the entire transaction — there is no partial execution.

### Developer Tooling

**AgroLedger SDK** — A TypeScript/JavaScript SDK for agri-fintech developers (Releaf, Apollo Agriculture, Twiga, Thrive Agric) to integrate without rebuilding the contract stack:

```bash
npm install @agroledger/sdk
```

```typescript
import { AgroLedger } from '@agroledger/sdk';

const client = new AgroLedger({ network: 'mainnet', signer: wallet });

// Issue a crop token from a certified lot
const token = await client.cropToken.issue({
  warehouseId: 'WH-LAG-0042',
  commodity: 'MAIZE',
  quantityKg: 10_000,
  oracleAttestation: signedAttestation,
});

// Open a collateral vault and draw USDC
const vault = await client.collateralVault.open({
  cropToken: token.address,
  borrowAmountUsdc: 4_500,
});
```

**On-chain Indexer** — A Soroban event indexer that tracks full crop lot provenance: from warehouse deposit → token issuance → collateral lock → forward sale → settlement. Queryable via GraphQL.

```graphql
query {
  cropLot(lotId: "WH-LAG-0042-LOT-8821") {
    commodity
    quantityKg
    currentHolder
    collateralizedAmount
    priceHistory(last: 30) { timestamp price }
    transfers { from to amountKg timestamp }
  }
}
```

### End-User Applications

**Farmer App (USSD + smartphone)** — Works on 2G feature phones via USSD short codes. Farmers see their crop value in local currency, available credit, and pending payments. The word "blockchain" does not appear anywhere in the interface.

```
*384*1# → Check crop value
*384*2# → Draw credit (USDC → naira via cNGN)
*384*3# → Receive remittance
*384*4# → Authorize forward sale
```

**Warehouse Portal** — A web dashboard for warehouse operators to manage lot intake, issue oracle-signed attestations, track tokenized inventory, and view real-time price exposures.

**Commodity Buyer B2B Portal** — A portal for mills, processors, and exporters to browse available lots, place forward hedges, execute spot purchases, and view delivery schedules against on-chain settlement status.

---

## Project Structure

```
agroledger-protocol/
│
├── contracts/                          # Soroban smart contracts (Rust)
│   ├── crop_token/                     # SEP-0041 fungible token for warehouse lots
│   │   ├── src/
│   │   │   ├── lib.rs                  # Contract entry point + issuance logic
│   │   │   ├── metadata.rs             # Lot metadata: commodity, quantity, expiry
│   │   │   └── transfer.rs             # Compliance-gated transfer hooks
│   │   └── Cargo.toml
│   │
│   ├── collateral_vault/               # Lock CropTokens, draw USDC credit
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── vault.rs                # Open / repay / liquidate logic
│   │   │   └── ltv.rs                  # Loan-to-value ratio + liquidation triggers
│   │   └── Cargo.toml
│   │
│   ├── compliance_registry/            # On-chain transfer allow-list
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── allowlist.rs            # Jurisdiction-keyed permit store
│   │   │   └── fatf.rs                 # Travel Rule memo validation
│   │   └── Cargo.toml
│   │
│   ├── cross_border_router/            # Stellar path payments + FATF wrapping
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── router.rs               # Path-payment execution + fee routing
│   │   │   └── stablecoins.rs          # USDC · cNGN · cXOF · cGHS · cKES asset IDs
│   │   └── Cargo.toml
│   │
│   ├── warehouse_oracle/               # Certified lot attestation + price feeds
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── attestation.rs          # Multi-sig inspector sign-off (≥3 for >50 MT)
│   │   │   └── price_feed.rs           # AFEX · GCX · CME price push
│   │   └── Cargo.toml
│   │
│   ├── harvest_vault/                  # Yield-bearing vault (storage fees + LP)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── vault.rs                # Deposit / withdraw / rebalance
│   │   │   └── yield.rs                # Storage yield accrual + LP fee compounding
│   │   └── Cargo.toml
│   │
│   ├── commodity_amm/                  # Soroban AMM with seasonal bonding curve
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── curve.rs                # Custom bonding curve with seasonal variance
│   │   │   └── pool.rs                 # LP token issuance + swap execution
│   │   └── Cargo.toml
│   │
│   ├── forward_hedge/                  # Forward purchase contracts for buyers
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── hedge.rs                # Sealed-bid placement + settlement
│   │   │   └── settlement.rs           # Physical vs USDC settlement logic
│   │   └── Cargo.toml
│   │
│   └── privacy_passport/               # ZK credential verifier (Verite / Reclaim)
│       ├── src/
│       │   ├── lib.rs
│       │   ├── passport.rs             # Nullifier hash registration + verification
│       │   └── revocation.rs           # Authority-signed revocation
│       └── Cargo.toml
│
├── sdk/                                # TypeScript/JavaScript developer SDK
│   ├── src/
│   │   ├── index.ts                    # Main AgroLedger client export
│   │   ├── clients/
│   │   │   ├── cropToken.ts
│   │   │   ├── collateralVault.ts
│   │   │   ├── crossBorderRouter.ts
│   │   │   ├── harvestVault.ts
│   │   │   ├── commodityAmm.ts
│   │   │   └── privacyPassport.ts
│   │   ├── hooks/                      # React hooks for frontend integration
│   │   │   ├── useFarmerPortfolio.ts
│   │   │   ├── useCollateralVault.ts
│   │   │   └── useForwardHedge.ts
│   │   └── types/                      # Shared TypeScript types + contract ABIs
│   ├── tests/
│   └── package.json
│
├── indexer/                            # On-chain event indexer + GraphQL API
│   ├── src/
│   │   ├── ingest/                     # Soroban event ingestion workers
│   │   ├── schema/                     # GraphQL schema definitions
│   │   ├── resolvers/                  # Crop lot, transfer, vault resolvers
│   │   └── db/                         # PostgreSQL migrations + query layer
│   ├── Dockerfile
│   └── package.json
│
├── apps/                               # End-user applications
│   ├── ussd/                           # USSD gateway handler (2G / feature phone)
│   │   ├── src/
│   │   │   ├── menus/                  # Menu trees: check value, draw credit, receive
│   │   │   └── gateway.ts              # Africa's Talking / Hubtel USSD adapter
│   │   └── package.json
│   │
│   ├── warehouse-portal/               # Warehouse operator web dashboard
│   │   ├── src/
│   │   │   ├── pages/                  # Lot intake, attestation, inventory views
│   │   │   └── components/
│   │   └── package.json
│   │
│   └── buyer-portal/                   # Commodity buyer B2B web app
│       ├── src/
│       │   ├── pages/                  # Browse lots, place hedges, track delivery
│       │   └── components/
│       └── package.json
│
├── oracle/                             # WarehouseOracle sidecar service
│   ├── src/
│   │   ├── attestation/                # Lot certification signing + submission
│   │   ├── price/                      # AFEX / GCX / CME price feed polling
│   │   └── multisig/                   # Inspector co-signing coordination
│   └── Cargo.toml
│
├── scripts/                            # Deployment, seeding, and migration scripts
│   ├── deploy/
│   │   ├── testnet.sh
│   │   └── mainnet.sh
│   ├── seed/
│   │   ├── warehouses.ts               # Seed test warehouse operators
│   │   ├── lots.ts                     # Seed test crop lots + price feeds
│   │   └── passports.ts                # Seed test KYC credentials
│   └── migrate/
│
├── docs/                               # Extended documentation
│   ├── contracts/                      # Per-contract interface references
│   ├── compliance/                     # FATF Travel Rule integration guide
│   ├── oracle/                         # Warehouse oracle deployment guide
│   └── sdk/                            # SDK usage + React hook examples
│
├── Cargo.toml                          # Workspace manifest (all Rust crates)
├── Cargo.lock
├── package.json                        # Workspace root (SDK + apps)
├── .env.example                        # Environment variable template
└── README.md
```

---

## Scaffold Reference

Everything below is copy-paste ready. These files are the minimum required to run `cargo build`, `npm install`, and `stellar contract deploy` without errors from a fresh clone.

---

### Workspace `Cargo.toml`

Place at the repo root. Lists every Soroban crate as a workspace member so `cargo build --release` compiles all contracts in one pass.

```toml
[workspace]
resolver = "2"
members  = [
  "contracts/crop_token",
  "contracts/collateral_vault",
  "contracts/compliance_registry",
  "contracts/cross_border_router",
  "contracts/warehouse_oracle",
  "contracts/harvest_vault",
  "contracts/commodity_amm",
  "contracts/forward_hedge",
  "contracts/privacy_passport",
  "oracle",
]

[workspace.dependencies]
soroban-sdk = { version = "21.0.0", features = ["testutils"] }
serde       = { version = "1",      features = ["derive"]    }
serde_json  = "1"

[profile.release]
opt-level     = "z"
overflow-checks = true
debug         = false
strip         = "symbols"
codegen-units = 1
lto           = true

[profile.release-with-logs]
inherits  = "release"
debug-assertions = true
```

> **Pin `soroban-sdk` to a single version across all crates.** Diverging versions produce linker errors at WASM compile time that are hard to diagnose.

---

### Contract `Cargo.toml` (per crate)

Use this template for every contract under `contracts/`. Replace `crop_token` / `CropToken` with the crate name.

```toml
[package]
name    = "crop-token"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]          # required — Soroban compiles to a WASM cdylib

[dependencies]
soroban-sdk = { workspace = true }

[dev-dependencies]
soroban-sdk = { workspace = true, features = ["testutils"] }

[features]
testutils = ["soroban-sdk/testutils"]
```

For contracts that call other contracts (e.g. `collateral_vault` calling `compliance_registry`), add the dependency crate:

```toml
[dependencies]
soroban-sdk        = { workspace = true }
compliance-registry = { path = "../compliance_registry", features = ["testutils"] }
```

---

### Root `package.json`

Place at the repo root. Declares the Node workspace so `npm install` hoists shared dependencies across SDK and all apps in one pass.

```json
{
  "name": "agroledger-protocol",
  "version": "0.1.0",
  "private": true,
  "workspaces": [
    "sdk",
    "apps/ussd",
    "apps/warehouse-portal",
    "apps/buyer-portal",
    "indexer"
  ],
  "scripts": {
    "build":            "npm run build --workspaces",
    "test":             "npm run test --workspaces",
    "test:integration": "cd sdk && npm run test:integration",
    "deploy:local":     "node scripts/deploy/local.js",
    "deploy:testnet":   "node scripts/deploy/testnet.js",
    "seed:local":       "node scripts/seed/index.js --network local",
    "lint":             "eslint '**/*.ts' --ignore-path .eslintignore"
  },
  "devDependencies": {
    "typescript":  "^5.4.0",
    "eslint":      "^8.57.0",
    "@types/node": "^20.0.0"
  },
  "engines": {
    "node": ">=18.0.0",
    "npm":  ">=9.0.0"
  }
}
```

Each workspace package (`sdk/package.json`, `apps/*/package.json`) should declare its own `name`, `version`, and `dependencies`. Cross-workspace imports use the package name directly:

```json
{
  "name": "@agroledger/warehouse-portal",
  "dependencies": {
    "@agroledger/sdk": "*"
  }
}
```

---

### `.env.example`

Copy to `.env` before running any script or service. Never commit `.env` — it is in `.gitignore`.

```bash
# ── Stellar Network ────────────────────────────────────────────────────────────
STELLAR_NETWORK=testnet                    # local | testnet | mainnet
STELLAR_RPC_URL=https://soroban-testnet.stellar.org
STELLAR_HORIZON_URL=https://horizon-testnet.stellar.org
STELLAR_NETWORK_PASSPHRASE="Test SDF Network ; September 2015"

# ── Deployer / Admin Keys ──────────────────────────────────────────────────────
DEPLOYER_SECRET_KEY=S...                   # Stellar secret key (S...)
ADMIN_ADDRESS=G...                         # Stellar public key (G...)

# ── Deployed Contract IDs (populated after first deploy) ──────────────────────
CROP_TOKEN_CONTRACT_ID=
COLLATERAL_VAULT_CONTRACT_ID=
COMPLIANCE_REGISTRY_CONTRACT_ID=
CROSS_BORDER_ROUTER_CONTRACT_ID=
WAREHOUSE_ORACLE_CONTRACT_ID=
HARVEST_VAULT_CONTRACT_ID=
COMMODITY_AMM_CONTRACT_ID=
FORWARD_HEDGE_CONTRACT_ID=
PRIVACY_PASSPORT_CONTRACT_ID=

# ── Stablecoin Asset IDs (Stellar asset contract addresses) ───────────────────
USDC_CONTRACT_ID=CCW67TSZV3SSS2HXMBQ5JFGCKJNXKZM7UQUWUZPUTHXSTZLEO7SJMI75   # testnet
CNGN_CONTRACT_ID=                          # cNGN (naira) — fill after issuance
CXOF_CONTRACT_ID=                          # cXOF (CFA franc)
CGHS_CONTRACT_ID=                          # cGHS (cedi)
CKES_CONTRACT_ID=                          # cKES (shilling)

# ── Oracle Sidecar ─────────────────────────────────────────────────────────────
ORACLE_SIGNING_SECRET=S...                 # Dedicated oracle keypair (not deployer)
ORACLE_MULTISIG_THRESHOLD=3                # Inspectors required for lots > 50 MT
AFEX_API_KEY=                              # AFEX price feed API key
AFEX_API_URL=https://api.afexnigeria.com/v1
GCX_API_KEY=                               # Ghana Commodity Exchange
CME_API_KEY=                               # CME DataMine (optional; for USD benchmark)
PRICE_FEED_INTERVAL_SECONDS=300            # Push prices every 5 minutes

# ── Compliance / KYC ──────────────────────────────────────────────────────────
KYC_PROVIDER=reclaim                       # reclaim | verite | self
RECLAIM_APP_ID=
RECLAIM_APP_SECRET=
VERITE_ISSUER_DID=
FATF_TRAVEL_RULE_THRESHOLD_USD=10000

# ── Indexer ───────────────────────────────────────────────────────────────────
DATABASE_URL=postgresql://agroledger:password@localhost:5432/agroledger
INDEXER_START_LEDGER=0                     # Ledger to begin ingesting from

# ── USSD Gateway ──────────────────────────────────────────────────────────────
AFRICASTALKING_API_KEY=
AFRICASTALKING_USERNAME=sandbox
USSD_SERVICE_CODE=*384#

# ── App ───────────────────────────────────────────────────────────────────────
PORT=3000
LOG_LEVEL=info                             # debug | info | warn | error
```

After deploying contracts for the first time, run:

```bash
npm run deploy:testnet   # prints CONTRACT_IDs — paste them back into .env
```

---

### Inter-Contract Address Wiring

Soroban contracts have no global registry. Each contract that calls another must receive the callee's address at initialization time, stored in contract storage. This is the pattern used across AgroLedger:

**Pattern — constructor-style `initialize` function:**

```rust
// In collateral_vault/src/lib.rs
#[contractimpl]
impl CollateralVault {

    /// Called once after deployment. Stores addresses of dependent contracts.
    pub fn initialize(
        env: Env,
        admin: Address,
        compliance_registry: Address,   // ComplianceRegistry contract ID
        usdc_token: Address,            // USDC SEP-0041 contract ID
        warehouse_oracle: Address,      // WarehouseOracle contract ID
    ) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin,               &admin);
        env.storage().instance().set(&DataKey::ComplianceRegistry,  &compliance_registry);
        env.storage().instance().set(&DataKey::UsdcToken,           &usdc_token);
        env.storage().instance().set(&DataKey::WarehouseOracle,     &warehouse_oracle);
    }

    /// Reads the stored address and invokes ComplianceRegistry atomically.
    pub fn open(env: Env, crop_token: Address, borrow_amount_usdc: i128) -> BytesN<32> {
        let registry: Address = env.storage().instance()
            .get(&DataKey::ComplianceRegistry).unwrap();

        // Cross-contract call — reverts entire tx if compliance check fails
        let compliant: bool = env.invoke_contract(
            &registry,
            &Symbol::new(&env, "verify"),
            (env.current_contract_address(), Symbol::new(&env, "NG")).into_val(&env),
        );
        assert!(compliant, "compliance check failed");

        // ... vault logic
    }
}
```

**Initialization order matters.** Deploy and initialize in this sequence to avoid circular dependency failures:

```
1. PrivacyPassport        (no dependencies)
2. ComplianceRegistry     (depends on: PrivacyPassport)
3. WarehouseOracle        (no dependencies)
4. CropToken              (depends on: WarehouseOracle, ComplianceRegistry)
5. CollateralVault        (depends on: CropToken, ComplianceRegistry, WarehouseOracle)
6. CrossBorderRouter      (depends on: ComplianceRegistry)
7. CommodityAmm           (depends on: CropToken)
8. HarvestVault           (depends on: CommodityAmm, CropToken)
9. ForwardHedge           (depends on: CropToken, CollateralVault)
```

The deploy script at `scripts/deploy/testnet.js` runs initialization calls in this order automatically and writes all contract IDs to `.env`.

---

### Test Scaffolding

Soroban tests live in two places: inline `#[cfg(test)]` modules inside each contract (unit tests) and a separate `tests/` directory at the workspace root (integration tests).

**Unit test module — add to the bottom of every `lib.rs`:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
        Address, Env,
    };

    fn create_env() -> (Env, Address, Address) {
        let env   = Env::default();
        let admin = Address::generate(&env);
        let user  = Address::generate(&env);
        (env, admin, user)
    }

    #[test]
    fn test_initialize() {
        let (env, admin, _) = create_env();
        let contract_id = env.register_contract(None, CropToken);
        let client = CropTokenClient::new(&env, &contract_id);
        // test initialize ...
    }

    #[test]
    fn test_issue_requires_oracle_sig() {
        // test that issuance without valid oracle sig panics
    }

    #[test]
    fn test_transfer_blocked_without_passport() {
        // test compliance gate on transfer
    }
}
```

Run unit tests for a single contract:

```bash
cargo test -p crop-token
```

Run all contract tests with output:

```bash
cargo test --workspace -- --nocapture
```

**Integration test structure** at `tests/`:

```
tests/
├── setup.ts           # Deploy all contracts to local node, return clients
├── cropToken.test.ts  # Issue, transfer, burn flows
├── vault.test.ts      # Open vault, draw credit, liquidate at LTV > 85%
├── router.test.ts     # Cross-border paths, FATF memo, stablecoin conversion
├── compliance.test.ts # Passport registration, jurisdiction blocking
└── e2e.test.ts        # Full farmer flow: deposit → credit → remittance → repay
```

Run integration tests (requires local Stellar node via Docker):

```bash
npm run test:integration
```

---

### Oracle Sidecar Config

The `oracle/` service is a standalone Rust binary (not a Soroban contract) that runs alongside the deployed contracts. It polls external price APIs and pushes signed attestations on-chain.

**`oracle/config.toml`** — place alongside the binary or mount as a volume in Docker:

```toml
[stellar]
rpc_url            = "${STELLAR_RPC_URL}"
network_passphrase = "${STELLAR_NETWORK_PASSPHRASE}"
signing_secret     = "${ORACLE_SIGNING_SECRET}"

[contracts]
warehouse_oracle = "${WAREHOUSE_ORACLE_CONTRACT_ID}"
crop_token       = "${CROP_TOKEN_CONTRACT_ID}"

[price_feeds]
interval_seconds = 300

  [price_feeds.afex]
  url     = "${AFEX_API_URL}"
  api_key = "${AFEX_API_KEY}"
  commodities = ["MAIZE", "COCOA", "SOY", "PALM_OIL"]

  [price_feeds.gcx]
  url     = "https://api.gcx.com.gh/v1"
  api_key = "${GCX_API_KEY}"
  commodities = ["COCOA", "MAIZE"]

  [price_feeds.cme]
  url     = "https://datamine.cmegroup.com/cme/api/v1"
  api_key = "${CME_API_KEY}"
  commodities = ["MAIZE", "SOY"]              # USD benchmark only

[attestation]
multisig_threshold = 3                        # inspectors required for lots > 50 MT
inspector_public_keys = [
  "G...",   # Inspector 1
  "G...",   # Inspector 2
  "G...",   # Inspector 3
]

[logging]
level = "${LOG_LEVEL}"
```

**Running the oracle sidecar:**

```bash
# Build
cargo build -p oracle --release

# Run with config
ORACLE_SIGNING_SECRET=S... ./target/release/oracle --config oracle/config.toml

# Or via Docker
docker build -t agroledger-oracle ./oracle
docker run --env-file .env -v ./oracle/config.toml:/app/config.toml agroledger-oracle
```

**How attestation signing works:**

The oracle does not have admin authority over the contracts. It signs lot attestation payloads with `ORACLE_SIGNING_SECRET`, and the `WarehouseOracle` contract verifies that signature against a pre-registered oracle public key before accepting any price update or lot submission. Rotating the oracle key requires an admin transaction — the oracle never self-registers.

---

## Getting Started

### Prerequisites

- Node.js ≥ 18
- Rust + `cargo` (for Soroban contract compilation)
- Stellar CLI (`stellar` ≥ 0.9)
- A funded Stellar testnet account

### Installation

```bash
git clone https://github.com/agroledger/protocol
cd protocol
npm install
```

### Compile Contracts

```bash
cd contracts
cargo build --target wasm32-unknown-unknown --release
```

### Deploy to Testnet

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/crop_token.wasm \
  --source <YOUR_SECRET_KEY> \
  --network testnet

stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/collateral_vault.wasm \
  --source <YOUR_SECRET_KEY> \
  --network testnet
```

### Run Tests

```bash
cargo test                  # Soroban unit tests
npm run test:integration    # SDK integration tests against testnet
```

### Local Development

```bash
# Start a local Stellar Quickstart node
docker run --rm -it -p 8000:8000 stellar/quickstart --testnet

# Deploy all contracts to local node
npm run deploy:local

# Seed test data (warehouse operators, crop lots, price feeds)
npm run seed:local
```

---

## Contract Reference

### CropToken

```rust
pub fn issue(env, warehouse_id, lot_id, commodity, quantity_kg, oracle_sig) -> Address
pub fn transfer(env, from, to, amount)          // compliance-gated
pub fn burn(env, lot_id)                        // triggered on physical lot sale
pub fn get_lot_metadata(env, lot_id) -> LotMeta
pub fn get_price(env, lot_id) -> i128           // in USDC (7 decimal places)
```

### CollateralVault

```rust
pub fn open(env, crop_token, borrow_amount_usdc) -> VaultId
pub fn repay(env, vault_id, repay_amount)
pub fn liquidate(env, vault_id)                 // triggered at LTV > 85%
pub fn get_vault(env, vault_id) -> VaultState
```

### CrossBorderRouter

```rust
pub fn route(env, from, to, send_asset, recv_asset, amount, travel_rule_data) -> PathResult
pub fn estimate(env, send_asset, recv_asset, amount) -> Vec<PathQuote>
```

### PrivacyPassport

```rust
pub fn register(env, nullifier_hash, credential_proof, jurisdiction) -> PassportId
pub fn verify(env, passport_id, required_jurisdiction) -> bool
pub fn revoke(env, passport_id, authority_sig)
```

---

## SDK

Full SDK documentation: [docs.agroledger.io](https://docs.agroledger.io)

The SDK exposes typed clients for every contract, handles Stellar horizon interaction, manages transaction signing, and provides React hooks for frontend integration.

```typescript
// React hook example
const { cropTokens, isLoading } = useFarmerPortfolio(farmerAddress);
const { drawCredit, isPending } = useCollateralVault();
```

---

## Tokenomics

AgroLedger does not have a native governance token at launch. Protocol fees accumulate in a treasury multi-sig controlled by a founding council of warehouse operators, agri-fintech partners, and development organizations. Fee distribution:

| Recipient | Share | Source |
|---|---|---|
| Liquidity providers | 70% | AMM trading fees |
| Warehouse operators | 15% | Storage yield share |
| Protocol treasury | 10% | Routing fees |
| Insurance fund | 5% | All fee sources |

The insurance fund covers oracle failures and smart contract incidents up to the fund's balance. Governance over fee parameters and insurance fund deployment is scheduled for a DAO migration in 2026.

---

## Supported Networks

| Network | Status | RPC |
|---|---|---|
| Stellar Mainnet | Live | `https://horizon.stellar.org` |
| Stellar Testnet | Live | `https://horizon-testnet.stellar.org` |
| Stellar Futurenet | Dev | `https://horizon-futurenet.stellar.org` |

---

## Roadmap

**Q3 2025 — Testnet Launch**
- Core contracts deployed (CropToken, CollateralVault, ComplianceRegistry)
- Warehouse operator onboarding (Nigeria pilot: 3 AFEX-certified warehouses)
- USSD farmer app live in Lagos and Kano

**Q4 2025 — Mainnet Launch**
- Full contract suite live on Stellar mainnet
- SDK v1.0 release
- Ghana and Kenya regulatory sandbox approvals

**Q1 2026 — DeFi Layer**
- Commodity AMM live with maize and cocoa pairs
- Harvest Vault open to external LPs
- Diaspora remittance vault routing (UK, US, Germany corridors)

**Q2 2026 — Expansion**
- Forward hedge contracts for mill and processor buyers
- cXOF integration for Francophone West Africa
- Multi-commodity expansion: soy, palm oil, sesame

**H2 2026 — Ecosystem**
- SDK partner integrations (Releaf, Apollo Agriculture, Twiga)
- DAO governance migration
- Cross-chain bridges (Ethereum, Celo) for institutional LP access

---

## Contributing

Contributions are welcome from developers, warehouse operators, agri-fintech teams, and compliance specialists.

```bash
# Fork and clone
git clone https://github.com/<your-fork>/agroledger-protocol

# Create a feature branch
git checkout -b feat/your-feature

# Run the full test suite before submitting
cargo test && npm run test:integration

# Submit a pull request against main
```

Please read [CONTRIBUTING.md](./CONTRIBUTING.md) before submitting. All contract changes require two independent security reviews. Oracle integrations require approval from the warehouse operator council.

For security disclosures, email **security@agroledger.io** — do not open a public issue.

---

## License

MIT — see [LICENSE](./LICENSE).

---

*Built on [Stellar](https://stellar.org) and [Soroban](https://soroban.stellar.org). Designed for Africa.*