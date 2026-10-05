#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VERSION=""
CHECKSUM=""
INSTALL_DIR="${TOOL_INSTALL_DIR:-$HOME/.local/bin}"
TMP_DIR=""

usage() {
    cat >&2 <<'USAGE'
usage: install-shellcheck.sh --version <version> --sha256 <digest> [--install-dir <directory>]
USAGE
}

while [ "$#" -gt 0 ]; do
    case "$1" in
    --version)
        [ "$#" -ge 2 ] || { usage; exit 2; }
        VERSION="$2"
        shift 2
        ;;
    --sha256)
        [ "$#" -ge 2 ] || { usage; exit 2; }
        CHECKSUM="$2"
        shift 2
        ;;
    --install-dir)
        [ "$#" -ge 2 ] || { usage; exit 2; }
        INSTALL_DIR="$2"
        shift 2
        ;;
    -h | --help)
        usage
        exit 0
        ;;
    *)
        usage
        exit 2
        ;;
    esac
done

if [ -z "$VERSION" ] || [ -z "$CHECKSUM" ]; then
    usage
    exit 2
fi

VERSION="${VERSION#v}"
if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z][0-9A-Za-z.-]*)?$ ]]; then
    echo "invalid ShellCheck version: $VERSION" >&2
    exit 1
fi
if [[ ! "$CHECKSUM" =~ ^[0-9a-f]{64}$ ]]; then
    echo "invalid SHA-256 digest" >&2
    exit 1
fi

resolve_platform() {
    case "$(uname -s):$(uname -m)" in
    Darwin:arm64 | Darwin:aarch64)
        platform="darwin.aarch64"
        ;;
    Darwin:x86_64 | Darwin:amd64)
        platform="darwin.x86_64"
        ;;
    Linux:arm64 | Linux:aarch64)
        platform="linux.aarch64"
        ;;
    Linux:x86_64 | Linux:amd64)
        platform="linux.x86_64"
        ;;
    *)
        echo "unsupported ShellCheck platform: $(uname -s) $(uname -m)" >&2
        exit 1
        ;;
    esac
}

main() {
    local version_no_v="${VERSION#v}"
    local release_dir="shellcheck-v${version_no_v}"
    local archive
    local url
    local installed
    local candidate
    local line
    local reported_version=""
    local version_output

    archive="${release_dir}.${platform}.tar.xz"
    url="https://github.com/koalaman/shellcheck/releases/download/v${version_no_v}/${archive}"

    TMP_DIR="$(mktemp -d)"
    trap 'rm -rf "$TMP_DIR"' EXIT
    mkdir -p "$INSTALL_DIR"
    curl --proto '=https' --proto-redir '=https' --tlsv1.2 -fsSL \
        --retry 5 --retry-all-errors --retry-delay 2 \
        --connect-timeout 15 --max-time 120 \
        -o "$TMP_DIR/$archive" "$url"
    bash "$SCRIPT_DIR/verify-file-checksum.sh" sha256 "$CHECKSUM" "$TMP_DIR/$archive"
    tar -xJf "$TMP_DIR/$archive" -C "$TMP_DIR"
    candidate="$TMP_DIR/$release_dir/shellcheck"
    chmod +x "$candidate"
    version_output="$("$candidate" --version 2>&1)"
    while IFS= read -r line; do
        case "$line" in
        "version: "*)
            reported_version="${line#version: }"
            break
            ;;
        esac
    done <<<"$version_output"
    if [ "$reported_version" != "$VERSION" ]; then
        echo "installed ShellCheck does not report the pinned version" >&2
        echo "expected: $VERSION" >&2
        echo "actual:   $version_output" >&2
        exit 1
    fi

    installed="$INSTALL_DIR/shellcheck"
    mv "$candidate" "$installed"
    printf '%s\n' "$installed"
}

resolve_platform
main
