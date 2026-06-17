# React Hooks

AgroLedger provides React hooks for integrating with React and Next.js applications. These hooks handle loading, error, and transaction states.

## Hook Reference

| Hook | Description |
|---|---|
| `useFarmerPortfolio` | Fetch farmer's CropTokens, vaults, and active hedges |
| `useCollateralVault` | Open vaults, draw credit, and repay |
| `useForwardHedge` | Place and accept forward hedges |

## Setup

```tsx
import { AgroLedger } from '@agroledger/sdk';
import { Keypair } from '@stellar/stellar-sdk';

const agro = new AgroLedger({
  network: 'testnet',
  signer: Keypair.fromSecret('S...'),
});

agro.connectCropToken('C...');
agro.connectCollateralVault('C...');
agro.connectForwardHedge('C...');
```

## `useFarmerPortfolio`

Fetches a farmer's complete portfolio including CropTokens, vaults, and hedges.

```tsx
import { useFarmerPortfolio } from '@agroledger/sdk';

function FarmerDashboard({ farmerAddress }: { farmerAddress: string }) {
  const { tokens, vaults, hedges, isLoading, error, refresh } =
    useFarmerPortfolio(
      farmerAddress,
      agro.cropToken,
      agro.collateralVault,
      agro.forwardHedge,
    );

  if (isLoading) return <Spinner />;
  if (error) return <Error message={error} />;

  return (
    <div>
      <h2>Crop Tokens ({tokens.length})</h2>
      <ul>
        {tokens.map((t) => (
          <li key={t.lotId}>
            {t.meta.commodity}: {t.meta.quantity_kg} kg
          </li>
        ))}
      </ul>

      <h2>Vaults ({vaults.length})</h2>
      <ul>
        {vaults.map((v) => (
          <li key={v.id}>
            Debt: {v.state.debt_amount} USDC
          </li>
        ))}
      </ul>

      <h2>Active Hedges ({hedges.length})</h2>
      <ul>
        {hedges.map((h) => (
          <li key={h.id}>
            {h.state.commodity}: {h.state.status}
          </li>
        ))}
      </ul>

      <button onClick={refresh}>Refresh</button>
    </div>
  );
}
```

### Return Values

| Property | Type | Description |
|---|---|---|
| `tokens` | `{ lotId: string; meta: LotMeta }[]` | Farmer's CropTokens |
| `vaults` | `{ id: number; state: VaultState }[]` | Farmer's vaults |
| `hedges` | `{ id: number; state: HedgeState }[]` | Farmer's hedges |
| `isLoading` | `boolean` | Loading state |
| `error` | `string | null` | Error message |
| `refresh` | `() => Promise<void>` | Refetch all data |

## `useCollateralVault`

Manages vault operations with loading and error states.

```tsx
import { useCollateralVault } from '@agroledger/sdk';

function BorrowPanel() {
  const { openVault, repay, liquidate, isPending, error } =
    useCollateralVault(agro.collateralVault);

  const handleBorrow = async () => {
    try {
      const vaultId = await openVault(
        userAddress,
        cropTokenId,
        'MAIZE',
        1,       // passport ID
        'NG',    // jurisdiction
        50000,   // collateral
        100000,  // borrow amount
      );
      console.log(`Vault opened: ${vaultId}`);
    } catch (err) {
      console.error(err);
    }
  };

  return (
    <div>
      <button onClick={handleBorrow} disabled={isPending}>
        {isPending ? 'Processing...' : 'Borrow USDC'}
      </button>
      {error && <div className="error">{error}</div>}
    </div>
  );
}
```

### Return Values

| Property | Type | Description |
|---|---|---|
| `openVault` | `(user, cropToken, commodity, passportId, jurisdiction, collateral, borrowAmount) => Promise<number>` | Open a new vault |
| `repay` | `(user, vaultId, amount) => Promise<string>` | Repay a vault |
| `liquidate` | `(liquidator, vaultId) => Promise<string>` | Liquidate a vault |
| `isPending` | `boolean` | Transaction in progress |
| `error` | `string | null` | Error message |

## `useForwardHedge`

Manages forward hedge placement and acceptance.

```tsx
import { useForwardHedge } from '@agroledger/sdk';

function HedgePanel() {
  const { placeHedge, acceptHedge, isPending, error } =
    useForwardHedge(agro.forwardHedge);

  const handlePlaceHedge = async () => {
    // Compute commitment = sha256(price + salt)
    const commitment = computeCommitment(price, salt);
    const hedgeId = await placeHedge(
      buyerAddress,
      'MAIZE',
      1000,
      commitment,
      expiryTimestamp,
    );
    console.log(`Hedge placed: ${hedgeId}`);
  };

  const handleAcceptHedge = async (hedgeId: number) => {
    await acceptHedge(hedgeId, farmerAddress);
    console.log(`Hedge ${hedgeId} accepted`);
  };

  return (
    <div>
      <button onClick={handlePlaceHedge} disabled={isPending}>
        Place Hedge
      </button>
      {error && <div className="error">{error}</div>}
    </div>
  );
}
```

### Return Values

| Property | Type | Description |
|---|---|---|
| `placeHedge` | `(buyer, commodity, quantity, commitment, expiry) => Promise<number>` | Place a new hedge |
| `acceptHedge` | `(hedgeId, farmer) => Promise<string>` | Accept a hedge |
| `isPending` | `boolean` | Transaction in progress |
| `error` | `string | null` | Error message |

## Best Practices

1. **Always handle the error state** — transactions can fail due to compliance, insufficient balance, or network issues
2. **Disable buttons while `isPending`** — prevents double-submission
3. **Call `refresh()` on `useFarmerPortfolio`** after any state-changing operation to keep the UI in sync
4. **Use TypeScript** — all hooks are fully typed with interfaces from `@agroledger/sdk`
5. **Wrap in error boundaries** — React error boundaries catch unexpected rendering errors
