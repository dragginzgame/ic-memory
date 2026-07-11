# Durable Commit Protocol

The allocation ledger is stored under the default native anchor:

```text
MemoryManager ID 0
  -> Cell<StableCellLedgerRecord>
  -> LedgerCommitStore
```

`LedgerCommitStore` uses two redundant commit slots. Each slot contains a
generation number, marker, checksum, and encoded ledger payload. Recovery
chooses the highest generation only after every present slot passes marker and
checksum validation. Any present corrupt slot and any equal-generation
divergence fail closed; recovery never rolls back to an older allocation
history.

The default runtime serializes both slots together inside one
`ic-stable-structures::Cell`; they are not independently atomic physical writes.
ICP message execution provides atomic stable-memory commit and rollback. The
redundant slots and checksums therefore provide generation selection and
localized corruption detection rather than transaction atomicity. Detected
corruption requires operator intervention; it is not an automatic rollback
signal.

The logical payload is wrapped in a small envelope before ledger decode:

```text
ICMEMLED
  || payloadLength
  || CBOR(AllocationLedger)
```

The only logical payload accepted by the current crate is the crate-owned CBOR
`AllocationLedger` DTO, guarded by the envelope magic and payload length.

This ordering still matters: physical recovery selects the authoritative
committed slot before the logical envelope is decoded, and the envelope is
classified before CBOR ledger decode. The decoded DTO still is not authority.
It becomes useful for allocation authority only after the physical generation
matches the logical ledger generation, committed-integrity validation succeeds,
and `RecoveredLedger` construction succeeds.
