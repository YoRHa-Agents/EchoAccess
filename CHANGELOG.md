# Changelog

## v0.1.3 — Deep Fix & Feature Iteration (2026-04-04)

Major release bridging the gap between the core library and user-facing interfaces. Web dashboard upgraded from demo to functional, TUI made interactive, CLI commands wired to real data, and new sync group management and conflict resolution capabilities added.

### Web Dashboard (Major Overhaul)

- **Dynamic file list**: Files now load from `/api/files` instead of hardcoded HTML
- **Configuration management**: Full config panel with tabs for General, Session, Trigger, Cloud, and Update sections
- **Config CRUD**: Read/write config via `GET/PUT /api/config`, saved to `config.toml`
- **File tracking**: Add/remove tracked files via `/api/files/add` and `/api/files/remove`
- **Real sync operations**: Upload/download buttons call `/api/sync/upload` and `/api/sync/download`
- **Live status**: `/api/status` returns real runtime state (session, cloud, file counts)
- **Profile listing**: `/api/profiles` scans and loads profile TOML files
- **Session management**: Lock/unlock session via `/api/session`
- **Toast notifications**: Success/error feedback for all operations
- **Loading states**: Button loading indicators and skeleton loading for file list
- **Responsive layout**: Mobile-friendly with proper breakpoints
- **Empty states**: Helpful messages when no files are tracked

### TUI (Terminal User Interface)

- **Working event loop**: Full crossterm + ratatui interactive terminal
- **Three views**: Dashboard, Sync, and Profiles with tab-based navigation
- **Keyboard navigation**: Tab/Shift-Tab switch views, number keys (1-3) for direct access, q to quit
- **NieR: Automata theme**: Consistent styling across all views
- **Graceful exit**: Proper terminal state restoration on exit

### Sync Groups & Batch Operations

- **SyncGroup model**: Named groups with path prefixes, include/exclude globs, and tags
- **GroupStore**: In-memory group CRUD with path resolution
- **Batch sync**: `/api/sync/batch` syncs all files in a group at once
- **Group membership**: `/api/groups/{id}/members` shows which tracked files belong to a group
- **Tag filtering**: Groups support tags for selective operations

### Conflict Resolution

- **ConflictStore**: Track, list, and resolve file conflicts
- **Resolution strategies**: Accept ours, theirs, base, or provide custom merged content
- **ConflictView**: Rich view with base/ours/theirs content for UI display
- **API endpoints**: `/api/conflicts` lists conflicts, `/api/conflicts/resolve` resolves them

### CLI Improvements

- **`config show`**: Now loads and displays actual `config.toml` content
- **`init`**: Creates default `config.toml` on first run
- **`status`**: Shows real cloud connection status from config
- **`profile list`**: Scans profiles directory and lists found profiles
- **`profile show`**: Displays profile details including sync rules
- **`sync check`**: Reports engine status and profile count
- **`sync upload/download`**: Shows cloud config status, reports SDK integration status

### Architecture

- **AppState**: Shared state struct with `Arc<RwLock<...>>` for concurrent access
- **21 API endpoints**: Up from 4 (health, status, config, files, sync, profiles, session, groups, conflicts)
- **Core library wiring**: AppConfig, SessionManager, SyncEngine, GroupStore, ConflictStore all connected to web layer

### Tests

- **127 tests passing** (up from 100)
- Added 5 group API tests (CRUD, members, batch sync)
- Added 2 conflict API tests (list, resolve)
- Added 13 core tests (groups: 7, conflicts: 6)
- Added 2 TUI tests (key handling, view navigation)
- All existing 94 core tests continue to pass

---

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
