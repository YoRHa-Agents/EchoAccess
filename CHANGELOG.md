# Changelog

## v0.1.0 — Initial Release (2026-04-04)

First release of EchoAccess, a cross-platform configuration file synchronization tool.

### Core Library (echoax-core)

- **Profile System**: TOML-based device profiles with sync rules, field overrides, and masking
- **Storage Layer**: `CloudBackend` trait with SQLite (local metadata) and S3-compatible (Aliyun OSS) backends
- **Crypto Core**: age file encryption, AES-256-GCM field-level encryption, argon2 KDF, SessionManager with auto-lock
- **Sync Engine**: 3-way merge (diffy), conflict detection, approval queue
- **Permission Manager**: Sensitivity-based policies, POSIX permission apply/verify, Windows stub
- **Trigger System**: File watcher, scheduled sync, manual trigger
- **Device Discovery**: SSH config parser, device push stubs, bootstrap stubs
- **Export/Import**: Encrypted `.echoax.age` archive format
- **Auto-Update**: GitHub Releases version checker + installer stubs
- **Multi-Backend**: Git backend stub implementing CloudBackend trait

### CLI (echoax-cli)

- clap-based subcommand architecture
- Commands: `init`, `status`, `sync upload/download/check`, `profile list/show/validate`, `config show/path`

### TUI (echoax-tui)

- NieR: Automata inspired theme (10-token warm palette)
- Dashboard, sync status, and profiles views
- ratatui + crossterm integration

### Web API (echoax-web)

- axum REST server on port 9876
- Endpoints: `GET /api/health`, `GET /api/status`

### Infrastructure

- Cargo workspace with 4 crates
- GitHub Actions CI (check, fmt, clippy, test)
- GitHub Actions release workflow (4 platform targets)
- GitHub Pages with mdBook user guide
- NieR-themed landing page
- 95 tests passing
