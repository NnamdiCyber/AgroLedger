# WarehouseOracle Contract

Certified lot attestation with multi-sig inspector verification and price feed management.

## Data Types

```rust
struct InspectorSet {
    inspectors: Vec<Address>,  // List of inspector addresses
    threshold: u32,            // Minimum signatures required
}

struct LotState {
    warehouse_id: Symbol,
    lot_id: Symbol,
    commodity: Symbol,
    quantity_kg: u64,
    approved: bool,
    approved_at: u64,
}

struct PriceData {
    price: u64,     // Price in USDC (7 decimals)
    timestamp: u64, // Unix timestamp
}

enum DataKey {
    Admin,
    OraclePubkey,
    InspectorSet,
    LotCounter,
    Lot(u64),
    LotLookup(Symbol, Symbol),
    Price(Symbol),
}
```

## Public Functions

### `initialize`

Initialize the contract with admin, oracle key, and inspector set.

```rust
fn initialize(env: Env, admin: Address, oracle_pubkey: Address, inspectors: InspectorSet)
```

| Parameter | Type | Description |
|---|---|---|
| `admin` | Address | Contract admin |
| `oracle_pubkey` | Address | Oracle public key for price feed authentication |
| `inspectors` | InspectorSet | Set of inspector addresses and signature threshold |

**Auth:** `admin.require_auth()`

---

### `submit_lot`

Submit a lot attestation with inspector multi-sig verification.

```rust
fn submit_lot(
    env: Env,
    warehouse_id: Symbol,
    lot_id: Symbol,
    commodity: Symbol,
    quantity_kg: u64,
    inspector_sigs: Vec<Address>,
) -> u64
```

| Parameter | Type | Description |
|---|---|---|
| `warehouse_id` | Symbol | Warehouse identifier (e.g., "WH001") |
| `lot_id` | Symbol | Lot identifier (e.g., "LOT001") |
| `commodity` | Symbol | Commodity type (e.g., "MAIZE") |
| `quantity_kg` | u64 | Quantity in kilograms |
| `inspector_sigs` | Vec<Address> | Inspector addresses providing signatures |

**Returns:** `u64` — auto-incremented lot number

**Auth:** Each inspector in `inspector_sigs` must call `require_auth()`

**Panics:**
- `"insufficient signatures"` — if `inspector_sigs.len()` < threshold (or < 3 for lots > 50 MT)

**Events:**

| Topic | Data |
|---|---|
| `(LotSubmitted, lot_num)` | (warehouse_id, lot_id, commodity, quantity_kg) |

---

### `push_price`

Push a price update for a commodity. Must be called by the oracle key.

```rust
fn push_price(env: Env, commodity: Symbol, price_usdc: u64, timestamp: u64)
```

| Parameter | Type | Description |
|---|---|---|
| `commodity` | Symbol | Commodity type |
| `price_usdc` | u64 | Price in USDC (7 decimals) |
| `timestamp` | u64 | Unix timestamp of the price |

**Auth:** `oracle_pubkey.require_auth()`

**Events:**

| Topic | Data |
|---|---|
| `(PriceUpdated, commodity)` | (price_usdc, timestamp) |

---

### `get_price`

Get the latest price for a commodity.

```rust
fn get_price(env: Env, commodity: Symbol) -> PriceData
```

| Parameter | Type | Description |
|---|---|---|
| `commodity` | Symbol | Commodity type |

**Returns:** `PriceData` — (price, timestamp). Returns (0, 0) if no price exists.

**Auth:** None (read-only)

---

### `verify_lot`

Check if a specific lot has been approved by warehouse and lot ID.

```rust
fn verify_lot(env: Env, warehouse_id: Symbol, lot_id: Symbol) -> bool
```

| Parameter | Type | Description |
|---|---|---|
| `warehouse_id` | Symbol | Warehouse identifier |
| `lot_id` | Symbol | Lot identifier |

**Returns:** `bool` — true if the lot exists and is approved

**Auth:** None (read-only)
