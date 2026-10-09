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
check_dependency_cache() {
    local status
    local fetch=(fetch --locked --offline)
    # Only the documented release entry points select network preparation.
    # Cargo still honours explicit offline environment/configuration settings.
    if [[ "${RELEASE_CACHE_PREPARE:-0}" == 1 ]]; then
        fetch=(fetch --locked)
    fi
    if cargo "+$toolchain" "${fetch[@]}" "$@"; then
        # The compiled adapter and its validation children stay offline.
        unset RELEASE_CACHE_PREPARE
        return
    else
        status=$?
    fi
    echo 'Dependency preparation failed. Run make fetch-dependencies to populate the selected cache; resolve any reported lockfile or network errors first. Explicit offline settings remain in effect.' >&2
    exit "$status"
}
manifest_version="$(awk '
    /^\[workspace\.package\]$/ { package = 1; next }
    /^\[/ { package = 0 }
    package && /^version = "[^"]+"$/ { gsub(/^version = "|"$/, ""); print }
' Cargo.toml)"
lock_version="$(awk '
    /^\[\[package\]\]$/ { root = 0 }
    /^name = "ic-memory"$/ { root = 1 }
    root && /^version = "[^"]+"$/ { gsub(/^version = "|"$/, ""); print }
' Cargo.lock)"
if [[ "$manifest_version" == "$lock_version" ]]; then
    check_dependency_cache
    exec cargo "+$toolchain" run --locked --offline --quiet --example repo-tool -- "$@"
fi
[[ "$lock_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || {
    echo 'cannot bootstrap repo-tool: expected one canonical root lock version' >&2
    exit 1
}
# --no-deps metadata does not resolve or update the mismatched lock. Respect
# Cargo's selected target directory, including the consumer's configuration.
# Preserve path bytes, removing only the parser's output record terminator.
# The sentinel is appended only after the complete pipeline succeeds.
target="$(cargo "+$toolchain" metadata --locked --offline --no-deps --format-version 1 |
    "${YQ:-yq}" -p=json -r '.target_directory' && printf '.')"
target="${target%$'\n.'}"
[[ "$target" == /* ]] || { echo 'invalid bootstrap target directory' >&2; exit 1; }
mkdir -p "$target/repo-tool-bootstrap"
scratch="$(mktemp -d "$target/repo-tool-bootstrap/attempt.XXXXXX")"
echo "Retained repo-tool bootstrap metadata: $scratch" >&2
cp Cargo.lock LICENSE "$scratch/"
mkdir -p "$scratch/crates/ic-memory"
cp crates/ic-memory/Cargo.toml crates/ic-memory/README.md "$scratch/crates/ic-memory/"
awk -v version="$lock_version" '
    /^\[workspace\.package\]$/ { package = 1; print; next }
    /^\[/ { package = 0 }
    package && /^version = "[^"]+"$/ { print "version = \"" version "\""; next }
    { print }
' Cargo.toml > "$scratch/Cargo.toml"
ln -s "$PWD/crates/ic-memory/src" "$scratch/crates/ic-memory/src"
ln -s "$PWD/crates/ic-memory/examples" "$scratch/crates/ic-memory/examples"
# This only makes compilation metadata coherent; it grants no release authority.
# The adapter runs in the real checkout and checks source, intent and evidence.
check_dependency_cache --manifest-path "$scratch/Cargo.toml"
exec cargo "+$toolchain" run --locked --offline --quiet \
    --manifest-path "$scratch/Cargo.toml" --target-dir "$target/repo-tool-bootstrap/build" \
    --example repo-tool -- "$@"
