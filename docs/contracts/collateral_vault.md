# CollateralVault Contract

Lock CropTokens as collateral and draw USDC credit with LTV-based liquidation.

## Data Types

```rust
struct VaultState {
    owner: Address,
    crop_token: Address,
    collateral_amount: i128,
    debt_amount: i128,
    commodity: Symbol,
    opened_at: u64,
}

enum DataKey {
    Admin,
    ComplianceRegistry,
    UsdcToken,
    WarehouseOracle,
    VaultCounter,
    Vault(u64),
}
```

## Public Functions

### `initialize`

Initialize the contract with admin and dependent contract addresses.

```rust
fn initialize(
    env: Env,
    admin: Address,
    compliance_registry: Address,
    usdc_token: Address,
    warehouse_oracle: Address,
)
```

| Parameter | Type | Description |
|---|---|---|
| `admin` | Address | Contract admin |
| `compliance_registry` | Address | ComplianceRegistry contract ID |
| `usdc_token` | Address | USDC token contract ID |
| `warehouse_oracle` | Address | WarehouseOracle contract ID |

**Auth:** `admin.require_auth()`

---

### `open`

Open a new vault, deposit collateral, and borrow USDC.

```rust
fn open(
    env: Env,
    user: Address,
    crop_token: Address,
    commodity: Symbol,
    passport_id: u64,
    jurisdiction: Symbol,
    collateral_amount: i128,
    borrow_amount_usdc: i128,
) -> u64
```

| Parameter | Type | Description |
|---|---|---|
| `user` | Address | Vault owner |
| `crop_token` | Address | CropToken contract address |
| `commodity` | Symbol | Commodity type (for price feed) |
| `passport_id` | u64 | PrivacyPassport ID |
| `jurisdiction` | Symbol | Jurisdiction for compliance |
| `collateral_amount` | i128 | Amount of CropTokens to lock |
| `borrow_amount_usdc` | i128 | Amount of USDC to borrow |

**Returns:** `u64` — vault ID

**Auth:** `user.require_auth()`

**Panics:**
- `"Collateral must be positive"` — if collateral_amount <= 0
- `"Borrow amount must be positive"` — if borrow_amount_usdc <= 0
- `"Compliance check failed"` — if ComplianceRegistry.verify() returns false

**Cross-contract calls:**
- `ComplianceRegistry.verify()` — compliance check
- `CropToken.transfer()` — lock collateral into vault
- `USDC.transfer()` — lend borrowed amount to user

**Events:**

| Topic | Data |
|---|---|
| `(VaultOpened, vault_id)` | borrow_amount_usdc |

---

### `repay`

Repay borrowed USDC to unlock collateral.

```rust
fn repay(env: Env, user: Address, vault_id: u64, amount: i128)
```

| Parameter | Type | Description |
|---|---|---|
| `user` | Address | Vault owner |
| `vault_id` | u64 | Vault ID |
| `amount` | i128 | Amount of USDC to repay |

**Auth:** `user.require_auth()`

**Panics:**
- `"Repay amount must be positive"` — if amount <= 0
- `"Not vault owner"` — if caller is not the vault owner
- `"Repay amount exceeds debt"` — if amount > debt

**Notes:** If fully repaid, the full collateral amount is transferred back to the user.

**Events:**

| Topic | Data |
|---|---|
| `(VaultRepaid, vault_id)` | amount |

---

### `liquidate`

Liquidate an unhealthy vault (LTV > 85%).

```rust
fn liquidate(env: Env, liquidator: Address, vault_id: u64)
```

| Parameter | Type | Description |
|---|---|---|
| `liquidator` | Address | Address performing liquidation |
| `vault_id` | u64 | Vault ID |

**Auth:** `liquidator.require_auth()`

**Panics:**
- `"Vault is not liquidatable"` — if computed LTV <= 85%

**Cross-contract calls:**
- `WarehouseOracle.get_price()` — fetch current commodity price
- `USDC.transfer()` — liquidator pays off debt
- `CropToken.transfer()` — liquidator receives collateral

**Events:**

| Topic | Data |
|---|---|
| `(VaultLiquidated, vault_id)` | collateral_amount |

---

### `get_vault`

Get the state of a vault.

```rust
fn get_vault(env: Env, vault_id: u64) -> VaultState
```

| Parameter | Type | Description |
|---|---|---|
| `vault_id` | u64 | Vault ID |

**Returns:** `VaultState`

**Auth:** None (read-only)

**Panics:** If vault_id does not exist

## LTV Calculation

Loan-to-value ratio computed as:

```
LTV = (debt_usdc * 100) / (crop_token_amount * price_per_token)
```

- Liquidation threshold: 85%
- Price sourced from WarehouseOracle.get_price()
- Returns 0 if price is 0 or collateral value is 0
