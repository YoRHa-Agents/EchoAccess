# Changelog

## v0.1.2 — Unified Binary (2026-04-04)

Bugfix: validate dashboard route before opening browser to an existing instance.

### Bugfix

- **Stale process detection**: The "already running" health check now verifies version match AND dashboard availability, preventing 404 when a stale or incompatible process occupies the port

---

## v0.1.1 — Unified Binary (2026-04-04)

Consolidates all interfaces into a single `echo_access` binary based on v0.1.0 release feedback.

### Unified Binary

- **Single binary**: `echo_access` now serves as the sole entry point for CLI, TUI, and Web dashboard
- **Default mode**: Running `echo_access` without arguments starts the Web UI and opens the browser automatically
- **TUI integration**: `echo_access tui` launches the NieR: Automata terminal interface (previously a separate `echoax-tui` binary)
- **Web subcommand**: `echo_access web --port 9876` starts the web dashboard with configurable port
- **Removed `echoax-web`**: Standalone Web API crate removed; its functionality is fully embedded in the unified binary

### Web Dashboard Improvements

- **Reliable browser opening**: `open::that` errors are now reported with a fallback message showing the URL
- **Already-running detection**: If the server is already running, opens the browser directly instead of failing
- **Better user feedback**: Clear messages distinguish "starting new server" vs "server already running"

### Architecture Changes

- `echoax-tui` converted from binary crate to library crate (exposes `echoax_tui::run()`)
- `echoax-web` crate removed from workspace (redundant with CLI-embedded web server)
- Cargo workspace reduced from 4 crates to 3 (`echoax-core`, `echoax-cli`, `echoax-tui`)

### Tests

- 100 tests passing (up from 95)
- Added 4 web router tests (dashboard, health, status, favicon)
- Added 2 TUI library tests (run smoke test, app lifecycle)

---

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
