#!/usr/bin/env bash
# Qualify fixture completion, evidence retention and the actual archive round trip.
set -euo pipefail
ROOT="${BASH_SOURCE[0]}"
[[ "$ROOT" == /* ]] || ROOT="$PWD/$ROOT"
ROOT="$(cd -P "${ROOT%/*}/../.." && printf '%s/.' "$PWD")"
ROOT="${ROOT%/.}"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/memory-fixture-retention.XXXXXX")"
fixture="$(CDPATH="" cd -P -- "$fixture" && printf '%s/.' "$PWD")"
fixture="${fixture%/.}"
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$ROOT/scripts/ci/fixture-lifecycle.sh"
# shellcheck source=scripts/ci/fixture-retention-probes.sh
source "$ROOT/scripts/ci/fixture-retention-probes.sh"
fixture_complete=false
trap 'fixture_finish "$?" "$fixture" "Consumer retention probes retained"' EXIT
for name in release-adapters failure-evidence git-hooks release-runner; do
    fixture_probe_exits "$ROOT/scripts/ci/test-$name.sh" "$fixture" "$ROOT" "$name" || exit 1
done
# Run the actual checker against an owner returning the wrong failure status.
# It must stop after the first observation, before testing any other case.
assertion_root="$fixture/assertion-source"
mkdir -p "$assertion_root/scripts/ci"
cp "$ROOT/scripts/ci/test-consumer-fixture-retention.sh" "$assertion_root/scripts/ci/"
cp "$ROOT/scripts/ci/fixture-lifecycle.sh" "$assertion_root/scripts/ci/"
cp "$ROOT/scripts/ci/fixture-retention-probes.sh" "$assertion_root/scripts/ci/"
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
# macOS and callers may select a temporary root through a directory alias.
# Run the complete round trip so an earlier, unintended host-fixture failure
# cannot masquerade as the controlled installed-tool failure.
mkdir "$fixture/physical-temp"
ln -s physical-temp "$fixture/aliased-temp"
(cd "$fixture" && TMPDIR=aliased-temp "$BASH" "$ROOT/scripts/ci/test-failure-evidence.sh") \
    > "$fixture/aliased-evidence.log" 2>&1
echo 'Consumer fixture completion, failure status and evidence retention passed'
fixture_complete=true
