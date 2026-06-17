# Contract Reference

This directory contains per-contract interface references for all 9 AgroLedger Soroban smart contracts.

Each reference documents:
- **Public functions** — signatures, parameters, return values
- **Error conditions** — panics and assertions
- **Events** — emitted topics and data
- **Data types** — structs and enums used by the contract

| Contract | File | Description |
|---|---|---|
| PrivacyPassport | [privacy_passport.md](privacy_passport.md) | ZK credential verifier |
| ComplianceRegistry | [compliance_registry.md](compliance_registry.md) | Transfer allow-list + FATF |
| WarehouseOracle | [warehouse_oracle.md](warehouse_oracle.md) | Lot attestation + price feeds |
| CropToken | [crop_token.md](crop_token.md) | SEP-0041 token for warehouse lots |
| CollateralVault | [collateral_vault.md](collateral_vault.md) | Lock tokens, borrow USDC |
| CrossBorderRouter | [cross_border_router.md](cross_border_router.md) | Path payment + compliance |
| CommodityAmm | [commodity_amm.md](commodity_amm.md) | AMM with seasonal curve |
| HarvestVault | [harvest_vault.md](harvest_vault.md) | Yield-bearing vault |
| ForwardHedge | [forward_hedge.md](forward_hedge.md) | Forward purchase contracts |

## Conventions

- All amounts are in the smallest unit (e.g., USDC has 7 decimals, CropTokens represent kg)
- `Symbol` parameters are short strings like `"MAIZE"`, `"NG"`, `"USDC"`
- `u64` timestamps are Unix epoch seconds
- `Address` parameters are Stellar contract or account addresses (G... or C... format)
- Panic messages are descriptive assertions — Soroban catches panics and reverts the transaction
