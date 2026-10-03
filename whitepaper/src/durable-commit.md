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
  || ICMF
  || formatVersion
  || payloadLength
  || CBOR(AllocationLedger)
```

The only logical payload accepted by the current crate is the crate-owned CBOR
`AllocationLedger` DTO, guarded by the envelope family magic, current format
marker and version, and payload length. A recognized non-current format is
reported as unsupported without accepting or migrating it.

The 24-byte envelope header contains eight family-magic bytes, four marker
bytes, a little-endian `u32` format version and a little-endian `u64` payload
length. Version is currently `1`. Inside the outer record, each already-encoded
generation payload is a definite-length CBOR byte string, introduced in 0.14.3.
Earlier records must be recreated; no integer-array reader or migration bridge
is supplied. Later releases retain this representation unless another hard cut
is explicitly documented.

Maintained recovery checks the stable-cell value length before reading or
allocating its buffer, then performs an allocation-free CBOR syntax preflight
before serde decoding. Current ceilings include 16,777,216 logical ledger bytes,
16,777,240 bytes per opaque payload, 33,558,528 bytes per stable-cell record,
32 nested CBOR edges and 65,536 generation records. Bounded collection visitors
and integrity checks also constrain allocation and schema histories. Writers
enforce matching limits before commit mutation and capability publication.
See the [recovery limit table](https://github.com/dragginzgame/ic-memory/blob/main/docs/key-only-recovery.md#recovery-and-admission-limits)
for each enforcement point. Oversized state fails closed and cannot be replaced
with a fresh ledger.

This ordering still matters: physical recovery selects the authoritative
committed slot before the logical envelope is decoded, and the envelope is
classified before CBOR ledger decode. The decoded DTO still is not authority.
It becomes useful for allocation authority only after the physical generation
matches the logical ledger generation, committed-integrity validation succeeds,
and `RecoveredLedger` construction succeeds.
