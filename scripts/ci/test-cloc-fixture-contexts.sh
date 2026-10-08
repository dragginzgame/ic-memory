#!/usr/bin/env bash
set -euo pipefail
# Independent fixture admission, with all enclosing consumer selectors present.
ROOT="$0"
[[ "$ROOT" == /* ]] || ROOT="$PWD/$ROOT"
ROOT="$(cd -P "${ROOT%/*}/../.." && printf '%s/.' "$PWD")"
ROOT="${ROOT%/.}"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/cloc-fixture-contexts.XXXXXX")"
fixture="$(cd "$fixture" && pwd -P)"
trap 'if [[ $? == 0 ]]; then rm -rf "$fixture"; else printf "Failed LOC context fixture retained: %s\n" "$fixture" >&2; fi' EXIT
mkdir -p "$fixture/enclosing/.cargo" "$fixture/enclosing/scratch" "$fixture/output"
git init -q "$fixture/enclosing"
printf '[workspace]\nmembers = []\nresolver = "2"\n' > "$fixture/enclosing/Cargo.toml"
printf '[build]\ntarget-dir = "ancestor-output"\n' > "$fixture/enclosing/.cargo/config.toml"
for script in test-cloc.sh test-cloc-siblings.sh; do
    TMPDIR="$fixture/enclosing/scratch" CARGO_TARGET_DIR="$fixture/output" \
        bash "$ROOT/scripts/ci/$script" > "$fixture/$script.log" 2>&1
done
ln -s "$fixture/enclosing/scratch" "$fixture/scratch-alias"
for context in trailing-slash directory-alias; do
    selected_tmp="$fixture/enclosing/scratch/"
    if [[ "$context" == directory-alias ]]; then selected_tmp="$fixture/scratch-alias/"; fi
    TMPDIR="$selected_tmp" CARGO_TARGET_DIR="$fixture/output" \
        bash "$ROOT/scripts/ci/test-cloc-siblings.sh" > "$fixture/siblings-$context.log" 2>&1
done
echo 'LOC fixtures passed with enclosing Git/Cargo configuration, inherited outputs and physical/aliased TMPDIR roots'
