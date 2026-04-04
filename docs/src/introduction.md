# EchoAccess User Guide

EchoAccess is a cross-platform configuration file synchronization tool written in Rust. It synchronizes SSH configs, dotfiles, and application settings across multiple devices with field-level encryption, approval-based workflows, and a NieR: Automata-inspired terminal interface.

## What EchoAccess Does

- **Syncs configs** across Linux and macOS devices via S3-compatible cloud storage
- **Encrypts selectively** using age (whole-file) and AES-256-GCM (per-field)
- **Handles conflicts** with 3-way merge and user approval queues
- **Manages permissions** preserving file modes across platforms (0600 for SSH keys)
- **Pushes to devices** via SSH, discovering hosts from your `~/.ssh/config`

## Architecture Overview

EchoAccess ships as a single unified binary (`echo_access`) with three interface modes:

| Mode | Command | Purpose |
|------|---------|---------|
| Web (default) | `echo_access` | Web dashboard with auto-open browser |
| CLI | `echo_access <command>` | Command-line subcommands (clap) |
| TUI | `echo_access tui` | NieR: Automata terminal dashboard (ratatui) |

The codebase consists of three crates:

| Crate | Type | Purpose |
|-------|------|---------|
| `echoax-core` | Library | Shared logic for all interfaces |
| `echoax-cli` | Binary | Unified binary (`echo_access`) — CLI + Web dashboard |
| `echoax-tui` | Library | TUI terminal interface (ratatui + NieR theme) |
