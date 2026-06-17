# AgroLedger Architecture

## System Overview

AgroLedger is a composable protocol for on-chain trade finance of African agricultural commodities, built on Stellar/Soroban. The protocol consists of 9 smart contracts, a TypeScript SDK, an oracle sidecar service, and end-user applications.

```
                        END-USER APPLICATIONS
   USSD (2G/feature phone)  ·  Warehouse Portal  ·  Buyer B2B App
            AgroLedger SDK  ·  On-chain Indexer
                              │
                      PAYMENTS & SETTLEMENT
   Diaspora Remittance Vault  ·  Instant Credit Disbursement
   Harvest Auto-Repayment  ·  Multi-stablecoin (USDC · cNGN · cXOF)
                              │
                         DeFi LAYER
   Commodity AMM (custom bonding curve)  ·  Harvest Yield Vault
   Forward Hedge Contracts  ·  USDC/XLM/cXOF Liquidity Pools
                              │
                     SOROBAN CONTRACT CORE
   CropToken (SEP-0041)  ·  CollateralVault  ·  ComplianceRegistry
   CrossBorderRouter  ·  WarehouseOracle  ·  PrivacyPassport
            │                                          │
   REAL-WORLD ASSETS                    PRIVACY & COMPLIANCE
  Certified Warehouse                    ZK Credential  ·  KYC
  Price Oracle (AFEX)                    FATF Travel Rule
  Lot Certification                      SEC Sandbox (NG/GH/KE)
```

## Contract Dependency Graph

```
PrivacyPassport  (no dependencies)
       │
       ▼
ComplianceRegistry  (depends on: PrivacyPassport)
       │
       ├────────────────────────────────────────────┐
       ▼                                            ▼
CropToken  (depends on: WarehouseOracle,      CrossBorderRouter
           ComplianceRegistry)                 (depends on: ComplianceRegistry)
       │
       ├────────────────────────────┐
       ▼                            ▼
CollateralVault               CommodityAmm
(depends on: CropToken,        (depends on: CropToken)
 ComplianceRegistry,                  │
 WarehouseOracle)                     ▼
       │                        HarvestVault
       ▼                       (depends on: CommodityAmm,
ForwardHedge                              CropToken)
(depends on: CropToken,
 CollateralVault)
```

### Initialization Order

Contracts must be deployed and initialized in this exact order:

1. **PrivacyPassport** — no dependencies
2. **ComplianceRegistry** — depends on PrivacyPassport
3. **WarehouseOracle** — no dependencies
4. **CropToken** — depends on WarehouseOracle, ComplianceRegistry
5. **CollateralVault** — depends on CropToken, ComplianceRegistry, WarehouseOracle
6. **CrossBorderRouter** — depends on ComplianceRegistry
7. **CommodityAmm** — depends on CropToken
8. **HarvestVault** — depends on CommodityAmm, CropToken
9. **ForwardHedge** — depends on CropToken, CollateralVault

## Data Flow: Key Operations

### 1. Farmer Deposits Crop → Receives CropToken

```
Farmer → WarehouseOperator (off-chain)
  ↓
WarehouseOracle.submitLot(warehouse_id, lot_id, commodity, qty, inspector_sigs)
  ↓ Multi-sig verification (≥3 for >50 MT)
LotSubmitted event emitted
  ↓
CropToken.issue(warehouse_id, lot_id, commodity, qty, oracle_sig)
  ↓ Verifies oracle signature via WarehouseOracle.verify_lot()
CropTokenIssued event emitted
  ↓
Farmer receives CropToken (balance increased)
```

### 2. Farmer Borrows USDC Against CropToken

```
Farmer → CollateralVault.open(user, crop_token, commodity, passport_id,
                               jurisdiction, collateral_amount, borrow_amount)
  ↓
ComplianceRegistry.verify(passport_id, jurisdiction)
  ↓ Calls PrivacyPassport.verify()
  ↓
CropToken.transfer(user → vault, collateral_amount)
  ↓
USDC.transfer(vault → user, borrow_amount)
VaultOpened event emitted
  ↓
Farmer receives USDC loan
```

### 3. Farmer Repays Loan → Receives CropToken Back

```
Farmer → CollateralVault.repay(user, vault_id, amount)
  ↓
USDC.transfer(user → vault, amount)
  ↓
If fully repaid:
  CropToken.transfer(vault → user, collateral_amount)
VaultRepaid event emitted
  ↓
Farmer receives CropTokens back
```

### 4. Liquidation (LTV > 85%)

```
Liquidator → CollateralVault.liquidate(liquidator, vault_id)
  ↓
WarehouseOracle.get_price(commodity) → price
  ↓ compute_ltv(): if > 85%
USDC.transfer(liquidator → vault, debt_amount)
CropToken.transfer(vault → liquidator, collateral_amount)
VaultLiquidated event emitted
```

### 5. Cross-Border Payment

```
Sender → CrossBorderRouter.route(from, to, send_asset, recv_asset,
                                  amount, travel_rule_data)
  ↓
ComplianceRegistry.verify(passport_id, jurisdiction)
ComplianceRegistry.validate_travel_rule(amount, jurisdiction)
  ↓
send_asset.transfer(from → router, amount)
  ↓ 0.15% fee deducted
recv_asset.transfer(router → to, amount_after_fee)
RouteExecuted event emitted
```

### 6. AMM Swap

```
User → CommodityAmm.swap(user, commodity, amount_in, min_amount_out, sell_crop)
  ↓
token_in.transfer(user → amm, amount_in)
  ↓ calculate_swap(): bonding curve + seasonal factor
token_out.transfer(amm → user, amount_out)
SwapExecuted event emitted
```

### 7. Forward Hedge

```
Buyer → ForwardHedge.place_hedge(buyer, commodity, qty, commitment, expiry)
  ↓ Stores hash(price + salt) as commitment
HedgePlaced event emitted
  ↓
Farmer → ForwardHedge.accept_hedge(hedge_id, farmer)
HedgeAccepted event emitted
  ↓ (at expiry)
ForwardHedge.reveal(hedge_id, price, salt)
  ↓ Verifies hash matches commitment
ForwardHedge.settle(hedge_id, settlement_type, caller)
  ↓ Physical: CropToken.transfer(farmer → buyer, qty)
  ↓ Cash:     CropToken.transfer(buyer → farmer, settlement_amount)
HedgeSettled event emitted
```

## Storage Architecture

Each contract uses Soroban's persistent and instance storage:

- **Instance storage** — admin addresses, contract dependencies (addresses), counters, and lookup data
- **Persistent storage** — user balances (CropTokens, LP tokens, hCT tokens)

DataKey enum pattern is used consistently across all contracts for type-safe storage access.

## Event System

All state-changing operations emit Soroban events. Key events:

| Event | Contract | Topics |
|---|---|---|
| PassportRegistered | PrivacyPassport | (PassportRegistered, passport_id) |
| PassportRevoked | PrivacyPassport | (PassportRevoked, passport_id) |
| ComplianceCheck | ComplianceRegistry | (ComplianceCheck, passport_id) |
| JurisdictionAdded | ComplianceRegistry | (JurisdictionAdded, code) |
| JurisdictionRemoved | ComplianceRegistry | (JurisdictionRemoved, code) |
| LotSubmitted | WarehouseOracle | (LotSubmitted, lot_num) |
| PriceUpdated | WarehouseOracle | (PriceUpdated, commodity) |
| CropTokenIssued | CropToken | (CropTokenIssued, lot_id) |
| CropTokenBurned | CropToken | (CropTokenBurned, lot_id) |
| Transfer | CropToken | (Transfer, from, to) |
| VaultOpened | CollateralVault | (VaultOpened, vault_id) |
| VaultRepaid | CollateralVault | (VaultRepaid, vault_id) |
| VaultLiquidated | CollateralVault | (VaultLiquidated, vault_id) |
| RouteExecuted | CrossBorderRouter | (RouteExecuted, route_id, from) |
| AssetRegistered | CrossBorderRouter | (AssetRegistered, symbol) |
| SwapExecuted | CommodityAmm | (SwapExecuted, commodity, user) |
| LiquidityAdded | CommodityAmm | (LiquidityAdded, commodity, user) |
| LiquidityRemoved | CommodityAmm | (LiquidityRemoved, commodity, user) |
| Deposited | HarvestVault | (Deposited, user) |
| Withdrawn | HarvestVault | (Withdrawn, user) |
| YieldAccrued | HarvestVault | (YieldAccrued,) |
| HedgePlaced | ForwardHedge | (HedgePlaced, hedge_id) |
| HedgeAccepted | ForwardHedge | (HedgeAccepted, hedge_id) |
| HedgeSettled | ForwardHedge | (HedgeSettled, hedge_id) |
| HedgeCancelled | ForwardHedge | (HedgeCancelled, hedge_id) |

## Fee Model

| Operation | Fee | Recipient |
|---|---|---|
| Cross-border route | 0.15% (15 bps) | Protocol treasury |
| AMM swap | 0.30% (30 bps) | LP providers |
| Hedge cancellation (accepted) | 10% penalty | Counterparty |

## Directory Layout

```
agroledger-protocol/
├── contracts/                     # Soroban smart contracts
│   ├── privacy_passport/          # ZK credential verifier
│   ├── compliance_registry/       # Transfer allow-list + FATF
│   ├── warehouse_oracle/          # Lot attestation + price feeds
│   ├── crop_token/                # SEP-0041 token for warehouse lots
│   ├── collateral_vault/          # Lock tokens, borrow USDC
│   ├── cross_border_router/       # Path payment + compliance
│   ├── commodity_amm/             # AMM with seasonal curve
│   ├── harvest_vault/             # Yield-bearing vault
│   ├── forward_hedge/             # Forward purchase contracts
│   └── integration-tests/         # Cross-contract integration tests
├── sdk/                           # TypeScript developer SDK
│   ├── src/
│   │   ├── clients/               # Per-contract typed clients
│   │   ├── hooks/                 # React hooks
│   │   └── types/                 # Shared TypeScript types
│   └── tests/
├── oracle/                        # Rust sidecar binary
│   ├── src/
│   │   ├── price/                 # AFEX/GCX/CME price polling
│   │   └── attestation/           # Lot certification signing
│   └── config.toml
├── scripts/                       # Deploy + seed scripts
│   ├── deploy/
│   └── seed/
├── docs/                          # Documentation
│   ├── contracts/                 # Per-contract references
│   ├── compliance/                # FATF Travel Rule guide
│   ├── oracle/                    # Oracle deployment guide
│   └── sdk/                       # SDK usage docs
└── apps/                          # End-user applications (scaffolded)
    ├── ussd/
    ├── warehouse-portal/
    └── buyer-portal/
```

## Security Model

- **Authentication**: All state-changing functions require `require_auth()` from the caller
- **Authorization**: Admin-only functions check stored admin address against caller
- **Cross-contract calls**: Callee addresses stored at init time, invoked via `env.invoke_contract()`
- **Compliance gating**: `ComplianceRegistry.verify()` called atomically inside vault operations and transfers
- **Oracle trust**: Multi-sig inspector verification for lots >50 MT; price feed requires oracle key authentication
- **No panics**: All error paths use `assert!` with descriptive messages (Soroban catches panics and reverts)
