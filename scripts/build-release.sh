#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:-$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')}"
TARGETS=(
    "x86_64-unknown-linux-gnu"
    "aarch64-unknown-linux-gnu"
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
)

echo "Building EchoAccess v${VERSION} for ${#TARGETS[@]} targets"

mkdir -p dist

for target in "${TARGETS[@]}"; do
    echo ""
    echo "=== Building for ${target} ==="

    if rustup target list --installed | grep -q "^${target}$"; then
        cargo build --release --workspace --target "${target}" || {
            echo "WARN: Native build failed for ${target}, trying cross..."
            cross build --release --workspace --target "${target}" || {
                echo "SKIP: Cannot build for ${target}"
                continue
            }
        }
    else
        echo "Target ${target} not installed, trying cross..."
        cross build --release --workspace --target "${target}" || {
            echo "SKIP: Cannot build for ${target}"
            continue
        }
    fi

    archive="echoax-v${VERSION}-${target}"
    mkdir -p "dist/${archive}"

    for bin in echoax-cli echoax-tui echoax-web; do
        cp "target/${target}/release/${bin}" "dist/${archive}/" 2>/dev/null || true
    done

    cp README.md LICENSE "dist/${archive}/"

    (cd dist && tar czf "${archive}.tar.gz" "${archive}")
    (cd dist && sha256sum "${archive}.tar.gz" > "${archive}.tar.gz.sha256")

    echo "Created dist/${archive}.tar.gz"
done

echo ""
echo "=== Release artifacts ==="
ls -lh dist/*.tar.gz 2>/dev/null || echo "No archives created"
