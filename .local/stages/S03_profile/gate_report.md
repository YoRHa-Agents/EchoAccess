# Stage S3: Profile System — Gate Report

**Branch:** `stage/S03-profile`  
**Date:** 2026-04-03  
**Status:** PASS

## Deliverables

| Item | Notes |
|------|--------|
| `crates/echoax-core/src/profile/mod.rs` | Module tree + re-exports (`load_profile`, `validate_profile`, model types). |
| `crates/echoax-core/src/profile/model.rs` | `DeviceProfile`, `DeviceInfo`, `SyncRule`, `FieldOverride`; serde `Deserialize`/`Serialize`; `FieldOverride::new`. |
| `crates/echoax-core/src/profile/loader.rs` | `load_profile`, `validate_profile`; maps I/O and TOML errors to `EchoAccessError::Profile`. |
| `lib.rs` | `pub mod profile;` (alongside existing modules). |

## Tests

- **Loader:** valid profile load; invalid TOML; empty hostname; empty `sync_rules` (root `sync_rules = []` before `[device]` to satisfy TOML scoping); empty rule `source`/`target`; `field_overrides` map round-trip.
- **Model:** `device_profile_deserialize` (incl. dotted keys in overrides); `device_profile_serialize_roundtrip`.

## Verification

Run from repo root on branch `stage/S03-profile`:

```text
cargo build --workspace
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

Lib and integration tests for `echoax-core` (including all `profile::` tests) passed in this environment. If `cargo test --workspace` fails on **doc-tests** with missing `.rlib` / crate errors, retry with `cargo test --workspace -j 1` or `cargo clean` then retest (parallel rustdoc race).

## Git

If `stage/S03-profile` has no new commit yet, stage and commit:

```text
git add crates/echoax-core/src/profile crates/echoax-core/src/lib.rs
git commit -m "feat(core): add device profile TOML loader and validation"
```

## Notes

- TOML: keys after `[device]` belong to `device` until a new table header. Root-level `sync_rules = []` must appear before `[device]` when using an empty rules list in tests.
