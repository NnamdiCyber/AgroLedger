# AgroLedger SDK

TypeScript/JavaScript SDK for integrating AgroLedger smart contracts into web, mobile, and backend applications.

## Installation

```bash
npm install @agroledger/sdk
```

Requires `@stellar/stellar-sdk` as a peer dependency:

```bash
npm install @stellar/stellar-sdk
```

## Quick Start

### Initialization

```typescript
import { AgroLedger } from '@agroledger/sdk';
import { Keypair } from '@stellar/stellar-sdk';

const signer = Keypair.fromSecret('S...'); // Your Stellar secret key

// Initialize the SDK client
const agro = new AgroLedger({
  network: 'testnet',   // 'local' | 'testnet' | 'mainnet'
  signer,
});

// Connect to deployed contracts (use contract IDs from your .env)
agro.connectPrivacyPassport('C...');
agro.connectComplianceRegistry('C...');
agro.connectCropToken('C...');
agro.connectCollateralVault('C...');
agro.connectWarehouseOracle('C...');
agro.connectCrossBorderRouter('C...');
agro.connectCommodityAmm('C...');
agro.connectHarvestVault('C...');
agro.connectForwardHedge('C...');
```

### Register a Passport

```typescript
const passportId = await agro.privacyPassport.register(
  '0xabcd...',  // nullifier hash
  '0xef01...',  // credential proof
  'NG',          // jurisdiction
);
```

### Issue CropTokens

```typescript
await agro.cropToken.issue(
  'WH001',   // warehouse ID
  'LOT001',  // lot ID
  'MAIZE',   // commodity
  10000,     // quantity in kg
  '0x...',   // oracle signature
);
```

### Open a Vault and Borrow

```typescript
const vaultId = await agro.collateralVault.open(
  'G...',       // user address
  'C...',       // crop token contract ID
  'MAIZE',      // commodity
  1,            // passport ID
  'NG',         // jurisdiction
  5000,         // collateral amount
  100000,       // borrow amount (USDC)
);
```

### Transfer CropTokens

```typescript
await agro.cropToken.transfer(
  'G...',   // from
  'G...',   // to
  100,      // amount
);
```

### Cross-Border Payment

```typescript
const result = await agro.crossBorderRouter.route(
  'G...',     // from
  'G...',     // to
  'C...',     // send asset (USDC contract ID)
  'C...',     // receive asset (cNGN contract ID)
  100000,     // amount
  {
    passportId: 1,
    jurisdiction: 'NG',
  },
);

console.log(`Received: ${result.amount_received}, Fee: ${result.fee}`);
```

### AMM Swap

```typescript
const amountOut = await agro.commodityAmm.swap(
  'G...',       // user
  'MAIZE',      // commodity pool
  1000,         // amount in
  0,            // min amount out (0 = no slippage protection)
  true,         // sell crop (true) or buy crop (false)
);
```

### Deposit into Harvest Vault

```typescript
const hctTokens = await agro.harvestVault.deposit(
  'G...',   // user
  50000,    // CropToken amount
);

const apy = await agro.harvestVault.getApy();
console.log(`Current APY: ${apy / 100}%`);
```

### Forward Hedge

```typescript
// Buyer places a sealed-bid hedge
const commitment = sha256(price + salt); // compute off-chain
const hedgeId = await agro.forwardHedge.placeHedge(
  'G...',       // buyer
  'MAIZE',      // commodity
  1000,         // quantity
  commitment,
  2000000000,   // expiry timestamp
);

// Farmer accepts
await agro.forwardHedge.acceptHedge(hedgeId, 'G...');

// At expiry: reveal and settle
await agro.forwardHedge.reveal(hedgeId, price, salt);
await agro.forwardHedge.settle(hedgeId, 'Physical', 'G...');
```

## API Reference

### `AgroLedger` Class

| Method | Description |
|---|---|
| `AgroLedger(config)` | Create SDK instance with network config and signer |
| `connectPrivacyPassport(id)` | Connect to PrivacyPassport contract |
| `connectComplianceRegistry(id)` | Connect to ComplianceRegistry contract |
| `connectCropToken(id)` | Connect to CropToken contract |
| `connectCollateralVault(id)` | Connect to CollateralVault contract |
| `connectWarehouseOracle(id)` | Connect to WarehouseOracle contract |
| `connectCrossBorderRouter(id)` | Connect to CrossBorderRouter contract |
| `connectCommodityAmm(id)` | Connect to CommodityAmm contract |
| `connectHarvestVault(id)` | Connect to HarvestVault contract |
| `connectForwardHedge(id)` | Connect to ForwardHedge contract |

### Contract Clients

Each client supports:
- **`send(method, args, types)`** — submits a transaction (write operation)
- **`simulate(method, args, types)`** — simulates a transaction (read operation)

See `docs/contracts/` for the full function signatures of each contract.

## Error Handling

```typescript
try {
  await agro.cropToken.transfer(from, to, amount);
} catch (error) {
  if (error.message.includes('Compliance check failed')) {
    // Handle compliance failure
  } else if (error.message.includes('Insufficient balance')) {
    // Handle insufficient balance
  }
}
```

## Running Tests

```bash
cd sdk
npm test
```

## React Integration

See [React Hooks Documentation](./react-hooks.md) for using AgroLedger with React.
