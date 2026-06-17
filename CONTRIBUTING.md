# Contributing to AgroLedger

## Table of Contents

- [Development Setup](#development-setup)
- [Project Architecture](#project-architecture)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [Security Disclosures](#security-disclosures)
- [Community Guidelines](#community-guidelines)

## Development Setup

### Prerequisites

- Node.js >= 18.0.0, npm >= 9.0.0
- Rust stable (see `rust-toolchain.toml`) with `wasm32-unknown-unknown` target
- Stellar CLI >= 0.9 (`stellar`)
- Docker (for local Stellar Quickstart node)
- A funded Stellar testnet account

### Clone and Install

```bash
git clone https://github.com/agroledger/protocol
cd protocol
npm install
rustup target add wasm32-unknown-unknown
```

### Environment

```bash
cp .env.example .env
# Edit .env with your Stellar secret key and RPC endpoints
```

### Build Contracts

```bash
cargo build --workspace
```

### Run Tests

```bash
cargo test --workspace            # Rust unit + integration tests
npm run test                      # SDK tests
```

### Local Development Node

```bash
docker run --rm -it -p 8000:8000 stellar/quickstart --testnet
npm run deploy:local
npm run seed:local
```

## Project Architecture

The project is organized as a Rust workspace with Soroban smart contracts and a TypeScript SDK:

- `contracts/` — 9 Soroban smart contracts, each a `cdylib` crate
- `sdk/` — TypeScript SDK with typed clients for every contract
- `oracle/` — Rust sidecar binary for price feeds and lot attestation
- `scripts/` — Deployment, seeding, and migration scripts
- `docs/` — Per-contract references, compliance guides, deployment docs

See [ARCHITECTURE.md](./ARCHITECTURE.md) for a detailed overview.

## Coding Standards

### Rust

- Format: `cargo fmt` (4-space indent, 100-char width — see `rustfmt.toml`)
- Lint: `cargo clippy --workspace -- -D warnings`
- `#![no_std]` required for all contract crates
- No `unwrap()` in production paths — use `expect("descriptive message")` or proper error handling
- All public functions must have `require_auth()` where appropriate
- Events must be emitted for all state-changing operations
- Storage keys use the `DataKey` enum pattern

### TypeScript

- Format: `prettier` (default config)
- Strict TypeScript mode enabled
- All contract client methods must accept and return typed interfaces from `src/types/index.ts`
- React hooks must handle loading, error, and empty states

### Commit Messages

```
<type>(<scope>): <description>

feat(contract): add forward hedge settlement
fix(sdk): correct crop token balance decoding
docs(contracts): add collateral vault reference
```

Types: `feat`, `fix`, `docs`, `test`, `refactor`, `chore`.

## Testing

- Unit tests live in `#[cfg(test)]` modules within each contract's `lib.rs`
- Integration tests are in `contracts/integration-tests/tests/`
- Every public function must have at least one unit test
- Every error path must be tested (expect panic or return value)
- SDK tests live in `sdk/tests/`
- Run all tests before submitting: `cargo test --workspace && npm run test`

## Pull Request Process

1. Fork the repository and create a feature branch from `main`
2. Make your changes, following coding standards
3. Add or update tests covering your changes
4. Run `cargo test --workspace` and `npm run test` — all must pass
5. Run `cargo clippy --workspace -- -D warnings`
6. Submit a PR against `main` with a clear description of changes
7. All contract changes require two independent security reviews
8. Oracle integration changes require approval from the warehouse operator council

### PR Title Format

```
<type>(<scope>): <description>
```

### Checklist

- [ ] Tests added/updated and passing
- [ ] Documentation updated (docs, README, or inline)
- [ ] No `unwrap()` added in production code
- [ ] `require_auth()` present on all state-changing functions
- [ ] Events emitted for state changes
- [ ] `cargo clippy` clean
- [ ] No secrets or keys committed

## Security Disclosures

**Do not open a public GitHub issue for security vulnerabilities.**

Send disclosures to **security@agroledger.io**. We will acknowledge within 48 hours and aim to ship a fix within 7 days.

Our bug bounty program covers contract-level vulnerabilities with payouts up to $50,000 USDC. See [SECURITY.md] for details.

## Community Guidelines

- Be respectful and constructive
- Keep discussions focused on technical merit
- Help review others' PRs
- First-time contributors welcome — look for issues labeled `good first issue`

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
