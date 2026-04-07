# Configuration

EchoAccess configuration lives at `~/.config/echoax/config.toml`.

## Config File Structure

```toml
[general]
language = "en"
auto_start = false
log_level = "info"
port = 9876

[session]
timeout_secs = 900
auto_lock = true

[trigger]
hotkey = "ctrl+shift+s"
on_login = false

[cloud]
enabled = true
endpoint = "https://echo-access-data.oss-cn-beijing.aliyuncs.com"
region = "cn-beijing"
bucket = "echo-access-data"
access_key_id = "AKIAIOSFODNN7EXAMPLE"
secret_access_key = "SECRET"
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
- `port`: Web UI listen port (default: 9876). Changes take effect after restart.

### `[session]`
- `timeout_secs`: Auto-lock session after N seconds of inactivity (default: 900)
- `auto_lock`: Enable session auto-lock

### `[cloud]`
- `enabled`: Enable cloud sync
- `endpoint`: S3-compatible storage endpoint
- `region`: Storage region or provider-specific region hint
- `bucket`: Bucket or container name
- `access_key_id`: Access key identifier for authenticated access
- `secret_access_key`: Secret access key used with the access key id
- `sync_interval_secs`: Automatic sync interval

When cloud sync is enabled, the Web UI validates that `endpoint`, `bucket`, `access_key_id`, and `secret_access_key` are all present before testing the configuration.

### View Configuration

```bash
echo_access config show    # Display current config
echo_access config path    # Show config file location
```
