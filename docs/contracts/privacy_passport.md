# PrivacyPassport Contract

Zero-knowledge credential verifier compatible with Verite, Reclaim Protocol, and Self Protocol.

## Data Types

```rust
struct PassportState {
    nullifier_hash: BytesN<32>,   // Hash of KYC credential
    jurisdiction: Symbol,          // e.g., "NG", "GH", "KE"
    active: bool,                  // true = valid, false = revoked
    registered_at: u64,            // Unix timestamp of registration
}

enum DataKey {
    Admin,
    PassportCounter,
    Passport(u64),
}
```

## Public Functions

### `initialize`

Initialize the contract with an admin address.

```rust
fn initialize(env: Env, admin: Address)
```

| Parameter | Type | Description |
|---|---|---|
| `admin` | Address | Admin who can register and revoke passports |

**Auth:** `admin.require_auth()`

**Panics:** None

**Events:** None

---

### `register`

Register a new passport with a KYC nullifier hash and jurisdiction.

```rust
fn register(
    env: Env,
    nullifier_hash: BytesN<32>,
    credential_proof: BytesN<32>,
    jurisdiction: Symbol,
) -> u64
```

| Parameter | Type | Description |
|---|---|---|
| `nullifier_hash` | BytesN<32> | Hash of the user's KYC credential |
| `credential_proof` | BytesN<32> | Proof of credential validity |
| `jurisdiction` | Symbol | ISO country code (e.g., "NG") |

**Returns:** `u64` — auto-incremented passport ID

**Auth:** `admin.require_auth()`

**Panics:** None

**Events:**

| Topic | Data |
|---|---|
| `(PassportRegistered, passport_id)` | jurisdiction |

---

### `verify`

Verify a passport is active and matches the required jurisdiction.

```rust
fn verify(env: Env, passport_id: u64, required_jurisdiction: Symbol) -> bool
```

| Parameter | Type | Description |
|---|---|---|
| `passport_id` | u64 | Passport ID to verify |
| `required_jurisdiction` | Symbol | Jurisdiction to check against |

**Returns:** `bool` — true if passport is active and jurisdiction matches

**Auth:** None (read-only)

**Panics:** None (returns false for missing/revoked/mismatched passports)

**Events:** None

---

### `revoke`

Revoke a passport by ID. Only the admin can revoke.

```rust
fn revoke(env: Env, passport_id: u64)
```

| Parameter | Type | Description |
|---|---|---|
| `passport_id` | u64 | Passport ID to revoke |

**Auth:** `admin.require_auth()`

**Panics:** If passport_id does not exist

**Events:**

| Topic | Data |
|---|---|
| `(PassportRevoked, passport_id)` | () |
