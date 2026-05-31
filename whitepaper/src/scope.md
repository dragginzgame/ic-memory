# Scope

The crate is allocation-governance infrastructure. It does not implement stable
collections, prove application schema compatibility, authorize controllers, or
perform data migrations.

Its narrower responsibility is to validate declarations against historical
allocation facts before application memory is opened.

## Native Substrate

The native substrate is the `MemoryManager` from `ic-stable-structures`. Its
usable ID domain is:

$$
S = \{0,1,\ldots,254\}
$$

ID `255` is reserved by `ic-stable-structures` as the unallocated sentinel. In
the default runtime, `ic-memory` reserves IDs `0..=9` for governance and uses ID
`0` for the allocation ledger. Lower-level bootstrap owners that bypass the
default runtime must enforce equivalent policy themselves.

## Substrate Correspondence

The allocation slot model is intentionally aligned with the current
`ic-stable-structures` `MemoryManager` implementation. The upstream manager
defines `MemoryId` as a one-byte value, uses `255` as the unallocated-bucket
marker, and stores each bucket owner as one byte in the manager metadata.

Therefore `ic-memory`'s current slot descriptor is not an arbitrary namespace.
It is the exact durable ID domain that the underlying manager can distinguish.

The same implementation reserves the first stable-memory page for manager
metadata. Its V1 header stores the magic value `MGR`, a layout version, the
allocated-bucket count, the bucket size, 32 reserved bytes, and one page-count
entry for each managed memory `0..=254`. The bucket-owner table follows the
header. With the default bucket size of 128 Wasm pages and 32,768 managed
buckets, the manager can address 256 GiB of bucket-backed stable memory through
this layout.

`ic-memory`'s default ledger anchor also relies on
`ic-stable-structures::Cell`. The relevant Cell V1 envelope begins with magic
`SCL`, a one-byte layout version, and a four-byte value length before the
encoded value. `ic-memory` preflights that envelope before opening the ledger
Cell so corrupt ledger storage is classified as a bootstrap error instead of
escaping as a decode panic.
