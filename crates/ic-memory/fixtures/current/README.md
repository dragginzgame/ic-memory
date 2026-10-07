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

Slot encoding retains the current nested `slot` / `MemoryManagerId` shape.
Construction and decoding both reject sentinel ID 255; valid slot bytes are
unchanged by the 0.25 source API consolidation.

Current logical payload envelopes carry the `ICMEMLED` family magic followed
by the `ICMS` format marker, format version `1`, payload length, and CBOR
ledger bytes.

Logical ledgers contain a commit counter and current ownership records only.
Opaque committed payloads use definite-length CBOR byte strings. The 0.27.0
hard cut replaces the former logical layout and discriminator; retained
installations require explicit disposition before deployment. See
[the current ledger contract](https://github.com/dragginzgame/ic-memory/blob/main/docs/current-ledger.md).
The logical format version remains `1`; there is no old-shape reader.

Fixture groups:

- `*_payload_envelope.hex`: logical `LedgerPayloadEnvelope` bytes.
- `ledger_commit_store_single_active.cbor.hex`: full `LedgerCommitStore` bytes
  with one active allocation snapshot.
- `dual_slot_store_valid_newer.cbor.hex`: full dual-slot store where the newer
  generation is valid and authoritative.
- `dual_slot_store_corrupt_newer.cbor.hex`: full dual-slot store where the
  newer generation is corrupt and recovery must fail closed without rolling
  back to the prior valid generation.
- `stable_cell_record.cbor.hex`: `StableCellLedgerRecord` value bytes stored
  inside the `ic-stable-structures::Cell` envelope.
- `memory_manager_slot.cbor.hex`: checked `MemoryManagerSlot` bytes.
