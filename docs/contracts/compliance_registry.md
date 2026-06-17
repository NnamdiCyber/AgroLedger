# ComplianceRegistry Contract

On-chain transfer allow-list keyed to PrivacyPassport attestations with FATF Travel Rule validation.

## Data Types

```rust
enum DataKey {
    Admin,
    PrivacyPassport,       // Address of PrivacyPassport contract
    AllowedJurisdictions,  // Vec<Symbol>
}
```

## Public Functions

### `initialize`

Initialize the contract with an admin and PrivacyPassport address.

```rust
fn initialize(env: Env, admin: Address, privacy_passport: Address)
```

| Parameter | Type | Description |
|---|---|---|
| `admin` | Address | Admin who manages jurisdiction allow-list |
| `privacy_passport` | Address | Deployed PrivacyPassport contract ID |

**Auth:** `admin.require_auth()`

---

### `add_jurisdiction`

Add a jurisdiction code to the allow-list.

```rust
fn add_jurisdiction(env: Env, code: Symbol)
```

| Parameter | Type | Description |
|---|---|---|
| `code` | Symbol | ISO country code (e.g., "NG", "GH", "KE") |

**Auth:** `admin.require_auth()`

**Events:**

| Topic | Data |
|---|---|
| `(JurisdictionAdded, code)` | () |

---

### `remove_jurisdiction`

Remove a jurisdiction code from the allow-list.

```rust
fn remove_jurisdiction(env: Env, code: Symbol)
```

| Parameter | Type | Description |
|---|---|---|
| `code` | Symbol | ISO country code to remove |

**Auth:** `admin.require_auth()`

**Events:**

| Topic | Data |
|---|---|
| `(JurisdictionRemoved, code)` | () |

---

### `is_allowed`

Check if a jurisdiction is on the allow-list.

```rust
fn is_allowed(env: Env, jurisdiction: Symbol) -> bool
```

| Parameter | Type | Description |
|---|---|---|
| `jurisdiction` | Symbol | ISO country code |

**Returns:** `bool` — true if jurisdiction is allowed

**Auth:** None (read-only)

---

### `verify`

Verify a passport and jurisdiction are compliant. Checks both the jurisdiction allow-list and PrivacyPassport validity.

```rust
fn verify(env: Env, passport_id: u64, jurisdiction: Symbol) -> bool
```

| Parameter | Type | Description |
|---|---|---|
| `passport_id` | u64 | PrivacyPassport ID |
| `jurisdiction` | Symbol | Jurisdiction to check |

**Returns:** `bool` — true if jurisdiction is allowed AND passport is valid

**Auth:** None (cross-contract read)

**Panics:** None (returns false on failure)

**Events:**

| Topic | Data |
|---|---|
| `(ComplianceCheck, passport_id)` | jurisdiction |

---

### `validate_travel_rule`

Validate an amount against the FATF Travel Rule threshold ($10,000).

```rust
fn validate_travel_rule(env: Env, amount: i128, jurisdiction: Symbol) -> bool
```

| Parameter | Type | Description |
|---|---|---|
| `amount` | i128 | Transaction amount in smallest unit |
| `jurisdiction` | Symbol | Target jurisdiction |

**Returns:** `bool` — true if amount is below or at the $10,000 threshold

**Auth:** None (read-only)

**Panics:** None

**Details:** Amounts above $10,000 (10,000,000,000 in 7-decimal USDC) trigger a travel rule requirement. Transactions exceeding the threshold require additional memo data provided off-chain.
