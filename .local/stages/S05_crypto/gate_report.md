# Stage S05 — Crypto Core — Gate Report

**Date:** 2026-04-03  
**Branch:** `stage/S05-crypto` (intended; create from `main` per stage brief)  
**Tier:** 1 | **Wave:** 3 | **Milestone:** M1 partial

## Deliverables

| Item | Status |
|------|--------|
| `crates/echoax-core/src/crypto/mod.rs` — module + re-exports | Done |
| `crypto/kdf.rs` — Argon2 `derive_key`, `generate_salt` | Done |
| `crypto/session.rs` — `SessionManager` (`lock` / `unlock` / `ensure_unlocked` / `get_key`) | Done |
| `crypto/file_enc.rs` — `encrypt_file` / `decrypt_file` via `age` scrypt recipient | Done |
| `pub mod crypto` in `lib.rs` | Done |
| Dependencies: `age` (with `cli-common`), `argon2`, `rand`, `zeroize` | Done |

## Verification (run locally)

```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

Prior run in this environment: **build, clippy (`-D warnings`), and full workspace tests passed** (including `crypto::` unit tests). `age` passphrase tests use device-tuned scrypt (~few seconds).

## Security notes

- No logging or printing of key material, passphrases, or ciphertext in new code.
- Master key bytes are zeroized on `lock`, replacement `unlock`, and `Drop`.
- Wrong-passphrase decrypt surfaces `EchoAccessError::Crypto` without leaking secrets.

## Follow-ups (out of scope for this stage)

- Replace `PLACEHOLDER_SALT` in `SessionManager::unlock` with salt loaded from persisted profile/storage.
- Optional: wire `SessionConfig` (`timeout_secs`, `auto_lock`) for auto-lock timers.
- Constant-time comparisons if comparing derived keys or MACs in application code (not required for current age/argon2 usage paths).
