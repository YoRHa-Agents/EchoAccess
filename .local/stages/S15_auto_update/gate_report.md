# Stage S15: Auto-Update — Gate Report

**Branch:** `stage/S15-auto-update` (create from `main` if not present)  
**Date:** 2026-04-03

## Delivered

| Item | Status |
|------|--------|
| `crates/echoax-core/src/updater/mod.rs` | Added — re-exports `UpdateInfo`, `check_for_updates`, `semver_update_available`, `install_update` |
| `crates/echoax-core/src/updater/checker.rs` | Added — `UpdateInfo`, semver helper, async `check_for_updates` (no network) |
| `crates/echoax-core/src/updater/installer.rs` | Added — async `install_update` stub + `self_update::get_target()` via `black_box` |
| `lib.rs` | `pub mod updater;` |
| `echoax-core/Cargo.toml` | `self_update` 0.43 + `rustls`, `semver` 1 |
| `UpdateConfig` | `check_interval_hours: u64` (default 24), serde + tests updated |
| `.gitignore` | `.cargo-s15/` (local Cargo cache from isolated build attempt) |

## Unit tests (in crate)

- `update_info_construction`
- `semver_*` (newer / equal / older / invalid → `Network`)
- `check_stub_returns_no_update` (`#[tokio::test]`)
- `install_stub_ok` (`#[tokio::test]`)
- Config: full TOML with `check_interval_hours = 48`, empty default `24`

## Verification

Automated `cargo` invocations from this agent environment returned no captured output (tooling issue). **Run locally / in CI:**

```bash
cd /home/agent/workspace/EchoAccess
cargo build --workspace
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

Suggested focused run:

```bash
cargo test -p echoax-core updater
cargo test -p echoax-core config::model::tests
```

## Commit (suggested)

```bash
git add crates/echoax-core/Cargo.toml crates/echoax-core/src/lib.rs \
  crates/echoax-core/src/config/model.rs crates/echoax-core/src/updater/ \
  .gitignore .local/stages/S15_auto_update/gate_report.md
git commit -m "feat(core): Stage S15 auto-update module (stub check/install)"
```

## Notes

- API errors use `EchoAccessError::Network(String)` for bad semver (per S2 guidance).
- `check_for_updates` intentionally does not call `api.github.com` yet.
- `install_update` does not download or `self_replace`; it only logs and references `self_update` for linkage.
