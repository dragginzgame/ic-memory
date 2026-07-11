# ic-memory Current Wire Fixtures

These fixtures pin the current 0.7.5 durable wire shape, unchanged from 0.7.4.

Files ending in `.hex` contain lowercase hexadecimal bytes. Tests decode the
hex into the actual durable bytes, recover or validate them, and re-encode the
current output to catch accidental wire-format drift in reviewable text form.

Intentional protocol hard cuts replace these fixtures in place. The repository
does not retain versioned decoders, compatibility fixture directories, or
legacy wire aliases.

Fixture groups:

- `*_payload_envelope.hex`: logical `LedgerPayloadEnvelope` bytes.
- `ledger_commit_store_single_active.cbor.hex`: full `LedgerCommitStore` bytes
  with one active allocation generation.
- `dual_slot_store_valid_newer.cbor.hex`: full dual-slot store where the newer
  generation is valid and authoritative.
- `dual_slot_store_corrupt_newer.cbor.hex`: full dual-slot store where the
  newer generation is corrupt and recovery must use the prior valid generation.
- `stable_cell_record.cbor.hex`: `StableCellLedgerRecord` value bytes stored
  inside the `ic-stable-structures::Cell` envelope.
- `memory_manager_descriptor.cbor.hex`: `MemoryManager` slot descriptor bytes.
