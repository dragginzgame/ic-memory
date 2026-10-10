#!/usr/bin/env bash
# Shared companions: make/tools.mk
set -euo pipefail

ROOT="$0"
[[ "$ROOT" == /* ]] || ROOT="$PWD/$ROOT"
ROOT="$(cd -P "${ROOT%/*}/../.." && printf '%s/.' "$PWD")"
ROOT="${ROOT%/.}"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/shared-tool-commands.XXXXXX")"
fixture="$(cd "$fixture" && pwd -P)"
trap 'if [[ $? == 0 ]]; then rm -rf "$fixture"; else printf "Failed tool command fixture retained: %s\n" "$fixture" >&2; fi' EXIT
unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS MAKEFILES
consumer="$fixture/consumer with spaces"
snapshot="$consumer/vendor/shared-tooling"
mkdir -p "$snapshot/make" "$snapshot/scripts/dev" "$consumer/.tools/host/bin" "$consumer/.tools/ic/bin"
cp "$ROOT/make/tools.mk" "$snapshot/make/"
export TOOL_COMMAND_LOG="$fixture/commands"
export TOOL_COMMAND_CONSUMER="$consumer"
cat > "$fixture/tool-stub" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
name="${0##*/}"
printf '%s' "$name" >> "$TOOL_COMMAND_LOG"
printf ' <%s>' "$@" >> "$TOOL_COMMAND_LOG"
printf '\n' >> "$TOOL_COMMAND_LOG"
[[ "$PATH" == "$TOOL_COMMAND_CONSUMER/.tools/host/bin:$TOOL_COMMAND_CONSUMER/.tools/ic/bin:"* ]]
if [[ "${!#}" == --preflight ]]; then
    [[ "${TOOL_COMMAND_PREFLIGHT_FAIL:-}" != "$name" ]] || exit 24
    exit 0
fi
if [[ "${TOOL_COMMAND_FAIL:-}" == "$name" ]]; then exit 23; fi
SCRIPT
for tool in install-host-tools.sh install-ic-tools.sh install-rust-tools.sh cloc.sh; do
    cp "$fixture/tool-stub" "$snapshot/scripts/dev/$tool"
done
cat > "$consumer/Makefile" <<'MAKE'
SHARED_TOOLING_ROOT := $(CURDIR)/vendor/shared-tooling
include vendor/shared-tooling/make/tools.mk
.PHONY: help
help:
	@echo consumer help
MAKE
# Including common commands must not make bare `make` install anything.
[[ "$(make --no-print-directory -C "$consumer")" == 'consumer help' ]]
[[ ! -e "$TOOL_COMMAND_LOG" ]]

make --no-print-directory -j4 -C "$consumer" install-tools > "$fixture/install.log" 2>&1
cat > "$fixture/expected" <<EOF
install-ic-tools.sh <--consumer> <$consumer> <--pins> <$consumer/ci/ic-tools.tsv> <--preflight>
install-rust-tools.sh <--consumer> <$consumer> <--versions> <$consumer/ci/tool-versions.env> <--preflight>
install-host-tools.sh <--consumer> <$consumer> <--versions> <$consumer/ci/tool-versions.env>
install-ic-tools.sh <--consumer> <$consumer> <--pins> <$consumer/ci/ic-tools.tsv>
install-rust-tools.sh <--consumer> <$consumer> <--versions> <$consumer/ci/tool-versions.env>
EOF
cmp "$fixture/expected" "$TOOL_COMMAND_LOG"

: > "$TOOL_COMMAND_LOG"
make --no-print-directory -j4 -C "$consumer" tools-check \
    HOST_TOOL_VERSIONS="$consumer/reviewed host pins.env" IC_TOOL_PINS="$consumer/reviewed IC pins.tsv" \
    > "$fixture/check.log" 2>&1
cat > "$fixture/expected" <<EOF
install-host-tools.sh <--consumer> <$consumer> <--versions> <$consumer/reviewed host pins.env> <--check>
install-ic-tools.sh <--consumer> <$consumer> <--pins> <$consumer/reviewed IC pins.tsv> <--check>
install-rust-tools.sh <--consumer> <$consumer> <--versions> <$consumer/reviewed host pins.env> <--check>
EOF
cmp "$fixture/expected" "$TOOL_COMMAND_LOG"

# Every common failure stops the ordered aggregate, including under parallel Make.
for target in install-tools tools-check; do
  count=0
  [[ "$target" != install-tools ]] || count=2
  for failed in install-host-tools.sh install-ic-tools.sh install-rust-tools.sh; do
    count=$((count + 1))
    : > "$TOOL_COMMAND_LOG"
    if TOOL_COMMAND_FAIL="$failed" make --no-print-directory -j4 -C "$consumer" "$target" \
        > "$fixture/$target-failed.log" 2>&1; then
        echo "accepted failed common step: $target ($failed)" >&2; exit 1
    fi
    [[ "$(wc -l < "$TOOL_COMMAND_LOG" | tr -d ' ')" == "$count" ]]
  done
done

: > "$TOOL_COMMAND_LOG"
make --no-print-directory -C "$consumer" cloc > "$fixture/cloc.log" 2>&1
printf 'cloc.sh <%s>\n' "$consumer" > "$fixture/expected"
cmp "$fixture/expected" "$TOOL_COMMAND_LOG"
if TOOL_COMMAND_FAIL=cloc.sh make --no-print-directory -C "$consumer" cloc \
    > "$fixture/cloc-failed.log" 2>&1; then
    echo 'accepted failed LOC report' >&2; exit 1
fi

# A caller can select the sibling reporter without copying the common recipe.
cp "$fixture/tool-stub" "$snapshot/scripts/dev/cloc-siblings.sh"
: > "$TOOL_COMMAND_LOG"
make --no-print-directory -C "$consumer" cloc \
    CLOC_REPORT="$snapshot/scripts/dev/cloc-siblings.sh" CLOC_ROOT="$fixture" \
    > "$fixture/siblings.log" 2>&1
printf 'cloc-siblings.sh <%s>\n' "$fixture" > "$fixture/expected"
cmp "$fixture/expected" "$TOOL_COMMAND_LOG"

: > "$TOOL_COMMAND_LOG"
make --no-print-directory -C "$consumer" cloc CLOC_MANIFEST=testing/Cargo.toml \
    > "$fixture/manifest.log" 2>&1
printf 'cloc.sh <--manifest> <testing/Cargo.toml> <%s>\n' "$consumer" > "$fixture/expected"
cmp "$fixture/expected" "$TOOL_COMMAND_LOG"

# Ordinary setup/check/LOC commands above work without any fleet reporter.
# An omitted optional report must explain its owner, without invoking a sibling.
: > "$TOOL_COMMAND_LOG"
if make --no-print-directory -C "$consumer" cloc-tooling > "$fixture/no-fleet.log" 2>&1; then
    echo 'accepted unselected fleet report' >&2; exit 1
fi
grep -F 'Fleet tooling reports are optional: run make cloc-tooling in Shared Tooling' "$fixture/no-fleet.log"
[[ ! -s "$TOOL_COMMAND_LOG" ]]

# Explicitly selected fleet tooling remains available through the same include.
cat > "$snapshot/scripts/dev/cloc-tooling.pl" <<'PERL'
use strict;
use warnings;
open my $log, '>>', $ENV{TOOL_COMMAND_LOG} or die $!;
print {$log} "cloc-tooling.pl <$ARGV[0]>\n";
PERL
: > "$TOOL_COMMAND_LOG"
make --no-print-directory -C "$consumer" cloc-tooling CLOC_PARENT="$fixture" > "$fixture/tooling.log" 2>&1
printf 'cloc-tooling.pl <%s>\n' "$fixture" > "$fixture/expected"
cmp "$fixture/expected" "$TOOL_COMMAND_LOG"

# Product tools extend the ordered recipe rather than racing as prerequisites.
cat >> "$consumer/Makefile" <<'MAKE'
LOCAL_TOOL_INSTALL_TARGETS += install-product install-second
LOCAL_TOOL_CHECK_TARGETS += check-product check-second
.PHONY: install-product install-second check-product check-second
install-product install-second check-product check-second:
	@printf '%s\n' '$@' >> "$(TOOL_COMMAND_LOG)"
	@test "$(TOOL_COMMAND_FAIL)" != '$@'
MAKE
: > "$TOOL_COMMAND_LOG"
make --no-print-directory -j4 -C "$consumer" install-tools > "$fixture/rust-install.log" 2>&1
[[ "$(sed -n '5p' "$TOOL_COMMAND_LOG")" == "install-rust-tools.sh <--consumer> <$consumer> <--versions> <$consumer/ci/tool-versions.env>" ]]
printf 'install-product\ninstall-second\n' > "$fixture/product-expected"
tail -2 "$TOOL_COMMAND_LOG" > "$fixture/product-actual"
cmp "$fixture/product-expected" "$fixture/product-actual"
: > "$TOOL_COMMAND_LOG"
make --no-print-directory -j4 -C "$consumer" tools-check RUST_TOOL_VERSIONS="$consumer/rust pins.env" > "$fixture/rust-check.log" 2>&1
[[ "$(sed -n '3p' "$TOOL_COMMAND_LOG")" == "install-rust-tools.sh <--consumer> <$consumer> <--versions> <$consumer/rust pins.env> <--check>" ]]
printf 'check-product\ncheck-second\n' > "$fixture/product-expected"
tail -2 "$TOOL_COMMAND_LOG" > "$fixture/product-actual"
cmp "$fixture/product-expected" "$fixture/product-actual"
for target in install-tools tools-check; do
    phase=install; [[ "$target" != tools-check ]] || phase=check
    for failed in install-rust-tools.sh "$phase-product" "$phase-second"; do
        : > "$TOOL_COMMAND_LOG"
        if TOOL_COMMAND_FAIL="$failed" make --no-print-directory -j4 -C "$consumer" "$target" \
            > "$fixture/product-failure.log" 2>&1; then exit 1; fi
        case "$failed" in
            install-rust-tools.sh) expected_count=3 ;;
            *-product) expected_count=4 ;;
            *-second) expected_count=5 ;;
        esac
        [[ "$target" != install-tools ]] || expected_count=$((expected_count + 2))
        [[ "$(wc -l < "$TOOL_COMMAND_LOG" | tr -d ' ')" == "$expected_count" ]]
    done
done
# Neither preflight failure may reach any installation or product extension.
for failed in install-ic-tools.sh install-rust-tools.sh; do
    : > "$TOOL_COMMAND_LOG"
    if TOOL_COMMAND_PREFLIGHT_FAIL="$failed" make --no-print-directory -j4 -C "$consumer" install-tools \
        > "$fixture/preflight-failure.log" 2>&1; then exit 1; fi
    if grep -v '<--preflight>$' "$TOOL_COMMAND_LOG"; then exit 1; fi
done
# Keep later default-goal comparison on its current command record.
cp "$TOOL_COMMAND_LOG" "$fixture/expected"

# An explicitly selected default goal remains unchanged too.
printf '.DEFAULT_GOAL := help\n' > "$fixture/prefixed.mk"
cat "$consumer/Makefile" >> "$fixture/prefixed.mk"
cp "$fixture/prefixed.mk" "$consumer/Makefile"
[[ "$(make --no-print-directory -C "$consumer")" == 'consumer help' ]]
cmp "$fixture/expected" "$TOOL_COMMAND_LOG"
# Exercise the actual preflight owners through the actual parallel aggregate.
# Host setup remains a recording stub: refusal must happen before it is called.
mkdir -p "$snapshot/scripts/ci" "$consumer/ci" "$fixture/bin"
cp "$ROOT/scripts/dev/install-ic-tools.sh" "$ROOT/scripts/dev/install-rust-tools.sh" "$snapshot/scripts/dev/"
cp "$ROOT/scripts/ci/ic-tool-pins.awk" "$ROOT/scripts/ci/verify-file-checksum.sh" "$snapshot/scripts/ci/"
cp "$ROOT/ci/ic-tools.tsv" "$ROOT/ci/tool-versions.env" "$consumer/ci/"
cat > "$fixture/bin/uname" <<'SCRIPT'
#!/usr/bin/env bash
case "$1" in -s) echo Linux ;; -m) echo "${TOOL_PREFLIGHT_ARCH:-x86_64}" ;; esac
SCRIPT
for tool in cargo rustc; do
    cat > "$fixture/bin/$tool" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
[[ "$PWD" == "$TOOL_COMMAND_CONSUMER" && "$RUSTUP_AUTO_INSTALL" == 0 && "$1" == --version ]]
printf '%s\n' "${0##*/}" >> "$TOOL_PREFLIGHT_PROBES"
if [[ "${TOOL_PREFLIGHT_BROKEN:-}" == "${0##*/}" ]]; then echo 'selected toolchain unavailable' >&2; exit 29; fi
printf '%s 1.0.0\n' "${0##*/}"
SCRIPT
    chmod +x "$fixture/bin/$tool"
done
chmod +x "$fixture/bin/uname"
export TOOL_PREFLIGHT_PROBES="$fixture/preflight-probes"
# Keep only these prerequisites in PATH to model genuinely absent Rust commands.
for tool in bash make awk sort; do ln -s "$(command -v "$tool")" "$fixture/bin/$tool"; done
real_make="$(command -v make)"
for failure in platform rustc cargo missing-rustc missing-cargo; do
    : > "$TOOL_COMMAND_LOG"
    : > "$TOOL_PREFLIGHT_PROBES"
    arch=x86_64; broken=''
    case "$failure" in
        platform) arch=aarch64 ;;
        rustc|cargo) broken="$failure" ;;
        missing-*) mv "$fixture/bin/${failure#missing-}" "$fixture/saved-tool" ;;
    esac
    if PATH="$fixture/bin" TOOL_PREFLIGHT_ARCH="$arch" TOOL_PREFLIGHT_BROKEN="$broken" \
        "$real_make" --no-print-directory -j4 -C "$consumer" install-tools \
        > "$fixture/preflight-$failure.log" 2>&1; then exit 1; fi
    [[ ! -s "$TOOL_COMMAND_LOG" && ! -e "$consumer/.tools/rust" ]]
    case "$failure" in
        platform) [[ ! -s "$TOOL_PREFLIGHT_PROBES" ]]; grep -F 'Linux:aarch64' "$fixture/preflight-$failure.log" ;;
        *) grep -F "tool=${failure#missing-}" "$fixture/preflight-$failure.log"
           grep -Ei 'prepare' "$fixture/preflight-$failure.log" >/dev/null ;;
    esac
    case "$failure" in missing-*) mv "$fixture/saved-tool" "$fixture/bin/${failure#missing-}" ;; esac
done
# A successful cold check is silent and cannot create .tools or build artifacts.
cold="$fixture/cold consumer"; mkdir "$cold"
for mode in ic rust; do
    args=(--consumer "$cold" --preflight)
    if [[ "$mode" == ic ]]; then args+=(--pins "$consumer/ci/ic-tools.tsv")
    else args+=(--versions "$consumer/ci/tool-versions.env"); fi
    PATH="$fixture/bin" TOOL_COMMAND_CONSUMER="$cold" CARGO_NET_OFFLINE=true \
        bash "$snapshot/scripts/dev/install-$mode-tools.sh" "${args[@]}" > "$fixture/preflight-$mode.out"
    [[ ! -s "$fixture/preflight-$mode.out" && ! -e "$cold/.tools" ]]
done
# Repeat preflight over retained Rust state without probing or replacing its tools.
mkdir -p "$cold/.tools/rust/build" "$cold/.tools/rust/bin"
printf 'retained failure evidence\n' > "$cold/.tools/rust/build/evidence"
printf 'retained receipt\n' > "$cold/.tools/rust/.crates2.json"
cp -R "$cold/.tools" "$fixture/retained-tools"
PATH="$fixture/bin" TOOL_COMMAND_CONSUMER="$cold" bash "$snapshot/scripts/dev/install-rust-tools.sh" \
    --consumer "$cold" --versions "$consumer/ci/tool-versions.env" --preflight
diff -r "$fixture/retained-tools" "$cold/.tools"
# Admission modes are distinct; combining them never falls through to setup.
for mode in ic rust; do
    args=(--consumer "$cold" --check --preflight)
    status=0
    bash "$snapshot/scripts/dev/install-$mode-tools.sh" "${args[@]}" > "$fixture/conflicting-modes.log" 2>&1 || status=$?
    [[ "$status" == 2 ]]
done
echo 'Shared tool Make commands passed (substitute installers and reports)'
