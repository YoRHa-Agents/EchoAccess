# Changelog

## v0.1.6 — Release Artifact Audit & Build Refresh (2026-04-08)

Release-focused follow-up that validates the shipped artifact set, refreshes the release metadata to v0.1.6, and makes local full-matrix packaging closer to the official CI release path.

### Release Artifact Health

- **Manifest verification**: Confirmed the published `checksums.sha256` entries match all 5 official v0.1.5 release archives
- **Official asset set**: Verified the public release contains the expected Linux, macOS, and Windows MSVC artifacts with matching per-asset checksums
- **Release confidence**: Established a clean baseline before cutting the next version

### Local Release Build

- **Windows MSVC from non-Windows hosts**: `scripts/build-release.sh` now uses `cargo-xwin` for `x86_64-pc-windows-msvc` when run from Linux or macOS
- **Cross-platform parity**: Non-Windows targets continue to use `cargo zigbuild`, while Windows hosts keep using native `cargo build`
- **Combined checksums**: Local release builds now emit `dist/checksums.sha256` alongside the per-asset `.sha256` files

### Release Metadata

- **Workspace bump**: Updated the workspace version to `0.1.6`
- **Docs refresh**: Updated README, installation docs, landing page content, and dashboard version labels to reference `v0.1.6`

### Verification

- **Release build**: `./scripts/build-release.sh 0.1.6` completed successfully for all 5 official targets
- **Test suite**: `cargo test --workspace` passed with **169 tests**

## v0.1.5 — Port Management, Cloud Config & Export Filter (2026-04-07)

Feedback-driven release addressing v0.1.4 review: web UI port management, complete cloud configuration, and export with search/filter.

### Port Management

- **Configurable port**: `general.port` field in `config.toml` (default: 9876)
- **Runtime vs configured port**: Server info now shows both the live port and the saved port value
- **Restart awareness**: Saving a new port warns that the change takes effect on the next restart
- **Config-driven startup**: Server reads port from config on launch instead of hardcoded value

### Cloud Configuration (Complete)

- **Completeness checks**: Cloud config now validates `endpoint`, `bucket`, `access_key_id`, and `secret_access_key`
- **Accurate cloud state**: Web status now distinguishes `disabled`, `incomplete`, and `configured`
- **Clear test feedback**: Cloud test reports missing fields up front and explains that backend access is not integrated yet
- **Docs sync**: Configuration docs now show the full endpoint/bucket/credential shape instead of endpoint-only examples

### Export with Search/Filter

- **Profile-based preview**: Export preview now searches device profiles instead of unrelated tracked file state
- **Aligned export filtering**: `POST /api/export` exports the same filtered profiles shown in preview
- **Secret redaction**: Sensitive profile override values are redacted before archive encryption
- **Legacy-safe fallback**: Export still supports unstructured TOML by falling back to raw profile content when needed
- **Bilingual updates**: EN/ZH dashboard copy now reflects profile filtering, redaction, and cloud completeness status

### Tests

- **164 tests passing** (up from 155)
- Added cloud completeness, export preview, and secret-redaction coverage
- Preserved portability tests by keeping export compatibility with legacy TOML inputs

## v0.1.4 — i18n, Theme Toggle, Auto-Update & Multi-Platform CI (2026-04-05)

Feedback-driven release adding internationalization, visual customization, self-update capability, and cross-platform CI improvements.

### Internationalization (i18n)

- **Bilingual UI**: Full English/Chinese support with 140+ translation keys
- **Inline translation system**: `t(key)` helper with `TRANSLATIONS` registry, `data-i18n` attributes for static text
- **Language switcher**: EN/ZH toggle in header, persists to `localStorage` and `AppConfig.general.language`
- **Dynamic content**: All JS-generated elements (file list, toasts, logs) use `t()` calls

### Day/Night Theme Toggle

- **Light theme**: Warm parchment palette under `[data-theme="light"]` CSS selector (NieR-inspired light variant)
- **Theme toggle**: Sun/moon icon in header switches between dark and light modes
- **Persistence**: Theme preference saved to `localStorage` and `AppConfig.general.theme`
- **WCAG AA contrast**: Light palette designed for accessibility compliance

### Cloud & Server Management

- **Expanded Cloud tab**: Full management section with connection status indicator, endpoint config, sync interval
- **Test Connection**: `POST /api/cloud/test` validates cloud backend reachability
- **Server Info**: `GET /api/server/info` returns config path, port, version, uptime
- **Server info panel**: Read-only display of server metadata in Cloud tab

### Help Tooltips

- **22 bilingual help entries**: Coverage across all 7 section headings, action buttons, and config tabs
- **CSS tooltip component**: Pure CSS popover with hover/focus behavior, no external library
- **Accessible**: `aria-label`, `role="tooltip"`, keyboard navigation, Escape to dismiss
- **Mobile-friendly**: Tap-to-show on touch devices, responsive positioning

### Auto-Update

- **Version checking**: `echo_access update check` queries GitHub Releases API
- **Self-update**: `echo_access update install` downloads, verifies SHA-256, extracts, and replaces binary
- **Web API**: `GET /api/update/check` and `POST /api/update/install` endpoints
- **Dashboard UI**: Update Status section with check/install buttons, release notes preview
- **Platform-aware**: Automatically detects target triple and selects correct release asset
- **Core types**: `parse_github_release()`, `get_platform_target()`, `binary_name()`, `archive_extension()`

### Multi-Platform CI/CD

- **Cross-OS testing**: CI matrix expanded to ubuntu-latest, macos-latest, windows-latest
- **Windows release**: Added `x86_64-pc-windows-msvc` target (5 targets total)
- **Cargo caching**: `actions/cache@v4` for registry, git, and target directories
- **Build script**: `build-release.sh` now fail-fast with build summary, Windows .zip support

### Dependencies Added

- `sha2` 0.10 — SHA-256 checksum verification for update downloads
- `flate2` 1 — gzip decompression for tar.gz release archives
- `tar` 0.4 — tar archive extraction
- `zip` 2 — zip archive extraction (Windows releases)
- `tempfile` 3 — temporary file handling during updates

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

### Directory Scanner

- **Recursive directory scanning**: `scan_directory()` with configurable max depth and file limits
- **Smart exclusions**: Automatically skips `.git`, `node_modules`, `target`, `__pycache__`, and hidden directories
- **Scan API**: `POST /api/files/scan` discovers files in a directory for bulk tracking

### Architecture

- **AppState**: Shared state struct with `Arc<RwLock<...>>` for concurrent access
- **22 API endpoints**: Up from 4 (health, status, config, files, files/scan, sync, profiles, session, groups, conflicts)
- **Core library wiring**: AppConfig, SessionManager, SyncEngine, GroupStore, ConflictStore all connected to web and TUI layers
- **TUI panic hook**: Terminal state properly restored on crash

### Tests

- **135 tests passing** (up from 100)
- Added 5 group API tests (CRUD, members, batch sync)
- Added 3 conflict API tests (list, resolve, store)
- Added 13 core tests (groups: 7, conflicts: 6)
- Added 5 scanner tests (depth, max files, exclusions)
- Added 2 TUI tests (key handling, view navigation)
- Added 2 CLI sync resolve tests
- All existing tests continue to pass

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
