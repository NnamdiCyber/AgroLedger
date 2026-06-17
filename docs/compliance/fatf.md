# FATF Travel Rule Integration Guide

## Overview

The Financial Action Task Force (FATF) Travel Rule requires Virtual Asset Service Providers (VASPs) to share originator and beneficiary information for transactions exceeding a threshold (typically $1,000–$10,000 depending on jurisdiction). AgroLedger enforces this at the contract level through the `ComplianceRegistry.validate_travel_rule()` function.

## Threshold

The current on-chain threshold is **$10,000 USD** (10,000,000,000 in 7-decimal USDC). Transactions above this amount are blocked unless accompanied by valid Travel Rule data.

```
TRAVEL_RULE_THRESHOLD = 10_000_000_000  // $10,000 in 7-decimal USDC
```

## How It Works

1. **Sender initiates a transaction** via `CrossBorderRouter.route()` or `CropToken.transfer()`
2. **ComplianceRegistry.validate_travel_rule(amount, jurisdiction)** is called atomically
3. **If amount > $10,000**: the transaction reverts with `"Travel rule check failed"`
4. **If amount <= $10,000**: the transaction proceeds normally

## Encryption Pattern

For transactions above the threshold, the AgroLedger SDK encrypts originator/beneficiary data into the transaction memo using the recipient's Stellar public key:

```
memo = encrypt({
  originator_name: string,
  originator_address: string,
  beneficiary_name: string,
  beneficiary_address: string,
  timestamp: number,
  jurisdiction: string,
}, recipientPublicKey)
```

### Encryption Spec

- Algorithm: **ChaCha20-Poly1305** (libsodium secretbox)
- Key derivation: **X25519+Blake2b** shared secret from sender's keypair and recipient's public key
- Encoding: Base64URL-encoded ciphertext in the transaction memo field

### Off-Chain Verification

Recipients decrypt the memo using their private key to verify the Travel Rule data. The protocol does not store Travel Rule data on-chain to maintain privacy.

## Integration Example (TypeScript)

```typescript
import { encryptTravelRule } from '@agroledger/sdk';

// For transactions > $10,000, encrypt Travel Rule data
const travelRuleData = {
  originatorName: 'Farmer Cooperative Ltd',
  originatorAddress: '123 Farm Road, Kano, Nigeria',
  beneficiaryName: 'Lagos Mill Corp',
  beneficiaryAddress: '456 Industrial Ave, Lagos, Nigeria',
  amount: 15_000_000_000n, // $15,000 in 7-decimal USDC
  jurisdiction: 'NG',
};

const encryptedMemo = encryptTravelRule(
  travelRuleData,
  recipientPublicKey, // G... address
  senderKeypair,       // sender's Keypair
);

// Attach memo to transaction
const tx = await router.route(
  from,
  to,
  sendAsset,
  recvAsset,
  amount,
  { passportId, jurisdiction },
  encryptedMemo,  // travel rule memo
);
```

## Jurisdiction-Specific Rules

| Jurisdiction | Threshold | Regulator | Notes |
|---|---|---|---|
| Nigeria (NG) | $10,000 | SEC Nigeria | Sandbox participant |
| Ghana (GH) | $10,000 | SEC Ghana | Sandbox participant |
| Kenya (KE) | $10,000 | CMA Kenya | Sandbox participant |
| United States (US) | $3,000 | FinCEN | Blocked by default |
| CFA Zone (CI, SN, etc.) | $10,000 | BCEAO | Under review |

Jurisdictions not on the allow-list are blocked entirely. See `ComplianceRegistry.add_jurisdiction()`.

## Best Practices

1. **Always check the threshold** before sending — use `ComplianceRegistry.validate_travel_rule()` or the SDK's `estimate` function
2. **Encrypt Travel Rule data** for every cross-border transaction regardless of amount if the sender's jurisdiction requires it
3. **Never store PII on-chain** — all Travel Rule data must be encrypted in transaction memos
4. **Rotate encryption keys** periodically for memos
5. **Log Travel Rule data** off-chain for regulatory reporting (retention: 5 years)

## References

- [FATF Guidance for Virtual Assets](https://www.fatf-gafi.org/en/publications/Fatfrecommendations/Guidance-rba-virtual-assets.html)
- [Stellar Travel Rule Working Group](https://stellar.org/foundation)
- [OpenVASP](https://openvasp.org) — interoperability standard
