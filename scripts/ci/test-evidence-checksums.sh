#!/usr/bin/env bash
# Shared companions: scripts/ci/verify-evidence-checksums.sh scripts/ci/verify-file-checksum.sh
set -Eeuo pipefail

ROOT="${BASH_SOURCE[0]}"
[[ "$ROOT" == /* ]] || ROOT="$PWD/$ROOT"
ROOT="$(cd -P "${ROOT%/*}/../.." && printf '%s/.' "$PWD")"
ROOT="${ROOT%/.}"
TEMPORARY="$(mktemp -d "${TMPDIR:-/tmp}/blob-evidence-checksums.XXXXXX")"
fixture_complete=false
finish() {
    local status=$?
    # Bash 3.2 may report zero after nounset before assertions finish.
    [[ "$fixture_complete" == true || "$status" != 0 ]] || status=1
    if [[ "$status" == 0 ]]; then rm -rf "$TEMPORARY";
    else echo "Checksum fixture/evidence retained: $TEMPORARY" >&2; fi
    exit "$status"
}
trap finish EXIT
cp "$ROOT/scripts/ci/verify-evidence-checksums.sh" "$ROOT/scripts/ci/verify-file-checksum.sh" "$TEMPORARY/"
cd "$TEMPORARY"
printf abc > 'selected file'
digest=ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad
printf '%s  selected file\n' "$digest" > text.manifest
# Accept the binary marker and a final line without a newline as GNU checks do.
printf '%s *selected file' "$digest" > binary.manifest
for backend in sha256sum shasum; do
    # Native macOS has shasum; exercise GNU when available without requiring it.
    if ! command -v "$backend" >/dev/null 2>&1; then
        [[ "$backend" == sha256sum ]] && continue
        echo "missing fixture prerequisite: $backend" >&2; exit 1
    fi
    mkdir "path-$backend"
    for tool in bash dirname "$backend"; do ln -s "$(command -v "$tool")" "path-$backend/$tool"; done
    PATH="$TEMPORARY/path-$backend" "$BASH" verify-evidence-checksums.sh text.manifest binary.manifest > "$backend.log" 2>&1
done
expect_failure() {
    if "$BASH" verify-evidence-checksums.sh "$@" > refusal.log 2>&1; then
        echo 'Checksum verifier unexpectedly accepted fixture' >&2; exit 1
    fi
}
printf changed > 'selected file'
expect_failure text.manifest
[[ "$(cat 'selected file')" == changed ]] || exit 1
printf abc > 'selected file'
cp text.manifest before.manifest
printf '%s  absent\n' "$digest" > missing.manifest
expect_failure missing.manifest
expect_failure absent.manifest
for record in 'bad digest  selected file' "$digest selected file" "$digest  "; do
    printf '%s\n' "$record" > malformed.manifest
    expect_failure text.manifest malformed.manifest
done
: > empty.manifest
expect_failure empty.manifest
expect_failure
mkdir path-no-hash
for tool in bash dirname; do ln -s "$(command -v "$tool")" "path-no-hash/$tool"; done
if PATH="$TEMPORARY/path-no-hash" "$BASH" verify-evidence-checksums.sh text.manifest > unavailable.log 2>&1; then
    echo 'Checksum verifier accepted without a hashing implementation' >&2; exit 1
fi
cmp text.manifest before.manifest
[[ "$(cat 'selected file')" == abc ]] || exit 1
echo 'Evidence-checksum tests: PASS (read-only GNU/Perl paths and refusals).'
fixture_complete=true
