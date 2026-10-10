#!/usr/bin/env bash
set -euo pipefail
# This independent fixture owns its checkout, formatter and Make selections.
unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS MAKEFILES
unset VALIDATION_REPOSITORY_ROOT VALIDATION_RUNNER_SNAPSHOT_PATH

# Consumer-owned integration of the vendored hook with this repository's Make
# targets. Reuse an existing commit read-only; never create commits or tags.
root="${BASH_SOURCE[0]}"
[[ "$root" == /* ]] || root="$PWD/$root"
root="$(cd -P "${root%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
# Keep inherited hook/index variables from redirecting fixture Git operations.
while IFS= read -r variable; do unset "$variable"; done < <(git rev-parse --local-env-vars)
fixture="$(mktemp -d "${TMPDIR:-/tmp}/ic-memory-hooks.XXXXXX")"
fixture_complete=false
cleanup_fixture() {
    local fixture_status="$1" fixture_command="$2"
    [[ "$fixture_complete" == true || "$fixture_status" != 0 ]] || fixture_status=1
    if [[ "$fixture_status" == 0 ]]; then
        rm -rf -- "$fixture"
    else
        printf 'Hook fixture failed in %s: %s\nRetained fixture: %s\n' "$PWD" "$fixture_command" "$fixture" >&2
        if [[ -f output ]]; then cat output >&2; fi
    fi
    exit "$fixture_status"
}
trap 'cleanup_fixture "$?" "$BASH_COMMAND"' EXIT
source_commit="$(git -C "$root" rev-parse HEAD)"
source_objects="$(git -C "$root" rev-parse --git-path objects)"
case "$source_objects" in /*) ;; *) source_objects="$root/$source_objects" ;; esac
mkdir "$fixture/templates"
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_TEMPLATE_DIR="$fixture/templates"

new_fixture() {
    local path workspace member
    mkdir "$fixture/$1"
    cd "$fixture/$1"
    git init --quiet
    mkdir -p .git/objects/info
    printf '%s\n' "$source_objects" > .git/objects/info/alternates
    git update-ref HEAD "$source_commit"
    git read-tree HEAD
    git checkout-index --all
    # Synthetic manifests have their own dependency graph. Remove inherited
    # consumer locks so the formatter check still detects accidental creation.
    git rm --quiet --ignore-unmatch -- Cargo.lock testing/runtime-qualification/Cargo.lock
    mkdir -p ci make .githooks scripts/ci scripts/dev
    for path in Makefile make/tools.mk make/release.mk make/execution.mk ci/tool-versions.env rust-toolchain.toml .githooks/pre-commit scripts/dev/install-git-hooks.sh scripts/ci/check-format-tools.sh scripts/ci/run-formatting.sh scripts/ci/check-make-execution.sh; do
        cp -p "$root/$path" "$path"
    done
    for workspace in . testing/runtime-qualification; do
        mkdir -p "$workspace/crates/hook-fixture/src"
        cat > "$workspace/Cargo.toml" <<'CARGO'
[workspace]
resolver = "3"
members = ["crates/hook-fixture", "crates/alpha", "crates/zeta"]
exclude = ["testing/runtime-qualification"]

[workspace.package]
version = "0.0.0"
edition = "2024"

[workspace.dependencies]
# Keep this comment and the selected path/feature policy.
zeta = { path = "crates/zeta", default-features = false }
alpha = { path = "crates/alpha" }
CARGO
        cat > "$workspace/crates/hook-fixture/Cargo.toml" <<'CARGO'
[package]
name = "hook-fixture"
version.workspace = true
edition.workspace = true

[dependencies]
zeta.workspace = true
alpha.workspace = true
CARGO
        printf 'pub fn fixture( ){}\n' > "$workspace/crates/hook-fixture/src/lib.rs"
        for member in alpha zeta; do
            mkdir -p "$workspace/crates/$member/src"
            printf '[package]\nname = "%s"\nversion.workspace = true\nedition.workspace = true\n' "$member" > "$workspace/crates/$member/Cargo.toml"
            printf 'pub fn fixture() {}\n' > "$workspace/crates/$member/src/lib.rs"
        done
    done
    git add -- Makefile make/tools.mk make/release.mk make/execution.mk ci/tool-versions.env rust-toolchain.toml .githooks/pre-commit scripts/dev/install-git-hooks.sh scripts/ci/check-format-tools.sh scripts/ci/run-formatting.sh scripts/ci/check-make-execution.sh Cargo.toml crates/hook-fixture crates/alpha crates/zeta testing/runtime-qualification/Cargo.toml testing/runtime-qualification/crates/hook-fixture testing/runtime-qualification/crates/alpha testing/runtime-qualification/crates/zeta
}

expect_failure() {
    if "$@" > output 2>&1; then
        echo "hook fixture unexpectedly accepted a failing operation in $PWD: $*" >&2
        exit 1
    fi
}

new_fixture selected
printf 'unrelated working edit\n' >> README.md
cp README.md unrelated-before
printf 'untracked edit\n' > unrelated.rs
tree="$(git write-tree)"
expect_failure make --no-print-directory fmt-check
[[ "$(git write-tree)" == "$tree" && "$(cat crates/hook-fixture/src/lib.rs)" == 'pub fn fixture( ){}' ]] || exit 1
CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 bash .githooks/pre-commit > output
[[ "$(git show :crates/hook-fixture/src/lib.rs)" == 'pub fn fixture() {}' ]] || exit 1
[[ "$(git show :testing/runtime-qualification/crates/hook-fixture/src/lib.rs)" == 'pub fn fixture() {}' ]] || exit 1
cmp unrelated-before README.md
[[ "$(git show :README.md)" == "$(git show HEAD:README.md)" ]] || exit 1
[[ "$(cat unrelated.rs)" == 'untracked edit' && -z "$(git ls-files -- unrelated.rs)" ]] || exit 1
make --no-print-directory fmt-check > output
for workspace in . testing/runtime-qualification; do
    grep -qF '# Keep this comment and the selected path/feature policy.' "$workspace/Cargo.toml"
done
[[ ! -e Cargo.lock && ! -e testing/runtime-qualification/Cargo.lock && ! -e target && ! -e testing/runtime-qualification/target ]] || exit 1
tree="$(git write-tree)"
bash .githooks/pre-commit > output
[[ "$(git write-tree)" == "$tree" ]] || exit 1

for path in crates/hook-fixture/src/lib.rs Cargo.toml crates/hook-fixture/Cargo.toml Makefile ci/tool-versions.env; do
    new_fixture "partial-${path//\//-}"
    case "$path" in *.rs) comment='//' ;; *) comment='#' ;; esac
    # Every case must select this file even when consumer tooling matches HEAD.
    printf '\n%s staged fixture edit\n' "$comment" >> "$path"
    git add -- "$path"
    if git diff --cached --quiet -- "$path"; then
        echo "hook fixture did not select $path" >&2
        exit 1
    fi
    printf '\n%s unstaged fixture edit\n' "$comment" >> "$path"
    cp "$path" before
    tree="$(git write-tree)"
    expect_failure bash .githooks/pre-commit
    [[ "$(git write-tree)" == "$tree" && "$(cat testing/runtime-qualification/crates/hook-fixture/src/lib.rs)" == 'pub fn fixture( ){}' ]] || exit 1
    cmp before "$path"
done

new_fixture failed-formatter
printf '.PHONY: fmt\nfmt:\n\t@printf "changed\\n" > crates/hook-fixture/src/lib.rs\n\t@exit 23\n' > Makefile
git add -- Makefile
tree="$(git write-tree)"
cp crates/hook-fixture/src/lib.rs before
expect_failure bash .githooks/pre-commit
[[ "$(git write-tree)" == "$tree" ]] || exit 1
cmp before crates/hook-fixture/src/lib.rs

# Matching stdout does not admit a failed index observation. Fail each real Git
# write-tree call after printing its tree, without refreshing real fixture files.
real_git="$(command -v git)"
for observation in 1 2 3; do
    new_fixture "failed-tree-$observation"
    printf 'unrelated working edit\n' >> README.md
    cp README.md unrelated-before
    git write-tree > tree-before
    cp .git/index index-before
    cp crates/hook-fixture/src/lib.rs before
    mkdir mock-bin
    cat > mock-bin/git <<'GIT'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "$HOOK_TEST_STATE/commands"
if [[ "$*" == write-tree ]]; then
    count=0
    [[ ! -f "$HOOK_TEST_STATE/count" ]] || read -r count < "$HOOK_TEST_STATE/count"
    count=$((count + 1))
    printf '%s\n' "$count" > "$HOOK_TEST_STATE/count"
    "$HOOK_TEST_GIT" "$@"
    if [[ "$count" == "$HOOK_TEST_OBSERVATION" ]]; then
        echo 'injected write-tree observation failure' >&2
        exit 23
    fi
    exit 0
fi
exec "$HOOK_TEST_GIT" "$@"
GIT
    chmod +x mock-bin/git
    status=0
    PATH="$PWD/mock-bin:$PATH" HOOK_TEST_GIT="$real_git" \
        HOOK_TEST_STATE="$PWD/mock-bin" HOOK_TEST_OBSERVATION="$observation" \
        CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 \
        bash .githooks/pre-commit > output 2>&1 || status=$?
    [[ "$status" == 23 && "$(cat mock-bin/count)" == "$observation" ]] || exit 1
    [[ "$(tail -n 1 mock-bin/commands)" == write-tree ]] || exit 1
    cmp index-before .git/index
    cmp before crates/hook-fixture/src/lib.rs
    cmp unrelated-before README.md
done

for mode in i n q t v; do
    new_fixture "make-mode-$mode"
    printf '.PHONY: fmt\nfmt:\n\t@touch "%s/formatter-%s"\n\t@exit 23\n' "$fixture" "$mode" > Makefile
    git add -- Makefile
    tree="$(git write-tree)"
    cp crates/hook-fixture/src/lib.rs before
    expect_failure env MAKEFLAGS="$mode" bash .githooks/pre-commit
    [[ "$(git write-tree)" == "$tree" && ! -e "$fixture/formatter-$mode" ]] || exit 1
    cmp before crates/hook-fixture/src/lib.rs
done

new_fixture make-selection
# shellcheck disable=SC2016 # Make expands the selected variable in the export.
printf '.PHONY: fmt\nfmt:\n\t@printf "%%s\\n" "$(HOOK_SELECTION)" > "%s/formatter-selection"\n' "$fixture" > Makefile
git add -- Makefile
cat > parent.make <<'MAKE'
.PHONY: hook
hook:
	+@bash .githooks/pre-commit
MAKE
make --no-print-directory -j2 -f parent.make hook HOOK_SELECTION=kept > output
[[ "$(cat "$fixture/formatter-selection")" == kept ]] || exit 1

# Prepared checkout-local Rust tools must remain visible inside the isolated
# index export even when the caller has no Cargo tools on its shell PATH.
cargo_sort="$(command -v cargo-sort)"
rustup="$(command -v rustup)"
for admission in prepared missing wrong-version; do
    new_fixture "local-tools-$admission"
    mkdir -p .tools/rust/bin .tools/cargo-home
    for tool in cargo cargo-fmt rustc rustfmt rustup; do ln -s "$rustup" ".tools/rust/bin/$tool"; done
    case "$admission" in
        prepared) cp "$cargo_sort" .tools/rust/bin/cargo-sort ;;
        wrong-version)
            printf '#!/bin/sh\necho "cargo-sort 0.0.0"\n' > .tools/rust/bin/cargo-sort
            chmod +x .tools/rust/bin/cargo-sort ;;
    esac
    tree="$(git write-tree)"
    cp crates/hook-fixture/src/lib.rs before
    if [[ "$admission" == prepared ]]; then
        CARGO_HOME="$PWD/.tools/cargo-home" PATH=/usr/bin:/bin bash .githooks/pre-commit > output 2>&1
        [[ "$(git show :crates/hook-fixture/src/lib.rs)" == 'pub fn fixture() {}' ]] || exit 1
        [[ "$(git show :testing/runtime-qualification/crates/hook-fixture/src/lib.rs)" == 'pub fn fixture() {}' ]] || exit 1
    else
        expect_failure env CARGO_HOME="$PWD/.tools/cargo-home" PATH=/usr/bin:/bin bash .githooks/pre-commit
        [[ "$(git write-tree)" == "$tree" ]] || exit 1
        cmp before crates/hook-fixture/src/lib.rs
    fi
    [[ ! -e Cargo.lock && ! -e testing/runtime-qualification/Cargo.lock && ! -e target && ! -e testing/runtime-qualification/target ]] || exit 1
done

new_fixture installation
bash scripts/dev/install-git-hooks.sh > output
[[ "$(git config --local --get core.hooksPath)" == .githooks ]] || exit 1
bash scripts/dev/install-git-hooks.sh > output
git config --local core.hooksPath private-hooks
expect_failure bash scripts/dev/install-git-hooks.sh
[[ "$(git config --local --get core.hooksPath)" == private-hooks ]] || exit 1

# macOS temporary roots can have logical aliases (/var and /private/var).
# Exercise the public setup target through an alias without replacing hooks.
new_fixture installation-path-alias
ln -s "$PWD" "$fixture/installer-alias"
(
    cd "$fixture/installer-alias"
    make --no-print-directory install-hooks > output
)
[[ "$(git config --local --get core.hooksPath)" == .githooks ]] || exit 1

# Qualify the actual two-workspace formatter in a literal newline-ending root.
new_fixture $'installation-root\n'
bash scripts/dev/install-git-hooks.sh > output
bash scripts/dev/install-git-hooks.sh >> output
[[ "$(git config --local --get core.hooksPath)" == .githooks ]] || exit 1
CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 bash .githooks/pre-commit >> output
[[ "$(git show :crates/hook-fixture/src/lib.rs)" == 'pub fn fixture() {}' ]] || exit 1
[[ "$(git show :testing/runtime-qualification/crates/hook-fixture/src/lib.rs)" == 'pub fn fixture() {}' ]] || exit 1
[[ ! -e Cargo.lock && ! -e testing/runtime-qualification/Cargo.lock && ! -e target && ! -e testing/runtime-qualification/target ]] || exit 1

echo 'Consumer hook selection, partial staging, failure isolation, setup and both-workspace formatting passed'
fixture_complete=true
