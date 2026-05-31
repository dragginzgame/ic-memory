# Durable Commit Protocol

The allocation ledger is stored under the default native anchor:

```text
MemoryManager ID 0
  -> Cell<StableCellLedgerRecord>
  -> LedgerCommitStore
```

`LedgerCommitStore` uses dual protected commit slots. Each slot contains a
generation number, marker, checksum, and encoded ledger payload. Recovery
chooses the highest-generation valid slot. A corrupt newer slot cannot override
an older valid slot, and ambiguous equal-generation divergent slots fail
closed.

The logical payload is wrapped in a small envelope before ledger decode:

```text
ICMEMLED
  || payloadLength
  || CBOR(AllocationLedger)
```

The 0.7 protocol deliberately removed the earlier version-routing scaffold from
this envelope. The only logical payload accepted by the current crate is the
current crate-owned CBOR `AllocationLedger` DTO, guarded by the envelope magic
and payload length.

This ordering still matters: physical recovery selects a valid committed slot
before the logical envelope is decoded, and the envelope is classified before
CBOR ledger decode. The decoded DTO still is not authority. It becomes useful
for allocation authority only after the physical generation matches the logical
ledger generation, committed-integrity validation succeeds, and
`RecoveredLedger` construction succeeds.
