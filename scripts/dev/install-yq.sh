#!/usr/bin/env bash
# Consumer-owned selection; the canonical shared installer verifies the download.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
if [[ $# != 2 || "$1" != --install-dir || -z "$2" ]]; then
    echo 'usage: install-yq.sh --install-dir <directory>' >&2
    exit 2
fi
# shellcheck source=ci-tool-versions.env
source "$ROOT/ci-tool-versions.env"
case "$(uname -s):$(uname -m)" in
    Linux:x86_64|Linux:amd64) digest="$IC_MEMORY_YQ_SHA256_LINUX_AMD64" ;;
    Linux:aarch64|Linux:arm64) digest="$IC_MEMORY_YQ_SHA256_LINUX_ARM64" ;;
    Darwin:x86_64|Darwin:amd64) digest="$IC_MEMORY_YQ_SHA256_DARWIN_AMD64" ;;
    Darwin:arm64|Darwin:aarch64) digest="$IC_MEMORY_YQ_SHA256_DARWIN_ARM64" ;;
    *) echo 'unsupported yq host' >&2; exit 1 ;;
esac
exec bash "$ROOT/scripts/ci/install-yq.sh" --version "$IC_MEMORY_YQ_VERSION" \
    --sha256 "$digest" --install-dir "$2"
