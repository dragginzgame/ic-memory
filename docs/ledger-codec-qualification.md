# Bounded ledger codec qualification

This records the compatible 0.26.1 candidate prepared after released
`f4b470e4f4ccbc672dbdf9513d236b8ecc63c73c` (0.26.0). It qualifies local codec
behavior and native allocation measurements, not publication or installed IC
execution. The manifest remains 0.26.0 until maintainer release preparation.

## Scope and tradeoffs

[`LedgerPayloadEnvelope::encode_ledger`](../src/ledger/payload.rs) previously
serialized into a growing `Vec` and checked its byte length afterward. A
structurally valid ledger with 65,536 generations and 256-byte fingerprints
exceeds the existing 16 MiB encoded limit. Its eventual refusal did not prevent
the temporary output buffer from growing beyond that limit.

The private writer now checks each slice before appending, limits geometric
capacity reservations to 16 MiB plus the 24-byte envelope, and discards partial
output on failure. Ciborium still performs one canonical serialization pass;
there is no extra size-counting traversal or intermediate payload copy. The
writer supplies an all-or-error `write_all` implementation for Ciborium's IO
adapter. The same typed ledger-byte error returns before physical mutation.

[`deserialize_bounded_vec`](../src/cbor.rs) previously ignored admitted sequence
length hints. It now reserves the definite collection length after checking its
ceiling. Non-empty collections below the ceiling have one spare entry, so the
owned bootstrap staging path can append its next generation without immediately
doubling a full vector. An empty collection reserves nothing; an unhinted
sequence keeps incremental growth and refusal before excess-element decoding.
This can add one element's capacity to a recovered collection; it is not a
universal reduction in peak recovery heap.

No public API, wire discriminator, encoded bytes, structural limits, integrity
ordering, dependency selection or toolchain changes. Existing corruption,
physical/logical generation binding, failed persistence, retry and publication
boundaries remain in place. No compatibility reader or new runtime mode exists.

The bounded writer adds checks per serialized slice. Native timings varied
substantially across runs, including slower valid commits and changes to the
unchanged record-encoding phase. No speedup or precise timing regression is
established. IC instruction costs, cycles, raw Wasm size, stable-memory footprint,
whole bootstrap, PocketIC upgrade rollback and native macOS execution were not
remeasured. Wasm compilation alone does not qualify these effects. Full release
gates remain maintainer-owned.

## Matched native allocation measurements

Linux x86-64, Rust 1.99.0 (`b940084d7eb6a299eb4bfeb8e34901bc051e7ac4`, LLVM
23.1.1), unchanged root lockfile, standard Cargo release profile, one test thread.
The baseline uses 0.26.0 production source with the extended measurement harness;
the candidate uses the source hashes below and that same harness. Logical input
fields and ten-iteration operations match. Inputs and the two protected slots
used by record encoding/recovery are constructed before counting begins.

The [`measurement harness`](../tests/allocation_measurements.rs) counts this
thread's requested allocations through `System`, not RSS or transient internal
allocator copying. Every measured operation releases its measured allocations.
Commit measurements start from an empty store and include encoding, protected
slot construction and the returned owned ledger. The fingerprint fixtures use
65,536 generation records, no allocation records, and either 128-byte strings
(admissible) or 256-byte strings (byte-limit refusal). They do not claim to model
an entire application or runtime.

| Operation | Baseline | Candidate | Interpretation |
| --- | ---: | ---: | --- |
| Recover 65,536 generations: reallocations per call | 14 | 0 | Admitted length reserves the full history |
| Recover 65,536 generations: peak requested heap | 4,194,304 | 4,194,304 | Count ceiling leaves no spare entry |
| Recover 1,024 generations: peak requested heap | 65,536 | 65,600 | One spare 64-byte generation entry |
| Commit 65,536 generations, 128-byte fingerprints: peak requested heap | 37,748,736 | 29,360,152 | About 8 MiB less peak heap, roughly 22% |
| Refuse 65,536 generations, 256-byte fingerprints: peak requested heap | 26,607,616 | 16,777,240 | About 9.4 MiB less peak heap, roughly 37% |

The final matched results, including timings and unchanged record-encoding and
validation phases, are retained in
[`ledger-codec-native-allocations.csv`](measurements/ledger-codec-native-allocations.csv).
This extends the earlier
[runtime IO and record-encoding evidence](runtime-io-qualification.md); it does
not relabel that evidence as current-source installed qualification.

## Inputs and retained evidence

SHA-256 at final candidate measurement:

| Input | SHA-256 |
| --- | --- |
| Root `Cargo.toml` | `f90815f13d3a9262dc614d3070614836f005fd1be224342b8340d49fe496f7f5` |
| Selected root `Cargo.lock` | `b01ebcdd187964fedfd4992e472466189620b8f30f0043184e535f7b809f1f9e` |
| Candidate `src/cbor.rs` | `c61b33d85395dfc6f822922bcba8049540ca0550fc56943c5f510589627e43f7` |
| Candidate `src/ledger/payload.rs` | `b3839ea0718f2b96d9bc9d9164a5d1b8bd86233c5a46fd3a6372060f9a8bcafc` |
| Shared measurement harness | `765a704331f92600926f4760c82c71edcbf7fc6f17cd1016315b6843007e5cf4` |
| Candidate native measurement binary | `63d1e04d7088b78d32292b34e76ed184167d3a9c61feac2f82a1a400540531f5` |

Local measurement logs are retained at
`/tmp/ic-memory-0.26.1-codec-baseline.log` and
`/tmp/ic-memory-0.26.1-codec-candidate-final.log`. Prototype writer logs, a failed
measurement attempt using unavailable private APIs, and the initial strict-test
Clippy failure remain under the same `/tmp/ic-memory-0.26.1-` prefix. The private
API attempt was removed rather than widening the crate's public surface; the
lint fixture was reordered. These temporary logs are local evidence, not
packaged artifacts. The matched CSV and source bindings above remain in the
repository for review.

## Focused behavior qualification

- Rust 1.99.0: 94 ledger, 3 CBOR, 65 runtime, 11 stable-cell and 12 bootstrap unit
  tests pass. The final writer's five payload tests also pass after its IO adapter
  refinement. This includes unchanged-store byte-limit refusal, corruption and
  persisted fixtures, generation/history exhaustion, persistence failure and
  deterministic retry.
- New checks cover exact-ceiling/empty writer operations, oversized refusal
  without partial append, byte equality with independent canonical CBOR plus
  envelope encoding, and definite collection boundaries with rejection before
  decoding any over-limit element. The unhinted excess-element test still passes.
- Strict Rust 1.99.0 Clippy passes for the library and test targets. Rust 1.88.0
  passes the 94 ledger and three CBOR tests and checks the library and native
  measurement target offline with the selected lockfile.
- Rust 1.99.0 library compilation for `wasm32-unknown-unknown` passes. This is a
  compile check, not installed IC or Wasm-size qualification.

To reproduce the measurement and focused codec checks:

```bash
cargo +1.99.0 test --locked --offline --release --test allocation_measurements \
  -- --ignored --test-threads=1 --nocapture
cargo +1.99.0 test --locked --offline --lib ledger:: -- --test-threads=1
cargo +1.99.0 test --locked --offline --lib cbor:: -- --test-threads=1
cargo +1.99.0 clippy --locked --offline --lib --tests -- -D warnings
cargo +1.88.0 test --locked --offline --lib ledger:: -- --test-threads=1
cargo +1.88.0 test --locked --offline --lib cbor:: -- --test-threads=1
cargo +1.88.0 check --locked --offline --lib --test allocation_measurements
cargo +1.99.0 check --locked --offline --lib --target wasm32-unknown-unknown
```

Run the baseline measurement against released source plus the same harness;
running these commands twice against the candidate is not a matched baseline.
Do not infer a published release or a native macOS qualification from local
Linux passes. See the [host support contract](host-support.md) and the
[release guide](../RELEASING.md) for the remaining native and full-gate checks.

## 0.26.2 text preflight follow-up

Baseline: released `2988c827ea1f0b5f98b2402fa471ae2e4c8a8395` (0.26.1).
The candidate adds one text-length check to the existing allocation-free CBOR
walk. Current ledger fields and identifiers fit the existing 256-byte diagnostic
ceiling; stable-key grammar is stricter. Previously, the syntax walk only checked
that advertised text fit the input. Ciborium could then grow an owned `String`
before the fingerprint validator refused its length.

Reject oversized text before entering serde, even inside arrays, maps and tags.
Do not walk inside opaque byte strings: their protected-record decode and inner
logical-ledger decode remain separate trust boundaries. Physical checksum,
envelope and generation checks retain their ordering. Malformed logical text
still produces `LedgerCommitError::Codec`, refuses initialization over existing
state and leaves both slots unchanged. Direct caller-selected serde readers keep
their field validators; they do not acquire the maintained CBOR preflight.

No accepted durable shape, public API, byte ceiling, dependency selection or
toolchain changes. Valid recovery adds a text-length comparison to the existing
walk; its IC instruction and Wasm-size impact are unmeasured. The raw input and
opaque payload buffers still exist. This improvement bounds additional logical
decoding allocations on the malformed-text failure path, rather than removing
the input buffer or eliminating all recovery allocations.

### Matched native measurement

Use the same Linux x86-64, Rust 1.99.0, release-profile, ten-iteration and
single-thread counting method above. The new ignored
`ledger_text_recovery_allocations` test constructs a current one-generation
ledger with a correctly checksummed physical slot and envelope. Only its
fingerprint length varies. The 256-byte control recovers; the larger values
refuse with the same codec error category. Fixture construction, existing input
buffers and physical-store construction are outside the counting window.
Checksums still scan the protected payload during each measured recovery.

| Fingerprint bytes | Baseline peak requested heap | Candidate peak requested heap |
| ---: | ---: | ---: |
| 256 (valid control) | 384 | 384 |
| 257 | 430 | 200 |
| 1,024 | 1,197 | 200 |
| 1,048,576 | 1,048,749 | 200 |
| 8,388,608 | 8,388,781 | 200 |

The failure still allocates small error objects/formatting buffers. The CSV
retains allocation and reallocation counts too; the small 257-byte case has more
error-buffer reallocations despite its lower peak heap. No universal reduction
in allocation calls or native speedup is claimed. No IC instruction, cycle,
Wasm-size, stable-memory, installed upgrade or native macOS measurement was run.

Results:
[`ledger-text-native-allocations.csv`](measurements/ledger-text-native-allocations.csv).
Local logs: `/tmp/ic-memory-0.26.2-text-baseline.log` and
`/tmp/ic-memory-0.26.2-text-candidate.log`. The failing-before regression log and
the initial test-helper compilation failure are retained under the same prefix.
The helper call was corrected to reuse the existing canonical envelope fixture
encoder. The earlier 0.26.1 measurements above are unchanged.

SHA-256 bindings for this measurement:

| Input | SHA-256 |
| --- | --- |
| Root `Cargo.toml` | `f76ddda751d33dfed621e81d3dea21f018e9e7b585046279127e3c0c686e4f44` |
| Selected root `Cargo.lock` | `92a9d55aadc78ae619e76793bfeac696a25330675ad534456da37f01a3310e23` |
| Baseline `src/cbor.rs` | `c61b33d85395dfc6f822922bcba8049540ca0550fc56943c5f510589627e43f7` |
| Candidate `src/cbor.rs` | `7535dd7cf5f937dd3a7fbb918cfa6cfe66be5c077ae9b0283d02454abd98b5e0` |
| Candidate measurement harness | `06b6f85d3d198d3a312de7de6c017bee7bf6fb50c25c52bad48bb113d83cd4a4` |
| Baseline native measurement binary | `2086312626a821d732d9dfad012b7b5b22406071fd5ef482f285c0746de6f72d` |
| Candidate native measurement binary | `1918b0393f479a560a30e3e7fb0e788ea74c3599f2d8af31f1c9d9025990a5cb` |

Both variants use the same added measurement operations and logical input fields.
Formatting the harness between runs does not change their logic. The baseline
binary hash was saved before compiling the candidate.

### Focused checks

- The early-text regression fails against 0.26.1 before the guard is added and
  passes afterward. It rejects fully backed text as well as truncated/maximal
  length advertisements before a panic-on-entry deserializer can run.
- Boundary tests admit 256-byte text and opaque byte strings containing longer
  inner text. A protected-record regression decodes that opaque data unchanged,
  then refuses logical recovery and initialization without mutating slots.
- Rust 1.99.0 passes five CBOR, 95 ledger, 65 runtime, 11 stable-cell and 11
  diagnostic-metadata tests. Current fixtures, field-error behavior in direct
  serde readers, corrupt-state rejection, persistence failure and retry pass.
- Rust 1.88.0 passes the five CBOR and 95 ledger tests and checks the library and
  measurement target offline. Strict Rust 1.99.0 library/test Clippy and library
  compilation for `wasm32-unknown-unknown` pass.

Reproduce the specific measurement with:

```bash
cargo +1.99.0 test --locked --offline --release --test allocation_measurements \
  ledger_text_recovery_allocations -- --ignored --test-threads=1 --nocapture
```

For a matched baseline, add the same measurement harness to released 0.26.1
production source. The full release gates and native macOS qualification were
not run during this focused pass.
