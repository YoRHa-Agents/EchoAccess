# CLI Reference

## Usage

```
echo_access [OPTIONS] [COMMAND]
```

Running `echo_access` without any command starts the Web UI dashboard.

## Global Flags

| Flag | Description |
|------|-------------|
| `--verbose` | Enable verbose output |
| `--quiet` | Suppress non-error output |
| `--config <PATH>` | Override config file path |

## Commands

### `echo_access web`

Start the Web UI dashboard (this is also the default when no command is given).

| Flag | Description |
|------|-------------|
| `--port <PORT>` | Port to listen on (default: 9876) |
| `--no-open` | Don't auto-open the browser |

### `echo_access tui`

Launch the NieR: Automata-styled TUI terminal dashboard.

### `echo_access init`

Initialize EchoAccess on the current device. Creates `~/.config/echoax/` directory structure.

### `echo_access status`

Display current sync status, session state, and cloud connection.

### `echo_access sync`

| Subcommand | Description |
|------------|-------------|
| `sync upload` | Upload approved local changes to cloud |
| `sync download` | Download latest configs from cloud |
| `sync check` | Check for differences without syncing |

### `echo_access profile`

| Subcommand | Description |
|------------|-------------|
| `profile list` | List all configured device profiles |
| `profile show <name>` | Display profile details |
| `profile validate <path>` | Validate a TOML profile file |

### `echo_access config`

| Subcommand | Description |
|------------|-------------|
| `config show` | Display current configuration |
| `config path` | Print config directory path |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
