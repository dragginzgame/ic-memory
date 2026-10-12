#!/usr/bin/env bash
# Shared companions: make/consumer.mk scripts/ci/check-make-execution.sh scripts/ci/verify-file-checksum.sh .githooks/pre-commit
set -euo pipefail
unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS MAKEFILES
root="${BASH_SOURCE[0]}"
[[ "$root" == /* ]] || root="$PWD/$root"
root="$(cd -P "${root%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/make-consumer.XXXXXX")"
fixture_complete=false
finish() {
    local status=$?
    [[ "$fixture_complete" == true || "$status" != 0 ]] || status=1
    if [[ "$status" == 0 ]]; then rm -rf "$fixture"
    else printf 'Consumer Make fixture retained: %s\n' "$fixture" >&2; fi
    exit "$status"
}
trap finish EXIT
consumer="$fixture/consumer with spaces"
snapshot="$consumer/vendor/shared"
mkdir -p "$snapshot/make" "$snapshot/scripts/ci" "$snapshot/scripts/dev" \
    "$consumer/ci" "$consumer/scripts/ci" "$consumer/.githooks" \
    "$consumer/.tools/rust/common/bin"
for file in consumer tools release rust-format execution; do
    cp "$root/make/$file.mk" "$snapshot/make/"
done
for file in check-make-execution check-format-tools run-formatting verify-shared-tooling-snapshot; do
    cp "$root/scripts/ci/$file.sh" "$snapshot/scripts/ci/"
done
cp "$root/scripts/dev/install-git-hooks.sh" "$snapshot/scripts/dev/"
# Hooks retain their root-layout contract even when other tooling is nested.
cp "$root/.githooks/pre-commit" "$consumer/.githooks/"
for file in check-make-execution verify-shared-tooling-snapshot verify-file-checksum; do
    cp "$root/scripts/ci/$file.sh" "$consumer/scripts/ci/"
done
printf '[workspace]\n' > "$consumer/Cargo.toml"
printf 'SHARED_TOOLING_CARGO_SORT_VERSION=2.1.4\n' > "$consumer/ci/tool-versions.env"
export MAKE_CONSUMER_EVENTS="$fixture/events" RUNNER_TEMP="$fixture"
cat > "$consumer/Makefile" <<'MAKE'
override SHARED_TOOLING_ROOT := $(CURDIR)/vendor/shared
FORMAT_CARGO := fixture-cargo
RELEASE_RESUME_CHECK_COMMAND = bash "$(CURDIR)/resume-check.sh" "$(VERSION)"
include vendor/shared/make/consumer.mk
.PHONY: help local-admission
help:
	@echo consumer help
local-admission:
	@echo admission >> "$$MAKE_CONSUMER_EVENTS"; test "$${ADMISSION_FAIL:-0}" = 0
release-patch release-minor release-major release-resume: local-admission
release-patch release-minor release-major release-resume: export LOCAL_RELEASE_SELECTION := retained
MAKE
cat > "$consumer/.tools/rust/common/bin/fixture-cargo" <<'CARGO'
#!/usr/bin/env bash
set -eu
[[ "$CARGO_NET_OFFLINE" == true && "$RUSTUP_AUTO_INSTALL" == 0 ]]
case "$*" in
    'sort --version') echo 'cargo-sort 2.1.4'; exit 0 ;;
    'fmt --version') exit 0 ;;
esac
printf '%s\n' "$*" >> "$MAKE_CONSUMER_EVENTS"
CARGO
chmod +x "$consumer/.tools/rust/common/bin/fixture-cargo"
cat > "$snapshot/scripts/ci/run-release.sh" <<'RUNNER'
#!/usr/bin/env bash
set -eu
[[ "$LOCAL_RELEASE_SELECTION" == retained ]]
printf '%s\n' "$*" >> "$MAKE_CONSUMER_EVENTS"
exit "${RUNNER_RESULT:-0}"
RUNNER
cat > "$consumer/resume-check.sh" <<'CHECK'
#!/usr/bin/env bash
set -eu
printf 'receipt %s\n' "$1" >> "$MAKE_CONSUMER_EVENTS"
exit "${RECEIPT_RESULT:-0}"
CHECK
[[ "$(make -s -C "$consumer")" == 'consumer help' && ! -e "$MAKE_CONSUMER_EVENTS" ]]
[[ "$(make -s -C "$consumer" fmt-check)" == 'Checking formatting... ok' ]]
printf 'sort --workspace --check\nfmt --all -- --check\n' > "$fixture/expected"
cmp "$fixture/expected" "$MAKE_CONSUMER_EVENTS"

# Product prerequisites and target-specific selections survive the common route.
for kind in patch minor major resume; do
    : > "$MAKE_CONSUMER_EVENTS"
    make -s -C "$consumer" "release-$kind" VERSION=0.1.2 RELEASE_REMOTE=review RELEASE_BRANCH=topic
    if [[ "$kind" == resume ]]; then args='resume 0.1.2 review topic'
    else args="$kind review topic"; fi
    printf 'admission\n%s\n' "$args" > "$fixture/expected"
    if [[ "$kind" == resume ]]; then printf 'receipt 0.1.2\n' >> "$fixture/expected"; fi
    cmp "$fixture/expected" "$MAKE_CONSUMER_EVENTS"
done
: > "$MAKE_CONSUMER_EVENTS"
if ADMISSION_FAIL=1 make -s -C "$consumer" release-patch > "$fixture/admission.log" 2>&1; then exit 1; fi
printf 'admission\n' > "$fixture/expected"
cmp "$fixture/expected" "$MAKE_CONSUMER_EVENTS"
if RUNNER_RESULT=23 make -s -C "$consumer" release-patch > "$fixture/runner.log" 2>&1; then exit 1; fi
: > "$MAKE_CONSUMER_EVENTS"
if RUNNER_RESULT=23 make -s -C "$consumer" release-resume VERSION=0.1.2 > "$fixture/resume-failure.log" 2>&1; then exit 1; fi
printf 'admission\nresume 0.1.2 origin main\n' > "$fixture/expected"
cmp "$fixture/expected" "$MAKE_CONSUMER_EVENTS"
: > "$MAKE_CONSUMER_EVENTS"
if RECEIPT_RESULT=24 make -s -C "$consumer" release-resume VERSION=0.1.2 > "$fixture/receipt-failure.log" 2>&1; then exit 1; fi
printf 'receipt 0.1.2\n' >> "$fixture/expected"
cmp "$fixture/expected" "$MAKE_CONSUMER_EVENTS"
: > "$MAKE_CONSUMER_EVENTS"
if make -s -C "$consumer" release-patch release-minor > "$fixture/conflict.log" 2>&1; then exit 1; fi
[[ ! -s "$MAKE_CONSUMER_EVENTS" ]]

# A complete custom policy receives exact arguments and inherited Cargo context.
cat > "$consumer/custom.mk" <<'MAKE'
FORMAT_WRITE_COMMAND = bash "$(CURDIR)/custom formatter.sh" --write 'workspace with spaces'
FORMAT_CHECK_COMMAND = bash "$(CURDIR)/custom formatter.sh" --check 'workspace with spaces'
fmt fmt-check: export CARGO_HOME := $(CURDIR)/selected cargo home
include Makefile
MAKE
cat > "$consumer/custom formatter.sh" <<'FORMAT'
#!/usr/bin/env bash
set -eu
[[ "$CARGO_HOME" == "$PWD/selected cargo home" ]]
[[ "$CARGO_NET_OFFLINE" == true && "$RUSTUP_AUTO_INSTALL" == 0 ]]
[[ $# == 2 && "$2" == 'workspace with spaces' ]]
if [[ "${REQUIRE_JOBSERVER:-0}" == 1 ]]; then
    [[ "$MAKEFLAGS" =~ --jobserver-(auth|fds)=([0-9]+),([0-9]+) ]]
    reader="${BASH_REMATCH[2]}"; writer="${BASH_REMATCH[3]}"
    : <&"$reader"; : >&"$writer"
fi
echo "$1" >> "$MAKE_CONSUMER_EVENTS"
echo custom-detail >&2
exit "${FORMAT_RESULT:-0}"
FORMAT
parallel=(-j2)
if make --help | grep -q -- --jobserver-style; then parallel+=(--jobserver-style=pipe); fi
: > "$MAKE_CONSUMER_EVENTS"
for target in fmt fmt-check; do
    REQUIRE_JOBSERVER=1 make -s "${parallel[@]}" -C "$consumer" -f custom.mk "$target" > "$fixture/$target.log"
done
printf '%s\n' --write --check > "$fixture/expected"
cmp "$fixture/expected" "$MAKE_CONSUMER_EVENTS"
if FORMAT_RESULT=23 make -s -C "$consumer" -f custom.mk fmt-check > "$fixture/format-failure.log" 2>&1; then exit 1; fi
grep -F 'Checking formatting... FAILED (exit 23)' "$fixture/format-failure.log"
grep -Fx custom-detail "$fixture"/formatting.*
for mode in -i -n -t -q; do
    : > "$MAKE_CONSUMER_EVENTS"
    if make -s "$mode" -C "$consumer" -f custom.mk fmt > "$fixture/mode.log" 2>&1; then exit 1; fi
    [[ ! -s "$MAKE_CONSUMER_EVENTS" ]]
done
for assignment in 'FORMAT_WRITE_COMMAND=echo unexpected' 'FORMAT_CHECK_COMMAND=echo unexpected' \
    'SHARED_TOOLING_RUST=' 'SHARED_TOOLING_RUST=0 1' 'SHARED_TOOLING_RUST=%'; do
    if make -s -C "$consumer" help "$assignment" > "$fixture/invalid.log" 2>&1; then exit 1; fi
done

# Explicit setup changes only this disposable clone, not the nested snapshot.
mkdir "$fixture/templates"
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_TEMPLATE_DIR="$fixture/templates"
git init -q "$consumer"
make -s -C "$consumer" install-hooks
[[ "$(git -C "$consumer" config --local --get core.hooksPath)" == .githooks ]]
git -C "$consumer" config --local core.hooksPath other-hooks
if make -s -C "$consumer" install-hooks > "$fixture/hook-refusal.log" 2>&1; then exit 1; fi
[[ "$(git -C "$consumer" config --local --get core.hooksPath)" == other-hooks ]]

# The nested entrypoint verifies the consumer and its selected manifest offline.
{
    printf 'format\t1\nsource\tfixture\nrevision\t%040d\n' 0
    for path in scripts/ci/verify-file-checksum.sh scripts/ci/verify-shared-tooling-snapshot.sh; do
        digest="$(bash "$root/scripts/ci/verify-file-checksum.sh" --print sha256 "$consumer/$path")"
        mode=-; [[ ! -x "$consumer/$path" ]] || mode=x
        printf 'file\t%s\t%s\t%s\n' "$digest" "$mode" "$path"
    done
} > "$consumer/reviewed snapshot"
make -s -C "$consumer" shared-tooling-check 'SHARED_TOOLING_MANIFEST=reviewed snapshot'
printf '\n# drift\n' >> "$consumer/scripts/ci/verify-file-checksum.sh"
if make -s -C "$consumer" shared-tooling-check 'SHARED_TOOLING_MANIFEST=reviewed snapshot' > "$fixture/snapshot-failure.log" 2>&1; then exit 1; fi

# Non-Rust owners retain tooling/releases without advertising Rust operations.
[[ "$(make -s -C "$consumer" SHARED_TOOLING_RUST=0)" == 'consumer help' ]]
for target in fmt fmt-check install-hooks; do
    if make -s -C "$consumer" SHARED_TOOLING_RUST=0 "$target" > "$fixture/non-rust.log" 2>&1; then exit 1; fi
done
echo 'Consumer Make entrypoint passed (substitute release/format commands; no installation or release)'
fixture_complete=true
