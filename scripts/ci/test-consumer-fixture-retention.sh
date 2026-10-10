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
        [[ "$status" == "$expected" ]] || exit 1
        retained="$(cat "$RETENTION_EXIT_PATH")"
        [[ -n "$retained" ]] || exit 1
        if [[ "$expected" == 0 ]]; then [[ ! -e "$retained" ]] || exit 1
        else [[ -d "$retained" ]] || exit 1; fi
    done
done
# Run the actual checker against an owner returning the wrong failure status.
# It must stop after the first observation, before testing any other case.
assertion_root="$fixture/assertion-source"
mkdir -p "$assertion_root/scripts/ci"
cp "$ROOT/scripts/ci/test-consumer-fixture-retention.sh" "$assertion_root/scripts/ci/"
for name in release-adapters failure-evidence git-hooks release-runner; do
    cat > "$assertion_root/scripts/ci/test-$name.sh" <<'PROBE'
#!/usr/bin/env bash
set -euo pipefail
fixture="$(mktemp -d "${TMPDIR:-/tmp}/contradicted-owner.XXXXXX")"
printf 'called\n' >> "$RETENTION_ASSERTION_EVENTS"
trap 'exit 23' EXIT
PROBE
done
status=0
RETENTION_ASSERTION_EVENTS="$fixture/assertion-events" TMPDIR="$fixture" \
    "$BASH" "$assertion_root/scripts/ci/test-consumer-fixture-retention.sh" \
    > "$fixture/assertion.log" 2>&1 || status=$?
[[ "$status" == 1 && "$(cat "$fixture/assertion-events")" == called ]] || exit 1
echo 'Consumer fixture completion, failure status and evidence retention passed'
fixture_complete=true
