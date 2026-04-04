# Stage S4: Storage Layer — Gate Report

**Tier:** 1 | **Wave:** 3 | **Milestone:** M1 partial  
**Records path:** `.local/stages/S04_storage/`  
**Target branch:** `stage/S04-storage` (create from your current `main`, then commit these changes there — do not push to protected `main` without MR).

## Deliverables

| Path | Description |
|------|-------------|
| `crates/echoax-core/src/storage/mod.rs` | Submodules + `CloudBackend`, `SqliteStore`, `S3Backend`, record types |
| `crates/echoax-core/src/storage/traits.rs` | `CloudBackend`: `upload`, `download`, `delete`, `list`, `exists` (async, `async_trait`) |
| `crates/echoax-core/src/storage/sqlite.rs` | `SqliteStore` + `sync_versions` / `devices` tables, `PRAGMA user_version` migration to v1 |
| `crates/echoax-core/src/storage/s3.rs` | `S3Backend` full trait impl (`aws-sdk-s3` + `endpoint_url`, static keys, region inference for `oss-*` URLs) |
| `crates/echoax-core/src/lib.rs` | `pub mod storage;` |
| `crates/echoax-core/Cargo.toml` | `async-trait`, `rusqlite` (bundled), `aws-sdk-s3`, `aws-config`, `tokio` |

## Unit tests (in crate)

- `storage::s3::tests::cloud_backend_trait_is_object_safe` — `Box<dyn CloudBackend>` + noop impl  
- `storage::sqlite::tests::*` — migration / `sync_versions` / `devices` CRUD on in-memory DB  
- `storage::s3::tests::s3_backend_construction` — client construction (no live OSS)

## Verification command

Run locally (required for gate sign-off if CI not run here):

```bash
cargo build --workspace && cargo clippy --workspace -- -D warnings && cargo test --workspace
```

## Gate

**PENDING LOCAL CI** — Implementation is in tree; re-run the commands above on your machine or in CI to mark **PASS**.
