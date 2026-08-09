# passwd-derive

Deterministic password derivation library used by [NoPassPlz](https://github.com/greekfetacheese/no-pass-plz).

Given a master username and password, this crate produces a high-entropy seed once (via Argon2id), then derives unlimited unique site passwords from that seed by index without ever storing the passwords themselves.

## How it works

### 1. Seed generation (`PasswordDeriver::new`)

1. Validate credentials (non-empty username/password, password matches confirmation).
2. Salt = `SHA3-512(username)`.
3. Seed = `Argon2id(password, salt)` → 64-byte (`512`-bit) secret, held in a locked `SecureArray`.

Default Argon2 parameters (from `default_argon2()`):

| Parameter     | Value                          |
|---------------|--------------------------------|
| Memory cost   | `4_096_000` KiB (~4 GiB)       |
| Time cost     | `32` iterations                |
| Parallelism   | `1`                            |
| Hash length   | `64` bytes (recommended)       |

On typical hardware these defaults take on the order of ~2 minutes 15 seconds (as of 2026). You can pass a custom `Argon2` config for faster tests or different security trade-offs.

### 2. Password derivation (`derive_at`)

For index `n`:

```text
HMAC-SHA3-512(key = seed, message = n.to_be_bytes()) → 64 bytes → hex string (128 chars)
```

Same master credentials + same index always yield the same password. Different indexes yield independent passwords.

### 3. Cleanup (`erase`)

Zeroizes the in-memory seed when you are done.

Sensitive material is handled with [`secure-types`](https://crates.io/crates/secure-types) (`SecureString`, `SecureArray`, `SecureVec`) and explicit `zeroize` where intermediate buffers are used.

## Example

```rust
use passwd_derive::{PasswordDeriver, default_argon2};
use secure_types::SecureString;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let deriver = PasswordDeriver::new(
        SecureString::from("my-username"),
        SecureString::from("my-long-master-password"),
        SecureString::from("my-long-master-password"),
        default_argon2(),
    )?;

    // Site / account passwords by index
    let pass_0 = deriver.derive_at(0); // SecureString, 128 hex chars
    let pass_1 = deriver.derive_at(1);

    // When finished
    // let mut deriver = deriver;
    // deriver.erase();

    let _ = (pass_0, pass_1);
    Ok(())
}
```

For unit tests, use a cheap Argon2 config:

```rust
use argon2_rs::Argon2;
use passwd_derive::PasswordDeriver;
use secure_types::SecureString;

let argon2 = Argon2::new(16_000, 1, 1);
let deriver = PasswordDeriver::new(
    SecureString::from("username"),
    SecureString::from("password"),
    SecureString::from("password"),
    argon2,
)
.unwrap();

assert_eq!(
    deriver.derive_at(0).unlock_str(|s| s.to_string()),
    "24edd00e13bba1a55bf1ec2c74961e5545426e3c9dee7c012a58a7832a53c8ca\
     321a7a8cbe58127b1b927548a1f5378184951b6c7cf3b3f18405677c66bcda4b"
);
```

## API surface

| Item                    | Description |
|-------------------------|-------------|
| `PasswordDeriver`       | Holds the Argon2 seed and config. |
| `PasswordDeriver::new`  | Build from master username + password (+ confirm) and an `Argon2` instance. |
| `PasswordDeriver::derive_at` | Deterministic password for `index` as a hex `SecureString`. |
| `PasswordDeriver::erase`| Wipe the seed from memory. |
| `default_argon2()`      | Production Argon2id settings (slow by design). |
| `M_COST` / `T_COST` / `P_COST` | Constants backing the defaults. |

## Security notes

- **Master credentials are the root of trust.** If they are forgotten, derived passwords cannot be recovered. If they are weak or leaked, all derived passwords are at risk.
- **No passwords are persisted by this crate.** Only the caller decides what (if anything) to store; NoPassPlz only keeps non-secret index metadata (title, description, etc.).
- **Default KDF cost is intentionally high** to slow offline guessing of the master password. Do not lower it in production without understanding the trade-off.
- **Outputs are hex-encoded 512-bit values** — high entropy, but not “human memorable”; treat them as machine passwords. 

## License

MIT OR Apache-2.0
