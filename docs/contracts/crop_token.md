# CropToken Contract

SEP-0041 compliant fungible token representing one metric ton of a certified warehouse lot.

## Data Types

```rust
struct LotMeta {
    warehouse_id: Symbol,
    lot_id: Symbol,
    commodity: Symbol,
    quantity_kg: u64,
    oracle_attestation: Bytes,
    expiry: u64,
    price: i128,
}

enum DataKey {
    Admin,
    WarehouseOracle,     // Address of WarehouseOracle contract
    ComplianceRegistry,  // Address of ComplianceRegistry contract
    LotMeta(Symbol),     // lot_id -> LotMeta
    Balance(Address),    // Address -> i128 balance
    AddressPassport(Address), // (passport_id, jurisdiction) for compliance
}
```

## Public Functions

### `initialize`

Initialize the contract with admin, oracle, and compliance registry addresses.

```rust
fn initialize(env: Env, admin: Address, warehouse_oracle: Address, compliance_registry: Address)
```

| Parameter | Type | Description |
|---|---|---|
| `admin` | Address | Contract admin |
| `warehouse_oracle` | Address | WarehouseOracle contract ID |
| `compliance_registry` | Address | ComplianceRegistry contract ID |

**Auth:** `admin.require_auth()`

**Panics:** `"Already initialized"` if called more than once

---

### `issue`

Issue CropTokens for a verified warehouse lot.

```rust
fn issue(
    env: Env,
    warehouse_id: Symbol,
    lot_id: Symbol,
    commodity: Symbol,
    quantity_kg: u64,
    oracle_sig: Bytes,
) -> Address
```

| Parameter | Type | Description |
|---|---|---|
| `warehouse_id` | Symbol | Warehouse identifier |
| `lot_id` | Symbol | Lot identifier |
| `commodity` | Symbol | Commodity type |
| `quantity_kg` | u64 | Quantity in kilograms |
| `oracle_sig` | Bytes | Oracle attestation signature |

**Returns:** `Address` — the CropToken contract address

**Auth:** `admin.require_auth()`

**Panics:**
- `"Lot not verified by oracle"` — if the lot is not verified via WarehouseOracle.verify_lot()

**Events:**

| Topic | Data |
|---|---|
| `(CropTokenIssued, lot_id)` | (commodity, quantity_kg) |

---

### `transfer`

Transfer CropTokens with compliance gating.

```rust
fn transfer(env: Env, from: Address, to: Address, amount: i128)
```

| Parameter | Type | Description |
|---|---|---|
| `from` | Address | Sender address |
| `to` | Address | Recipient address |
| `amount` | i128 | Amount to transfer |

**Auth:** `from.require_auth()`

**Panics:**
- `"Insufficient balance"` — if sender balance < amount
- `"Compliance check failed"` — if the sender has a linked passport and the compliance check fails

**Notes:** If the sender has a linked passport (via `link_passport`), the transfer calls `ComplianceRegistry.verify()` atomically. Contract-to-contract transfers (e.g., vault liquidation) skip the compliance check.

**Events:**

| Topic | Data |
|---|---|
| `(Transfer, from, to)` | amount |

---

### `burn`

Burn CropTokens when a physical lot is sold.

```rust
fn burn(env: Env, lot_id: Symbol)
```

| Parameter | Type | Description |
|---|---|---|
| `lot_id` | Symbol | Lot identifier to burn |

**Auth:** `admin.require_auth()`

**Panics:**
- `"Insufficient balance to burn"` — if admin balance < lot quantity

**Events:**

| Topic | Data |
|---|---|
| `(CropTokenBurned, lot_id)` | amount |

---

### `get_lot_metadata`

Get the metadata for a specific lot.

```rust
fn get_lot_metadata(env: Env, lot_id: Symbol) -> LotMeta
```

| Parameter | Type | Description |
|---|---|---|
| `lot_id` | Symbol | Lot identifier |

**Returns:** `LotMeta`

**Auth:** None (read-only)

**Panics:** If lot metadata does not exist

---

### `balance`

Get the CropToken balance for an address.

```rust
fn balance(env: Env, id: Address) -> i128
```

| Parameter | Type | Description |
|---|---|---|
| `id` | Address | Account address |

**Returns:** `i128` — balance (0 if none)

**Auth:** None (read-only)

---

### `link_passport`

Link a passport to an address for compliance-gated transfers.

```rust
fn link_passport(env: Env, address: Address, passport_id: u64, jurisdiction: Symbol)
```

| Parameter | Type | Description |
|---|---|---|
| `address` | Address | Address to link passport to |
| `passport_id` | u64 | PrivacyPassport ID |
| `jurisdiction` | Symbol | Jurisdiction code |

**Auth:** `admin.require_auth()`
