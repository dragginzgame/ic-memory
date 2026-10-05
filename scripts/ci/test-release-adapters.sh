#!/usr/bin/env bash
# Exercise the consumer Make boundary with effect-free helper/runner substitutes.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FIXTURE="$(mktemp -d "${TMPDIR:-/tmp}/ic-memory-release-adapters.XXXXXX")"
trap 'rm -rf "$FIXTURE"' EXIT
cp "$ROOT/Makefile" "$ROOT/rust-toolchain.toml" "$ROOT/ci-tool-versions.env" "$FIXTURE/"
mkdir -p "$FIXTURE/scripts/ci" "$FIXTURE/custom target"
cat > "$FIXTURE/helper" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
case "$1" in
    target) printf '%s/custom target\n' "$PWD" ;;
    version) echo 0.25.14 ;;
    release-*)
        [[ "$RELEASE_KIND" == minor && "$RELEASE_PREVIOUS" == 0.25.14 && "$RELEASE_VERSION" == 0.26.0 ]]
        [[ "$RELEASE_DATE" == 2026-10-05 && "$RELEASE_SOURCE" == aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa ]]
        [[ "$RELEASE_REMOTE" == fixture && "$RELEASE_BRANCH" == reviewed ]]
        printf '%s\n' "$1" >> events
        if [[ "$1" == release-verify ]]; then
            printf 'gate stdout %s\n' "${GATE_STATUS:-0}"
            printf 'gate stderr %s\n' "${GATE_STATUS:-0}" >&2
            exit "${GATE_STATUS:-0}"
        fi
        ;;
    *) exit 2 ;;
esac
STUB
cat > "$FIXTURE/scripts/ci/run-release.sh" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$@" > dispatch
STUB
chmod +x "$FIXTURE/helper"
cd "$FIXTURE"
for kind in patch minor major; do
    make --no-print-directory "release-$kind" RELEASE_REMOTE=fixture RELEASE_BRANCH=reviewed
    printf '%s\n' "$kind" fixture reviewed > expected
    cmp expected dispatch
done
make --no-print-directory release-resume VERSION=0.26.0 RELEASE_REMOTE=fixture RELEASE_BRANCH=reviewed
printf '%s\n' resume 0.26.0 fixture reviewed > expected
cmp expected dispatch
cp dispatch saved-dispatch
if make --no-print-directory release-patch release-minor > conflict.log 2>&1; then
    echo 'multiple release selections accepted' >&2; exit 1
fi
cmp saved-dispatch dispatch
selection=(RELEASE_KIND=minor RELEASE_PREVIOUS=0.25.14 RELEASE_VERSION=0.26.0
    RELEASE_DATE=2026-10-05 RELEASE_SOURCE=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
    RELEASE_REMOTE=fixture RELEASE_BRANCH=reviewed TOOL=./helper)
[[ "$(make --no-print-directory -s release-version TOOL=./helper)" == 0.25.14 ]]
targets=(release-preflight release-prepare-version release-prepared-check release-files
    release-commit-check release-committed-check release-tagged-check release-push-check)
for target in "${targets[@]}"; do
    make --no-print-directory "$target" "${selection[@]}"
done
printf '%s\n' "${targets[@]}" > expected
cmp expected events
if make --no-print-directory release-verify "${selection[@]}" GATE_STATUS=7 > failed.log 2>&1; then
    echo 'failed verification accepted' >&2; exit 1
fi
logs=("custom target/release-validation/attempts/"verify.*)
[[ "${#logs[@]}" == 1 ]]
cp "${logs[0]}" saved-failure
make --no-print-directory release-verify "${selection[@]}" GATE_STATUS=0 > success.log 2>&1
logs=("custom target/release-validation/attempts/"verify.*)
[[ "${#logs[@]}" == 2 ]]
failed=0
passed=0
for log in "${logs[@]}"; do
    if cmp -s saved-failure "$log"; then failed=$((failed + 1)); fi
    if [[ "$(cat "$log")" == $'gate stdout 0\ngate stderr 0' ]]; then passed=$((passed + 1)); fi
done
[[ "$failed" == 1 && "$passed" == 1 ]]
echo 'Consumer release Make dispatch, selection forwarding and attempt retention passed (substitutes only).'
