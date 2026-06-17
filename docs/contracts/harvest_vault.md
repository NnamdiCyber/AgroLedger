# HarvestVault Contract

Yield-bearing vault that auto-compounds warehouse storage income and AMM LP fees.

## Data Types

```rust
enum DataKey {
    Admin,
    CropToken,
    CommodityAmm,
    UsdcToken,
    TotalCropDeposited,
    TotalHctSupply,
    TotalYieldUsdc,
    BalanceHct(Address),
    LastAccrual,
}
```

## Public Functions

### `initialize`

Initialize the vault with token and AMM addresses.

```rust
fn initialize(
    env: Env,
    admin: Address,
    crop_token: Address,
    commodity_amm: Address,
    usdc_token: Address,
)
```

| Parameter | Type | Description |
|---|---|---|
| `admin` | Address | Contract admin |
| `crop_token` | Address | CropToken contract ID |
| `commodity_amm` | Address | CommodityAmm contract ID |
| `usdc_token` | Address | USDC token contract ID |

**Auth:** `admin.require_auth()`

**Panics:** `"Already initialized"` if called more than once

---

### `deposit`

Deposit CropTokens and receive yield-bearing hCT receipt tokens.

```rust
fn deposit(env: Env, user: Address, amount: i128) -> i128
```

| Parameter | Type | Description |
|---|---|---|
| `user` | Address | Depositor address |
| `amount` | i128 | Amount of CropTokens to deposit |

**Returns:** `i128` — amount of hCT tokens minted

**Auth:** `user.require_auth()`

**Panics:** `"Deposit amount must be positive"` — if amount <= 0

**Notes:** First depositor establishes the 1:1 ratio. Subsequent depositors receive hCT proportional to their share of total deposits.

**Events:**

| Topic | Data |
|---|---|
| `(Deposited, user)` | (amount, hct_minted) |

---

### `withdraw`

Withdraw CropTokens and accrued USDC yield by burning hCT tokens.

```rust
fn withdraw(env: Env, user: Address, hct_amount: i128) -> (i128, i128)
```

| Parameter | Type | Description |
|---|---|---|
| `user` | Address | Withdrawer address |
| `hct_amount` | i128 | Amount of hCT tokens to burn |

**Returns:** `(i128, i128)` — (crop_tokens_withdrawn, usdc_yield_withdrawn)

**Auth:** `user.require_auth()`

**Panics:**
- `"Withdraw amount must be positive"` — if hct_amount <= 0
- `"Insufficient hCT balance"` — if user hCT balance < hct_amount

**Events:**

| Topic | Data |
|---|---|
| `(Withdrawn, user)` | (crop_out, yield_out) |

---

### `accrue_yield`

Accrue yield since the last accrual event. Admin-only.

```rust
fn accrue_yield(env: Env, admin: Address) -> i128
```

| Parameter | Type | Description |
|---|---|---|
| `admin` | Address | Contract admin |

**Returns:** `i128` — amount of yield accrued

**Auth:** `admin.require_auth()`

**Events:**

| Topic | Data |
|---|---|
| `(YieldAccrued,)` | yield_amount |

---

### `get_apy`

Get the current APY in basis points.

```rust
fn get_apy(env: Env) -> u32
```

**Returns:** `u32` — APY in basis points (800 = 8.00%)

**Auth:** None (read-only)

---

### `rebalance`

Trigger yield accrual and compounding. Admin-only.

```rust
fn rebalance(env: Env, admin: Address)
```

| Parameter | Type | Description |
|---|---|---|
| `admin` | Address | Contract admin |

**Auth:** `admin.require_auth()`

---

### Read-Only Getters

```rust
fn get_hct_balance(env: Env, user: Address) -> i128
fn get_total_crop_deposited(env: Env) -> i128
fn get_total_hct_supply(env: Env) -> i128
fn get_total_yield(env: Env) -> i128
```

**Auth:** None (read-only)

## Yield Calculation

```
yield = total_crop_deposited * BASE_APY_BPS * elapsed_seconds / (SECONDS_PER_YEAR * 10000)
```

- Base APY: 800 bps (8.00%)
- Yield accrues in USDC
- Yield is proportional to hCT holdings at withdrawal time
