# Quick Start

## 1. Launch the Web Dashboard

```bash
echo_access
```

This starts the web server on port 9876 and opens your browser automatically.
If the server is already running, it simply opens the browser to the existing instance.

## 2. Initialize

```bash
echo_access init
```

This creates the config directory at `~/.config/echoax/` with default settings.

## 3. Create a Device Profile

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

## 4. Validate the Profile

```bash
echo_access profile validate ~/.config/echoax/profiles/my-device.toml
```

## 5. Check Status

```bash
echo_access status
```

## 6. Sync

```bash
echo_access sync upload    # Push local configs to cloud
echo_access sync download  # Pull configs from cloud
echo_access sync check     # Check for differences
```

## 7. TUI Mode

```bash
echo_access tui
```

Launches the NieR: Automata-styled terminal dashboard for interactive use.
