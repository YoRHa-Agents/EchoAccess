# Installation

## From GitHub Releases

Download the latest release for your platform:

| Platform | Architecture | Download |
|----------|-------------|----------|
| Linux | x86_64 | `echoax-v0.1.2-x86_64-unknown-linux-gnu.tar.gz` |
| Linux | aarch64 | `echoax-v0.1.2-aarch64-unknown-linux-gnu.tar.gz` |
| macOS | Intel | `echoax-v0.1.2-x86_64-apple-darwin.tar.gz` |
| macOS | Apple Silicon | `echoax-v0.1.2-aarch64-apple-darwin.tar.gz` |

```bash
# Example: Linux x86_64
curl -fsSL https://github.com/shendeguize/EchoAccess/releases/latest/download/echoax-v0.1.2-x86_64-unknown-linux-gnu.tar.gz | tar xz
sudo mv echoax-*/echo_access /usr/local/bin/echo_access
```

## From Source

Requirements: Rust 1.94+ with cargo.

```bash
git clone https://github.com/shendeguize/EchoAccess.git
cd EchoAccess
cargo build --release -p echoax-cli
```

The binary is produced at `target/release/echo_access`.

## Verify Installation

```bash
echo_access --help
```
