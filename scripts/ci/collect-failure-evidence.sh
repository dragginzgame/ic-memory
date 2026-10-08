#!/usr/bin/env bash
set -euo pipefail

# Consumer policy only: the shared helper owns archive creation and path safety.
[[ $# == 2 ]] || { echo 'usage: collect-failure-evidence.sh TEMP-ROOT REPOSITORY-ROOT' >&2; exit 2; }
# Anchor the script path before cd so inherited CDPATH cannot enter ROOT.
# The sentinel keeps command substitution from trimming pathname newlines.
ROOT="$0"
[[ "$ROOT" == /* ]] || ROOT="$PWD/$ROOT"
ROOT="$(cd -P "${ROOT%/*}/../.." && printf '%s/.' "$PWD")"
ROOT="${ROOT%/.}"
temp_root="$1"
repository_root="$2"
# Preserve operand bytes; physical resolution and path admission belong to the
# shared archiver. Anchoring avoids interpreting relative roots as cd options.
[[ "$temp_root" == /* ]] || temp_root="$PWD/$temp_root"
[[ "$repository_root" == /* ]] || repository_root="$PWD/$repository_root"
[[ -d "$temp_root" && -d "$repository_root" ]] || { echo 'evidence roots must be existing directories' >&2; exit 1; }
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
