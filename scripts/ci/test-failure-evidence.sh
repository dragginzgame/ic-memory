#!/usr/bin/env bash
set -euo pipefail
shopt -s nullglob
# Anchor the script path before cd so inherited CDPATH cannot enter ROOT.
# The sentinel keeps command substitution from trimming pathname newlines.
ROOT="$0"
[[ "$ROOT" == /* ]] || ROOT="$PWD/$ROOT"
ROOT="$(cd -P "${ROOT%/*}/../.." && printf '%s/.' "$PWD")"
ROOT="${ROOT%/.}"
[[ $# -le 1 ]] || { echo 'usage: test-failure-evidence.sh [NEW-ROUNDTRIP-DIRECTORY]' >&2; exit 2; }
retained=false
if [[ $# == 1 ]]; then
    [[ ! -e "$1" && ! -L "$1" ]] || { echo 'round-trip directory must be new' >&2; exit 1; }
    mkdir -- "$1"
    fixture="$1"
    retained=true
else
    fixture="$(mktemp -d "${TMPDIR:-/tmp}/failure-evidence.XXXXXX")"
fi
fixture_complete=false
finish() {
    local status=$?
    [[ "$fixture_complete" == true || "$status" != 0 ]] || status=1
    if [[ "$status" == 0 && "$retained" == false ]]; then rm -rf "$fixture"
    else printf 'Failure-evidence fixture retained: %s\n' "$fixture" >&2; fi
    exit "$status"
}
trap finish EXIT
# Child installers use physical paths; give their fixture assertions the same root.
[[ "$fixture" == /* ]] || fixture="$PWD/$fixture"
fixture="$(cd -P "$fixture" && printf '%s/.' "$PWD")"
fixture="${fixture%/.}"

# Memory owns the hosted upload/discovery/download bindings; Shared owns retries.
yq -o=json '.jobs.test' "$ROOT/.github/workflows/ci.yml" > "$fixture/workflow.json"
jq -e '
    [.steps[] | select(.id == "evidence-discovery")] as $gates
    | [.steps[] | select((.uses // "") | startswith("actions/download-artifact@"))] as $downloads
    | .permissions.actions == "read"
    and ($gates | length) == 1 and ($gates[0] |
        .if == "${{ !cancelled() && steps.evidence-upload.outputs.artifact-id != '\'''\'' }}"
        and .run == "bash scripts/ci/wait-for-artifact.sh \"$EVIDENCE_REPOSITORY\" \"$EVIDENCE_RUN\" \"$EVIDENCE_ARTIFACT\""
        and .["timeout-minutes"] == 2 and (."continue-on-error" // false) == false
        and .env.GH_TOKEN == "${{ github.token }}"
        and .env.EVIDENCE_REPOSITORY == "${{ github.repository }}"
        and .env.EVIDENCE_RUN == "${{ github.run_id }}"
        and .env.EVIDENCE_ARTIFACT == "${{ steps.evidence-upload.outputs.artifact-id }}")
    and ($downloads | length) == 1 and ($downloads[0] |
        .if == "${{ !cancelled() && steps.evidence-discovery.outcome == '\''success'\'' }}"
        and (."continue-on-error" // false) == false
        and .with["artifact-ids"] == "${{ steps.evidence-upload.outputs.artifact-id }}"
        and .with.repository == "${{ github.repository }}"
        and .with["run-id"] == "${{ github.run_id }}"
        and .with["github-token"] == "${{ github.token }}"
        and .with["digest-mismatch"] == "error")
' "$fixture/workflow.json" > /dev/null

mkdir -p "$fixture/temp/ic-memory-fixtures" "$fixture/repository"
temp_root="$fixture/temp"
repository_root="$fixture/repository"
export SHARED_TOOLING_STORE="$fixture/shared-store"

# A real retained host-tool fixture, with all downloads/installations substituted.
if TMPDIR="$temp_root/ic-memory-fixtures" TEST_VERSION_STATUS=9 \
    bash "$ROOT/scripts/ci/test-host-tools.sh" > "$temp_root/tools-setup.log" 2>&1; then
    echo 'expected the controlled host-tool failure' >&2; exit 1
else
    status=$?
fi
[[ "$status" == 1 ]] || exit 1
printf 'original_status=%s\n' "$status" > "$temp_root/validation.log"
printf 'selected dependency diagnostics\n' > "$temp_root/dependencies.log"
printf 'runtime setup failure\n' > "$temp_root/runtime-setup.log"
printf 'common tool admission failure\n' > "$temp_root/tools-check.log"
printf 'runtime host setup diagnostics\n' > "$temp_root/runtime-host-setup.log"
printf 'runtime host admission failure\n' > "$temp_root/runtime-host-check.log"
printf 'runtime IC setup diagnostics\n' > "$temp_root/runtime-ic-setup.log"
printf 'runtime IC admission failure\n' > "$temp_root/runtime-ic-check.log"
printf 'runtime diagnostics\n' > "$temp_root/runtime.log"
printf 'formatter diagnostics\n' > "$temp_root/formatting.failure"
printf 'unselected temporary file\n' > "$temp_root/unrelated"
mkdir -p "$repository_root/target/qualification" "$repository_root/target/release-validation" \
    "$repository_root/.tools/host-set.test/bin" "$repository_root/.tools/ic-set.test/bin" \
    "$repository_root/.tools/rust/build/cargo-attempt.test" \
    "$repository_root/.tools/rust/cargo-sort-2.1.4-bin-cargo-sort-release/installed/build" \
    "$repository_root/.tools/rust/common-set.test/bin" \
    "$repository_root/.tools/ic-testkit-server/failed-attempt" \
    "$repository_root/.git" "$repository_root/target/unrelated"
printf 'CLI build failure\n' > "$repository_root/.tools/rust/build/cargo-attempt.test/install.log"
printf 'successful Cargo build evidence\n' > "$repository_root/.tools/rust/cargo-sort-2.1.4-bin-cargo-sort-release/installed/build/output"
printf 'selected Cargo receipt\n' > "$repository_root/.tools/rust/cargo-sort-2.1.4-bin-cargo-sort-release/installed/selection.json"
ln -s ../../cargo-sort-2.1.4-bin-cargo-sort-release/installed/bin/cargo-sort "$repository_root/.tools/rust/common-set.test/bin/cargo-sort"
ln -s common-set.test "$repository_root/.tools/rust/common"
printf 'server setup failure\n' > "$repository_root/.tools/ic-testkit-server/failed-attempt/version.stderr"
printf 'qualification\n' > "$repository_root/target/qualification/"$'line\nbreak:payload'
chmod 640 "$repository_root/target/qualification/"$'line\nbreak:payload'
printf 'release validation\n' > "$repository_root/target/release-validation/attempt.log"
printf '#!/bin/sh\nexit 0\n' > "$repository_root/.tools/host-set.test/bin/tool"
chmod 755 "$repository_root/.tools/host-set.test/bin/tool"
printf 'IC diagnostics\n' > "$repository_root/.tools/ic-set.test/receipt"
printf 'private configuration\n' > "$repository_root/.git/config"
printf 'unselected target\n' > "$repository_root/target/unrelated/file"
ln -s ../unrelated/file "$repository_root/target/qualification/link"
ln -s host-set.test "$repository_root/.tools/host"

# Select a shared bundle through the canonical reference contract. Duplicate
# local references retain it once; unrelated store entries and unknown links
# must never widen this checkout's archive selection.
shared_bundle="$(source "$ROOT/scripts/dev/tool-store.sh"; tool_store_init host evidence; printf '%s/set.evidence\n' "$store_slot")"
(umask 077; mkdir -p "$shared_bundle/bin" "$SHARED_TOOLING_STORE/unrelated")
printf 'selected shared payload\n' > "$shared_bundle/bin/tool"
printf 'unrelated store payload\n' > "$SHARED_TOOLING_STORE/unrelated/private"
ln -s "$shared_bundle" "$repository_root/.tools/host-set.shared"
ln -s "$shared_bundle" "$repository_root/.tools/host-set.sharedDuplicate"
ln -s "$SHARED_TOOLING_STORE/unrelated" "$repository_root/.tools/ic-set.unknown"
rust_bundle="$(source "$ROOT/scripts/dev/tool-store.sh"; tool_store_init rust evidence; printf '%s/set.evidence\n' "$store_slot")"
(umask 077; mkdir -p "$rust_bundle/build")
printf 'shared Cargo build failure\n' > "$rust_bundle/build/install.log"
# A build-identity refusal retains both observations without activating a receipt.
printf 'expected native inventory\n' > "$rust_bundle/native-environment.txt"
printf 'observed native inventory\n' > "$rust_bundle/native-environment.observed.txt"
expected_native="$(bash "$ROOT/scripts/ci/verify-file-checksum.sh" --print sha256 "$rust_bundle/native-environment.txt")"
observed_native="$(bash "$ROOT/scripts/ci/verify-file-checksum.sh" --print sha256 "$rust_bundle/native-environment.observed.txt")"
printf '{"build":{"native":"%s"}}\n' "$expected_native" > "$rust_bundle/inputs.json"
printf '{"native":"%s"}\n' "$observed_native" > "$rust_bundle/build-identity.observed.json"
ln -s "$rust_bundle" "$repository_root/.tools/rust/rust-set.shared"
ln -s "$rust_bundle" "$repository_root/.tools/rust/rust-set.sharedDuplicate"
ln -s "$SHARED_TOOLING_STORE/unrelated" "$repository_root/.tools/rust/rust-set.unknown"

# Relative entry points must ignore inherited CDPATH when locating helpers.
archive="$(cd "$ROOT" && GITHUB_OUTPUT="$fixture/full-output" CDPATH="$ROOT" bash scripts/ci/collect-failure-evidence.sh "$temp_root" "$repository_root")"
[[ -f "$archive" && "${archive##*/}" == evidence.tar.gz ]] || exit 1
[[ "$(cat "$fixture/full-output")" == collection-complete=true ]] || exit 1
bash "$ROOT/scripts/ci/verify-file-checksum.sh" --print sha256 "$archive" > "$fixture/archive.sha256"
(cd "$ROOT" && CDPATH="$ROOT" bash scripts/ci/verify-failure-evidence.sh "$fixture" "$archive")

# Retries create separate archives and preserve original evidence and gate status.
cp "$archive" "$fixture/saved.tar.gz"
retry="$(bash "$ROOT/scripts/ci/collect-failure-evidence.sh" "$temp_root" "$repository_root")"
[[ "$archive" != "$retry" && -f "$retry" ]] || exit 1
cmp "$archive" "$fixture/saved.tar.gz"
[[ "$(cat "$temp_root/validation.log")" == original_status=1 ]] || exit 1

# Compact selection keeps unknown/unverified bundles and the primary failure.
compact="$(bash "$ROOT/scripts/ci/collect-failure-evidence.sh" "$temp_root" "$repository_root" compact)"
cp "$fixture/archive.sha256" "$fixture/original.sha256"
bash "$ROOT/scripts/ci/verify-file-checksum.sh" --print sha256 "$compact" > "$fixture/archive.sha256"
bash "$ROOT/scripts/ci/verify-failure-evidence.sh" "$fixture" "$compact"
mv "$fixture/original.sha256" "$fixture/archive.sha256"
# Matching archive bytes cannot hide a lost or changed refusal observation.
cp "$rust_bundle/native-environment.observed.txt" "$fixture/original-observed-native.txt"
printf 'wrong observed inventory\n' > "$rust_bundle/native-environment.observed.txt"
if bash "$ROOT/scripts/ci/verify-failure-evidence.sh" "$fixture" "$archive" > "$fixture/contradicted-observation.log" 2>&1; then
    echo 'expected observed build-identity refusal' >&2; exit 1
fi
mv "$fixture/original-observed-native.txt" "$rust_bundle/native-environment.observed.txt"
# A linked observation can compare equal through the live fixture while its
# bytes never enter the archive. Download verification must refuse that shape.
cp "$rust_bundle/native-environment.observed.txt" "$fixture/linked-observation.txt"
rm "$rust_bundle/native-environment.observed.txt"
ln -s "$fixture/linked-observation.txt" "$rust_bundle/native-environment.observed.txt"
linked_archive="$(bash "$ROOT/scripts/ci/collect-failure-evidence.sh" "$temp_root" "$repository_root")"
cp "$fixture/archive.sha256" "$fixture/original.sha256"
bash "$ROOT/scripts/ci/verify-file-checksum.sh" --print sha256 "$linked_archive" > "$fixture/archive.sha256"
if bash "$ROOT/scripts/ci/verify-failure-evidence.sh" "$fixture" "$linked_archive" > "$fixture/linked-observation.log" 2>&1; then
    echo 'expected linked observation refusal' >&2; exit 1
fi
mv "$fixture/original.sha256" "$fixture/archive.sha256"
rm "$rust_bundle/native-environment.observed.txt"
mv "$fixture/linked-observation.txt" "$rust_bundle/native-environment.observed.txt"
# Rust-only checkouts must invoke the selector without any host/IC references.
mkdir -p "$fixture/rust-only/.tools" "$fixture/rust-only-temp"
cp -R "$repository_root/.tools/rust" "$fixture/rust-only/.tools/"
rust_archive="$(GITHUB_OUTPUT="$fixture/rust-only-output" bash "$ROOT/scripts/ci/collect-failure-evidence.sh" "$fixture/rust-only-temp" "$fixture/rust-only")"
[[ "$(cat "$fixture/rust-only-output")" == collection-complete=true ]] || exit 1
mkdir "$fixture/rust-only-unpacked"
tar -xzf "$rust_archive" -C "$fixture/rust-only-unpacked"
rust_path="${rust_bundle#"$SHARED_TOOLING_STORE/"}"
for path in build/install.log inputs.json native-environment.txt \
    build-identity.observed.json native-environment.observed.txt; do
    cmp "$rust_bundle/$path" "$fixture/rust-only-unpacked/$rust_path/$path"
done
cmp "$repository_root/.tools/rust/build/cargo-attempt.test/install.log" "$fixture/rust-only-unpacked/.tools/rust/build/cargo-attempt.test/install.log"
tar -tzf "$rust_archive" > "$fixture/rust-only-members"
[[ "$(awk '{ sub(/^\.\//, "") } $0 == ".tools/rust/" { count++ } END { print count+0 }' "$fixture/rust-only-members")" == 1 ]] || exit 1
[[ "$(awk -v member="$rust_path/build/install.log" '{ sub(/^\.\//, "") } $0 == member { count++ } END { print count+0 }' "$fixture/rust-only-members")" == 1 ]] || exit 1
# Valid archive bytes and a matching input log must not admit the wrong status.
# The verifier's conditional assertions need explicit failure on Bash 3.2.
cp "$temp_root/validation.log" "$fixture/original-validation.log"
printf 'original_status=2\n' > "$temp_root/validation.log"
contradicted="$(bash "$ROOT/scripts/ci/collect-failure-evidence.sh" "$temp_root" "$repository_root")"
cp "$fixture/archive.sha256" "$fixture/original.sha256"
bash "$ROOT/scripts/ci/verify-file-checksum.sh" --print sha256 "$contradicted" > "$fixture/archive.sha256"
if bash "$ROOT/scripts/ci/verify-failure-evidence.sh" "$fixture" "$contradicted" > "$fixture/contradicted-status.log" 2>&1; then
    echo 'expected original failure-status refusal' >&2; exit 1
fi
mv "$fixture/original.sha256" "$fixture/archive.sha256"
mv "$fixture/original-validation.log" "$temp_root/validation.log"
if bash "$ROOT/scripts/ci/collect-failure-evidence.sh" "$temp_root" "$repository_root" invalid > "$fixture/invalid-mode.log" 2>&1; then
    echo 'expected invalid selection refusal' >&2; exit 1
fi

# An early failure with no selected inputs produces no artifact, not an empty tar.
mkdir "$fixture/empty-temp" "$fixture/empty-repository"
[[ -z "$(bash "$ROOT/scripts/ci/collect-failure-evidence.sh" "$fixture/empty-temp" "$fixture/empty-repository")" ]] || exit 1
empty_entries=("$fixture/empty-temp"/*)
[[ ${#empty_entries[@]} == 0 ]] || exit 1

# Rejected store admission must not discard the independent failure logs. The
# selector still owns which tool references may be retained; never follow an
# unknown link while recovering product evidence.
for selection_mode in full compact; do
    for rejected_store in relative redirected; do
        rejected="$fixture/rejected-$selection_mode-$rejected_store"
        mkdir -p "$rejected/temp" "$rejected/repository/.tools/host-set.retained" \
            "$rejected/repository/.tools/rust/build" "$rejected/private" "$rejected/unpacked"
        cp "$temp_root/validation.log" "$rejected/temp/validation.log"
        printf 'local installer diagnostics\n' > "$rejected/repository/.tools/host-set.retained/install.log"
        printf 'local Cargo failure\n' > "$rejected/repository/.tools/rust/build/install.log"
        printf 'must not be collected\n' > "$rejected/private/secret"
        ln -s "$rejected/private" "$rejected/repository/.tools/ic-set.unknown"
        rejected_path=relative
        if [[ "$rejected_store" == redirected ]]; then
            ln -s private "$rejected/store"
            rejected_path="$rejected/store"
        fi
        rejected_archive="$(SHARED_TOOLING_STORE="$rejected_path" GITHUB_OUTPUT="$rejected/outputs" \
            bash "$ROOT/scripts/ci/collect-failure-evidence.sh" "$rejected/temp" \
            "$rejected/repository" "$selection_mode" 2> "$rejected/error")"
        [[ -f "$rejected_archive" && -s "$rejected/error" && \
            "$(cat "$rejected/outputs")" == collection-complete=false ]] || exit 1
        tar -xzf "$rejected_archive" -C "$rejected/unpacked"
        cmp "$temp_root/validation.log" "$rejected/unpacked/validation.log"
        statuses=("$rejected/unpacked"/tool-evidence.*/selection-status)
        [[ ${#statuses[@]} == 1 && "$(cat "${statuses[0]}")" == 1 ]] || exit 1
        cmp "$rejected/temp/"tool-evidence.*/selection.log "${statuses[0]%/*}/selection.log"
        [[ -s "${statuses[0]%/*}/selection.log" ]] || exit 1
        cmp "$rejected/repository/.tools/host-set.retained/install.log" "$rejected/unpacked/.tools/host-set.retained/install.log"
        cmp "$rejected/repository/.tools/rust/build/install.log" "$rejected/unpacked/.tools/rust/build/install.log"
        [[ -L "$rejected/unpacked/.tools/ic-set.unknown" && \
            "$(readlink "$rejected/unpacked/.tools/ic-set.unknown")" == "$rejected/private" ]] || exit 1
        tar -tzf "$rejected_archive" > "$rejected/members"
        if grep -q '/secret$' "$rejected/members"; then echo 'unadmitted store payload collected' >&2; exit 1; fi
    done
done

# A redirected Rust leaf is retained without its outside build payload.
for selection_mode in full compact; do
    redirected="$fixture/redirected-rust-$selection_mode"
    mkdir -p "$redirected/temp" "$redirected/repository/.tools" "$redirected/private" "$redirected/unpacked"
    cp "$temp_root/validation.log" "$redirected/temp/validation.log"
    printf 'outside Rust payload\n' > "$redirected/private/secret"
    ln -s "$redirected/private" "$redirected/repository/.tools/rust"
    redirected_archive="$(GITHUB_OUTPUT="$redirected/outputs" bash "$ROOT/scripts/ci/collect-failure-evidence.sh" \
        "$redirected/temp" "$redirected/repository" "$selection_mode" 2> "$redirected/error")"
    [[ "$(cat "$redirected/outputs")" == collection-complete=false && -s "$redirected/error" ]] || exit 1
    tar -xzf "$redirected_archive" -C "$redirected/unpacked"
    cmp "$temp_root/validation.log" "$redirected/unpacked/validation.log"
    [[ -L "$redirected/unpacked/.tools/rust" && \
        "$(readlink "$redirected/unpacked/.tools/rust")" == "$redirected/private" ]] || exit 1
    tar -tzf "$redirected_archive" > "$redirected/members"
    if grep -q '/secret$' "$redirected/members"; then echo 'redirected Rust payload collected' >&2; exit 1; fi
done

# Redirected checkout roots retain their final links and independent evidence,
# never outside payloads. Physical evidence from the other root stays selected.
for selection_mode in full compact; do
    for redirected_root in target .tools both; do
        redirected="$fixture/redirected-$redirected_root-$selection_mode"
        mkdir -p "$redirected/temp" "$redirected/repository" "$redirected/unpacked"
        cp "$temp_root/validation.log" "$redirected/temp/validation.log"
        for path in rust ic-testkit-server host-set.retained qualification release-validation; do
            mkdir -p "$redirected/outside/$path"
            printf 'outside private payload\n' > "$redirected/outside/$path/private"
        done
        aliases=("$redirected_root")
        [[ "$redirected_root" != both ]] || aliases=(target .tools)
        for alias in "${aliases[@]}"; do
            ln -s "$redirected/outside" "$redirected/repository/$alias"
        done
        physical=()
        case "$redirected_root" in
            target) physical=(.tools/rust .tools/ic-testkit-server .tools/host-set.retained) ;;
            .tools) physical=(target/qualification target/release-validation) ;;
        esac
        for path in ${physical[@]+"${physical[@]}"}; do
            mkdir -p "$redirected/repository/$path"
            printf 'independent physical evidence\n' > "$redirected/repository/$path/local"
        done
        redirected_archive="$(GITHUB_OUTPUT="$redirected/outputs" \
            bash "$ROOT/scripts/ci/collect-failure-evidence.sh" "$redirected/temp" \
            "$redirected/repository" "$selection_mode" 2> "$redirected/error")"
        [[ -f "$redirected_archive" && -s "$redirected/error" && \
            "$(cat "$redirected/outputs")" == collection-complete=false ]] || exit 1
        tar -xzf "$redirected_archive" -C "$redirected/unpacked"
        cmp "$temp_root/validation.log" "$redirected/unpacked/validation.log"
        for path in ${physical[@]+"${physical[@]}"}; do
            cmp "$redirected/repository/$path/local" "$redirected/unpacked/$path/local"
        done
        # Listing archive members avoids traversing the preserved absolute links.
        tar -tzf "$redirected_archive" > "$redirected/members"
        for alias in "${aliases[@]}"; do
            [[ -L "$redirected/unpacked/$alias" && \
                "$(readlink "$redirected/unpacked/$alias")" == "$redirected/outside" ]] || exit 1
            while IFS= read -r member; do
                [[ "$member" != "./$alias/"* ]] || { echo 'redirected payload must not be archived' >&2; exit 1; }
            done < "$redirected/members"
        done
        diagnostics=("$redirected/unpacked"/tool-evidence.*/local-selection.log)
        [[ ${#diagnostics[@]} == 1 && -s "${diagnostics[0]}" ]] || exit 1
        cmp "$redirected/error" "${diagnostics[0]}"
    done
done

# A selector may fail after emitting admitted pairs. Retain those complete
# selections, its exact status/stderr and product logs; ignore a trailing root
# without a path. Substitute only selection, keeping the real collector/archive.
for selection_mode in full compact; do
    partial_root="$fixture/partial-selector-$selection_mode"
    mkdir -p "$partial_root/scripts/ci" "$partial_root/temp" "$partial_root/repository/.tools/host-set.selected" \
        "$partial_root/unpacked"
    cp "$ROOT/scripts/ci/collect-failure-evidence.sh" "$ROOT/scripts/ci/archive-evidence.sh" "$partial_root/scripts/ci/"
    cat > "$partial_root/scripts/ci/select-tool-evidence.sh" <<'SELECTOR'
#!/usr/bin/env bash
if [[ "$1" == compact ]]; then
    snapshot_root="${0%/scripts/ci/select-tool-evidence.sh}"
    [[ "$4" == "$snapshot_root/ci/tool-versions.env" &&
        "$5" == "$snapshot_root/ci/ic-tools.tsv" ]] || exit 18
fi
printf '%s\0%s\0%s\0' "$2" '.tools/host-set.selected' "$2"
printf 'controlled partial selector failure\n' >&2
exit 17
SELECTOR
    cp "$temp_root/validation.log" "$partial_root/temp/validation.log"
    printf 'admitted tool diagnostics\n' > "$partial_root/repository/.tools/host-set.selected/install.log"
    partial_archive="$(GITHUB_OUTPUT="$partial_root/outputs" bash "$partial_root/scripts/ci/collect-failure-evidence.sh" \
        "$partial_root/temp" "$partial_root/repository" "$selection_mode" 2> "$partial_root/error")"
    [[ -f "$partial_archive" && "$(cat "$partial_root/outputs")" == collection-complete=false ]] || exit 1
    tar -xzf "$partial_archive" -C "$partial_root/unpacked"
    cmp "$temp_root/validation.log" "$partial_root/unpacked/validation.log"
    cmp "$partial_root/repository/.tools/host-set.selected/install.log" "$partial_root/unpacked/.tools/host-set.selected/install.log"
    statuses=("$partial_root/unpacked"/tool-evidence.*/selection-status)
    [[ ${#statuses[@]} == 1 && "$(cat "${statuses[0]}")" == 17 && \
        "$(cat "${statuses[0]%/*}/selection.log")" == 'controlled partial selector failure' ]] || exit 1
done

# Caller selection must not trim a newline from a root before shared admission.
mkdir "$fixture/selection" "$fixture/selection"$'\n' "$fixture/selection-repo" "$fixture/selection-unpacked"
printf 'selected root\n' > "$fixture/selection"$'\n/validation.log'
printf 'wrong root\n' > "$fixture/selection/validation.log"
selected_archive="$(bash "$ROOT/scripts/ci/collect-failure-evidence.sh" "$fixture/selection"$'\n' "$fixture/selection-repo")"
tar -xzf "$selected_archive" -C "$fixture/selection-unpacked"
cmp "$fixture/selection"$'\n/validation.log' "$fixture/selection-unpacked/validation.log"

# A failed archiver reports failure and retains both its partial output and inputs.
mkdir "$fixture/bin"
printf '#!%s\nprintf "partial archive"\nexit 23\n' "$BASH" > "$fixture/bin/tar"
chmod +x "$fixture/bin/tar"
if PATH="$fixture/bin:$PATH" GITHUB_OUTPUT="$fixture/failed-fields" bash "$ROOT/scripts/ci/collect-failure-evidence.sh" \
    "$temp_root" "$repository_root" > "$fixture/failed-output" 2> "$fixture/failed-error"; then
    echo 'expected archive failure' >&2; exit 1
fi
[[ ! -s "$fixture/failed-output" && -s "$fixture/failed-error" && ! -s "$fixture/failed-fields" ]] || exit 1
partial=0
printf 'partial archive' > "$fixture/partial-content"
for path in "$temp_root"/ic-memory-evidence.*/evidence.tar.gz; do
    if cmp -s "$path" "$fixture/partial-content"; then partial=$((partial + 1)); fi
done
[[ "$partial" == 1 && "$(cat "$temp_root/validation.log")" == original_status=1 ]] || exit 1
# The hosted verifier must reject wrong/corrupt payloads before extracting them.
printf 'wrong archive\n' > "$fixture/corrupt.tar.gz"
unpacked_before=("$fixture"/unpacked.*)
if bash "$ROOT/scripts/ci/verify-failure-evidence.sh" "$fixture" "$fixture/corrupt.tar.gz" > "$fixture/corrupt.log" 2>&1; then
    echo 'expected checksum refusal' >&2; exit 1
fi
unpacked_after=("$fixture"/unpacked.*)
(( ${#unpacked_before[@]} == ${#unpacked_after[@]} )) || exit 1
if bash "$ROOT/scripts/ci/test-failure-evidence.sh" "$fixture" > "$fixture/occupied.log" 2>&1; then
    echo 'expected occupied fixture refusal' >&2; exit 1
fi
[[ "$(cat "$temp_root/validation.log")" == original_status=1 ]] || exit 1
if [[ "$retained" == true && -n "${GITHUB_OUTPUT:-}" ]]; then
    printf 'path=%s\n' "$archive" >> "$GITHUB_OUTPUT"
fi
echo 'Consumer evidence selections, retained host failure, archive round trip and retry/failure preservation passed'
fixture_complete=true
