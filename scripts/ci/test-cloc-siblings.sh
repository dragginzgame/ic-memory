#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd -P)"
# Synthetic workspaces own their output paths, not the enclosing consumer.
unset CARGO_TARGET_DIR
FIXTURE="$(mktemp -d "${TMPDIR:-/tmp}/shared-tooling-cloc-siblings-test.XXXXXX")"
FIXTURE="$(cd "$FIXTURE" && pwd -P)"
trap 'if [[ $? == 0 ]]; then rm -rf "$FIXTURE"; else printf "Failed sibling LOC fixture retained: %s\n" "$FIXTURE" >&2; fi' EXIT

parent="$FIXTURE/parent [projects]"
alpha="$parent/alpha repo [copy]"
zeta="$parent/zeta"
mkdir -p "$alpha/crates/library/src" "$alpha/apps/demo/service/src" \
    "$alpha/apps/demo/service/tests" "$alpha/.cargo" "$parent/.notes" \
    "$parent/not-a-repository" "$FIXTURE/gitdirs" "$FIXTURE/elsewhere"
git init -q "$alpha"
git init -q "$parent/.notes"
git init -q --separate-git-dir "$FIXTURE/gitdirs/zeta" "$zeta"
ln -s "$alpha" "$parent/alias"

cat >"$alpha/Cargo.toml" <<'TOML'
[workspace]
members = ["crates/library", "apps/demo/service"]
resolver = "2"
TOML
for spec in 'crates/library library' 'apps/demo/service service'; do
    read -r path name <<<"$spec"
    cat >"$alpha/$path/Cargo.toml" <<TOML
[package]
name = "$name"
version = "0.1.0"
edition = "2021"
TOML
done
cat >"$alpha/crates/library/src/lib.rs" <<'RUST'
pub fn library() -> u8 { 1 }
#[test]
fn inline_test() {}
RUST
printf 'pub fn application() {}\n' >"$alpha/apps/demo/service/src/lib.rs"
printf '#[test]\nfn smoke() {}\n' >"$alpha/apps/demo/service/tests/smoke.rs"
cat >"$alpha/.cargo/config.toml" <<'TOML'
[build]
target-dir = "crates/library/build [generated]"
TOML
mkdir -p "$alpha/crates/library/build [generated]/tests"
printf '#[test]\nfn generated() {}\n' >"$alpha/crates/library/build [generated]/tests/generated.rs"
(cd "$alpha" && cargo generate-lockfile --offline)
cp "$alpha/Cargo.lock" "$FIXTURE/alpha.lock"
cat >"$alpha/Makefile" <<'MAKE'
$(error repository Make targets must not run during reporting)
MAKE

cat >"$zeta/Cargo.toml" <<'TOML'
[package]
name = "zeta"
version = "0.1.0"
edition = "2021"

[workspace]
TOML
mkdir -p "$zeta/src" "$zeta/scripts/dev" "$zeta/.cargo"
printf '[build]\ntarget-dir = "target"\n' > "$zeta/.cargo/config.toml"
printf '#[test]\nfn another_test() {}\n' >"$zeta/src/lib.rs"
cp "$ROOT/scripts/dev/cloc.sh" "$ROOT/scripts/dev/cloc-siblings.sh" "$zeta/scripts/dev/"

output="$(bash "$ROOT/scripts/dev/cloc-siblings.sh" "$parent")"
[[ "$(printf '%s\n' "$output" | wc -l | tr -d ' ')" == 7 ]]
[[ "$(printf '%s\n' "$output" | awk 'NR == 3 { print $1, $2, $3, $4, $5, $6 }')" == '.notes N/A N/A N/A N/A N/A' ]]
[[ "$(printf '%s\n' "$output" | awk 'NR == 4 { print $(NF-4), $(NF-3), $(NF-2), $(NF-1), $NF }')" == '4 2 33.3% 2 1' ]]
[[ "$(printf '%s\n' "$output" | awk 'NR == 5 { print $1, $2, $3, $4, $5, $6 }')" == 'zeta 2 0 0.0% 1 1' ]]
# Percentages are recomputed from combined LOC, not averaged from repository rows.
[[ "$(printf '%s\n' "$output" | awk 'END { print $1, $2, $3, $4, $5, $6 }')" == 'TOTAL 6 2 25.0% 3 2' ]]
[[ -f "$zeta/.git" && ! -e "$zeta/Cargo.lock" ]]
cmp "$alpha/Cargo.lock" "$FIXTURE/alpha.lock"

# The default parent follows the installed script, not the caller's directory.
default_output="$(cd "$FIXTURE/elsewhere" && bash "$zeta/scripts/dev/cloc-siblings.sh")"
[[ "$default_output" == "$output" ]]

# A missing shared prerequisite fails once, before printing a misleading table.
# Use the copied scripts so this checkout's installed tools cannot mask absence.
mkdir "$FIXTURE/no-cloc"
for tool in bash dirname git cargo jq; do
    ln -s "$(command -v "$tool")" "$FIXTURE/no-cloc/$tool"
done
if PATH="$FIXTURE/no-cloc" "$BASH" "$zeta/scripts/dev/cloc-siblings.sh" "$parent" \
    >"$FIXTURE/missing.out" 2>"$FIXTURE/missing.err"; then
    echo 'missing cloc was accepted' >&2
    exit 1
fi
[[ ! -s "$FIXTURE/missing.out" && "$(grep -c 'missing LOC tools: cloc' "$FIXTURE/missing.err")" == 1 ]]
grep -F 'make install-host-tools' "$FIXTURE/missing.err" >/dev/null
# Explicit installation beside the report supplies cloc without a PATH export.
mkdir -p "$zeta/.tools/host/bin"
ln -s "$(command -v cloc)" "$zeta/.tools/host/bin/cloc"
PATH="$FIXTURE/no-cloc" "$BASH" "$zeta/scripts/dev/cloc.sh" --check-tools

# A broken workspace reports failure without hiding later successful rows.
git init -q "$parent/middle-broken"
printf '[workspace\n' >"$parent/middle-broken/Cargo.toml"
if bash "$ROOT/scripts/dev/cloc-siblings.sh" "$parent" >"$FIXTURE/failed.out" 2>"$FIXTURE/failed.err"; then
    echo 'broken Cargo workspace was reported as successful' >&2
    exit 1
fi
[[ "$(awk '$1 == "middle-broken" { print $2, $3, $4, $5, $6 }' "$FIXTURE/failed.out")" == 'ERROR ERROR ERROR ERROR ERROR' ]]
[[ "$(awk '$1 == "zeta" { print $2, $3, $4, $5, $6 }' "$FIXTURE/failed.out")" == '2 0 0.0% 1 1' ]]
[[ "$(awk 'END { print $1, $2, $3, $4, $5, $6, $7 }' "$FIXTURE/failed.out")" == 'TOTAL (partial) 6 2 25.0% 3 2' ]]
grep -F "$parent/middle-broken" "$FIXTURE/failed.err" >/dev/null
cmp "$alpha/Cargo.lock" "$FIXTURE/alpha.lock"

# No measured workspace remains distinct from a measured empty Rust package.
mkdir "$FIXTURE/no-rust"
git init -q "$FIXTURE/no-rust/notes"
output="$(bash "$ROOT/scripts/dev/cloc-siblings.sh" "$FIXTURE/no-rust")"
[[ "$(printf '%s\n' "$output" | awk 'END { print $0 }')" =~ TOTAL[[:space:]]+N/A[[:space:]]+N/A[[:space:]]+N/A[[:space:]]+N/A[[:space:]]+N/A$ ]]
mkdir -p "$FIXTURE/empty-rust/empty/src"
git init -q "$FIXTURE/empty-rust/empty"
cp "$zeta/Cargo.toml" "$FIXTURE/empty-rust/empty/Cargo.toml"
cp -R "$zeta/.cargo" "$FIXTURE/empty-rust/empty/"
: > "$FIXTURE/empty-rust/empty/src/lib.rs"
output="$(bash "$ROOT/scripts/dev/cloc-siblings.sh" "$FIXTURE/empty-rust")"
[[ "$(printf '%s\n' "$output" | awk 'END { print $1, $2, $3, $4, $5, $6 }')" == 'TOTAL 0 0 0.0% 0 0' ]]

bash "$ROOT/scripts/dev/cloc-siblings.sh" --help >/dev/null
for path in "$FIXTURE/absent" "$FIXTURE/elsewhere"; do
    if bash "$ROOT/scripts/dev/cloc-siblings.sh" "$path" >"$FIXTURE/invalid.out" 2>"$FIXTURE/invalid.err"; then
        echo 'invalid or empty parent was accepted' >&2
        exit 1
    fi
done

echo 'sibling cloc tests passed'
