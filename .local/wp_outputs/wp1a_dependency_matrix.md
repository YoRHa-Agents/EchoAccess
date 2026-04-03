# WP-1A Output: Module Dependency Matrix

## echoax-core Internal Module Dependencies

| Module | Depends On | Confidence | Notes |
|--------|-----------|------------|-------|
| **config/** | (none) | High | Shared leaf: serde, toml, dirs. All modules import config types. |
| **error.rs** | (none) | High | Shared leaf: thiserror. All modules import error types. |
| **ui/adapter.rs** | error, config | High | UIAdapter trait references AppState (config) and Result (error). Trait only — impls in binary crates. |
| **profile/** | config, error | High | TOML loader uses config for paths. Profile/SyncRule data models. |
| **storage/** | config, error | High | CloudBackend trait + SQLite/S3 impls. Uses config for connection settings. |
| **crypto/** | config, error, **ui** | High | SessionManager.ensure_unlocked() calls ui_adapter.prompt_password(). KDF uses config for params. |
| **sync/** | profile, storage, crypto, permission, **ui**, config, error | High | Core orchestration: reads profiles, persists to storage, encrypts via crypto, verifies permissions, prompts via UIAdapter for conflicts. |
| **permission/** | config, error | High | Policy definitions from config. Platform-specific impls via #[cfg]. No other core module deps. |
| **trigger/** | sync, config, error | High | Triggers sync operations. Uses config for intervals/debounce. |
| **device/** | profile, crypto, config, error | High | Generates draft profiles, encrypts payloads for SSH push. Uses openssh. |
| **updater/** | config, error | High | Uses config for GitHub repo, channel, check interval. Standalone module. |
| **portability/** | profile, crypto, config, error | High | Exports/imports profiles + re-encrypts master key. |

## Cross-Cutting Dependencies (shared leaves)

All modules depend on `config/` and `error.rs`. These are **frozen after S2** (Core Abstractions Stage).

## Critical Dependency: crypto -> ui

`SessionManager.ensure_unlocked()` calls `self.ui_adapter.prompt_password()` (Architecture.md §4.2, lines 324-332). This means **crypto depends on the UIAdapter trait definition** (not any specific implementation). The trait must be defined in S2 before S5 (Crypto Core) can implement SessionManager.

## Crate Dependencies

| Binary Crate | Depends On | Specific Deps |
|-------------|-----------|---------------|
| echoax-cli | echoax-core | + clap, clap_complete, indicatif |
| echoax-tui | echoax-core | + ratatui, crossterm |
| echoax-web | echoax-core | + axum, tower-http |

All three binary crates depend on echoax-core as a single crate. No cross-binary-crate dependencies.

## Hidden Dependencies Found

1. **crypto -> ui**: SessionManager needs UIAdapter for password prompt (confirmed in §4.2)
2. **sync -> ui**: Conflict resolution needs UIAdapter for user prompts (confirmed in §4.9 UIAdapter trait: prompt_conflict_resolution)
3. **sync -> permission**: Post-sync permission verification required (confirmed in §7.2: "Sensitivity::Critical files verify() after sync")
4. **device -> profile**: Device discovery generates draft profiles (confirmed in §4.6)
5. **portability -> storage**: Optional SQLite snapshot export (confirmed in §3.4 ExportArchive.sync_state)

## Validated Dependency DAG (no cycles)

```
config, error (shared leaves)
    |
    +-> ui/adapter (trait def)
    |
    +-> profile
    |     |
    +-> storage
    |     |
    +-> crypto (depends on ui for password prompt)
    |     |       |
    +-> permission |
    |     |       |
    |     +-------+-> sync (depends on profile, storage, crypto, permission, ui)
    |                   |
    |                   +-> trigger
    |
    +-> device (depends on profile, crypto)
    +-> updater (standalone)
    +-> portability (depends on profile, crypto, optionally storage)
```

No circular dependencies detected. The graph is a clean DAG.
