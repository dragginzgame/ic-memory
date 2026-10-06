#!/usr/bin/env bash
set -euo pipefail
# This independent fixture owns its checkout, formatter and Make selections.
unset MAKEFLAGS MFLAGS MAKEOVERRIDES VALIDATION_REPOSITORY_ROOT VALIDATION_RUNNER_SNAPSHOT_PATH

# Consumer-owned integration of the vendored hook with this repository's Make
# targets. Reuse an existing commit read-only; never create commits or tags.
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
# Keep inherited hook/index variables from redirecting fixture Git operations.
while IFS= read -r variable; do unset "$variable"; done < <(git rev-parse --local-env-vars)
fixture="$(mktemp -d "${TMPDIR:-/tmp}/ic-memory-hooks.XXXXXX")"
cleanup_fixture() {
    local fixture_status="$1" fixture_command="$2"
    if [[ "$fixture_status" == 0 ]]; then
        rm -rf -- "$fixture"
    else
        printf 'Hook fixture failed in %s: %s\nRetained fixture: %s\n' "$PWD" "$fixture_command" "$fixture" >&2
        if [[ -f output ]]; then cat output >&2; fi
    fi
    return "$fixture_status"
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
    mkdir -p ci .githooks scripts/dev testing/runtime-qualification/src
    for path in Makefile ci/tool-versions.env rust-toolchain.toml .githooks/pre-commit scripts/dev/install-git-hooks.sh; do
        cp -p "$root/$path" "$path"
    done
    cat > Cargo.toml <<'CARGO'
[workspace]
exclude = ["testing/runtime-qualification"]
[package]
name = "hook-fixture"
version = "0.0.0"
edition = "2024"
autoexamples = false
autotests = false
autobenches = false
autobins = false
CARGO
    cat > testing/runtime-qualification/Cargo.toml <<'CARGO'
[workspace]
[package]
name = "hook-host-fixture"
version = "0.0.0"
edition = "2024"
CARGO
    printf 'pub fn fixture( ){}\n' > src/lib.rs
    printf 'pub fn fixture( ){}\n' > testing/runtime-qualification/src/lib.rs
    for workspace in . testing/runtime-qualification; do
        cat >> "$workspace/Cargo.toml" <<'CARGO'

[workspace.dependencies]
# Keep this comment and the selected path/feature policy.
zeta = { path = "zeta", default-features = false }
alpha = { path = "alpha" }

[dependencies]
zeta.workspace = true
alpha.workspace = true
CARGO
        for member in alpha zeta; do
            mkdir -p "$workspace/$member/src"
            printf '[package]\nname = "%s"\nversion = "0.0.0"\nedition = "2024"\n' "$member" > "$workspace/$member/Cargo.toml"
            printf 'pub fn fixture() {}\n' > "$workspace/$member/src/lib.rs"
        done
    done
    git add -- Makefile ci/tool-versions.env rust-toolchain.toml .githooks/pre-commit scripts/dev/install-git-hooks.sh Cargo.toml src/lib.rs alpha zeta testing/runtime-qualification/Cargo.toml testing/runtime-qualification/src/lib.rs testing/runtime-qualification/alpha testing/runtime-qualification/zeta
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
[[ "$(git write-tree)" == "$tree" && "$(cat src/lib.rs)" == 'pub fn fixture( ){}' ]]
CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 bash .githooks/pre-commit > output
[[ "$(git show :src/lib.rs)" == 'pub fn fixture() {}' ]]
[[ "$(git show :testing/runtime-qualification/src/lib.rs)" == 'pub fn fixture() {}' ]]
cmp unrelated-before README.md
[[ "$(git show :README.md)" == "$(git show HEAD:README.md)" ]]
[[ "$(cat unrelated.rs)" == 'untracked edit' && -z "$(git ls-files -- unrelated.rs)" ]]
make --no-print-directory fmt-check > output
for workspace in . testing/runtime-qualification; do
    grep -qF '# Keep this comment and the selected path/feature policy.' "$workspace/Cargo.toml"
done
[[ ! -e Cargo.lock && ! -e testing/runtime-qualification/Cargo.lock && ! -e target && ! -e testing/runtime-qualification/target ]]
tree="$(git write-tree)"
bash .githooks/pre-commit > output
[[ "$(git write-tree)" == "$tree" ]]

for path in src/lib.rs Cargo.toml Makefile ci/tool-versions.env; do
    new_fixture "partial-$(basename "$path")"
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
    [[ "$(git write-tree)" == "$tree" && "$(cat testing/runtime-qualification/src/lib.rs)" == 'pub fn fixture( ){}' ]]
    cmp before "$path"
done

new_fixture failed-formatter
printf '.PHONY: fmt\nfmt:\n\t@false\n' > Makefile
git add -- Makefile
tree="$(git write-tree)"
cp src/lib.rs before
expect_failure bash .githooks/pre-commit
[[ "$(git write-tree)" == "$tree" ]]
cmp before src/lib.rs

new_fixture installation
bash scripts/dev/install-git-hooks.sh > output
[[ "$(git config --local --get core.hooksPath)" == .githooks ]]
bash scripts/dev/install-git-hooks.sh > output
git config --local core.hooksPath private-hooks
expect_failure bash scripts/dev/install-git-hooks.sh
[[ "$(git config --local --get core.hooksPath)" == private-hooks ]]

# macOS temporary roots can have logical aliases (/var and /private/var).
# Exercise the public setup target through an alias without replacing hooks.
new_fixture installation-path-alias
ln -s "$PWD" "$fixture/installer-alias"
(
    cd "$fixture/installer-alias"
    make --no-print-directory install-hooks > output
)
[[ "$(git config --local --get core.hooksPath)" == .githooks ]]

echo 'Consumer hook selection, partial staging, failure isolation, setup and both-workspace formatting passed'
