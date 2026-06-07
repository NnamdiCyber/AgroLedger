
# AgroLedger — 16-Day Development Sprint to 50% Completion

**Goal:** From zero source code to a deployable, tested foundation with core contracts, SDK, tooling, and contributor documentation. After this sprint, the protocol should be functional on Stellar testnet with meaningful test coverage and clear contribution paths.

---

## Sprint Structure

| Phase | Days | Focus | Output |
|-------|------|-------|--------|
| **Phase 1** | 1–4 | Project scaffold + Zero-dependency contracts | Monorepo structure, CI, `PrivacyPassport`, `ComplianceRegistry`, `WarehouseOracle` — compiled, tested |
| **Phase 2** | 5–8 | Core RWA + Credit contracts | `CropToken`, `CollateralVault`, `CrossBorderRouter` — compiled, unit tested, basic integration tests |
| **Phase 3** | 9–12 | DeFi + Hedge contracts | `CommodityAmm`, `HarvestVault`, `ForwardHedge` — compiled, unit tested |
| **Phase 4** | 13–16 | SDK, Deploy tooling, Docs, Polish | TypeScript SDK, deploy scripts, oracle sidecar skeleton, contributor docs, end-to-end integration test |

**50% completion definition:**
- All 9 contracts exist, compile, and have unit tests passing
- Inter-contract calls wired (address injection pattern)
- Minimal SDK with typed clients for all contracts
- Deploy scripts for local/testnet (one-command deploy)
- End-to-end flow: issue CropToken → open vault → borrow → repay → transfer
- Contribution guide, architecture doc, and test guide written

---

## Phase 1 — Scaffold & Zero-Dependency Contracts (Days 1–4)

### Day 1 — Monorepo Setup

| Task | Detail |
|------|--------|
| Create `Cargo.toml` workspace | Workspace with resolver = "2", all 9 contract paths + oracle, pinned `soroban-sdk = "21.0.0"` |
| Create `package.json` root | Workspaces: sdk, apps/ussd, apps/warehouse-portal, apps/buyer-portal, indexer |
| Create `.env.example` | Copy from README — all contract IDs, API keys, stablecoin addresses |
| Create `.gitignore` | `target/`, `node_modules/`, `.env`, `*.wasm`, `.stellar/` |
| Create `rust-toolchain.toml` | Pin `nightly-2025-03-01` or stable channel that Soroban 21 requires |
| Create `rustfmt.toml` | Project-wide formatting rules |
| Scaffold all 9 contract directories | Each gets `Cargo.toml` (cdylib), `src/lib.rs` (empty `#![no_std]` skeleton with contract trait) |
| Scaffold SDK directory | `sdk/package.json`, `src/index.ts` (stub), `tsconfig.json` |
| Install Rust toolchain | `rustup target add wasm32-unknown-unknown` |
| Verify `cargo build --workspace` | All 9 crates compile to WASM (empty contracts) — **first green build** |

### Day 2 — PrivacyPassport Contract

| Task | Detail |
|------|--------|
| `contracts/privacy_passport/src/lib.rs` | Contract entry point + `initialize(admin)` |
| `contracts/privacy_passport/src/passport.rs` | `register(nullifier_hash, credential_proof, jurisdiction) -> PassportId`, `verify(passport_id, required_jurisdiction) -> bool` |
| `contracts/privacy_passport/src/revocation.rs` | `revoke(passport_id, authority_sig)` — authority-signed revocation, emits event |
| Storage design | `DataKey::Passport(PassportId) -> PassportState { nullifier_hash, jurisdiction, active, registered_at }` |
| Unit tests | `test_register`, `test_verify_valid`, `test_verify_revoked`, `test_verify_wrong_jurisdiction`, `test_revoke_unauthorized` |
| Events | Emit `PassportRegistered`, `PassportRevoked` |

### Day 3 — ComplianceRegistry Contract ✅

| Task | Detail |
|------|--------|
| `contracts/compliance_registry/src/lib.rs` | `initialize(admin, privacy_passport)` |
| `contracts/compliance_registry/src/allowlist.rs` | Jurisdiction allow-list: `add_jurisdiction(admin, code)`, `remove_jurisdiction(admin, code)`, `is_allowed(jurisdiction) -> bool` |
| `contracts/compliance_registry/src/fatf.rs` | `validate_travel_rule(amount, jurisdiction) -> bool` — checks amount > $10K threshold requires valid memo |
| Cross-contract calls | Calls `PrivacyPassport.verify()` during compliance check |
| `verify(origin, jurisdiction) -> bool` | Main entry: checks passport validity + jurisdiction allow-list |
| Unit tests | `test_allowlist_add_remove`, `test_verify_passport_required`, `test_verify_blocked_jurisdiction`, `test_travel_rule_threshold` |
| Events | Emit `ComplianceCheck`, `JurisdictionAdded`, `JurisdictionRemoved` |

### Day 4 — WarehouseOracle Contract + CI Setup

| Task | Detail |
|------|--------|
| `contracts/warehouse_oracle/src/lib.rs` | `initialize(admin, oracle_pubkey, inspectors)` |
| `contracts/warehouse_oracle/src/attestation.rs` | `submit_lot(warehouse_id, lot_id, commodity, quantity_kg, inspector_sigs)`, multi-sig verification (≥3 sigs for >50 MT) |
| `contracts/warehouse_oracle/src/price_feed.rs` | `push_price(commodity, price_usdc, timestamp)`, `get_price(commodity) -> (price, timestamp)` — single-source push with oracle signature |
| Unit tests | `test_submit_lot`, `test_submit_lot_requires_multisig`, `test_push_price_oracle_sig`, `test_get_price` |
| GitHub Actions CI | `.github/workflows/ci.yml` — `cargo build --workspace` + `cargo test --workspace` on push/PR to `main` |
| Events | Emit `LotSubmitted`, `PriceUpdated` |

**End of Phase 1 — Checkpoint:** 3 contracts fully implemented and tested. Project compiles. CI green. `cargo test --workspace` passes.

---

## Phase 2 — Core RWA & Credit Contracts (Days 5–8)

### Day 5 — CropToken Contract

| Task | Detail |
|------|--------|
| `contracts/crop_token/src/lib.rs` | `initialize(admin, warehouse_oracle, compliance_registry)` |
| `contracts/crop_token/src/metadata.rs` | Lot metadata storage: `LotMeta { commodity, quantity_kg, warehouse_id, oracle_attestation, expiry, price }` |
| `contracts/crop_token/src/transfer.rs` | `transfer(from, to, amount)` — compliance-gated: calls `ComplianceRegistry.verify()` before executing `token_admin.transfer()` |
| `issue(warehouse_id, lot_id, commodity, quantity_kg, oracle_sig) -> Address` | Verifies oracle signature via cross-contract call to `WarehouseOracle`, mints SEP-0041 token |
| `burn(lot_id)` | Destroys token on physical lot sale, admin-only |
| `get_lot_metadata(lot_id) -> LotMeta` | Public read function |
| Unit tests | `test_issue_valid_sig`, `test_issue_invalid_sig_panics`, `test_transfer_compliance_gated`, `test_burn`, `test_get_metadata` |
| Event | Emit `CropTokenIssued`, `CropTokenBurned` |

### Day 6 — CollateralVault Contract

| Task | Detail |
|------|--------|
| `contracts/collateral_vault/src/lib.rs` | `initialize(admin, compliance_registry, usdc_token, warehouse_oracle)` |
| `contracts/collateral_vault/src/vault.rs` | `open(crop_token, borrow_amount_usdc) -> VaultId` — compliance check, lock CropToken, mint USDC. `repay(vault_id, amount)` — repay USDC, unlock CropToken. `liquidate(vault_id)` — triggered if LTV > 85% |
| `contracts/collateral_vault/src/ltv.rs` | Loan-to-value calculation: `compute_ltv(crop_token_amount, crop_price, debt_usdc) -> u32` — queries `WarehouseOracle.get_price()`, triggers liquidation if > 85% |
| Cross-contract calls | `ComplianceRegistry.verify()`, `WarehouseOracle.get_price()`, `CropToken.transfer()` |
| Unit tests | `test_open_vault`, `test_repay_full`, `test_repay_partial`, `test_liquidate_healthy_fails`, `test_liquidate_unhealthy`, `test_compliance_reverts` |

### Day 7 — CrossBorderRouter Contract

| Task | Detail |
|------|--------|
| `contracts/cross_border_router/src/lib.rs` | `initialize(admin, compliance_registry)` |
| `contracts/cross_border_router/src/router.rs` | `route(from, to, send_asset, recv_asset, amount, travel_rule_data) -> PathResult` — Stellar path-payment emulation: validate compliance + travel rule, execute asset swap via SDEX or internal pool, deduct 0.15% fee |
| `contracts/cross_border_router/src/stablecoins.rs` | Asset ID constants + registration: USDC, cNGN, cXOF, cGHS, cKES. `register_asset(admin, symbol, contract_id)`, `get_asset(symbol) -> Address` |
| `estimate(send_asset, recv_asset, amount) -> Vec<PathQuote>` | Quote simulation (no state change) |
| Unit tests | `test_route_same_asset`, `test_route_cross_border`, `test_route_blocked_jurisdiction`, `test_route_travel_rule_threshold`, `test_register_asset` |
| Event | Emit `RouteExecuted`, `AssetRegistered` |

### Day 8 — Integration Wiring & First End-to-End Test

| Task | Detail |
|------|--------|
| `tests/setup.rs` | Rust integration test helper: deploys all 6 Phase 1–2 contracts to test env, calls `initialize()` in correct order |
| `tests/cropToken.test.rs` | Full flow: deploy → register passport → issue CropToken → transfer → burn |
| `tests/vault.test.rs` | Full flow: issue CropToken → open vault → draw USDC → repay → verify tokens unlocked |
| `tests/router.test.rs` | Full flow: register assets → route payment → verify balances |
| `tests/compliance.test.rs` | Full flow: register passport → verify → revoke → verify fails |
| Fix inter-contract wiring bugs | Any issues uncovered by integration tests |

**End of Phase 2 — Checkpoint:** 6 contracts fully implemented, unit tested, passing integration tests in Rust test harness. Core farmer flow works: deposit → tokenize → borrow → repay → transfer. `cargo test --workspace` green.

---

## Phase 3 — DeFi & Hedge Contracts (Days 9–12)

### Day 9 — CommodityAMM Contract

| Task | Detail |
|------|--------|
| `contracts/commodity_amm/src/lib.rs` | `initialize(admin, crop_token)` |
| `contracts/commodity_amm/src/curve.rs` | Custom bonding curve: `calculate_swap(commodity, amount_in, reserve_in, reserve_out, timestamp) -> amount_out` — incorporates seasonal price variance derived from historical data (simplified: linear interpolation between pre-harvest discount / post-harvest premium) |
| `contracts/commodity_amm/src/pool.rs` | `create_pool(commodity)` — LP token issuance. `swap(commodity, amount_in, min_amount_out) -> amount_out` — swap execution. `add_liquidity(commodity, amount_crop, amount_usdc) -> lp_tokens`. `remove_liquidity(lp_tokens) -> (crop, usdc)` |
| Unit tests | `test_swap_basic`, `test_swap_price_impact`, `test_seasonal_curve_variance`, `test_add_remove_liquidity`, `test_swap_slippage_protection` |
| Event | Emit `SwapExecuted`, `LiquidityAdded`, `LiquidityRemoved` |

### Day 10 — HarvestVault Contract

| Task | Detail |
|------|--------|
| `contracts/harvest_vault/src/lib.rs` | `initialize(admin, crop_token, commodity_amm, usdc_token)` |
| `contracts/harvest_vault/src/vault.rs` | `deposit(crop_token_amount) -> hCT_tokens` — user deposits CropTokens, receives yield-bearing receipt tokens. `withdraw(hCT_tokens) -> (crop_tokens, usdc_yield)` — redeem principal + yield |
| `contracts/harvest_vault/src/yield.rs` | Yield accrual: `accrue_yield()` — harvest AMM LP fees + simulated storage yield. `get_apy() -> u32` — current APY in basis points. `rebalance()` — trigger yield compounding |
| Unit tests | `test_deposit_withdraw`, `test_yield_accrual`, `test_multiple_depositors`, `test_rebalance` |
| Event | Emit `Deposited`, `Withdrawn`, `YieldAccrued` |

### Day 11 — ForwardHedge Contract

| Task | Detail |
|------|--------|
| `contracts/forward_hedge/src/lib.rs` | `initialize(admin, crop_token, collateral_vault)` |
| `contracts/forward_hedge/src/hedge.rs` | `place_hedge(commodity, quantity, max_price, expiry) -> HedgeId` — sealed-bid: buyer commits to max price, contract stores commitment hash. `accept_hedge(farmer, hedge_id)` — farmer accepts bid terms |
| `contracts/forward_hedge/src/settlement.rs` | `settle(hedge_id)` — at expiry: physical settlement (trigger CropToken.burn) or USDC settlement (transfer difference between spot and agreed price). `cancel(hedge_id)` — cancel before expiry with penalty |
| Sealed-bid mechanism | Buyer submits `hash(price + salt)`, later reveals `(price, salt)` — contract verifies hash match |
| Unit tests | `test_place_hedge`, `test_accept_hedge`, `test_settle_physical`, `test_settle_usdc`, `test_cancel_before_expiry`, `test_reveal_mismatch_panics` |
| Event | Emit `HedgePlaced`, `HedgeAccepted`, `HedgeSettled`, `HedgeCancelled` |

### Day 12 — Phase 3 Integration Tests

| Task | Detail |
|------|--------|
| Expand `tests/setup.rs` | Deploy all 9 contracts in correct dependency order |
| `tests/amm.test.rs` | Full flow: create pool → add liquidity → swap with seasonal curve → remove liquidity |
| `tests/harvest.test.rs` | Full flow: deposit CropTokens → wait (simulate yield) → withdraw with yield |
| `tests/hedge.test.rs` | Full flow: place sealed-bid hedge → accept → settle |
| `tests/e2e.test.rs` | Full farmer-to-buyer flow: deposit lot → issue CropToken → open vault → borrow USDC → place forward hedge → settle → repay vault |
| Performance sanity | Verify gas/op counts are within Soroban limits per transaction |

**End of Phase 3 — Checkpoint:** All 9 contracts fully implemented and unit tested. End-to-end Rust integration test passes for full farmer-to-buyer flow. `cargo test --workspace` green.

---

## Phase 4 — SDK, Tooling, Docs & Polish (Days 13–16)

### Day 13 — TypeScript SDK Core

| Task | Detail |
|------|--------|
| `sdk/src/types/index.ts` | TypeScript types mirroring contract types: `LotMeta`, `VaultState`, `PathQuote`, `PassportState`, `HedgeState`, all enums |
| `sdk/src/clients/cropToken.ts` | Typed client: `issue()`, `transfer()`, `burn()`, `getLotMetadata()`, `getPrice()` — uses `stellar-sdk` to build/submit Soroban contract calls |
| `sdk/src/clients/collateralVault.ts` | `open()`, `repay()`, `liquidate()`, `getVault()` |
| `sdk/src/clients/privacyPassport.ts` | `register()`, `verify()`, `revoke()` |
| `sdk/src/clients/complianceRegistry.ts` | `verify()`, `isAllowed()` |
| `sdk/src/index.ts` | Main `AgroLedger` class — accepts `{ network, signer }`, exposes all contract clients as properties, handles wallet/Stellar RPC setup |
| `sdk/package.json` | Deps: `@stellar/stellar-sdk`, `soroban-client`. Build: `tsc`. |
| `sdk/tsconfig.json` | Strict mode, ES2022 target, `dist/` output |

### Day 14 — SDK Remaining Clients + React Hooks

| Task | Detail |
|------|--------|
| `sdk/src/clients/crossBorderRouter.ts` | `route()`, `estimate()`, `registerAsset()`, `getAsset()` |
| `sdk/src/clients/commodityAmm.ts` | `createPool()`, `swap()`, `addLiquidity()`, `removeLiquidity()` |
| `sdk/src/clients/harvestVault.ts` | `deposit()`, `withdraw()`, `getApy()` |
| `sdk/src/clients/forwardHedge.ts` | `placeHedge()`, `acceptHedge()`, `settle()`, `cancel()` |
| `sdk/src/clients/warehouseOracle.ts` | `submitLot()`, `pushPrice()`, `getPrice()` |
| `sdk/src/hooks/useFarmerPortfolio.ts` | React hook: fetch farmer's CropTokens, vaults, active hedges |
| `sdk/src/hooks/useCollateralVault.ts` | `drawCredit()`, `repay()` — wraps client calls in React mutation pattern |
| `sdk/src/hooks/useForwardHedge.ts` | `placeHedge()`, `acceptHedge()` — React mutations |
| `sdk/tests/clients.test.ts` | Unit tests with mocked Stellar RPC (or integration tests against local node) |
| `npm run test` passes | SDK test suite green |

### Day 15 — Deploy Tooling + Oracle Sidecar

| Task | Detail |
|------|--------|
| `scripts/deploy/local.js` | Deploy all 9 contracts to local Stellar Quickstart node in correct order, call `initialize()` with correct addresses, write `.env` |
| `scripts/deploy/testnet.js` | Same for Stellar testnet — prompts for funded account, writes `.env` |
| `scripts/deploy/testnet.sh` | Shell version for pure CLI workflow |
| `scripts/seed/warehouses.ts` | Seed 3 test warehouse operators |
| `scripts/seed/lots.ts` | Seed test crop lots + price feeds |
| `scripts/seed/passports.ts` | Seed test KYC credentials |
| `oracle/src/main.rs` | Oracle sidecar skeleton: reads `config.toml`, initializes Stellar RPC client, background loop for price polling (AFEX/GCX/CME stubs), lot attestation signing endpoint |
| `oracle/src/price/mod.rs` | Price feed poller: trait + stub implementations for AFEX, GCX, CME APIs |
| `oracle/src/attestation/mod.rs` | Attestation signing: loads secret key, signs lot payload, submits to `WarehouseOracle` contract |
| `oracle/config.toml` | From README scaffold — template config file |
| `npm run deploy:local` works | One command deploys all contracts to local node |

### Day 16 — Contributor Docs, Polish & Launch

| Task | Detail |
|------|--------|
| `CONTRIBUTING.md` | How to set up local dev environment, coding standards, PR process, security disclosure policy |
| `ARCHITECTURE.md` | High-level architecture diagram (ASCII from README), contract dependency graph, data flow for key operations |
| `docs/contracts/` | Per-contract reference: all public functions, parameters, error codes, events |
| `docs/compliance/fatf.md` | FATF Travel Rule integration guide — how memos work, encryption patterns |
| `docs/oracle/deployment.md` | Warehouse oracle deployment guide — hardware requirements, key rotation, monitoring |
| `docs/sdk/README.md` | SDK quickstart: install, `AgroLedger` client init, basic usage examples |
| `docs/sdk/react-hooks.md` | React hook usage with code examples |
| Final audit pass | Review all `unwrap()` calls — replace with proper error handling or `expect()` with descriptive messages. Ensure all `require_auth()` calls are present. Verify no secrets in code. |
| `cargo test --workspace` | Final green build |
| `npm run test` | Final green SDK test |
| Tag `v0.5.0` | Milestone release tag |
| Write GitHub issue templates | Bug report, feature request, security disclosure |

**End of Phase 4 — Checkpoint:** All 9 contracts live. SDK published-ready. One-command deploy. Contributor docs complete. End-to-end flow demoable. `cargo test --workspace` + `npm run test` green.

---

## Summary: What "50% Complete" Means After 16 Days

| Dimension | Delivered | Not Yet (Remaining 50%) |
|-----------|-----------|------------------------|
| **Contracts** | All 9 exist, compile, unit tested, integration tested | Mainnet-grade auditing, formal verification, fuzz testing |
| **SDK** | Typed clients for all contracts, React hooks | Production error handling, retry logic, analytics |
| **Apps** | — | USSD gateway, Warehouse Portal, Buyer Portal (3 apps) |
| **Indexer** | — | GraphQL schema, event ingestion workers, PostgreSQL |
| **Oracle** | Skeleton with price polling stubs | Production API integrations, monitoring, alerting |
| **Deploy** | One-command local + testnet deploy | Mainnet deploy with multi-sig governance |
| **Compliance** | Core contract logic + FATF rule | Real KYC provider integration, regulatory filings |
| **Docs** | Architecture, contributor guide, per-contract refs | Full SDK reference, video tutorials, API docs site |
| **Security** | Basic auth checks, no panics | Third-party audit, bug bounty, insurance fund |

---

## Key Principles

1. **Working software over polish** — every day ends with compilable, tested code. No "I'll fix it later."
2. **Contract-first** — Rust contracts are the source of truth. SDK, indexer, and apps follow.
3. **Tests are not optional** — every `#[test]` proves a requirement. No contract is "done" without >=80% line coverage on its core logic.
4. **Inter-contract wiring from day 1** — no contract exists in isolation. Integration tests catch address/interface mismatches early.
5. **Contributor readiness** — after day 16, a new developer should be able to clone, `cargo build`, `npm install`, and run tests in under 5 minutes.
