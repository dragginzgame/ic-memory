# Lean Model

The executable model is committed at `whitepaper/lean/IcMemory.lean`. It
captures the authority boundary that the Rust code must maintain:

- stable keys do not move slots,
- physical slots are not reused by other keys,
- retired allocations do not become active again,
- opening through a post-commit authority is backed by an active ledger record,
- successful generation staging advances the abstract generation.

The model is intentionally protocol-level. It avoids Rust-specific
implementation details such as CBOR decoding, physical dual-slot recovery,
runtime registration, declaration-count limits, schema metadata history,
mandatory generation parent links, and `u64` overflow. In particular, the Lean
`OpenAuthority` type is a post-commit model object, not a field carried inside
Rust's `ValidatedAllocations` struct.

The current model also encodes several safety facts as assumptions on
`SafeLedger`, `TombstoneLedger`, and `OpenAuthority`. The checked theorems then
show what follows from those assumptions. That makes the file useful as
executable protocol documentation, but it should not be described as a proof
that the Rust implementation preserves the invariants.

```lean
{{#include ../lean/IcMemory.lean}}
```
