#!/usr/bin/env bash
set -euo pipefail
shopt -s nullglob
ROOT="$(cd "$(dirname "$0")/../.." && pwd -P)"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/failure-evidence.XXXXXX")"
trap 'if [[ $? == 0 ]]; then rm -rf "$fixture"; else printf "Failure-evidence fixture retained: %s\n" "$fixture" >&2; fi' EXIT
mkdir -p "$fixture/temp/ic-memory-fixtures" "$fixture/repository" "$fixture/unpacked"
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
printf 'unselected temporary file\n' > "$temp_root/unrelated"
mkdir -p "$repository_root/target/qualification" "$repository_root/target/release-validation" \
    "$repository_root/.tools/host-set.test/bin" "$repository_root/.tools/ic-set.test/bin" \
    "$repository_root/.git" "$repository_root/target/unrelated"
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

archive="$(bash "$ROOT/scripts/ci/collect-failure-evidence.sh" "$temp_root" "$repository_root")"
[[ -f "$archive" && "${archive##*/}" == evidence.tar.gz ]]
tar -xzf "$archive" -C "$fixture/unpacked"
retained=("$fixture/unpacked/ic-memory-fixtures"/host-tools-test.*/Linux:x86_64/.tools/host-set.*/bin/yq)
[[ ${#retained[@]} == 1 && -x "${retained[0]}" ]]
for path in tools-setup.log dependencies.log validation.log; do cmp "$temp_root/$path" "$fixture/unpacked/$path"; done
cmp "$repository_root/target/qualification/"$'line\nbreak:payload' "$fixture/unpacked/target/qualification/"$'line\nbreak:payload'
[[ "$(perl -e 'printf "%o", (stat($ARGV[0]))[2] & 0777' "$fixture/unpacked/target/qualification/"$'line\nbreak:payload')" == 640 ]]
[[ -x "$fixture/unpacked/.tools/host-set.test/bin/tool" && -f "$fixture/unpacked/.tools/ic-set.test/receipt" ]]
cmp "$repository_root/target/release-validation/attempt.log" "$fixture/unpacked/target/release-validation/attempt.log"
[[ -L "$fixture/unpacked/target/qualification/link" && ! -e "$fixture/unpacked/target/qualification/link" ]]
[[ "$(readlink "$fixture/unpacked/target/qualification/link")" == ../unrelated/file ]]
[[ ! -e "$fixture/unpacked/.git" && ! -e "$fixture/unpacked/unrelated" && ! -e "$fixture/unpacked/target/unrelated" && ! -e "$fixture/unpacked/.tools/host" ]]

# Retries create separate archives and preserve original evidence and gate status.
cp "$archive" "$fixture/saved.tar.gz"
retry="$(bash "$ROOT/scripts/ci/collect-failure-evidence.sh" "$temp_root" "$repository_root")"
[[ "$archive" != "$retry" && -f "$retry" ]]
cmp "$archive" "$fixture/saved.tar.gz"
[[ "$(cat "$temp_root/validation.log")" == original_status=1 ]]

# An early failure with no selected inputs produces no artifact, not an empty tar.
mkdir "$fixture/empty-temp" "$fixture/empty-repository"
[[ -z "$(bash "$ROOT/scripts/ci/collect-failure-evidence.sh" "$fixture/empty-temp" "$fixture/empty-repository")" ]]
empty_entries=("$fixture/empty-temp"/*)
[[ ${#empty_entries[@]} == 0 ]]

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
echo 'Consumer evidence selections, retained host failure, archive round trip and retry/failure preservation passed'
