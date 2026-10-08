#!/usr/bin/env bash
# Exercise common dispatch and consumer-specific release adapters without effects.
set -euo pipefail
# This independent fixture owns its selections, not the invoking release's.
unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS MAKEFILES
unset VALIDATION_REPOSITORY_ROOT VALIDATION_RUNNER_SNAPSHOT_PATH
export RELEASE_DELIVERY=direct
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
bash "$ROOT/scripts/ci/check-release-commands.sh" "$ROOT" rust-toolchain.toml ci/tool-versions.env make/tools.mk
FIXTURE="$(mktemp -d "${TMPDIR:-/tmp}/ic-memory-release-adapters.XXXXXX")"
trap 'if [[ $? == 0 ]]; then rm -rf "$FIXTURE"; else printf "Consumer release-adapter fixture retained: %s\n" "$FIXTURE" >&2; fi' EXIT
mkdir -p "$FIXTURE/ci" "$FIXTURE/make"
cp "$ROOT/Makefile" "$ROOT/rust-toolchain.toml" "$FIXTURE/"
cp "$ROOT/ci/tool-versions.env" "$FIXTURE/ci/"
cp "$ROOT/make/tools.mk" "$FIXTURE/make/"
mkdir -p "$FIXTURE/scripts/ci"
cat > "$FIXTURE/scripts/ci/run-release.sh" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
[[ "$RELEASE_DELIVERY" == direct ]]
printf '%s\n' "$*" >> runner-events
STUB
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
(cd "$FIXTURE"; make --no-print-directory release-patch)
[[ "$(cat "$FIXTURE/runner-events")" == 'patch origin main' ]] || exit 1
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
cp "$ROOT/make/tools.mk" "$FIXTURE/bootstrap/make/"
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
    metadata)
        [[ "$*" == '+1.99.0 metadata --locked --offline --no-deps --format-version 1' ]]
        echo metadata >> calls
        echo '{"target_directory":"fixture"}'
        ;;
    run)
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

[[ "$(make --no-print-directory -s release-version)" == 0.12.3 ]]
[[ "$(cat calls)" == normal ]]
check_publication_environment
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
[[ "$(cat calls)" == $'normal\nmetadata\nbootstrap' ]]
check_publication_environment
cmp original-manifest Cargo.toml
cmp prepared-lock Cargo.lock
if BOOTSTRAP_STATUS=7 make --no-print-directory -s release-version > refusal.log 2>&1; then
    echo 'launcher ignored a failed locked bootstrap' >&2; exit 1
fi
cmp original-manifest Cargo.toml
cmp prepared-lock Cargo.lock
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
