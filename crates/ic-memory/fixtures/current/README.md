<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-memory/ic-memory-readme-header.svg" alt="IC Memory — Stops upgrades from mixing up stored data" width="100%">
</p>

# ic-memory Current Wire Fixtures

These fixtures pin the current durable wire shape.

Files ending in `.hex` contain lowercase hexadecimal bytes. Tests decode the
hex into the actual durable bytes, recover or validate them, and re-encode the
current output to catch accidental wire-format drift in reviewable text form.

Intentional protocol hard cuts replace these fixtures in place. The repository
contains exactly one current fixture set and one current decoder path.

Slots encode their numeric ID directly. Construction and decoding both reject
sentinel ID 255. The 0.41.0 hard cut replaces the former nested representation.

The current `ICMSNAP1` physical snapshot contains the logical ledger directly as
a bounded definite-length CBOR byte string. Its marker, counter, payload length
and bytes are checksum-bound. There is no additional logical envelope header.
Logical ledgers contain a counter and key/slot/state records only; store schema
metadata is application-owned.

Earlier images are retired by the maintainer-authorized 0.45.0 schema-free cut.
Fresh/reset memory is required; incompatible bytes fail without automatic
clearing. See [the current ledger contract](../../../../docs/current-ledger.md).
There is one current fixture set and no earlier-format reader.

Fixture groups:

- `*_ledger.cbor.hex`: direct logical `AllocationLedger` CBOR bytes.
- `ledger_commit_store_single_active.cbor.hex`: full `LedgerCommitStore` bytes
  with one active allocation snapshot, also used directly as the Cell value.
- `snapshot_store_valid.cbor.hex`: one valid current physical snapshot.
- `snapshot_store_corrupt.cbor.hex`: one corrupt current snapshot; recovery
  refuses it without initializing or publishing allocations.
- `memory_manager_slot.cbor.hex`: checked `MemoryManagerSlot` bytes.

The 0.45.0 cut retains one required `current` snapshot field, uses the distinct
`ICMSNAP1` marker, removes stored schema fields and regenerates payloads/checksums.
The Cell value is the commit store itself, without another record wrapper. Numeric checked slots remain unchanged. Manager and Cell
layouts retain the pinned substrate contract. No previous payload or automatic
reset path is retained.
