#!/usr/bin/env bash
# Shared companions: scripts/ci/verify-file-checksum.sh
set -euo pipefail

# Exact-ID readback of our single-file evidence artifact; no listing/name lookup.
[[ $# == 5 ]] || { echo 'usage: download-evidence.sh OWNER/REPO RUN_ID ARTIFACT_ID SHA256 NEW_DIRECTORY' >&2; exit 2; }
repository="$1"; run_id="$2"; artifact_id="$3"; digest="$4"; destination="$5"
[[ "$repository" =~ ^[A-Za-z0-9_-]+/[A-Za-z0-9_.-]+$ && "$run_id" =~ ^[1-9][0-9]*$ &&
   "$artifact_id" =~ ^[1-9][0-9]*$ && "$digest" =~ ^[0-9a-f]{64}$ &&
   "$destination" == /* && "$destination" != *$'\r'* && "$destination" != *$'\n'* ]] || {
    echo 'Invalid evidence identity, digest or destination' >&2; exit 2;
}
[[ -n "${GH_TOKEN:-}" ]] || { echo 'An explicit GH_TOKEN with actions:read is required' >&2; exit 2; }
parent="$(cd -P "${destination%/*}/" && printf '%s/.' "$PWD")"; parent="${parent%/.}"
[[ "$parent" != *$'\r'* && "$parent" != *$'\n'* ]] || exit 2
destination="$parent/${destination##*/}"
mkdir "$destination"
ROOT="${BASH_SOURCE[0]%/*}"
# All attempts remain available for diagnosing failure or transient visibility.
request() {
    local endpoint="$1" output="$2" status=0
    retryable=false
    gh api --hostname github.com --method GET "$endpoint" > "$output" 2> "$output.errors" || status=$?
    cat "$output.errors" >&2
    [[ "$status" != 0 ]] || return 0
    # gh reports HTTP status on stderr. Retry only a reported 404, never a
    # transport/auth/parser/integrity error. Exhaustion still fails closed.
    if [[ "$status" == 1 ]] && grep -Eq '^gh: .*\(HTTP 404\)$' "$output.errors"; then retryable=true; fi
    return "$status"
}
for delay in 0 2 4 8 16 30; do
    [[ "$delay" == 0 ]] || sleep "$delay"
    attempt="$destination/attempt-$delay"
    mkdir "$attempt"
    status=0
    request "repos/$repository/actions/artifacts/$artifact_id" "$attempt/metadata.json" || status=$?
    if [[ "$retryable" == true ]]; then continue; fi
    [[ "$status" == 0 ]] || exit "$status"
    jq -e --arg id "$artifact_id" --arg run "$run_id" --arg digest "sha256:$digest" '
        (.id | tostring) == $id and (.workflow_run.id | tostring) == $run and
        .expired == false and .digest == $digest
    ' "$attempt/metadata.json" >/dev/null || { echo 'Evidence artifact identity or digest mismatch' >&2; exit 1; }
    request "repos/$repository/actions/artifacts/$artifact_id/zip" "$attempt/artifact.zip" || status=$?
    if [[ "$retryable" == true ]]; then continue; fi
    [[ "$status" == 0 ]] || exit "$status"
    bash "$ROOT/verify-file-checksum.sh" sha256 "$digest" "$attempt/artifact.zip"
    # Never extract arbitrary ZIP paths. Our collector uploads exactly this file.
    unzip -Z1 "$attempt/artifact.zip" > "$attempt/members"
    printf 'evidence.tar.gz\n' > "$attempt/expected-members"
    cmp "$attempt/members" "$attempt/expected-members" || { echo 'Unexpected evidence artifact members' >&2; exit 1; }
    unzip -p "$attempt/artifact.zip" evidence.tar.gz > "$attempt/evidence.tar.gz"
    mv "$attempt/evidence.tar.gz" "$destination/evidence.tar.gz"
    printf 'Downloaded verified artifact %s from %s run %s\n' "$artifact_id" "$repository" "$run_id"
    exit 0
done
echo 'Exact artifact download exhausted; attempts retained in the destination' >&2
exit 1
