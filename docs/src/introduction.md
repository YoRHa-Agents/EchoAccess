# EchoAccess User Guide

EchoAccess is a cross-platform configuration file synchronization tool written in Rust. It synchronizes SSH configs, dotfiles, and application settings across multiple devices with field-level encryption, approval-based workflows, and a NieR: Automata-inspired terminal interface.

## What EchoAccess Does

- **Syncs configs** across Linux and macOS devices via S3-compatible cloud storage
- **Encrypts selectively** using age (whole-file) and AES-256-GCM (per-field)
- **Handles conflicts** with 3-way merge and user approval queues
- **Manages permissions** preserving file modes across platforms (0600 for SSH keys)
- **Pushes to devices** via SSH, discovering hosts from your `~/.ssh/config`

## Architecture Overview

EchoAccess consists of four components:

| Component | Binary | Purpose |
|-----------|--------|---------|
| Core library | — | Shared logic for all interfaces |
| CLI | `echoax-cli` | Command-line interface (clap) |
| TUI | `echoax-tui` | Terminal dashboard (ratatui) |
| Web API | `echoax-web` | REST API (axum) on port 9876 |
