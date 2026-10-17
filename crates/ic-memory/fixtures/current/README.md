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

The current `ICMKEY01` physical snapshot contains a bounded definite-length CBOR
byte string whose payload is the allocation-record array. The header alone owns
the generation. Its marker, generation, payload length and bytes are checksum-bound;
recovery constructs the logical ledger using that checked generation. Record
fields are permanent key and numeric slot only.

Earlier images, including ICMSNAP1 and pending ICMREC01, are retired by the maintainer-authorized
0.48.0 cut. Fresh/reset memory is required; incompatible bytes fail without
automatic clearing. See [the current ledger contract](../../../../docs/current-ledger.md).
There is one current fixture set and no earlier-format reader.

Fixture groups:

- `*_ledger.cbor.hex`: record-array payload bytes; generation is supplied by the
  enclosing protected snapshot, or explicitly by the payload fixture test.
- `ledger_commit_store_single.cbor.hex`: full `LedgerCommitStore` bytes
  with one allocation snapshot, also used directly as the Cell value.
- `snapshot_store_valid.cbor.hex`: one valid current physical snapshot.
- `snapshot_store_corrupt.cbor.hex`: one corrupt current snapshot; recovery
  refuses it without initializing or publishing allocations.
- `memory_manager_slot.cbor.hex`: checked `MemoryManagerSlot` bytes.

Fixtures are regenerated through the maintained bounded payload and commit-store
encoders. The Cell value remains the commit store itself. Numeric checked slots,
manager and Cell layouts retain the pinned substrate contract. No previous
payload decoder or automatic reset path is retained.
