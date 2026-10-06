# Shared owner adoption — 2026-10-06

## Scope and identities

Source: released ic-memory 0.27.3, commit
`19efb34ce8fce0b297b4ff0a7a5bd4920255ce5f`, with uncommitted adoption edits.
Trigger: maintainer requested the issue recommendations and less local code,
using ic-host-tools and Shared Tooling where they already own the contract.

Shared source: clean committed 0.1.6 revision
`a37771f1b6b5fc9a88ed6ab3b705bdda35cd8fa3`. Canonical refresh exported 44
files from a frozen clone; [.shared-tooling.snapshot](../../../.shared-tooling.snapshot)
records their exact hashes and modes. The sibling checkout was not modified.
During final inspection, new optional verification helpers and guide edits were
uncommitted upstream. Their current adoption guidance was read; it explicitly
requires a reviewed committed source before exporting those helpers. They are
not included or attributed to the recorded 0.1.6 revision.
Host artifact dependency: registry `ic-host-tools 0.1.10`, not the sibling's
uncommitted tooling/documentation changes. Rust pin/MSRV remain 1.99.0/1.88.0.

Methods: [common contract](../../../audits/README.md),
[code hygiene](../../../audits/code-hygiene.md) and
[local overlay](code-hygiene.md). SHA-256 identities:

| Definition | SHA-256 |
| --- | --- |
| Common contract | `926d5a39ea62a3becd19ff0326c63987b2929d3a1a728e2a0ef8a7180824d231` |
| Code hygiene | `518d890bf485bef06c316b4cf26ac6bf357ed52bcce995e8d9b1232a103d0ea3` |
| Local overlay | `d790d608eeec1c750c7b40a69fcbc3363e8aad0253c6c7cabcee463e21bbb10c` |
| Frozen prior definition | `84c8631fe89788feb20f4e4184703956240f2ca4e3a4ded50ad68f9abeaa1b5a` |

This is source adoption, an existing-report walk and focused Linux qualification,
not a newly executed product audit. Verdict: **PASS** for those recorded checks.
Native macOS consumer adoption remains unqualified; configured CI is not evidence.
Historical composite scores are **N/A (method change)**.

## Prior-obligation mapping and representative report walk

The original definition is preserved byte-for-byte in [frozen evidence](frozen/README.md).
The [2026-10-03 report](code-hygiene-report-2026-10-03.md) was read as historical
evidence, not rerun or relabeled. Its APIs and measurements describe its source.

| Prior obligation / report anchor | Current owner |
| --- | --- |
| Public API and trust-state inventories; authority constructors | Shared hygiene review 1; overlay public roots and trust transitions |
| Serde/decoded-data classification and validated constructors | Shared review 1/3; overlay distinguishes inert DTOs from capabilities |
| Panic, unwrap, unsafe and error inventory | Shared review 2/3; overlay stable-input reachability, Storable and operator-action distinctions |
| Negative tests, recent behavioral fixes and capability boundaries | Shared review 3/6; overlay selected behavioral/compile-fail tests and serialized declaration tests |
| README, advanced docs, examples and archived evidence | Shared review 4; overlay product caveats, shortest path and historical identities |
| Dependency, feature, module and artifact hygiene | Shared review 4/5; overlay separates runtime, host adapters and PocketIC graph |
| Performance/Wasm tables and installed lifecycle claims | Owning domain qualification; no new performance or installed-state verdict |
| Automatic mechanical repairs, unconditional broad commands, risk score | Retired generic instructions; common authority/evidence contract now owns these |

The report's bootstrap/decoder negative tests are examples of evidence that can
support a finding when source identities match. Their old PASS does not qualify
the present implementation. Product trust obligations were retained before the
generic checklist was replaced; no report tree was moved or historical report edited.

## Ownership and simplification

- Shared installers own host/IC setup, checksums, versions, activation, locks and
  failed/prior candidate retention. Make exposes their six standard setup/check
  targets; `test-tools` executes their unchanged substitutes. Checks never install.
- `ci/tool-versions.env` owns jq/yq, cargo-sort, Actionlint and ShellCheck selections;
  `ci/ic-tools.tsv` owns the IC set. All previously selected formatter/lint/parser
  versions and hashes are retained. The local duplicate catalog is removed.
- Retire `scripts/dev/install-yq.sh` and `scripts/ci/install-yq.sh`, using common
  host setup instead. The only removed named function is **usage** in the latter;
  common `scripts/dev/install-host-tools.sh::usage` owns setup argument help.
  The consumer installer and catalog declared no functions, methods or types.
- `Repository::sha256` remains a receipt-string adapter, now an associated function
  using `ic_host_tools::artifact::hash_file`. The shasum dispatch/output parser and
  substitute branch are removed. `retained_package` uses `Sha256Digest` parsing;
  malformed digest authority rejects before target discovery.
- Keep product receipt validation, exact release reconciliation, atomic metadata
  replacement and raw Wasm ceilings local. They have no equivalent upstream owner.
  No generic tool-execution framework, new schema, store history or runtime abstraction
  was added.

## Focused qualification

Host: Linux x86-64. All commands ran against the dirty adoption scope with the
selected root lock. Its SHA-256 is
`8d8bdf4082a9422ceb83d7782d7735d18c17f2c992a18857d3eacc9d753c25bf`.
Every existing package version is preserved. The host-only dependency adds 22
locked packages; the independent runtime workspace lock is unchanged.

- Real `make install-tools tools-check` passed: jq/yq and Quill 0.5.4, ICP CLI
  1.6.0, didc 0.6.2, ic-wasm 0.11.1, PocketIC 16.0.0 and wasm-opt 132.
  A subsequent normal-sandbox setup/check passed offline using the selected sets.
- First setup failed at sandbox DNS before activation. Its candidate remains at
  `.tools/host-set.hjnO1R`. The explicit network-capable retry passed; no failed
  evidence was cleaned. Real installation/version checks do not prove PocketIC
  application compatibility or tool deployment behavior.
- `make test-tools` passed canonical host/IC/evidence fixtures, including corruption,
  version mismatch, interruption, retained candidates and simulated host mappings.
  These mappings do not substitute for native macOS.
- `make check-pins test-pins verify-shared-tooling` passed.
- `make test-release-adapters test-release-runner test-hooks` passed with command
  substitutes/disposable indexes and no commits, tags or pushes.
- `make fmt-check lint-tooling` passed. Initial focused lint found the new shared
  shell catalog needs an explicit Bash selection; Make now supplies
  `shellcheck --shell=bash`. The vendored catalog remains unchanged.
- Local link destinations passed across 34 current/adoption documents (205 links),
  excluding literal documentation examples and preserving historical reports.
- `cargo +1.99.0 clippy --locked --offline --example repo-tool -- -D warnings`
  passed. The initial check found the hash adapter no longer needs a receiver;
  callers now use its associated function.
- `cargo +1.99.0 test --locked --offline --example repo-tool` and the same command
  on 1.88.0 passed 23 tests each, including retained/corrupt evidence, publication
  guards, selected-commit recovery and invalid digest paths.
- Locked offline no-deps metadata passed for both maintained workspaces.
  Focused Wasm checks for `repo-tool` and `wasm-core-size-probe` passed.
  The Wasm normal/build/dev tree contains none of ic-host-tools, sha2, flate2,
  wasmparser or rustix. Canister source and normal dependencies are unchanged.

Full gates, package/release effects, Wasm size measurements, installed PocketIC
execution and native macOS were not run. No measured Wasm/instruction/stable-memory
delta is claimed. CI is configured to install/check both tool sets and run the
existing native gates on Ubuntu/macOS Intel/macOS ARM.

## Compatibility and follow-up owner

Pending **0.28.0** removes the documented standalone yq setup command and requires
the local parser set for full validation. Run `make install-tools` (or just
`make install-host-tools` for parser checks); Rust/cargo-sort bootstrap remains
separate. This developer CLI hard cut has no library API, stable-memory format
or receipt schema change. The package version remains 0.27.3 until maintainer release.

Implementation/adoption evidence belongs to
[#11](https://github.com/dragginzgame/ic-memory/issues/11) and
[#12](https://github.com/dragginzgame/ic-memory/issues/12).
[#10](https://github.com/dragginzgame/ic-memory/issues/10) retains its distinct
matching native release-workflow qualification requirement; this tooling batch
does not qualify a live release or close issues. Matching consumer CI remains
the native host evidence owner. No upstream message or issue mutation was performed.

Post-batch inspection of [CI run 37452984282](https://github.com/dragginzgame/ic-memory/actions/runs/37452984282),
attempt 1 at the released source above, found all three native test jobs failing
in `test-pins` with `rg: command not found`. Tooling lint and all three MSRV jobs
passed. Add explicit conditional apt/Homebrew ripgrep bootstrap to the new setup
steps; retain the offline fixtures unchanged. Focused local pin/tool fixtures and
workflow lint pass, but only matching new native CI can qualify that repair.
The actual YAML bootstrap step also passed Linux/macOS install selection, existing-rg
skip and unsupported-host rejection through command substitutes; evidence remains
under `target/shared-tooling-adoption/bootstrap.p9y7tW`. These substitutes performed
no package installation and do not qualify native Homebrew/apt execution.

## Follow-up: test-target lint qualification

The maintainer's gate on adoption commit
`1794de9bc1ada301c86f20ed9d174cdf3285700a` caught the new test's empty-vector
assertion lint. The earlier example-only Clippy command above did not include
the example's test target. Change that assertion to an explicit zero call count,
preserving the proof that invalid digests reject before target discovery.
On that source plus the unstaged assertion fix,
`cargo +1.99.0 clippy --locked --offline --example repo-tool --tests -- -D warnings`,
the selected `retained_package_paths_require_digest_authority_before_target_discovery`
test, `make fmt-check` and `git diff --check` passed. No full gate or release
command was executed; this evidence does not relabel the original narrower check.
