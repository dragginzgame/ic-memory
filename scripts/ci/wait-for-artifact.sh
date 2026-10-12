#!/usr/bin/env bash
set -euo pipefail

# Read-only discovery gate for a freshly uploaded exact ID. The caller retains
# ownership of upload identity, download transport and independent payload checks.
[[ $# == 3 ]] || { echo 'usage: wait-for-artifact.sh OWNER/REPO RUN_ID ARTIFACT_ID' >&2; exit 2; }
repository="$1"
run_id="$2"
artifact_id="$3"
[[ "$repository" =~ ^[A-Za-z0-9_-]+/[A-Za-z0-9_.-]+$ &&
   "$run_id" =~ ^[1-9][0-9]*$ && "$artifact_id" =~ ^[1-9][0-9]*$ ]] || {
    echo 'Invalid repository, run or artifact identity' >&2; exit 2;
}
[[ -n "${GH_TOKEN:-}" ]] || { echo 'An explicit GH_TOKEN with actions:read is required' >&2; exit 2; }

# Six complete listings, at most 60 seconds of backoff. HTTP/auth/schema errors
# are terminal; only an absent exact ID in a successful listing is retried.
# The workflow also bounds this entire step, including API requests, to 2 minutes.
for delay in 0 2 4 8 16 30; do
    if [[ "$delay" != 0 ]]; then sleep "$delay"; fi
    listing="$(gh api --hostname github.com --method GET --paginate \
        "repos/$repository/actions/runs/$run_id/artifacts?per_page=100")"
    state="$(jq -ser --arg id "$artifact_id" --arg run "$run_id" '
        if length == 0 or any(.[]; (.artifacts | type) != "array") then
            error("Malformed artifact listing")
        elif any(.[] | .artifacts[]; type != "object" or (.id | type) != "number") then
            error("Malformed artifact record")
        else
            [.[] | .artifacts[] | select((.id | tostring) == $id)] as $matches
            | if ($matches | length) == 0 then "missing"
              elif ($matches | length) != 1 then error("Duplicate artifact identity")
              elif $matches[0].expired != false then error("Artifact expired or missing expiry status")
              elif ($matches[0].workflow_run.id | tostring) != $run then error("Artifact run identity mismatch")
              else "visible" end
        end
    ' <<< "$listing")"
    if [[ "$state" == visible ]]; then
        printf 'Artifact %s is visible in %s run %s\n' "$artifact_id" "$repository" "$run_id"
        exit 0
    fi
    printf 'Artifact %s is not yet visible in %s run %s\n' "$artifact_id" "$repository" "$run_id" >&2
done
echo 'Exact artifact discovery exhausted; download was not authorized' >&2
exit 1
