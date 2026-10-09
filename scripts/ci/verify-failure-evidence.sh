#!/usr/bin/env bash
set -euo pipefail
shopt -s nullglob
[[ $# == 2 ]] || { echo 'usage: verify-failure-evidence.sh FIXTURE ARCHIVE' >&2; exit 2; }
# Anchor the script path before cd so inherited CDPATH cannot enter ROOT.
# The sentinel keeps command substitution from trimming pathname newlines.
ROOT="$0"
[[ "$ROOT" == /* ]] || ROOT="$PWD/$ROOT"
ROOT="$(cd -P "${ROOT%/*}/../.." && printf '%s/.' "$PWD")"
ROOT="${ROOT%/.}"
fixture="$1"
temp_root="$fixture/temp"
repository_root="$fixture/repository"
# The oracle stays outside the upload selection. Reject different archive bytes
# before extraction, then assert the consumer's original payload and metadata.
read -r digest < "$fixture/archive.sha256"
bash "$ROOT/scripts/ci/verify-file-checksum.sh" sha256 "$digest" "$2"
unpacked="$(mktemp -d "$fixture/unpacked.XXXXXX")"
tar -xzf "$2" -C "$unpacked"
retained=("$unpacked/ic-memory-fixtures"/host-tools-test.*/Linux:x86_64/.tools/host-set.*/bin/yq)
[[ ${#retained[@]} == 1 && -x "${retained[0]}" ]]
for path in tools-setup.log dependencies.log validation.log runtime-setup.log runtime.log; do cmp "$temp_root/$path" "$unpacked/$path"; done
[[ "$(cat "$unpacked/validation.log")" == original_status=1 ]]
cmp "$repository_root/target/qualification/"$'line\nbreak:payload' "$unpacked/target/qualification/"$'line\nbreak:payload'
[[ "$(perl -e 'printf "%o", (stat($ARGV[0]))[2] & 0777' "$unpacked/target/qualification/"$'line\nbreak:payload')" == 640 ]]
[[ -x "$unpacked/.tools/host-set.test/bin/tool" && -f "$unpacked/.tools/ic-set.test/receipt" ]]
cmp "$repository_root/.tools/host-set.test/bin/tool" "$unpacked/.tools/host-set.test/bin/tool"
cmp "$repository_root/.tools/ic-set.test/receipt" "$unpacked/.tools/ic-set.test/receipt"
cmp "$repository_root/target/release-validation/attempt.log" "$unpacked/target/release-validation/attempt.log"
cmp "$repository_root/.tools/rust/build/cargo-attempt.test/install.log" "$unpacked/.tools/rust/build/cargo-attempt.test/install.log"
cmp "$repository_root/.tools/ic-testkit-server/failed-attempt/version.stderr" "$unpacked/.tools/ic-testkit-server/failed-attempt/version.stderr"
[[ -L "$unpacked/target/qualification/link" && ! -e "$unpacked/target/qualification/link" ]]
[[ "$(readlink "$unpacked/target/qualification/link")" == ../unrelated/file ]]
[[ ! -e "$unpacked/.git" && ! -e "$unpacked/unrelated" && ! -e "$unpacked/target/unrelated" && ! -e "$unpacked/.tools/host" ]]
echo 'Consumer archive digest, original failure logs, bytes, modes, links and selection verified'
