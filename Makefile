.PHONY: test \
        version ensure-clean validate validate-toolchain wasm-size test-wasm-size test-release-flow patch minor \
        release-patch release-minor release-stage release-commit release-push \
        package publish publish-dry-run

# Share the development/CI pin; callers can still override it explicitly.
VALIDATION_TOOLCHAIN ?= $(shell python3 -c 'import tomllib; from pathlib import Path; print(tomllib.loads(Path("rust-toolchain.toml").read_text())["toolchain"]["channel"])')
RELEASE := python3 scripts/release.py

test:
	cargo test -- --test-threads=1

# Source and the next numbered CHANGELOG.md entry must already be committed.
version:
	@$(RELEASE) version

ensure-clean:
	@$(RELEASE) ensure-clean

validate:
	$(MAKE) --no-print-directory test-release-flow
	$(MAKE) --no-print-directory validate-toolchain
	cargo +$$($(RELEASE) msrv) check --locked --all-targets

# Shared by CI and maintainer validation; the MSRV is checked separately.
validate-toolchain:
	$(MAKE) --no-print-directory test-wasm-size
	cargo +$(VALIDATION_TOOLCHAIN) fmt --check
	cargo +$(VALIDATION_TOOLCHAIN) clippy --all-targets -- -D warnings
	cargo +$(VALIDATION_TOOLCHAIN) test --locked -- --test-threads=1
	cargo +$(VALIDATION_TOOLCHAIN) check --locked --target wasm32-unknown-unknown --tests
	$(MAKE) --no-print-directory wasm-size
	cargo +$(VALIDATION_TOOLCHAIN) package --locked

wasm-size:
	cargo +$(VALIDATION_TOOLCHAIN) build --locked --profile wasm-size --target wasm32-unknown-unknown \
		--example wasm-core-size-probe --example wasm-diagnostics-size-probe --example wasm-key-only-size-probe --example wasm-admission-size-probe \
		--example wasm-runtime-integration-size-probe
	@set -eu; \
	target_dir=$$(cargo +$(VALIDATION_TOOLCHAIN) metadata --locked --offline --no-deps --format-version 1 | python3 -c 'import json, sys; print(json.load(sys.stdin)["target_directory"])'); \
	artifact_dir="$$target_dir/wasm32-unknown-unknown/wasm-size/examples"; \
	core_bytes=$$(wc -c < "$$artifact_dir/wasm_core_size_probe.wasm"); \
	diagnostics_bytes=$$(wc -c < "$$artifact_dir/wasm_diagnostics_size_probe.wasm"); \
	echo "Core raw Wasm: $$core_bytes bytes (budget: 260000 bytes)"; \
	echo "Diagnostics raw Wasm: $$diagnostics_bytes bytes (budget: 315000 bytes)"; \
	test "$$core_bytes" -le 260000; \
	test "$$diagnostics_bytes" -le 315000; \
	key_only_bytes=$$(wc -c < "$$artifact_dir/wasm_key_only_size_probe.wasm"); \
	echo "Key-only raw Wasm: $$key_only_bytes bytes (budget: 260000 bytes)"; \
	test "$$key_only_bytes" -le 260000; \
	admission_bytes=$$(wc -c < "$$artifact_dir/wasm_admission_size_probe.wasm"); \
	echo "Admission raw Wasm: $$admission_bytes bytes (budget: 264000 bytes)"; \
	test "$$admission_bytes" -le 264000; \
	integration_bytes=$$(wc -c < "$$artifact_dir/wasm_runtime_integration_size_probe.wasm"); \
	echo "Runtime integration raw Wasm: $$integration_bytes bytes (budget: 270000 bytes)"; \
	test "$$integration_bytes" -le 270000

test-wasm-size:
	PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tests -p 'test_wasm_size.py'

test-release-flow:
	PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tests -p 'test_release.py'

patch:
	$(RELEASE) patch

minor:
	$(RELEASE) minor

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
	$(RELEASE) stage

release-commit:
	$(RELEASE) commit

release-push:
	$(RELEASE) push

package: ensure-clean
	cargo package

publish:
	$(RELEASE) publish

publish-dry-run:
	$(RELEASE) publish --dry-run
