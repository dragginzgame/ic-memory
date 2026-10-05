.PHONY: test version ensure-clean fetch-dependencies verify-shared-tooling \
        fmt-check lint-tooling test-tooling validate validate-toolchain wasm-size \
        patch minor release-patch release-minor release-stage release-commit \
        qualify-release release-push package publish publish-dry-run

# Bootstrap from the repository's simple, checked-in toolchain declaration.
# The Rust helper parses the full TOML for its own compiler identity checks.
VALIDATION_TOOLCHAIN ?= $(shell sed -n 's/^channel = "\([^"]*\)"$$/\1/p' rust-toolchain.toml)
TOOL := cargo +$(VALIDATION_TOOLCHAIN) run --locked --offline --quiet --example repo-tool --

# Network preparation is separate from offline checks; select Cargo.lock first.
fetch-dependencies:
	cargo +$(VALIDATION_TOOLCHAIN) fetch --locked

verify-shared-tooling:
	bash scripts/ci/verify-shared-tooling-snapshot.sh

fmt-check:
	cargo +$(VALIDATION_TOOLCHAIN) fmt --check

# Install exact, checksum-verified tools separately; these checks are offline.
lint-tooling:
	$${ACTIONLINT_BIN:-actionlint} .github/workflows/*.yml
	$${SHELLCHECK_BIN:-shellcheck} ci-tool-versions.env scripts/ci/*.sh

test-tooling:
	cargo +$(VALIDATION_TOOLCHAIN) test --locked --offline --example repo-tool

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
	$(MAKE) --no-print-directory verify-shared-tooling test-tooling fmt-check
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

patch:
	$(TOOL) patch

minor:
	$(TOOL) minor

# Maintainer-only orchestration: these targets commit, tag and push.
release-patch:
	$(MAKE) patch
	$(MAKE) release-stage
	$(MAKE) release-commit
	$(MAKE) release-push

release-minor:
	$(MAKE) minor
	$(MAKE) release-stage
	$(MAKE) release-commit
	$(MAKE) release-push

release-stage:
	$(TOOL) stage

release-commit:
	$(TOOL) commit

qualify-release:
	$(TOOL) qualify-release

release-push:
	$(TOOL) push

package: ensure-clean
	cargo +$(VALIDATION_TOOLCHAIN) package --locked --offline

publish:
	$(TOOL) publish

publish-dry-run:
	$(TOOL) publish --dry-run
