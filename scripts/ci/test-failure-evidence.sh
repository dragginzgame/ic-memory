#!/usr/bin/env bash
set -euo pipefail
shopt -s nullglob
# Anchor the script path before cd so inherited CDPATH cannot enter ROOT.
# The sentinel keeps command substitution from trimming pathname newlines.
ROOT="$0"
[[ "$ROOT" == /* ]] || ROOT="$PWD/$ROOT"
ROOT="$(cd -P "${ROOT%/*}/../.." && printf '%s/.' "$PWD")"
ROOT="${ROOT%/.}"
[[ $# -le 1 ]] || { echo 'usage: test-failure-evidence.sh [NEW-ROUNDTRIP-DIRECTORY]' >&2; exit 2; }
retained=false
if [[ $# == 1 ]]; then
    [[ ! -e "$1" && ! -L "$1" ]] || { echo 'round-trip directory must be new' >&2; exit 1; }
    mkdir -- "$1"
    fixture="$1"
    [[ "$fixture" == /* ]] || fixture="$PWD/$fixture"
    retained=true
else
    fixture="$(mktemp -d "${TMPDIR:-/tmp}/failure-evidence.XXXXXX")"
fi
fixture_complete=false
finish() {
    local status=$?
    [[ "$fixture_complete" == true || "$status" != 0 ]] || status=1
    if [[ "$status" == 0 && "$retained" == false ]]; then rm -rf "$fixture"
    else printf 'Failure-evidence fixture retained: %s\n' "$fixture" >&2; fi
    exit "$status"
}
trap finish EXIT
mkdir -p "$fixture/temp/ic-memory-fixtures" "$fixture/repository"
temp_root="$fixture/temp"
repository_root="$fixture/repository"

# A real retained host-tool fixture, with all downloads/installations substituted.
if TMPDIR="$temp_root/ic-memory-fixtures" TEST_VERSION_STATUS=9 \
    bash "$ROOT/scripts/ci/test-host-tools.sh" > "$temp_root/tools-setup.log" 2>&1; then
    echo 'expected the controlled host-tool failure' >&2; exit 1
else
    status=$?
fi
[[ "$status" == 1 ]]
printf 'original_status=%s\n' "$status" > "$temp_root/validation.log"
printf 'selected dependency diagnostics\n' > "$temp_root/dependencies.log"
printf 'runtime setup failure\n' > "$temp_root/runtime-setup.log"
printf 'common tool admission failure\n' > "$temp_root/tools-check.log"
printf 'runtime host setup diagnostics\n' > "$temp_root/runtime-host-setup.log"
printf 'runtime host admission failure\n' > "$temp_root/runtime-host-check.log"
printf 'runtime diagnostics\n' > "$temp_root/runtime.log"
printf 'formatter diagnostics\n' > "$temp_root/formatting.failure"
printf 'unselected temporary file\n' > "$temp_root/unrelated"
mkdir -p "$repository_root/target/qualification" "$repository_root/target/release-validation" \
    "$repository_root/.tools/host-set.test/bin" "$repository_root/.tools/ic-set.test/bin" \
    "$repository_root/.tools/rust/build/cargo-attempt.test" \
    "$repository_root/.tools/ic-testkit-server/failed-attempt" \
    "$repository_root/.git" "$repository_root/target/unrelated"
printf 'CLI build failure\n' > "$repository_root/.tools/rust/build/cargo-attempt.test/install.log"
printf 'server setup failure\n' > "$repository_root/.tools/ic-testkit-server/failed-attempt/version.stderr"
printf 'qualification\n' > "$repository_root/target/qualification/"$'line\nbreak:payload'
chmod 640 "$repository_root/target/qualification/"$'line\nbreak:payload'
printf 'release validation\n' > "$repository_root/target/release-validation/attempt.log"
printf '#!/bin/sh\nexit 0\n' > "$repository_root/.tools/host-set.test/bin/tool"
chmod 755 "$repository_root/.tools/host-set.test/bin/tool"
printf 'IC diagnostics\n' > "$repository_root/.tools/ic-set.test/receipt"
printf 'private configuration\n' > "$repository_root/.git/config"
printf 'unselected target\n' > "$repository_root/target/unrelated/file"
ln -s ../unrelated/file "$repository_root/target/qualification/link"
ln -s host-set.test "$repository_root/.tools/host"

# Relative entry points must ignore inherited CDPATH when locating helpers.
archive="$(cd "$ROOT" && CDPATH="$ROOT" bash scripts/ci/collect-failure-evidence.sh "$temp_root" "$repository_root")"
[[ -f "$archive" && "${archive##*/}" == evidence.tar.gz ]]
bash "$ROOT/scripts/ci/verify-file-checksum.sh" --print sha256 "$archive" > "$fixture/archive.sha256"
(cd "$ROOT" && CDPATH="$ROOT" bash scripts/ci/verify-failure-evidence.sh "$fixture" "$archive")

# Retries create separate archives and preserve original evidence and gate status.
cp "$archive" "$fixture/saved.tar.gz"
retry="$(bash "$ROOT/scripts/ci/collect-failure-evidence.sh" "$temp_root" "$repository_root")"
[[ "$archive" != "$retry" && -f "$retry" ]]
cmp "$archive" "$fixture/saved.tar.gz"
[[ "$(cat "$temp_root/validation.log")" == original_status=1 ]]

# Compact selection keeps unknown/unverified bundles and the primary failure.
compact="$(bash "$ROOT/scripts/ci/collect-failure-evidence.sh" "$temp_root" "$repository_root" compact)"
cp "$fixture/archive.sha256" "$fixture/original.sha256"
bash "$ROOT/scripts/ci/verify-file-checksum.sh" --print sha256 "$compact" > "$fixture/archive.sha256"
bash "$ROOT/scripts/ci/verify-failure-evidence.sh" "$fixture" "$compact"
mv "$fixture/original.sha256" "$fixture/archive.sha256"
if bash "$ROOT/scripts/ci/collect-failure-evidence.sh" "$temp_root" "$repository_root" invalid > "$fixture/invalid-mode.log" 2>&1; then
    echo 'expected invalid selection refusal' >&2; exit 1
fi

# An early failure with no selected inputs produces no artifact, not an empty tar.
mkdir "$fixture/empty-temp" "$fixture/empty-repository"
[[ -z "$(bash "$ROOT/scripts/ci/collect-failure-evidence.sh" "$fixture/empty-temp" "$fixture/empty-repository")" ]]
empty_entries=("$fixture/empty-temp"/*)
[[ ${#empty_entries[@]} == 0 ]]

# Caller selection must not trim a newline from a root before shared admission.
mkdir "$fixture/selection" "$fixture/selection"$'\n' "$fixture/selection-repo" "$fixture/selection-unpacked"
printf 'selected root\n' > "$fixture/selection"$'\n/validation.log'
printf 'wrong root\n' > "$fixture/selection/validation.log"
selected_archive="$(bash "$ROOT/scripts/ci/collect-failure-evidence.sh" "$fixture/selection"$'\n' "$fixture/selection-repo")"
tar -xzf "$selected_archive" -C "$fixture/selection-unpacked"
cmp "$fixture/selection"$'\n/validation.log' "$fixture/selection-unpacked/validation.log"

# A failed archiver reports failure and retains both its partial output and inputs.
mkdir "$fixture/bin"
printf '#!%s\nprintf "partial archive"\nexit 23\n' "$BASH" > "$fixture/bin/tar"
chmod +x "$fixture/bin/tar"
if PATH="$fixture/bin:$PATH" bash "$ROOT/scripts/ci/collect-failure-evidence.sh" \
    "$temp_root" "$repository_root" > "$fixture/failed-output" 2> "$fixture/failed-error"; then
    echo 'expected archive failure' >&2; exit 1
fi
[[ ! -s "$fixture/failed-output" && -s "$fixture/failed-error" ]]
partial=0
printf 'partial archive' > "$fixture/partial-content"
for path in "$temp_root"/ic-memory-evidence.*/evidence.tar.gz; do
    if cmp -s "$path" "$fixture/partial-content"; then partial=$((partial + 1)); fi
done
[[ "$partial" == 1 && "$(cat "$temp_root/validation.log")" == original_status=1 ]]
# The hosted verifier must reject wrong/corrupt payloads before extracting them.
printf 'wrong archive\n' > "$fixture/corrupt.tar.gz"
unpacked_before=("$fixture"/unpacked.*)
if bash "$ROOT/scripts/ci/verify-failure-evidence.sh" "$fixture" "$fixture/corrupt.tar.gz" > "$fixture/corrupt.log" 2>&1; then
    echo 'expected checksum refusal' >&2; exit 1
fi
unpacked_after=("$fixture"/unpacked.*)
(( ${#unpacked_before[@]} == ${#unpacked_after[@]} ))
if bash "$ROOT/scripts/ci/test-failure-evidence.sh" "$fixture" > "$fixture/occupied.log" 2>&1; then
    echo 'expected occupied fixture refusal' >&2; exit 1
fi
[[ "$(cat "$temp_root/validation.log")" == original_status=1 ]]
if [[ "$retained" == true && -n "${GITHUB_OUTPUT:-}" ]]; then
    printf 'path=%s\n' "$archive" >> "$GITHUB_OUTPUT"
fi
echo 'Consumer evidence selections, retained host failure, archive round trip and retry/failure preservation passed'
fixture_complete=true
