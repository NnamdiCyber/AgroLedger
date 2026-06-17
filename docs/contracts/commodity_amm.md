# CommodityAmm Contract

Soroban-native AMM with commodity-specific bonding curves incorporating seasonal price variance.

## Data Types

```rust
struct PoolInfo {
    commodity: Symbol,
    reserve_crop: i128,
    reserve_usdc: i128,
    total_lp_supply: i128,
    created_at: u64,
}

enum DataKey {
    Admin,
    CropToken,
    UsdcToken,
    PoolInfo(Symbol),
    BalanceLP(Address, Symbol),
    TotalLpSupply(Symbol),
}
```

## Public Functions

### `initialize`

Initialize the AMM with admin, CropToken, and USDC token addresses.

```rust
fn initialize(env: Env, admin: Address, crop_token: Address, usdc_token: Address)
```

| Parameter | Type | Description |
|---|---|---|
| `admin` | Address | Contract admin |
| `crop_token` | Address | CropToken contract ID |
| `usdc_token` | Address | USDC token contract ID |

**Auth:** `admin.require_auth()`

**Panics:** `"Already initialized"` if called more than once

---

### `create_pool`

Create a new liquidity pool for a commodity.

```rust
fn create_pool(env: Env, admin: Address, commodity: Symbol)
```

| Parameter | Type | Description |
|---|---|---|
| `admin` | Address | Contract admin |
| `commodity` | Symbol | Commodity type (e.g., "MAIZE") |

**Auth:** `admin.require_auth()`

**Panics:**
- `"Only admin can create pools"` — if caller is not admin
- `"Pool already exists"` — if a pool for this commodity already exists

---

### `swap`

Execute a token swap against a commodity pool.

```rust
fn swap(
    env: Env,
    user: Address,
    commodity: Symbol,
    amount_in: i128,
    min_amount_out: i128,
    sell_crop: bool,
) -> i128
```

| Parameter | Type | Description |
|---|---|---|
| `user` | Address | Trader address |
| `commodity` | Symbol | Commodity pool to trade against |
| `amount_in` | i128 | Amount of input token |
| `min_amount_out` | i128 | Minimum output amount (slippage protection) |
| `sell_crop` | bool | true = sell CropTokens for USDC, false = buy CropTokens with USDC |

**Returns:** `i128` — amount of output token received

**Auth:** `user.require_auth()`

**Panics:**
- `"Amount must be positive"` — if amount_in <= 0
- `"Min amount out must be non-negative"` — if min_amount_out < 0
- `"Pool does not exist"` — if pool not created
- `"Slippage: amount_out below min"` — if calculated output < min_amount_out
- `"Insufficient liquidity"` — if pool reserves are zero

**Fee:** 0.30% (30 bps)

**Events:**

| Topic | Data |
|---|---|
| `(SwapExecuted, commodity, user)` | (amount_in, amount_out) |

---

### `add_liquidity`

Add liquidity to a commodity pool and receive LP tokens.

```rust
fn add_liquidity(
    env: Env,
    user: Address,
    commodity: Symbol,
    amount_crop: i128,
    amount_usdc: i128,
) -> (i128, i128, i128)
```

| Parameter | Type | Description |
|---|---|---|
| `user` | Address | Liquidity provider |
| `commodity` | Symbol | Commodity pool |
| `amount_crop` | i128 | CropToken amount to deposit |
| `amount_usdc` | i128 | USDC amount to deposit |

**Returns:** `(i128, i128, i128)` — (actual_crop_deposited, actual_usdc_deposited, lp_tokens_minted)

**Auth:** `user.require_auth()`

**Panics:**
- `"Amounts must be positive"` — if either amount <= 0
- `"Pool does not exist"` — if pool not created
- `"No LP tokens to mint"` — if calculated LP tokens = 0

**Events:**

| Topic | Data |
|---|---|
| `(LiquidityAdded, commodity, user)` | (actual_crop, actual_usdc, lp_tokens) |

---

### `remove_liquidity`

Remove liquidity and burn LP tokens to receive underlying assets.

```rust
fn remove_liquidity(
    env: Env,
    user: Address,
    commodity: Symbol,
    lp_tokens: i128,
    min_crop: i128,
    min_usdc: i128,
) -> (i128, i128)
```

| Parameter | Type | Description |
|---|---|---|
| `user` | Address | Liquidity provider |
| `commodity` | Symbol | Commodity pool |
| `lp_tokens` | i128 | Amount of LP tokens to burn |
| `min_crop` | i128 | Minimum CropTokens to receive (slippage protection) |
| `min_usdc` | i128 | Minimum USDC to receive (slippage protection) |

**Returns:** `(i128, i128)` — (crop_out, usdc_out)

**Auth:** `user.require_auth()`

**Panics:**
- `"LP tokens must be positive"` — if lp_tokens <= 0
- `"Pool does not exist"` — if pool not created
- `"Insufficient LP tokens"` — if user balance < lp_tokens
- `"Crop output below minimum"` — if crop_out < min_crop
- `"USDC output below minimum"` — if usdc_out < min_usdc

**Events:**

| Topic | Data |
|---|---|
| `(LiquidityRemoved, commodity, user)` | (crop_out, usdc_out) |

---

### `get_pool`

Get pool information for a commodity.

```rust
fn get_pool(env: Env, commodity: Symbol) -> PoolInfo
```

**Auth:** None (read-only)

**Panics:** If pool does not exist

---

### `get_lp_balance`

Get LP token balance for a user in a specific commodity pool.

```rust
fn get_lp_balance(env: Env, user: Address, commodity: Symbol) -> i128
```

**Auth:** None (read-only)

## Seasonal Curve

The AMM uses a custom bonding curve that incorporates seasonal price variance:

- **Pre-harvest (Jan-Jun)**: Prices are discounted (lower output for CropToken sellers)
- **Post-harvest (Jul-Dec)**: Prices include a premium (higher output for CropToken sellers)
- The factor ranges linearly from 95% (Jan) to 105% (Dec) of the base constant-product price
- Base curve: constant product with 0.30% fee
- Formula: `amount_out = amount_in_with_fee * reserve_out / (reserve_in + amount_in_with_fee) * seasonal_factor`
