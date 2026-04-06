#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:-$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')}"
TARGETS=(
    "x86_64-unknown-linux-gnu"
    "aarch64-unknown-linux-gnu"
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
    "x86_64-pc-windows-msvc"
)

echo "Building EchoAccess v${VERSION} for ${#TARGETS[@]} targets"

mkdir -p dist

succeeded=()
failed=()

for target in "${TARGETS[@]}"; do
    echo ""
    echo "=== Building for ${target} ==="

    if ! cargo zigbuild --release -p echoax-cli --target "${target}"; then
        echo "ERROR: cargo zigbuild failed for ${target}" >&2
        failed+=("${target}")
        continue
    fi

    archive="echoax-v${VERSION}-${target}"
    mkdir -p "dist/${archive}"

    if [[ "${target}" == *windows-msvc* ]]; then
        if ! cp "target/${target}/release/echo_access.exe" "dist/${archive}/"; then
            echo "ERROR: expected binary missing: target/${target}/release/echo_access.exe" >&2
            failed+=("${target}")
            continue
        fi
    else
        if ! cp "target/${target}/release/echo_access" "dist/${archive}/"; then
            echo "ERROR: expected binary missing: target/${target}/release/echo_access" >&2
            failed+=("${target}")
            continue
        fi
    fi

    cp README.md LICENSE "dist/${archive}/"

    if [[ "${target}" == *windows-msvc* ]]; then
        if ! (cd dist && zip -qr "${archive}.zip" "${archive}"); then
            echo "ERROR: zip failed for ${target}" >&2
            failed+=("${target}")
            continue
        fi
        if command -v sha256sum >/dev/null 2>&1; then
            (cd dist && sha256sum "${archive}.zip" > "${archive}.zip.sha256")
        elif command -v shasum >/dev/null 2>&1; then
            (cd dist && shasum -a 256 "${archive}.zip" > "${archive}.zip.sha256")
        else
            echo "ERROR: neither sha256sum nor shasum found; cannot write checksum for ${archive}.zip" >&2
            failed+=("${target}")
            continue
        fi
        echo "Created dist/${archive}.zip"
    else
        if ! (cd dist && tar czf "${archive}.tar.gz" "${archive}"); then
            echo "ERROR: tar failed for ${target}" >&2
            failed+=("${target}")
            continue
        fi
        if command -v sha256sum >/dev/null 2>&1; then
            if ! (cd dist && sha256sum "${archive}.tar.gz" > "${archive}.tar.gz.sha256"); then
                echo "ERROR: sha256sum failed for ${target}" >&2
                failed+=("${target}")
                continue
            fi
        elif command -v shasum >/dev/null 2>&1; then
            if ! (cd dist && shasum -a 256 "${archive}.tar.gz" > "${archive}.tar.gz.sha256"); then
                echo "ERROR: shasum failed for ${target}" >&2
                failed+=("${target}")
                continue
            fi
        else
            echo "ERROR: neither sha256sum nor shasum found; cannot write checksum for ${archive}.tar.gz" >&2
            failed+=("${target}")
            continue
        fi
        echo "Created dist/${archive}.tar.gz"
    fi

    succeeded+=("${target}")
done

total=${#TARGETS[@]}
ok=${#succeeded[@]}
echo ""
echo "=== Build summary ==="
echo "Built ${ok}/${total} targets successfully"

if [ ${#succeeded[@]} -gt 0 ]; then
    echo "Succeeded:"
    for t in "${succeeded[@]}"; do
        echo "  - ${t}"
    done
fi

if [ ${#failed[@]} -gt 0 ]; then
    echo "Failed:"
    for t in "${failed[@]}"; do
        echo "  - ${t}"
    done
    exit 1
fi

echo ""
echo "=== Release artifacts ==="
shopt -s nullglob
archives=(dist/*.tar.gz dist/*.zip)
if [ ${#archives[@]} -eq 0 ]; then
    echo "No archives in dist/"
else
    ls -lh "${archives[@]}"
fi
