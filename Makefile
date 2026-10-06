.DEFAULT_GOAL := help
.PHONY: help test version ensure-clean fetch-dependencies verify-shared-tooling check-pins test-pins \
        install-tools tools-check install-host-tools host-tools-check install-ic-tools ic-tools-check test-tools \
        fmt fmt-check check-format-tools install-hooks lint-tooling test-hooks test-tooling validate validate-toolchain wasm-size \
        test-release-runner test-release-adapters release-patch release-minor release-major release-resume \
        release-version release-preflight release-verify release-prepare-version \
        release-prepared-check release-files release-commit-check release-committed-check \
        release-tagged-check release-push-check qualify-release package publish publish-dry-run

RELEASE_REMOTE ?= origin
RELEASE_BRANCH ?= main
ifneq ($(word 2,$(filter release-patch release-minor release-major release-resume,$(MAKECMDGOALS))),)
$(error Select exactly one release target)
endif

# Bootstrap from the repository's simple, checked-in toolchain declaration.
# The Rust helper parses the full TOML for its own compiler identity checks.
VALIDATION_TOOLCHAIN ?= $(shell sed -n 's/^channel = "\([^"]*\)"$$/\1/p' rust-toolchain.toml)
TOOL := bash scripts/dev/run-repo-tool.sh $(VALIDATION_TOOLCHAIN)
FORMAT_CARGO := RUSTUP_AUTO_INSTALL=0 CARGO_NET_OFFLINE=true cargo +$(VALIDATION_TOOLCHAIN)
CARGO_SORT_VERSION := $(shell sed -n 's/^export SHARED_TOOLING_CARGO_SORT_VERSION=//p' ci/tool-versions.env)
IC_TOOL_PINS ?= ci/ic-tools.tsv
HOST_TOOL_VERSIONS ?= ci/tool-versions.env
export PATH := $(CURDIR)/.tools/host/bin:$(CURDIR)/.tools/ic/bin:$(PATH)

help:
	@echo 'Setup: install-tools (host then IC tools), fetch-dependencies, install-hooks; prepare Rust/cargo-sort separately.'
	@echo 'Focused checks: tools-check, test-tools, verify-shared-tooling, check-pins, test-pins, test-tooling, test-release-adapters, test-release-runner, test-hooks, fmt-check, lint-tooling.'
	@echo 'Formatting: fmt. Full gates require explicit qualification: validate, validate-toolchain.'
	@echo 'Maintainer releases: release-patch, release-minor, release-major; normal targets recover unfinished releases.'

install-hooks:
	bash scripts/dev/install-git-hooks.sh

# Explicit network setup; ordinary checks never install prerequisites.
install-tools:
	+$(MAKE) --no-print-directory install-host-tools
	+$(MAKE) --no-print-directory install-ic-tools

tools-check:
	+$(MAKE) --no-print-directory host-tools-check
	+$(MAKE) --no-print-directory ic-tools-check

install-host-tools:
	bash scripts/dev/install-host-tools.sh --versions "$(HOST_TOOL_VERSIONS)"

host-tools-check:
	bash scripts/dev/install-host-tools.sh --versions "$(HOST_TOOL_VERSIONS)" --check

install-ic-tools:
	bash scripts/dev/install-ic-tools.sh --pins "$(IC_TOOL_PINS)"

ic-tools-check:
	bash scripts/dev/install-ic-tools.sh --pins "$(IC_TOOL_PINS)" --check

test-tools:
	bash scripts/ci/test-host-tools.sh
	bash scripts/ci/test-ic-tools.sh
	bash scripts/ci/test-evidence-checksums.sh

# Network preparation is separate from offline checks; preserve tracked locks.
fetch-dependencies:
	cargo +$(VALIDATION_TOOLCHAIN) fetch --locked

verify-shared-tooling:
	bash scripts/ci/verify-shared-tooling-snapshot.sh

check-pins:
	RUSTUP_TOOLCHAIN=$(VALIDATION_TOOLCHAIN) RUSTUP_AUTO_INSTALL=0 CARGO_NET_OFFLINE=true bash scripts/ci/check-dependency-pins.sh --cargo-inheritance

test-pins:
	RUSTUP_TOOLCHAIN=$(VALIDATION_TOOLCHAIN) RUSTUP_AUTO_INSTALL=0 CARGO_NET_OFFLINE=true bash scripts/ci/test-dependency-pins.sh
	RUSTUP_TOOLCHAIN=$(VALIDATION_TOOLCHAIN) RUSTUP_AUTO_INSTALL=0 CARGO_NET_OFFLINE=true bash scripts/ci/test-cargo-metadata.sh

check-format-tools:
	@test "$$($(FORMAT_CARGO) sort --version)" = "cargo-sort $(CARGO_SORT_VERSION)" || \
		{ echo 'Install the pinned cargo-sort from ci/tool-versions.env before formatting.' >&2; exit 1; }

fmt: check-format-tools
	$(FORMAT_CARGO) sort --workspace
	$(FORMAT_CARGO) sort --workspace testing/runtime-qualification
	$(FORMAT_CARGO) fmt --all
	$(FORMAT_CARGO) fmt --manifest-path testing/runtime-qualification/Cargo.toml --all

fmt-check: check-format-tools
	$(FORMAT_CARGO) sort --workspace --check
	$(FORMAT_CARGO) sort --workspace --check testing/runtime-qualification
	$(FORMAT_CARGO) fmt --all -- --check
	$(FORMAT_CARGO) fmt --manifest-path testing/runtime-qualification/Cargo.toml --all -- --check

# Install exact, checksum-verified tools separately; these checks are offline.
lint-tooling:
	$${ACTIONLINT_BIN:-actionlint} .github/workflows/*.yml
	$${SHELLCHECK_BIN:-shellcheck} --shell=bash ci/tool-versions.env scripts/ci/*.sh scripts/dev/*.sh .githooks/pre-commit

test-tooling:
	cargo +$(VALIDATION_TOOLCHAIN) test --locked --offline --example repo-tool

test-release-runner:
	bash scripts/ci/test-release-runner.sh

test-release-adapters:
	bash scripts/ci/test-release-adapters.sh

test-hooks: check-format-tools
	RUSTUP_AUTO_INSTALL=0 CARGO_NET_OFFLINE=true bash scripts/ci/test-git-hooks.sh

test:
	cargo +$(VALIDATION_TOOLCHAIN) test --locked --offline -- --test-threads=1

version:
	@$(TOOL) version

ensure-clean:
	@$(TOOL) ensure-clean

# Full gates: explicitly requested qualification or configured CI only.
validate:
	$(MAKE) --no-print-directory validate-toolchain
	cargo +$$($(TOOL) msrv) check --locked --offline --all-targets

validate-toolchain:
	$(MAKE) --no-print-directory verify-shared-tooling host-tools-check check-pins test-pins test-tools test-tooling test-release-adapters test-release-runner test-hooks fmt-check
	cargo +$(VALIDATION_TOOLCHAIN) clippy --locked --offline --all-targets -- -D warnings
	cargo +$(VALIDATION_TOOLCHAIN) test --locked --offline -- --test-threads=1
	RUSTDOCFLAGS='-D warnings' cargo +$(VALIDATION_TOOLCHAIN) doc --locked --offline --no-deps
	cargo +$(VALIDATION_TOOLCHAIN) check --locked --offline --target wasm32-unknown-unknown --tests
	$(MAKE) --no-print-directory wasm-size
	cargo +$(VALIDATION_TOOLCHAIN) package --locked --offline

wasm-size:
	cargo +$(VALIDATION_TOOLCHAIN) build --locked --offline --profile wasm-size --target wasm32-unknown-unknown \
		--example wasm-core-size-probe --example wasm-diagnostics-size-probe --example wasm-key-only-size-probe --example wasm-admission-size-probe \
		--example wasm-runtime-integration-size-probe
	@$(TOOL) wasm-size

# Maintainer-only orchestration: these targets commit, tag and push.
release-patch release-minor release-major:
	+@bash scripts/ci/run-release.sh "$(@:release-%=%)" "$(RELEASE_REMOTE)" "$(RELEASE_BRANCH)"

release-resume:
	+@bash scripts/ci/run-release.sh resume "$(VERSION)" "$(RELEASE_REMOTE)" "$(RELEASE_BRANCH)"

release-version:
	@$(TOOL) version

release-preflight release-prepare-version release-prepared-check release-files release-commit-check release-committed-check release-tagged-check release-push-check:
	@$(TOOL) $@

# Preserve every gate attempt, including failures. The adapter owns the gate;
# these logs use Cargo's selected target directory, including overrides.
release-verify:
	@set -eu; \
	log_dir="$$($(TOOL) target)/release-validation/attempts"; \
	mkdir -p "$$log_dir"; \
	log_file="$$(mktemp "$$log_dir/verify.XXXXXX")"; \
	if $(TOOL) release-verify > "$$log_file" 2>&1; then result=0; else result=$$?; fi; \
	cat "$$log_file"; \
	echo "Retained full-gate log: $$log_file"; \
	exit "$$result"

qualify-release:
	$(TOOL) qualify-release

package: ensure-clean
	cargo +$(VALIDATION_TOOLCHAIN) package --locked --offline

publish:
	$(TOOL) publish

publish-dry-run:
	$(TOOL) publish --dry-run
