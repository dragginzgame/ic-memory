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
  || envelopeVersion
  || ledgerSchemaVersion
  || physicalFormatId
  || payloadLength
  || CBOR(AllocationLedger)
```

This ordering matters: version routing happens before deserializing the ledger
DTO. The decoded DTO still is not authority. It becomes useful for allocation
authority only after compatibility checks, committed-integrity validation, and
`RecoveredLedger` construction succeed.
