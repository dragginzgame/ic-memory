#!/usr/bin/env bash
# Exercise common dispatch and consumer-specific release adapters without effects.
set -euo pipefail
# This independent fixture owns its selections, not the invoking release's.
unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS MAKEFILES
unset VALIDATION_REPOSITORY_ROOT VALIDATION_RUNNER_SNAPSHOT_PATH
unset RELEASE_CACHE_PREPARE
export RELEASE_DELIVERY=direct
# Resolve relative script entry points without CDPATH output or newline loss.
ROOT="${BASH_SOURCE[0]}"
[[ "$ROOT" == /* ]] || ROOT="$PWD/$ROOT"
ROOT="$(cd -P "${ROOT%/*}/../.." && printf '%s/.' "$PWD")"
ROOT="${ROOT%/.}"
bash "$ROOT/scripts/ci/check-release-commands.sh" "$ROOT" rust-toolchain.toml ci/tool-versions.env make/tools.mk make/release.mk make/execution.mk scripts/ci/check-make-execution.sh
FIXTURE="$(mktemp -d "${TMPDIR:-/tmp}/ic-memory-release-adapters.XXXXXX")"
fixture_complete=false
finish() {
    local status=$?
    [[ "$fixture_complete" == true || "$status" != 0 ]] || status=1
    if [[ "$status" == 0 ]]; then rm -rf "$FIXTURE"
    else printf 'Consumer release-adapter fixture retained: %s\n' "$FIXTURE" >&2; fi
    exit "$status"
}
trap finish EXIT
mkdir -p "$FIXTURE/ci" "$FIXTURE/make"
cp "$ROOT/Makefile" "$ROOT/rust-toolchain.toml" "$FIXTURE/"
cp "$ROOT/ci/tool-versions.env" "$FIXTURE/ci/"
cp "$ROOT/make/tools.mk" "$ROOT/make/release.mk" "$ROOT/make/execution.mk" "$FIXTURE/make/"
mkdir -p "$FIXTURE/scripts/ci"
cp "$ROOT/scripts/ci/check-make-execution.sh" "$FIXTURE/scripts/ci/"
cat > "$FIXTURE/scripts/ci/run-release.sh" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
[[ "$RELEASE_DELIVERY" == direct ]]
[[ "$RELEASE_CACHE_PREPARE" == 1 ]]
printf '%s\n' "$*" >> runner-events
exit "${RELEASE_TEST_STATUS:-0}"
STUB
# Actual Make admission must stop before even a failing substitute runner.
for target in release-patch release-minor release-major release-resume; do
    for mode in -i -n -t -q --ignore-errors --dry-run --touch --question -kin; do
        for source in direct inherited; do
            for replacement in original empty harmless; do
                flags=(--no-print-directory)
                case "$replacement" in
                    empty) flags+=(MAKEFLAGS=) ;;
                    harmless) flags+=(MAKEFLAGS=-j2) ;;
                esac
                status=0
                if [[ "$source" == direct ]]; then
                    (cd "$FIXTURE"; RELEASE_TEST_STATUS=23 make "$mode" "$target" "${flags[@]}") \
                        > "$FIXTURE/mode-$target-$mode-$source-$replacement.log" 2>&1 || status=$?
                else
                    (cd "$FIXTURE"; MAKEFLAGS="$mode" RELEASE_TEST_STATUS=23 make "$target" "${flags[@]}") \
                        > "$FIXTURE/mode-$target-$mode-$source-$replacement.log" 2>&1 || status=$?
                fi
                [[ "$status" == 2 && ! -e "$FIXTURE/runner-events" ]] || exit 1
            done
        done
    done
done
for target in release-patch release-minor release-major release-resume; do
    for source in command-line makefile; do
        status=0
        if [[ "$source" == command-line ]]; then
            (cd "$FIXTURE"; make -i --no-print-directory "$target" MAKEFLAGS= MFLAGS=) \
                > "$FIXTURE/mflags-$target-$source.log" 2>&1 || status=$?
        else
            printf 'MFLAGS :=\n' > "$FIXTURE/erased-mflags.mk"
            (cd "$FIXTURE"; make -i --no-print-directory -f erased-mflags.mk -f Makefile "$target" MAKEFLAGS=) \
                > "$FIXTURE/mflags-$target-$source.log" 2>&1 || status=$?
        fi
        [[ "$status" == 2 && ! -e "$FIXTURE/runner-events" ]] || exit 1
    done
done
for target in release-patch release-minor release-major release-resume; do
    for delivery in pr invalid; do
        if (cd "$FIXTURE"; make --no-print-directory "$target" RELEASE_DELIVERY="$delivery") \
            > "$FIXTURE/command-$target-$delivery.log" 2>&1; then
            echo 'unsupported release delivery accepted' >&2; exit 1
        fi
        if (cd "$FIXTURE"; RELEASE_DELIVERY="$delivery" make --no-print-directory "$target") \
            > "$FIXTURE/environment-$target-$delivery.log" 2>&1; then
            echo 'inherited unsupported release delivery accepted' >&2; exit 1
        fi
        [[ ! -e "$FIXTURE/runner-events" ]] || exit 1
    done
done
# Parallel qualification must refuse missing tools before any build/test phase.
cat > "$FIXTURE/tool-ordering.mk" <<'MAKE'
.PHONY: verify-shared-tooling tools-check check-format-tools check-pins
verify-shared-tooling check-format-tools:
	@:
tools-check:
	@exit 23
check-pins:
	@echo reached > build-events
MAKE
status=0
(cd "$FIXTURE"; make -j2 --no-print-directory -f Makefile -f tool-ordering.mk validate-toolchain) \
    > "$FIXTURE/tool-ordering.log" 2>&1 || status=$?
[[ "$status" == 2 && ! -e "$FIXTURE/build-events" ]] || exit 1
(cd "$FIXTURE"; make --no-print-directory release-patch)
[[ "$(cat "$FIXTURE/runner-events")" == 'patch origin main' ]] || exit 1
# The previous inline assignment forced preparation for every release entrypoint.
# Preserve that contract even when a caller supplies a conflicting selection.
for target in release-patch release-minor release-major release-resume; do
    (cd "$FIXTURE"; RELEASE_CACHE_PREPARE=0 make --no-print-directory "$target" RELEASE_CACHE_PREPARE=0 VERSION=0.1.1)
done
# Memory's standard release entrypoints always select this checkout's runner.
# An external substitute makes a redirected dispatch harmless and observable.
external_root="$FIXTURE/external runner"
mkdir -p "$external_root/scripts/ci"
cat > "$external_root/scripts/ci/check-make-execution.sh" <<'STUB'
#!/usr/bin/env bash
echo escaped >> external-probe-events
exit 23
STUB
cat > "$external_root/scripts/ci/run-release.sh" <<'STUB'
#!/usr/bin/env bash
printf '%s\n' "$*" >> external-runner-events
exit 23
STUB
rm "$FIXTURE/runner-events"
for target in release-patch release-minor release-major release-resume; do
    (cd "$FIXTURE"; SHARED_TOOLING_ROOT="$external_root" make --no-print-directory "$target" VERSION=0.1.1 RELEASE_REMOTE=fixture RELEASE_BRANCH=reviewed)
    (cd "$FIXTURE"; make --no-print-directory "$target" SHARED_TOOLING_ROOT="$external_root" VERSION=0.1.1 RELEASE_REMOTE=fixture RELEASE_BRANCH=reviewed)
done
printf '%s\n' 'patch fixture reviewed' 'patch fixture reviewed' \
    'minor fixture reviewed' 'minor fixture reviewed' \
    'major fixture reviewed' 'major fixture reviewed' \
    'resume 0.1.1 fixture reviewed' 'resume 0.1.1 fixture reviewed' > "$FIXTURE/expected-runner-events"
cmp "$FIXTURE/expected-runner-events" "$FIXTURE/runner-events"
[[ ! -e "$FIXTURE/external-runner-events" ]]
[[ ! -e "$FIXTURE/external-probe-events" ]]
cat > "$FIXTURE/overrides.mk" <<'MAKE'
RELEASE_REMOTE := recursive
.PHONY: nested
nested:
	+@$(MAKE) release-patch
MAKE
recursive_make="$(command -v make) --no-print-directory -f Makefile -f overrides.mk"
rm "$FIXTURE/runner-events"
(cd "$FIXTURE"; make -j2 --no-print-directory -f Makefile -f overrides.mk nested \
    "MAKE=$recursive_make" "SHARED_TOOLING_ROOT=$external_root" RELEASE_CACHE_PREPARE=0)
[[ "$(cat "$FIXTURE/runner-events")" == 'patch recursive main' ]]
[[ ! -e "$FIXTURE/external-runner-events" && ! -e "$FIXTURE/external-probe-events" ]]
# Qualify descriptor handoff through actual consumer Cargo, formatter and helper
# recipes. All executable effects below are substitutes, including publication.
jobserver="$FIXTURE/jobserver"
mkdir -p "$jobserver/bin" "$jobserver/make" "$jobserver/ci" "$jobserver/scripts/ci"
cp "$ROOT/Makefile" "$ROOT/rust-toolchain.toml" "$jobserver/"
cp "$ROOT/make/tools.mk" "$ROOT/make/release.mk" "$ROOT/make/execution.mk" "$jobserver/make/"
cp "$ROOT/ci/tool-versions.env" "$jobserver/ci/"
cp "$ROOT/scripts/ci/check-make-execution.sh" "$ROOT/scripts/ci/check-format-tools.sh" \
    "$ROOT/scripts/ci/run-formatting.sh" "$jobserver/scripts/ci/"
export JOBSERVER_EVENTS="$jobserver/events"
cat > "$jobserver/bin/cargo" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
[[ "${MAKEFLAGS:-}" =~ --jobserver-(auth|fds)=([0-9]+),([0-9]+) ]]
reader="${BASH_REMATCH[2]}"; writer="${BASH_REMATCH[3]}"
: <&"$reader"
: >&"$writer"
printf '%s\n' "$*" >> "$JOBSERVER_EVENTS"
[[ "$*" != 'sort --version' ]] || echo 'cargo-sort 2.1.4'
exit 0
STUB
cat > "$jobserver/helper" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
cargo helper "$@"
case "$1" in target) echo "$PWD/target" ;; version) echo 0.35.1 ;; esac
STUB
chmod +x "$jobserver/bin/cargo" "$jobserver/helper"
parallel=(-j4)
if make --help | grep -q -- --jobserver-style; then parallel+=(--jobserver-style=pipe); fi
for target in fmt fmt-check test-tooling test fetch-dependencies wasm-size version \
    release-version release-preflight release-verify qualify-release package publish publish-dry-run; do
    (cd "$jobserver"; PATH="$jobserver/bin:$PATH" make --no-print-directory "${parallel[@]}" \
        "$target" TOOL=./helper) > "$jobserver/$target.log" 2>&1
done
grep -F 'sort --workspace testing/runtime-qualification' "$JOBSERVER_EVENTS"
grep -F 'fmt --manifest-path testing/runtime-qualification/Cargo.toml --all' "$JOBSERVER_EVENTS"
grep -F 'helper release-verify' "$JOBSERVER_EVENTS"
grep -F 'helper publish --dry-run' "$JOBSERVER_EVENTS"
mkdir -p "$FIXTURE/custom target"
cat > "$FIXTURE/helper" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
case "$1" in
    target) printf '%s/custom target\n' "$PWD" ;;
    version) echo 0.25.14 ;;
    release-*)
        [[ "$RELEASE_DELIVERY" == direct ]]
        [[ "$RELEASE_KIND" == minor && "$RELEASE_PREVIOUS" == 0.25.14 && "$RELEASE_VERSION" == 0.26.0 ]]
        [[ "$RELEASE_DATE" == 2026-10-05 && "$RELEASE_SOURCE" == aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa ]]
        [[ "$RELEASE_REMOTE" == fixture && "$RELEASE_BRANCH" == reviewed ]]
        case "$1" in
            release-committed-check|release-tagged-check|release-push-check)
                [[ "$RELEASE_COMMIT" == bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb ]]
                ;;
            *) [[ -z "$RELEASE_COMMIT" ]] ;;
        esac
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
chmod +x "$FIXTURE/helper"
cd "$FIXTURE"
selection=(RELEASE_KIND=minor RELEASE_PREVIOUS=0.25.14 RELEASE_VERSION=0.26.0
    RELEASE_DATE=2026-10-05 RELEASE_SOURCE=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
    RELEASE_REMOTE=fixture RELEASE_BRANCH=reviewed RELEASE_COMMIT= TOOL=./helper)
[[ "$(make --no-print-directory -s release-version TOOL=./helper)" == 0.25.14 ]]
targets=(release-preflight release-prepare-version release-prepared-check release-files
    release-commit-check release-committed-check release-tagged-check release-push-check)
for target in "${targets[@]}"; do
    case "$target" in
        release-committed-check|release-tagged-check|release-push-check)
            make --no-print-directory "$target" "${selection[@]}" RELEASE_COMMIT=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
            ;;
        *) make --no-print-directory "$target" "${selection[@]}" ;;
    esac
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

# The actual Make launcher must reach the adapter while only the root lock is
# prepared. Cargo/parser substitutes prove isolation and argument propagation.
mkdir -p "$FIXTURE/bootstrap/scripts/dev" "$FIXTURE/bootstrap/bin" \
    "$FIXTURE/bootstrap/crates/ic-memory/src" "$FIXTURE/bootstrap/crates/ic-memory/examples" "$FIXTURE/bootstrap/ci" "$FIXTURE/bootstrap/make"
cp "$ROOT/Makefile" "$ROOT/rust-toolchain.toml" "$FIXTURE/bootstrap/"
cp "$ROOT/ci/tool-versions.env" "$FIXTURE/bootstrap/ci/"
cp "$ROOT/make/tools.mk" "$ROOT/make/release.mk" "$ROOT/make/execution.mk" "$FIXTURE/bootstrap/make/"
mkdir -p "$FIXTURE/bootstrap/scripts/ci"
cp "$ROOT/scripts/ci/check-make-execution.sh" "$FIXTURE/bootstrap/scripts/ci/"
cp "$ROOT/scripts/dev/run-repo-tool.sh" "$FIXTURE/bootstrap/scripts/dev/"
cd "$FIXTURE/bootstrap"
cat > Cargo.toml <<'MANIFEST'
[workspace]
members = ["crates/ic-memory"]
[workspace.package]
version = "0.12.3"
[workspace.dependencies]
other = "0.12.3"
MANIFEST
cat > Cargo.lock <<'LOCK'
version = 4
[[package]]
name = "ic-memory"
version = "0.12.3"
[[package]]
name = "other"
version = "0.12.3"
LOCK
echo fixture > crates/ic-memory/README.md
echo fixture-license > LICENSE
cat > crates/ic-memory/Cargo.toml <<'MANIFEST'
[package]
name = "ic-memory"
version.workspace = true
MANIFEST
cp Cargo.toml original-manifest
cat > bin/cargo <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
[[ "$1" == +1.99.0 ]]
[[ "$RUSTUP_AUTO_INSTALL" == 0 && "${CARGO_NET_OFFLINE-unset}" == "$CALLER_OFFLINE" ]]
case "$2" in
    fetch)
        [[ "$3" == --locked ]]
        offset=4
        if [[ "${RELEASE_CACHE_PREPARE:-0}" != 1 ]]; then
            [[ "$4" == --offline ]]
            offset=5
        fi
        if [[ $# -ge "$offset" ]]; then
            [[ "${!offset}" == --manifest-path ]]
            offset=$((offset + 1))
            cmp expected-manifest "${!offset}"
            cmp Cargo.lock "$(dirname "${!offset}")/Cargo.lock"
            echo bootstrap-cache >> calls
        else
            [[ $# == $((offset - 1)) ]]
            echo normal-cache >> calls
        fi
        if [[ "${CACHE_STATUS:-0}" != 0 ]]; then
            echo 'simulated offline cache failure' >&2
            exit "$CACHE_STATUS"
        fi
        if [[ "${CACHE_MISSING:-0}" == 1 ]]; then
            if [[ "${RELEASE_CACHE_PREPARE:-0}" != 1 || "$CALLER_OFFLINE" == true ]]; then
                echo 'simulated offline cache failure' >&2
                exit 101
            fi
            echo prepared >> preparation-events
        fi
        ;;
    metadata)
        [[ "$*" == '+1.99.0 metadata --locked --offline --no-deps --format-version 1' ]]
        echo metadata >> calls
        echo '{"target_directory":"fixture"}'
        ;;
    run)
        [[ "${RELEASE_CACHE_PREPARE:-0}" == 0 ]]
        [[ "$3" == --locked && "$4" == --offline && "$5" == --quiet ]]
        if [[ "$6" == --manifest-path ]]; then
            [[ "$8" == --target-dir && "$9" == "$BOOTSTRAP_TARGET/repo-tool-bootstrap/build" ]]
            [[ "${10}" == --example && "${11}" == repo-tool && "${12}" == -- ]]
            [[ "${13}" == version || "${13}" == publish ]]
            cmp expected-manifest "$7"
            cmp Cargo.lock "$(dirname "$7")/Cargo.lock"
            cmp crates/ic-memory/Cargo.toml "$(dirname "$7")/crates/ic-memory/Cargo.toml"
            cmp crates/ic-memory/README.md "$(dirname "$7")/crates/ic-memory/README.md"
            cmp LICENSE "$(dirname "$7")/LICENSE"
            [[ "$(readlink "$(dirname "$7")/crates/ic-memory/src")" == "$PWD/crates/ic-memory/src" ]]
            [[ "$(readlink "$(dirname "$7")/crates/ic-memory/examples")" == "$PWD/crates/ic-memory/examples" ]]
            echo bootstrap >> calls
        else
            [[ "$6" == --example && "$7" == repo-tool && "$8" == -- ]]
            [[ "$9" == version || "$9" == publish ]]
            echo normal >> calls
        fi
        printf '%s\n' "${CARGO_NET_OFFLINE-unset}" > child-offline
        exit_code="${BOOTSTRAP_STATUS:-0}"
        [[ "$exit_code" == 0 ]] || exit "$exit_code"
        echo 0.12.3
        ;;
    *) exit 2 ;;
esac
STUB
cat > bin/yq <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
[[ "$*" == '-p=json -r .target_directory' ]]
[[ "$(cat)" == '{"target_directory":"fixture"}' ]]
printf '%s\n' "$BOOTSTRAP_TARGET"
STUB
chmod +x bin/cargo bin/yq
export PATH="$PWD/bin:$PATH" YQ="$PWD/bin/yq" BOOTSTRAP_TARGET="$PWD/custom target"
unset CARGO_NET_OFFLINE
export CALLER_OFFLINE=unset

check_publication_environment() {
    local offline
    cp calls saved-version-calls
    for offline in unset false true; do
        export CALLER_OFFLINE="$offline"
        if [[ "$offline" == unset ]]; then unset CARGO_NET_OFFLINE
        else export CARGO_NET_OFFLINE="$offline"; fi
        make --no-print-directory publish > "publish-$offline.log" 2>&1
        [[ "$(cat child-offline)" == "$offline" ]]
    done
    unset CARGO_NET_OFFLINE
    export CALLER_OFFLINE=unset
    cp saved-version-calls calls
}

check_release_cache_preparation() {
    local offline status
    cp calls saved-cache-calls
    for offline in unset false true; do
        export CALLER_OFFLINE="$offline"
        if [[ "$offline" == unset ]]; then unset CARGO_NET_OFFLINE
        else export CARGO_NET_OFFLINE="$offline"; fi
        : > calls
        : > preparation-events
        status=0
        CACHE_MISSING=1 RELEASE_CACHE_PREPARE=1 bash scripts/dev/run-repo-tool.sh 1.99.0 version \
            > "release-cache-$offline.log" 2>&1 || status=$?
        if [[ "$offline" == true ]]; then
            [[ "$status" == 101 && ! -s preparation-events ]]
            grep -F 'Run make fetch-dependencies' "release-cache-$offline.log"
            if grep -E '^(normal|bootstrap)$' calls; then exit 1; fi
        else
            [[ "$status" == 0 && "$(cat preparation-events)" == prepared ]]
            grep -Fx 0.12.3 "release-cache-$offline.log"
            grep -E '^(normal|bootstrap)$' calls
        fi
    done
    unset CARGO_NET_OFFLINE
    export CALLER_OFFLINE=unset
    : > calls
    status=0
    CACHE_STATUS=9 RELEASE_CACHE_PREPARE=1 bash scripts/dev/run-repo-tool.sh 1.99.0 version \
        > release-network-failure.log 2>&1 || status=$?
    [[ "$status" == 9 ]]
    if grep -E '^(normal|bootstrap)$' calls; then exit 1; fi
    cp saved-cache-calls calls
}

[[ "$(make --no-print-directory -s release-version)" == 0.12.3 ]]
[[ "$(cat calls)" == $'normal-cache\nnormal' ]]
check_publication_environment
check_release_cache_preparation
cp calls successful-calls
status=0
CACHE_STATUS=101 bash scripts/dev/run-repo-tool.sh 1.99.0 version > cache-refusal.log 2>&1 || status=$?
[[ "$status" == 101 ]]
grep -F 'simulated offline cache failure' cache-refusal.log
grep -F 'Run make fetch-dependencies' cache-refusal.log
printf '%s\n' normal-cache >> successful-calls
cmp successful-calls calls
cmp original-manifest Cargo.toml
cp calls saved-calls
cat > expected-manifest <<'MANIFEST'
[workspace]
members = ["crates/ic-memory"]
[workspace.package]
version = "0.13.0"
[workspace.dependencies]
other = "0.12.3"
MANIFEST
awk '!changed && $0 == "version = \"0.12.3\"" { $0 = "version = \"0.13.0\""; changed = 1 } { print }' \
    Cargo.lock > prepared-lock
cp prepared-lock Cargo.lock
[[ "$(make --no-print-directory -s release-version 2> bootstrap.log)" == 0.12.3 ]]
printf '%s\n' metadata bootstrap-cache bootstrap >> saved-calls
cmp saved-calls calls
check_publication_environment
check_release_cache_preparation
cmp original-manifest Cargo.toml
cmp prepared-lock Cargo.lock
cp calls successful-calls
status=0
CACHE_STATUS=101 bash scripts/dev/run-repo-tool.sh 1.99.0 version > bootstrap-cache-refusal.log 2>&1 || status=$?
[[ "$status" == 101 ]]
grep -F 'simulated offline cache failure' bootstrap-cache-refusal.log
grep -F 'Run make fetch-dependencies' bootstrap-cache-refusal.log
printf '%s\n' metadata bootstrap-cache >> successful-calls
cmp successful-calls calls
cmp original-manifest Cargo.toml
cmp prepared-lock Cargo.lock
if BOOTSTRAP_STATUS=7 make --no-print-directory -s release-version > refusal.log 2>&1; then
    echo 'launcher ignored a failed locked bootstrap' >&2; exit 1
fi
cmp original-manifest Cargo.toml
cmp prepared-lock Cargo.lock
# Cargo's selected target is a literal path, including any terminal newline.
normal_target="$BOOTSTRAP_TARGET"
export BOOTSTRAP_TARGET="$PWD/newline target"$'\n\n'
[[ "$(make --no-print-directory -s release-version 2> newline-target.log)" == 0.12.3 ]]
[[ -d "$BOOTSTRAP_TARGET/repo-tool-bootstrap" ]]
[[ ! -e "$PWD/newline target" ]]
cmp original-manifest Cargo.toml
cmp prepared-lock Cargo.lock
# Plausible parser output never overrides its failed observation status.
cp bin/yq bin/failed-yq
printf 'exit 23\n' >> bin/failed-yq
cp calls failed-parser-calls
printf '%s\n' metadata >> failed-parser-calls
status=0
YQ="$PWD/bin/failed-yq" bash scripts/dev/run-repo-tool.sh 1.99.0 version > failed-parser.log 2>&1 || status=$?
[[ "$status" == 23 ]]
cmp failed-parser-calls calls
cmp original-manifest Cargo.toml
cmp prepared-lock Cargo.lock
export BOOTSTRAP_TARGET="$normal_target"
cp calls saved-calls
cat >> Cargo.lock <<'LOCK'
[[package]]
name = "ic-memory"
version = "0.14.0"
LOCK
cp Cargo.lock conflicting-lock
if make --no-print-directory -s release-version > conflicting.log 2>&1; then
    echo 'launcher accepted multiple root lock versions' >&2; exit 1
fi
cmp saved-calls calls
cmp original-manifest Cargo.toml
cmp conflicting-lock Cargo.lock
echo 'Consumer selection forwarding, attempt retention and launcher checks passed (substitutes only).'
fixture_complete=true
