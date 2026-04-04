# CLI Reference

## Global Flags

| Flag | Description |
|------|-------------|
| `--verbose` | Enable verbose output |
| `--quiet` | Suppress non-error output |
| `--config <PATH>` | Override config file path |

## Commands

### `echoax-cli init`

Initialize EchoAccess on the current device. Creates `~/.config/echoax/` directory structure.

### `echoax-cli status`

Display current sync status, session state, and cloud connection.

### `echoax-cli sync`

| Subcommand | Description |
|------------|-------------|
| `sync upload` | Upload approved local changes to cloud |
| `sync download` | Download latest configs from cloud |
| `sync check` | Check for differences without syncing |

### `echoax-cli profile`

| Subcommand | Description |
|------------|-------------|
| `profile list` | List all configured device profiles |
| `profile show <name>` | Display profile details |
| `profile validate <path>` | Validate a TOML profile file |

### `echoax-cli config`

| Subcommand | Description |
|------------|-------------|
| `config show` | Display current configuration |
| `config path` | Print config directory path |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
