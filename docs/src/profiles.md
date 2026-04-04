# Device Profiles

Each device has a TOML profile that declares which files to sync and how to handle platform differences.

## Profile Structure

```toml
[device]
os = "linux"        # linux, macos, windows
role = "server"     # server, desktop, dev, edge
hostname = "srv-01" # Unique device identifier

[[sync_rules]]
source = "ssh/config.base"          # Source file in cloud storage
target = "~/.ssh/config"            # Target path on this device
transforms = ["strip_gui_hosts"]    # Transform pipeline
masked_fields = ["Host desktop-*"]  # Fields to mask on this device

[sync_rules.field_overrides]
"user.email" = "ops@company.com"    # Per-device field values
```

## Sync Rules

Each `[[sync_rules]]` entry defines one file to synchronize:

| Field | Type | Description |
|-------|------|-------------|
| `source` | String | Path in cloud storage |
| `target` | String | Local file path (supports `~`) |
| `transforms` | String[] | Ordered list of transform names |
| `masked_fields` | String[] | Fields hidden on this device |
| `field_overrides` | Map | Key-value overrides per device |

## Managing Profiles

```bash
echoax-cli profile list               # List all profiles
echoax-cli profile show <name>        # Display profile details
echoax-cli profile validate <path>    # Validate TOML syntax + rules
```
