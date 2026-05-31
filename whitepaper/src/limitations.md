# Limitations

The Lean model states and checks abstract allocation-ownership lemmas over a
compact protocol model. The Rust crate enforces the corresponding invariants
through recovery, validation, staging, commit, and tests. This is not a formal
verification of the Rust implementation.

`ic-memory` also does not:

- prove semantic correctness of stored bytes,
- provide cryptographic tamper resistance,
- protect against malicious controllers,
- validate schema migrations,
- authorize endpoints,
- provide disaster recovery.

Its checksum protects against torn writes and accidental corruption only.

The present native slot model follows `ic-stable-structures` `MemoryManager`
IDs exactly. Moving beyond 255 virtual memories would require a different slot
descriptor and memory-manager layout, not merely a different `ic-memory`
policy.

`ic-memory` turns stable-memory allocation from implicit convention into
durable protocol state. Its central guarantee is simple: a stable key cannot
move, and a physical slot cannot be silently reused. By enforcing that
guarantee before application memory handles are opened, the crate closes a
practical upgrade hazard for multi-store Internet Computer canisters.
