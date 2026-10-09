.DEFAULT_GOAL := help
.PHONY: help test version ensure-clean fetch-dependencies verify-shared-tooling check-pins test-pins \
        install-tools tools-check install-host-tools host-tools-check install-ic-tools ic-tools-check test-tools test-failure-evidence \
        fmt fmt-check check-format-tools install-hooks lint-tooling test-hooks test-tooling validate validate-toolchain wasm-size \
        install-runtime-server runtime-server-check test-runtime \
        test-release-runner test-release-adapters \
        release-version release-preflight release-verify release-prepare-version \
        release-prepared-check release-files release-commit-check release-committed-check \
        release-tagged-check release-push-check qualify-release package publish publish-dry-run

export RELEASE_DELIVERY ?= direct
ifneq ($(filter release-%,$(MAKECMDGOALS)),)
ifneq ($(RELEASE_DELIVERY),direct)
$(error ic-memory release adapters support only RELEASE_DELIVERY=direct)
endif
endif

# Bootstrap from the repository's simple, checked-in toolchain declaration.
# The Rust helper parses the full TOML for its own compiler identity checks.
VALIDATION_TOOLCHAIN ?= $(shell sed -n 's/^channel = "\([^"]*\)"$$/\1/p' rust-toolchain.toml)
TOOL := bash scripts/dev/run-repo-tool.sh $(VALIDATION_TOOLCHAIN)
FORMAT_CARGO := RUSTUP_AUTO_INSTALL=0 CARGO_NET_OFFLINE=true cargo +$(VALIDATION_TOOLCHAIN)
CARGO_SORT_VERSION := $(shell sed -n 's/^export SHARED_TOOLING_CARGO_SORT_VERSION=//p' ci/tool-versions.env)
include make/tools.mk
include make/release.mk

# The published Testkit CLI owns PocketIC selection, admission and lifecycle.
# Keep this tool graph outside both maintained workspace lockfiles.
IC_TESTKIT_VERSION := 0.25.4
TESTKIT_CLI := RUSTUP_TOOLCHAIN=$(VALIDATION_TOOLCHAIN) bash scripts/dev/install-rust-tools.sh --consumer "$(CURDIR)" --package ic-testkit --version $(IC_TESTKIT_VERSION) --bin ic-testkit-server --profile release
TESTKIT_SERVER_DIRECTORY := $(CURDIR)/.tools/ic-testkit-server

help:
	@echo 'Setup: install-tools (host then IC tools), fetch-dependencies, install-hooks; prepare Rust/cargo-sort separately.'
	@echo 'Focused checks: tools-check, test-tools, test-failure-evidence, verify-shared-tooling, check-pins, test-pins, test-tooling, test-release-adapters, test-release-runner, test-hooks, fmt-check, lint-tooling.'
	@echo 'Formatting: fmt. Full gates require explicit qualification: validate, validate-toolchain.'
	@echo 'Installed runtime: install-runtime-server (explicit Testkit setup), runtime-server-check (offline), test-runtime (prepared server and locked caches).'
	@echo 'Report: cloc (root workspace; CLOC_MANIFEST selects an independent Cargo manifest). Fleet reports run in Shared Tooling.'
	@echo 'Maintainer releases: release-patch, release-minor, release-major; normal targets recover unfinished releases.'

install-hooks:
	bash scripts/dev/install-git-hooks.sh

# Explicit network setup; ordinary tool/library gates never provision a server.
install-runtime-server: host-tools-check
	@cli="$$($(TESTKIT_CLI))" && "$$cli" setup --directory "$(TESTKIT_SERVER_DIRECTORY)"

# Both CLI receipt/bytes and server archive/bytes are checked without downloads.
runtime-server-check:
	@cli="$$($(TESTKIT_CLI) --check)" && "$$cli" check --directory "$(TESTKIT_SERVER_DIRECTORY)"

# Focused installed IO/upgrade qualification, separate from the library gates.
# Build before launch so compilation does not consume the server lifetime.
test-runtime: runtime-server-check
	cargo +$(VALIDATION_TOOLCHAIN) build --locked --offline --profile wasm-size --target wasm32-unknown-unknown --example wasm-io-qualification
	cargo +$(VALIDATION_TOOLCHAIN) build --locked --offline --manifest-path testing/runtime-qualification/Cargo.toml --target-dir target/runtime-qualification
	@mkdir -p target/qualification/runtime
	@cli="$$($(TESTKIT_CLI) --check)" && \
	  evidence="$$(mktemp -d target/qualification/runtime/attempt.XXXXXX)" && \
	  shasum -a 256 Cargo.toml Cargo.lock .shared-tooling.snapshot \
	    testing/runtime-qualification/Cargo.toml testing/runtime-qualification/Cargo.lock \
	    target/wasm32-unknown-unknown/wasm-size/examples/wasm_io_qualification.wasm \
	    target/runtime-qualification/debug/ic-memory-runtime-qualification "$$cli" > "$$evidence/inputs.sha256" && \
	  { env -u POCKET_IC_BIN -u IC_TESTKIT_POCKET_IC_URL \
	    IC_MEMORY_QUALIFICATION_WASM="$(CURDIR)/target/wasm32-unknown-unknown/wasm-size/examples/wasm_io_qualification.wasm" \
	    "$$cli" run --directory "$(TESTKIT_SERVER_DIRECTORY)" --ttl 900 \
	    --server-stdout "$$evidence/server.stdout" --server-stderr "$$evidence/server.stderr" \
	    -- "$(CURDIR)/target/runtime-qualification/debug/ic-memory-runtime-qualification" > "$$evidence/runtime.log" 2>&1; \
	    status=$$?; cat "$$evidence/runtime.log"; exit "$$status"; }

test-failure-evidence:
	bash scripts/ci/test-evidence-archive.sh
	bash scripts/ci/test-failure-evidence.sh

test-tools: test-failure-evidence
	bash scripts/ci/test-tool-commands.sh
	bash scripts/ci/test-host-tools.sh
	bash scripts/ci/test-ic-tools.sh
	bash scripts/ci/test-rust-tools.sh
	bash scripts/ci/test-evidence-checksums.sh
	RUSTUP_TOOLCHAIN=$(VALIDATION_TOOLCHAIN) RUSTUP_AUTO_INSTALL=0 CARGO_NET_OFFLINE=true bash scripts/ci/test-cloc.sh

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
	RUSTUP_TOOLCHAIN="$(VALIDATION_TOOLCHAIN)" bash scripts/ci/check-format-tools.sh "$(CARGO_SORT_VERSION)"

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
	RUSTUP_AUTO_INSTALL=0 CARGO_NET_OFFLINE=true bash scripts/ci/test-format-tools.sh
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

# Preserve checkout-local routing and cache preparation for all caller settings.
# GNU Make 3.81 cannot combine target-specific override and export. Reclassify
# caller values first so these exported target policies can supersede them.
override SHARED_TOOLING_ROOT := $(SHARED_TOOLING_ROOT)
ifneq ($(origin RELEASE_CACHE_PREPARE),undefined)
override RELEASE_CACHE_PREPARE := $(RELEASE_CACHE_PREPARE)
endif
release-patch release-minor release-major release-resume: export SHARED_TOOLING_ROOT := $(CURDIR)
release-patch release-minor release-major release-resume: export RELEASE_CACHE_PREPARE := 1

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
