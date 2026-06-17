---
name: Bug Report
about: Report a bug in AgroLedger contracts, SDK, or tooling
title: '[BUG] '
labels: bug
assignees: ''
---

## Bug Description

A clear and concise description of the bug.

## Reproduction Steps

1. Deploy contracts to `[network: local/testnet]`
2. Call `[contract.function()]` with `[parameters]`
3. Observe `[error/behavior]`

## Expected Behavior

What should have happened.

## Actual Behavior

What actually happened (include error message, panic, or unexpected output).

## Environment

- Network: `local` / `testnet` / `mainnet`
- Rust version: `rustc --version`
- Soroban SDK version: `21.0.0`
- Node version: `node --version`
- Contract version/commit: `git rev-parse HEAD`

## Additional Context

- Full test output or transaction hash
- Relevant contract IDs
- Screenshots if applicable

## Severity

- `critical` — funds at risk, contract broken
- `high` — core feature broken
- `medium` — non-critical feature broken
- `low` — cosmetic or documentation
