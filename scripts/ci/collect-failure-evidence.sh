#!/usr/bin/env bash
set -euo pipefail

# Consumer policy only: the shared helper owns archive creation and path safety.
# CI activation requires the helper's reviewed, committed snapshot export.
[[ $# == 2 ]] || { echo 'usage: collect-failure-evidence.sh TEMP-ROOT REPOSITORY-ROOT' >&2; exit 2; }
ROOT="$(cd "$(dirname "$0")/../.." && pwd -P)"
temp_root="$(cd "$1" && pwd -P)"
repository_root="$(cd "$2" && pwd -P)"
inputs=()
shopt -s nullglob
for path in "$temp_root/ic-memory-fixtures" "$temp_root/tools-setup.log" \
    "$temp_root/dependencies.log" "$temp_root/validation.log"; do
    if [[ -e "$path" || -L "$path" ]]; then
        inputs+=("$temp_root" "${path#"$temp_root/"}")
    fi
done
for path in "$repository_root/target/qualification" "$repository_root/target/release-validation" \
    "$repository_root"/.tools/host-set.* "$repository_root"/.tools/ic-set.*; do
    if [[ -e "$path" || -L "$path" ]]; then
        inputs+=("$repository_root" "${path#"$repository_root/"}")
    fi
done
[[ ${#inputs[@]} != 0 ]] || exit 0
archive_dir="$(mktemp -d "$temp_root/ic-memory-evidence.XXXXXX")"
bash "$ROOT/scripts/ci/archive-evidence.sh" "$archive_dir/evidence.tar.gz" "${inputs[@]}"
