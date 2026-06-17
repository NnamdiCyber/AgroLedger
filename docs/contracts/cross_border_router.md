# CrossBorderRouter Contract

Stellar path-payment routing with FATF Travel Rule compliance and multi-stablecoin support.

## Data Types

```rust
struct PathResult {
    from: Address,
    to: Address,
    send_asset: Address,
    recv_asset: Address,
    amount_sent: i128,
    amount_received: i128,
    fee: i128,
}

struct PathQuote {
    send_asset: Address,
    recv_asset: Address,
    amount_out: i128,
    fee: i128,
}

struct TravelRuleData {
    passport_id: u64,
    jurisdiction: Symbol,
}

enum DataKey {
    Admin,
    ComplianceRegistry,
    RouteCounter,
    Asset(Symbol),
}
```

## Public Functions

### `initialize`

Initialize the contract with admin and compliance registry.

```rust
fn initialize(env: Env, admin: Address, compliance_registry: Address)
```

| Parameter | Type | Description |
|---|---|---|
| `admin` | Address | Contract admin |
| `compliance_registry` | Address | ComplianceRegistry contract ID |

**Auth:** `admin.require_auth()`

---

### `route`

Execute a cross-border payment route with compliance checks.

```rust
fn route(
    env: Env,
    from: Address,
    to: Address,
    send_asset: Address,
    recv_asset: Address,
    amount: i128,
    travel_rule_data: TravelRuleData,
) -> PathResult
```

| Parameter | Type | Description |
|---|---|---|
| `from` | Address | Sender address |
| `to` | Address | Recipient address |
| `send_asset` | Address | Source asset contract ID |
| `recv_asset` | Address | Destination asset contract ID |
| `amount` | i128 | Amount to send |
| `travel_rule_data` | TravelRuleData | Passport ID and jurisdiction |

**Returns:** `PathResult`

**Auth:** `from.require_auth()`

**Panics:**
- `"Amount must be positive"` — if amount <= 0
- `"Compliance check failed"` — if ComplianceRegistry.verify() returns false
- `"Travel rule check failed"` — if ComplianceRegistry.validate_travel_rule() returns false

**Fee:** 0.15% (15 bps) deducted from the sent amount

**Events:**

| Topic | Data |
|---|---|
| `(RouteExecuted, route_id, from)` | (amount, fee) |

---

### `estimate`

Get a quote for a potential route without executing it.

```rust
fn estimate(
    env: Env,
    send_asset: Address,
    recv_asset: Address,
    amount: i128,
) -> Vec<PathQuote>
```

| Parameter | Type | Description |
|---|---|---|
| `send_asset` | Address | Source asset contract ID |
| `recv_asset` | Address | Destination asset contract ID |
| `amount` | i128 | Amount to send |

**Returns:** `Vec<PathQuote>` — list of quotes (currently returns a single quote)

**Auth:** None (read-only)

**Panics:** `"Amount must be positive"` — if amount <= 0

---

### `register_asset`

Register a stablecoin asset with a symbol for routing.

```rust
fn register_asset(env: Env, admin: Address, symbol: Symbol, contract_id: Address)
```

| Parameter | Type | Description |
|---|---|---|
| `admin` | Address | Contract admin |
| `symbol` | Symbol | Asset symbol (e.g., "USDC", "cNGN") |
| `contract_id` | Address | Asset contract ID |

**Auth:** `admin.require_auth()`

**Events:**

| Topic | Data |
|---|---|
| `(AssetRegistered, symbol)` | contract_id |

---

### `get_asset`

Get the contract ID for a registered asset symbol.

```rust
fn get_asset(env: Env, symbol: Symbol) -> Address
```

| Parameter | Type | Description |
|---|---|---|
| `symbol` | Symbol | Asset symbol |

**Returns:** `Address` — contract ID

**Auth:** None (read-only)

**Panics:** If symbol has not been registered

## Supported Stablecoins

| Symbol | Description |
|---|---|
| USDC | USD Circle stablecoin |
| cNGN | Nigerian Naira stablecoin |
| cXOF | CFA Franc stablecoin |
| cGHS | Ghana Cedi stablecoin |
| cKES | Kenyan Shilling stablecoin |

## Fee Calculation

```
fee = amount * 15 / 10000  (0.15%)
amount_after_fee = amount - fee
```
