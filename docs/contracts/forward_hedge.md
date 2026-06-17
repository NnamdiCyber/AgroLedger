# ForwardHedge Contract

Forward purchase contracts enabling commodity buyers to lock future purchase prices against on-chain inventory using a sealed-bid mechanism.

## Data Types

```rust
struct HedgeState {
    buyer: Address,
    farmer: Address,
    commodity: Symbol,
    quantity: i128,
    commitment: BytesN<32>,  // hash(price + salt)
    expiry: u64,
    status: Symbol,           // "Placed" | "Accepted" | "SettledPhysical" | "SettledCash" | "Cancelled"
    placed_at: u64,
}

enum DataKey {
    Admin,
    CropToken,
    CollateralVault,
    HedgeCounter,
    Hedge(u64),
    RevealedPrice(u64),
}
```

## Status Lifecycle

```
Placed → Accepted → SettledPhysical
                  → SettledCash
        → Cancelled (before expiry)
```

## Public Functions

### `initialize`

Initialize the hedge contract with admin and dependent contracts.

```rust
fn initialize(env: Env, admin: Address, crop_token: Address, collateral_vault: Address)
```

| Parameter | Type | Description |
|---|---|---|
| `admin` | Address | Contract admin |
| `crop_token` | Address | CropToken contract ID |
| `collateral_vault` | Address | CollateralVault contract ID |

**Auth:** `admin.require_auth()`

**Panics:** `"Already initialized"` if called more than once

---

### `place_hedge`

Place a sealed-bid hedge as a buyer. The buyer commits to a max price without revealing it.

```rust
fn place_hedge(
    env: Env,
    buyer: Address,
    commodity: Symbol,
    quantity: i128,
    commitment: BytesN<32>,
    expiry: u64,
) -> u64
```

| Parameter | Type | Description |
|---|---|---|
| `buyer` | Address | Buyer address |
| `commodity` | Symbol | Commodity type |
| `quantity` | i128 | Quantity in kg |
| `commitment` | BytesN<32> | `sha256(price + salt)` — sealed bid |
| `expiry` | u64 | Expiry timestamp |

**Returns:** `u64` — hedge ID

**Auth:** `buyer.require_auth()`

**Panics:**
- `"Quantity must be positive"` — if quantity <= 0
- `"Expiry must be in the future"` — if expiry <= current timestamp

**Events:**

| Topic | Data |
|---|---|
| `(HedgePlaced, hedge_id)` | (buyer, commodity, quantity, expiry) |

---

### `accept_hedge`

Accept a hedge as a farmer, agreeing to the buyer's terms.

```rust
fn accept_hedge(env: Env, hedge_id: u64, farmer: Address)
```

| Parameter | Type | Description |
|---|---|---|
| `hedge_id` | u64 | Hedge ID |
| `farmer` | Address | Farmer address accepting the hedge |

**Auth:** `farmer.require_auth()`

**Panics:**
- `"Hedge must be in Placed status"` — if not in Placed state
- `"Hedge has expired"` — if current time >= expiry

**Events:**

| Topic | Data |
|---|---|
| `(HedgeAccepted, hedge_id)` | farmer |

---

### `reveal`

Reveal the sealed-bid price and salt to verify commitment hash.

```rust
fn reveal(env: Env, hedge_id: u64, price: i128, salt: i128)
```

| Parameter | Type | Description |
|---|---|---|
| `hedge_id` | u64 | Hedge ID |
| `price` | i128 | Agreed price per unit |
| `salt` | i128 | Random salt used in commitment |

**Auth:** None (anyone can reveal)

**Panics:**
- `"Hedge must be in Accepted status"` — if not accepted
- `"Revealed price and salt do not match commitment"` — if sha256(price + salt) != commitment

---

### `settle`

Settle a hedge at expiry via physical delivery or cash settlement.

```rust
fn settle(env: Env, hedge_id: u64, settlement_type: Symbol, caller: Address)
```

| Parameter | Type | Description |
|---|---|---|
| `hedge_id` | u64 | Hedge ID |
| `settlement_type` | Symbol | `"Physical"` or `"Cash"` |
| `caller` | Address | Address initiating the settlement |

**Auth:** `caller.require_auth()`

**Panics:**
- `"Hedge must be in Accepted status"` — if not accepted
- `"Hedge has not expired yet"` — if current time < expiry
- `"Price must be revealed before settlement"` — if price not revealed
- `"Invalid settlement type"` — if not Physical or Cash

**Settlement details:**
- **Physical**: `CropToken.transfer(farmer → buyer, quantity)` — farmer delivers tokens
- **Cash**: `CropToken.transfer(buyer → farmer, quantity * revealed_price / 1_000_000_000)` — buyer pays difference

**Events:**

| Topic | Data |
|---|---|
| `(HedgeSettled, hedge_id)` | settlement_type |

---

### `cancel`

Cancel a hedge before expiry with a 10% penalty on accepted hedges.

```rust
fn cancel(env: Env, hedge_id: u64, caller: Address)
```

| Parameter | Type | Description |
|---|---|---|
| `hedge_id` | u64 | Hedge ID |
| `caller` | Address | Address initiating the cancellation |

**Auth:** `caller.require_auth()`

**Panics:**
- `"Hedge already settled or cancelled"` — if not in Placed or Accepted state
- `"Cannot cancel after expiry"` — if current time >= expiry

**Penalty:** If hedge is in Accepted state, farmer pays a 10% penalty to buyer.

**Events:**

| Topic | Data |
|---|---|
| `(HedgeCancelled, hedge_id)` | caller |

---

### Read-Only Getters

```rust
fn get_hedge(env: Env, hedge_id: u64) -> HedgeState
fn get_revealed_price(env: Env, hedge_id: u64) -> i128
```

**Auth:** None (read-only)
