#!/usr/bin/env bash
set -euo pipefail

# Consumer policy only: the shared helper owns archive creation and path safety.
[[ $# == 2 || $# == 3 ]] || { echo 'usage: collect-failure-evidence.sh TEMP-ROOT REPOSITORY-ROOT [full|compact]' >&2; exit 2; }
mode="${3:-full}"
[[ "$mode" == full || "$mode" == compact ]] || { echo 'invalid tool evidence selection' >&2; exit 2; }
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
    "$temp_root/dependencies.log" "$temp_root/validation.log" \
    "$temp_root/runtime-setup.log" "$temp_root/runtime.log"; do
    if [[ -e "$path" || -L "$path" ]]; then
        inputs+=("$temp_root" "${path#"$temp_root/"}")
    fi
done
for path in "$repository_root/target/qualification" "$repository_root/target/release-validation" \
    "$repository_root/.tools/rust" "$repository_root/.tools/ic-testkit-server"; do
    if [[ -e "$path" || -L "$path" ]]; then
        inputs+=("$repository_root" "${path#"$repository_root/"}")
    fi
done
bundles=("$repository_root"/.tools/host-set.* "$repository_root"/.tools/ic-set.*)
if [[ ${#bundles[@]} != 0 ]]; then
    diagnostics="$(mktemp -d "$temp_root/tool-evidence.XXXXXX")"
    selector=(bash "$ROOT/scripts/ci/select-tool-evidence.sh" "$mode" "$repository_root" "$diagnostics")
    if [[ "$mode" == compact ]]; then
        selector+=("$repository_root/ci/tool-versions.env" "$repository_root/ci/ic-tools.tsv")
    fi
    "${selector[@]}" > "$diagnostics/selection"
    while IFS= read -r -d '' selection_root; do
        IFS= read -r -d '' selection_path
        inputs+=("$selection_root" "$selection_path")
    done < "$diagnostics/selection"
    if [[ -d "$diagnostics/host" || -d "$diagnostics/ic" ]]; then
        inputs+=("$temp_root" "${diagnostics#"$temp_root/"}")
    else
        rm "$diagnostics/selection"
        rmdir "$diagnostics"
    fi
fi
[[ ${#inputs[@]} != 0 ]] || exit 0
archive_dir="$(mktemp -d "$temp_root/ic-memory-evidence.XXXXXX")"
bash "$ROOT/scripts/ci/archive-evidence.sh" "$archive_dir/evidence.tar.gz" "${inputs[@]}"
