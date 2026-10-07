<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-memory/ic-memory-readme-header.svg" alt="IC Memory — Stops upgrades from mixing up stored data" width="100%">
</p>

# ic-memory

Durable stable-memory allocation governance for Internet Computer canisters.
It keeps allocation ownership explicit and validates stable-memory geometry so
upgrades cannot silently mix up stored data.

[Package guide and quick start](crates/ic-memory/README.md) ·
[Advanced integration](ADVANCED.md) · [Safety contract](SAFETY.md) ·
[Documentation](docs/README.md)

The virtual root workspace owns the library's metadata, dependencies and
`Cargo.lock`; the package lives in `crates/ic-memory/`. Run the existing Make
and Cargo commands from this repository root. The independent installed-runtime
qualification workspace remains at `testing/runtime-qualification/` with its
own selected lockfile.

See [host setup](docs/host-support.md) for prerequisites and
[release preparation](RELEASING.md) for maintainer commands.
