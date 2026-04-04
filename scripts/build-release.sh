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

    cargo zigbuild --release -p echoax-cli --target "${target}" || {
        echo "SKIP: Cannot build for ${target}"
        continue
    }

    archive="echoax-v${VERSION}-${target}"
    mkdir -p "dist/${archive}"

    cp "target/${target}/release/echo_access" "dist/${archive}/" 2>/dev/null || true
    cp README.md LICENSE "dist/${archive}/"

    (cd dist && tar czf "${archive}.tar.gz" "${archive}")
    (cd dist && sha256sum "${archive}.tar.gz" > "${archive}.tar.gz.sha256")

    echo "Created dist/${archive}.tar.gz"
done

echo ""
echo "=== Release artifacts ==="
ls -lh dist/*.tar.gz 2>/dev/null || echo "No archives created"
