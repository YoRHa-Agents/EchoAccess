# Installation

## From GitHub Releases

Download the latest release for your platform:

| Platform | Architecture | Download |
|----------|-------------|----------|
| Linux | x86_64 | `echoax-v0.1.0-x86_64-unknown-linux-gnu.tar.gz` |
| Linux | aarch64 | `echoax-v0.1.0-aarch64-unknown-linux-gnu.tar.gz` |
| macOS | Intel | `echoax-v0.1.0-x86_64-apple-darwin.tar.gz` |
| macOS | Apple Silicon | `echoax-v0.1.0-aarch64-apple-darwin.tar.gz` |

```bash
# Example: Linux x86_64
curl -fsSL https://github.com/nicholasjng/EchoAccess/releases/latest/download/echoax-v0.1.0-x86_64-unknown-linux-gnu.tar.gz | tar xz
sudo mv echoax-*/echoax-cli /usr/local/bin/echoax
```

## From Source

Requirements: Rust 1.94+ with cargo.

```bash
git clone https://github.com/nicholasjng/EchoAccess.git
cd EchoAccess
cargo build --release --workspace
```

Binaries are produced at `target/release/echoax-cli`, `target/release/echoax-tui`, `target/release/echoax-web`.

## Verify Installation

```bash
echoax-cli --help
```
