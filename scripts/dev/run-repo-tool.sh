#!/usr/bin/env bash
# Keep the Rust adapter reachable between atomic manifest/lockfile replacements.
set -euo pipefail
export RUSTUP_AUTO_INSTALL=0
# Cargo's --offline flags scope helper compilation/discovery. Preserve the
# caller's network setting for commands the helper dispatches, such as publish.
if [[ $# -lt 2 ]]; then
    echo 'usage: run-repo-tool.sh <toolchain> <command> [arguments]' >&2
    exit 2
fi
toolchain="$1"
shift
manifest_version="$(awk '
    /^\[package\]$/ { package = 1; next }
    /^\[/ { package = 0 }
    package && /^version = "[^"]+"$/ { gsub(/^version = "|"$/, ""); print }
' Cargo.toml)"
lock_version="$(awk '
    /^\[\[package\]\]$/ { root = 0 }
    /^name = "ic-memory"$/ { root = 1 }
    root && /^version = "[^"]+"$/ { gsub(/^version = "|"$/, ""); print }
' Cargo.lock)"
if [[ "$manifest_version" == "$lock_version" ]]; then
    exec cargo "+$toolchain" run --locked --offline --quiet --example repo-tool -- "$@"
fi
[[ "$lock_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || {
    echo 'cannot bootstrap repo-tool: expected one canonical root lock version' >&2
    exit 1
}
# --no-deps metadata does not resolve or update the mismatched lock. Respect
# Cargo's selected target directory, including the consumer's configuration.
target="$(cargo "+$toolchain" metadata --locked --offline --no-deps --format-version 1 |
    "${YQ:-yq}" -p=json -r '.target_directory')"
[[ "$target" == /* ]] || { echo 'invalid bootstrap target directory' >&2; exit 1; }
mkdir -p "$target/repo-tool-bootstrap"
scratch="$(mktemp -d "$target/repo-tool-bootstrap/attempt.XXXXXX")"
echo "Retained repo-tool bootstrap metadata: $scratch" >&2
cp Cargo.lock README.md "$scratch/"
awk -v version="$lock_version" '
    /^\[package\]$/ { package = 1; print; next }
    /^\[/ { package = 0 }
    package && /^version = "[^"]+"$/ { print "version = \"" version "\""; next }
    { print }
' Cargo.toml > "$scratch/Cargo.toml"
ln -s "$PWD/src" "$scratch/src"
ln -s "$PWD/examples" "$scratch/examples"
# This only makes compilation metadata coherent; it grants no release authority.
# The adapter runs in the real checkout and checks source, intent and evidence.
exec cargo "+$toolchain" run --locked --offline --quiet \
    --manifest-path "$scratch/Cargo.toml" --target-dir "$target/repo-tool-bootstrap/build" \
    --example repo-tool -- "$@"
