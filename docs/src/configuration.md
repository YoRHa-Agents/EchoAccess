# Configuration

EchoAccess configuration lives at `~/.config/echoax/config.toml`.

## Config File Structure

```toml
[general]
language = "en"
auto_start = false
log_level = "info"

[session]
timeout_secs = 900
auto_lock = true

[trigger]
hotkey = "ctrl+shift+s"
on_login = false

[cloud]
enabled = true
endpoint = "https://echo-access-data.oss-cn-beijing.aliyuncs.com"
sync_interval_secs = 3600

[update]
auto_check = true
check_interval_hours = 24
channel = "stable"
```

## Config Sections

### `[general]`
- `language`: UI language (default: "en")
- `auto_start`: Start EchoAccess on system login
- `log_level`: Logging verbosity: trace, debug, info, warn, error

### `[session]`
- `timeout_secs`: Auto-lock session after N seconds of inactivity (default: 900)
- `auto_lock`: Enable session auto-lock

### `[cloud]`
- `enabled`: Enable cloud sync
- `endpoint`: S3-compatible storage endpoint
- `sync_interval_secs`: Automatic sync interval

### View Configuration

```bash
echoax-cli config show    # Display current config
echoax-cli config path    # Show config file location
```
