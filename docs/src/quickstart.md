# Quick Start

## 1. Initialize

```bash
echoax-cli init
```

This creates the config directory at `~/.config/echoax/` with default settings.

## 2. Create a Device Profile

Create a profile at `~/.config/echoax/profiles/my-device.toml`:

```toml
[device]
os = "linux"
role = "server"
hostname = "my-server"

[[sync_rules]]
source = "ssh/config"
target = "~/.ssh/config"
transforms = []
masked_fields = []
```

## 3. Validate the Profile

```bash
echoax-cli profile validate ~/.config/echoax/profiles/my-device.toml
```

## 4. Check Status

```bash
echoax-cli status
```

## 5. Sync

```bash
echoax-cli sync upload    # Push local configs to cloud
echoax-cli sync download  # Pull configs from cloud
echoax-cli sync check     # Check for differences
```
