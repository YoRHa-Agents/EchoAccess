# EchoAccess

A cross-platform configuration file synchronization tool written in Rust.

Synchronize SSH configs, dotfiles, and application settings across multiple devices with field-level encryption, approval-based workflows, and a NieR: Automata-inspired TUI.

## Features

- **Profile-based sync**: Per-device TOML profiles with field-level overrides and masking
- **Dual encryption**: age file encryption + AES-256-GCM field-level encryption (SOPS-inspired)
- **3-way merge**: Automatic merge with conflict detection and user approval queue
- **Triple UI**: CLI (clap) + TUI (ratatui, NieR: Automata theme) + Web API (axum REST)
- **Cloud storage**: S3-compatible backends (Aliyun OSS, AWS S3, MinIO) via pluggable `CloudBackend` trait
- **SSH device push**: Discover devices from `~/.ssh/config`, push configs via SSH
- **Cross-platform permissions**: Policy-based permission management (POSIX + Windows stubs)
- **Auto-update**: GitHub Releases integration via `self_update` crate
- **Export/Import**: Encrypted `.echoax.age` archive format for settings portability

## Architecture

```
echoax/
├── crates/
│   ├── echoax-core/     # Shared library: 12 modules
│   │   ├── config/      # AppConfig (TOML)
│   │   ├── crypto/      # age + AES-GCM + argon2 KDF + SessionManager
│   │   ├── device/      # SSH device discovery + push
│   │   ├── error/       # EchoAccessError enum
│   │   ├── permission/  # Sensitivity policies + POSIX/Windows
│   │   ├── portability/ # Export/Import .echoax.age archives
│   │   ├── profile/     # DeviceProfile + SyncRule + TOML loader
│   │   ├── storage/     # CloudBackend trait + SQLite + S3 + Git
│   │   ├── sync/        # 3-way merge + conflicts + approval queue
│   │   ├── trigger/     # File watcher + scheduler + manual
│   │   ├── ui/          # UIAdapter trait + MockAdapter
│   │   └── updater/     # Auto-update via GitHub Releases
│   ├── echoax-cli/      # CLI binary (clap subcommands)
│   ├── echoax-tui/      # TUI binary (ratatui + NieR theme)
│   └── echoax-web/      # Web API binary (axum REST)
```

## Build

```bash
# Build all crates
cargo build --workspace

# Run CLI
cargo run -p echoax-cli -- --help

# Run tests
cargo test --workspace

# Lint
cargo clippy --workspace -- -D warnings
```

## CLI Usage

```bash
echoax init                     # Initialize device
echoax status                   # Show sync status
echoax sync upload|download     # Sync operations
echoax profile list|show|validate <path>  # Profile management
echoax config show|path         # Configuration
```

## License

See [LICENSE](LICENSE) for details.
