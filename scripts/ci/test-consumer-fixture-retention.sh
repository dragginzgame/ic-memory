#!/usr/bin/env bash
# Qualify actual fixture EXIT handlers without executing their test bodies.
set -euo pipefail
ROOT="${BASH_SOURCE[0]}"
[[ "$ROOT" == /* ]] || ROOT="$PWD/$ROOT"
ROOT="$(cd -P "${ROOT%/*}/../.." && printf '%s/.' "$PWD")"
ROOT="${ROOT%/.}"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/memory-fixture-retention.XXXXXX")"
fixture_complete=false
finish() {
    local status=$?
    [[ "$fixture_complete" == true || "$status" != 0 ]] || status=1
    if [[ "$status" == 0 ]]; then rm -rf "$fixture"
    else printf 'Consumer retention probes retained: %s\n' "$fixture" >&2; fi
    exit "$status"
}
trap finish EXIT
export RETENTION_SOURCE_ROOT="$ROOT" RETENTION_EXIT_PATH="$fixture/exit-path"
for name in release-adapters failure-evidence git-hooks release-runner; do
    for failure in nounset command nonzero premature completed failed-completion; do
        # shellcheck disable=SC2016 # Literal source for the disposable child.
        case "$failure" in
            nounset) injection='unset RETENTION_UNBOUND; printf "%s\n" "$RETENTION_UNBOUND"'; expected=1 ;;
            command) injection='false'; expected=1 ;;
            nonzero) injection='exit 23'; expected=23 ;;
            premature) injection='exit 0'; expected=1 ;;
            completed) injection='fixture_complete=true; exit 0'; expected=0 ;;
            failed-completion) injection='fixture_complete=true; exit 23'; expected=23 ;;
        esac
        RETENTION_INJECTION="$injection" awk '
            { print }
            /^(ROOT|root)=".*%\/\.\}"$/ {
                print "ROOT=\"$RETENTION_SOURCE_ROOT\""
                print "root=\"$RETENTION_SOURCE_ROOT\""
            }
            /^trap .* EXIT$/ && !injected {
                print "printf \"%s\\n\" \"${fixture:-${FIXTURE:-${FIXTURE_ROOT:-}}}\" > \"$RETENTION_EXIT_PATH\""
                print ENVIRON["RETENTION_INJECTION"]
                print "exit 99"
                injected=1
            }
            END { if (!injected) exit 1 }
        ' "$ROOT/scripts/ci/test-$name.sh" > "$fixture/probe.sh"
        status=0
        TMPDIR="$fixture" "$BASH" "$fixture/probe.sh" > "$fixture/$name-$failure.log" 2>&1 || status=$?
        [[ "$status" == "$expected" ]]
        retained="$(cat "$RETENTION_EXIT_PATH")"
        [[ -n "$retained" ]]
        if [[ "$expected" == 0 ]]; then [[ ! -e "$retained" ]]
        else [[ -d "$retained" ]]; fi
    done
done
echo 'Consumer fixture completion, failure status and evidence retention passed'
fixture_complete=true
