#!/usr/bin/env bash
# Shared companions: scripts/ci/archive-evidence.sh
set -euo pipefail
ROOT="$0"
[[ "$ROOT" == /* ]] || ROOT="$PWD/$ROOT"
ROOT="$(cd -P "${ROOT%/*}/../.." && printf '%s/.' "$PWD")"
ROOT="${ROOT%/.}"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/evidence-archive.XXXXXX")"
trap 'if [[ $? == 0 ]]; then rm -rf "$fixture"; else printf "Archive fixture retained: %s\n" "$fixture" >&2; fi' EXIT
mkdir -p "$fixture/source/fixtures/.git" "$fixture/other" "$fixture/unpacked"
printf 'private Git configuration\n' > "$fixture/source/fixtures/.git/config"
printf 'retained input\n' > "$fixture/source/fixtures/"$'line\nbreak:payload'
chmod 640 "$fixture/source/fixtures/"$'line\nbreak:payload'
printf '#!/bin/sh\nexit 0\n' > "$fixture/source/fixtures/executable"
chmod 755 "$fixture/source/fixtures/executable"
printf 'outside selection\n' > "$fixture/source/outside"
ln -s ../outside "$fixture/source/fixtures/link"
ln -s fixtures "$fixture/source/alias"
ln -s fixtures "$fixture/source/"$'parent\n'
printf 'original_status=23\n' > "$fixture/other/outcome.txt"
bash "$ROOT/scripts/ci/archive-evidence.sh" "$fixture/evidence.tar.gz" \
    "$fixture/source" fixtures "$fixture/other" outcome.txt > "$fixture/archive-path"
tar -xzf "$fixture/evidence.tar.gz" -C "$fixture/unpacked"
cmp "$fixture/source/fixtures/"$'line\nbreak:payload' "$fixture/unpacked/fixtures/"$'line\nbreak:payload'
[[ "$(perl -e 'printf "%o", (stat($ARGV[0]))[2] & 0777' "$fixture/unpacked/fixtures/"$'line\nbreak:payload')" == 640 ]]
[[ -x "$fixture/unpacked/fixtures/executable" && -L "$fixture/unpacked/fixtures/link" ]]
[[ "$(readlink "$fixture/unpacked/fixtures/link")" == ../outside && ! -e "$fixture/unpacked/fixtures/link" ]]
[[ ! -e "$fixture/unpacked/fixtures/.git" && ! -e "$fixture/unpacked/outside" ]]
cmp "$fixture/other/outcome.txt" "$fixture/unpacked/outcome.txt"
cp "$fixture/evidence.tar.gz" "$fixture/before.tar.gz"
if bash "$ROOT/scripts/ci/archive-evidence.sh" "$fixture/evidence.tar.gz" "$fixture/source" fixtures; then exit 1; fi
cmp "$fixture/before.tar.gz" "$fixture/evidence.tar.gz"
for path in missing ../outside /outside alias/executable $'parent\n/executable'; do
    if bash "$ROOT/scripts/ci/archive-evidence.sh" "$fixture/rejected.tar.gz" "$fixture/source" "$path"; then exit 1; fi
    [[ ! -e "$fixture/rejected.tar.gz" ]]
done
if bash "$ROOT/scripts/ci/archive-evidence.sh" "$fixture/rejected.tar.gz" \
    "$fixture/source" fixtures "$fixture/source" fixtures/executable; then exit 1; fi
if bash "$ROOT/scripts/ci/archive-evidence.sh" "$fixture/source/fixtures/self.tar.gz" "$fixture/source" fixtures; then exit 1; fi
[[ ! -e "$fixture/rejected.tar.gz" && ! -e "$fixture/source/fixtures/self.tar.gz" ]]
# Output symlinks cannot redirect writes, and failed writers retain diagnostics.
ln -s "$fixture/absent" "$fixture/output-link"
if bash "$ROOT/scripts/ci/archive-evidence.sh" "$fixture/output-link" "$fixture/source" fixtures; then exit 1; fi
[[ ! -e "$fixture/absent" && -L "$fixture/output-link" ]]
mkfifo "$fixture/occupied-pipe"
# Keep a reader open so an accidental small write fails the test without hanging.
exec 9<> "$fixture/occupied-pipe"
if bash "$ROOT/scripts/ci/archive-evidence.sh" "$fixture/occupied-pipe" "$fixture/other" outcome.txt; then exit 1; fi
exec 9>&-
[[ -p "$fixture/occupied-pipe" ]]
# Resolve relative operands literally, even under inherited directory search.
mkdir -p "$fixture/operands/-P" "$fixture/search/-P" "$fixture/operands/unpacked"
printf 'selected root\n' > "$fixture/operands/-P/--help"
printf 'wrong root\n' > "$fixture/search/-P/--help"
(cd "$fixture/operands" && CDPATH="$fixture/search" \
    bash "$ROOT/scripts/ci/archive-evidence.sh" --help -P --help) > "$fixture/operands.log"
tar -xzf "$fixture/operands/--help" -C "$fixture/operands/unpacked"
cmp "$fixture/operands/-P/--help" "$fixture/operands/unpacked/--help"
# Newlines at directory/output boundaries must not select similarly named paths.
mkdir "$fixture/root" "$fixture/"$'root\n' "$fixture/"$'destination\n'
printf 'wrong source\n' > "$fixture/root/payload"
printf 'selected source\n' > "$fixture/"$'root\n/payload'
archive="$fixture/"$'destination\n/archive\n'
bash "$ROOT/scripts/ci/archive-evidence.sh" "$archive" "$fixture/"$'root\n' payload > "$fixture/newlines.log"
[[ -f "$archive" && ! -e "$fixture/"$'destination\n/archive' ]]
tar -xOf "$archive" ./payload > "$fixture/selected-payload"
cmp "$fixture/"$'root\n/payload' "$fixture/selected-payload"
mkdir "$fixture/bin"
printf '#!%s\nprintf "partial archive"\nexit 23\n' "$BASH" > "$fixture/bin/tar"
chmod +x "$fixture/bin/tar"
if PATH="$fixture/bin:$PATH" bash "$ROOT/scripts/ci/archive-evidence.sh" "$fixture/partial.tar.gz" "$fixture/source" fixtures; then exit 1; fi
[[ "$(cat "$fixture/partial.tar.gz")" == 'partial archive' && -f "$fixture/source/fixtures/executable" ]]
# A whole filesystem selection contains every output. Use substituted tar so
# a regression cannot archive the host; refusal must precede output creation.
if PATH="$fixture/bin:$PATH" bash "$ROOT/scripts/ci/archive-evidence.sh" "$fixture/rejected.tar.gz" / .; then exit 1; fi
[[ ! -e "$fixture/rejected.tar.gz" ]]
echo 'Evidence archive bytes, modes, links, selection and failure retention passed'
