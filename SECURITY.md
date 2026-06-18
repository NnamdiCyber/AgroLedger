# Security Policy

## Reporting a Vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Send disclosures to **security@agroledger.io**.

You can also reach the security team via:

- **PGP Key**: [security@agroledger.io PGP key link]
- **Signal**: Contact a maintainer directly

### What to include

- Type of vulnerability (e.g., reentrancy, authorization bypass, integer overflow)
- Affected contract(s) or component(s)
- Steps to reproduce (contract function, parameters, network)
- Impact assessment and exploitability
- Any suggested fix (optional)

### Response Timeline

| Event | Target |
|---|---|
| Acknowledgment | 48 hours |
| Fix for critical vulnerabilities | 7 days |
| Public disclosure | After fix is deployed and verified |

## Bug Bounty

We operate a bug bounty program for contract-level vulnerabilities with payouts up to **$50,000 USDC**.

### Scope

- All Soroban smart contracts under `contracts/`
- Oracle sidecar under `oracle/`
- SDK under `sdk/`

### Out of Scope

- Issues in third-party dependencies
- Phishing or social engineering attacks
- Denial-of-service attacks on the Stellar network

### Payouts

| Severity | Payout |
|---|---|
| Critical (funds at risk, contract broken) | Up to $50,000 USDC |
| High (core feature broken) | Up to $10,000 USDC |
| Medium (non-critical feature broken) | Up to $2,500 USDC |
| Low (cosmetic or documentation) | Acknowledgment only |

Payouts are at the discretion of the AgroLedger security team and are paid in USDC on Stellar.

## Supported Versions

| Version | Supported |
|---|---|
| Mainnet (latest) | ✅ |
| Testnet (latest) | ✅ |
| Previous versions | ❌ |

## Safe Harbor

Any security research conducted in good faith and in accordance with this policy is considered authorized conduct. We will not pursue legal action against researchers who:

- Make a good-faith effort to avoid privacy violations and data destruction
- Do not exfiltrate or monetize data beyond what is necessary to demonstrate the vulnerability
- Report the vulnerability privately before public disclosure
