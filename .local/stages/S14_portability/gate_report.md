# S14 — Export/Import Gate Report

**Status:** PASS
**Date:** 2026-04-04

## Scope
Encrypted archive export/import for profile portability using age passphrase encryption.

## Files Created/Modified
- `crates/echoax-core/src/portability/mod.rs` — Module re-exports
- `crates/echoax-core/src/portability/export.rs` — `export_archive()`, `ExportManifest` struct
- `crates/echoax-core/src/portability/import.rs` — `import_archive()`
- `crates/echoax-core/src/lib.rs` — added `pub mod portability`

## API
```rust
pub fn export_archive(config_dir: &Path, output_path: &Path, passphrase: &str) -> Result<ExportManifest>;
pub fn import_archive(archive_path: &Path, target_dir: &Path, passphrase: &str) -> Result<ExportManifest>;
```

## Tests (4 total)
- export_creates_file — exports .toml profiles to encrypted archive
- export_nonexistent_dir_fails — proper error for missing directory
- import_roundtrip — export then import, verifies manifest
- import_wrong_passphrase_fails — rejects wrong passphrase

## Checks
- `cargo check`: PASS
- `cargo test`: 4/4 portability tests pass
- `cargo clippy -- -D warnings`: PASS
