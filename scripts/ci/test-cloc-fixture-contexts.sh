#!/usr/bin/env bash
set -euo pipefail
# Independent fixture admission, with all enclosing consumer selectors present.
ROOT="$(cd "$(dirname "$0")/../.." && pwd -P)"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/cloc-fixture-contexts.XXXXXX")"
trap 'if [[ $? == 0 ]]; then rm -rf "$fixture"; else printf "Failed LOC context fixture retained: %s\n" "$fixture" >&2; fi' EXIT
mkdir -p "$fixture/enclosing/.cargo" "$fixture/enclosing/scratch" "$fixture/output"
git init -q "$fixture/enclosing"
printf '[workspace]\nmembers = []\nresolver = "2"\n' > "$fixture/enclosing/Cargo.toml"
printf '[build]\ntarget-dir = "ancestor-output"\n' > "$fixture/enclosing/.cargo/config.toml"
for script in test-cloc.sh test-cloc-siblings.sh; do
    TMPDIR="$fixture/enclosing/scratch" CARGO_TARGET_DIR="$fixture/output" \
        bash "$ROOT/scripts/ci/$script" > "$fixture/$script.log" 2>&1
done
echo 'LOC fixtures passed inside an enclosing Git/Cargo workspace with inherited output selection'
