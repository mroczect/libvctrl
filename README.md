# libvctrl

A precision toolkit for building custom version control systems.

## Philosophy

- **Mechanism, not policy** – no assumptions about branches, workflows, or defaults.
- **Unbounded flexibility, high discipline** – everything is generic and replaceable,
  but every input is strictly validated.
- **Streaming & memory‑safe** – large objects are accessed via `std::io::Read`,
  reference listings use lazy iterators.
- **Full POSIX fidelity** – tree entries distinguish regular files, executables,
  symlinks, subdirectories, and submodules.

## Crates

| Crate                                       | Version | Description                                                             |
| ------------------------------------------- | ------- | ----------------------------------------------------------------------- |
| [`libvctrl_handler`](libvctrl_handler/)     | 4.0.0   | Fundamental contracts – traits, types, errors. **No implementations.**  |
| [`libvctrl_core`](libvctrl_core/)           | 2.0.0   | Reference implementations (memory store, SHA‑512 hasher, binary codec). |
| [`libvctrl_plumbing`](libvctrl_plumbing/)   | 0.1.0   | Atomic version control operations (generic over contracts).             |
| [`libvctrl_porcelain`](libvctrl_porcelain/) | 0.1.0   | High‑level convenience API.                                             |
| [`libvctrl_sha512`](libvctrl_sha512/)       | 2.0.0   | SHA‑512 / HMAC / HKDF implementations used by `libvctrl_core`.          |
| [`libvctrl`](libvctrl/)                     | 0.1.0   | Facade crate re‑exporting all of the above into a single namespace.     |

## Quick Start

Add the contracts crate to your `Cargo.toml` if you only need the types and traits:

```toml
[dependencies]
libvctrl_handler = "4.0"
```

For the full SDK (contracts + reference implementations + cryptographic hashing), add:

```toml
[dependencies]
libvctrl = "0.1"
```

Then use it in your code:

```rust
use libvctrl::{
    Blob, Commit, Tree, Hash, EntryKind,
    ObjectStore, MemoryStore,
    BinaryEncoder, BinaryDecoder,
    Sha512Hasher,
    VctrlError,
};
```

## Status

- **`libvctrl_handler` (v4.0.0)** – Stable. All 144 doctests and 20 unit tests pass.  
  Breaking changes from v3.x include streaming object reads, iterator‑based ref listings, `&mut self` for signing, full POSIX `EntryKind`, and platform‑independent `u64` constants.

- **`libvctrl_core` (v2.0.0)** – Stable. All 13 unit tests, 40 integration tests, and 8 proptest fuzz tests pass.  
  Fully compatible with `libvctrl_handler` v4.0.0. Provides in‑memory stores, binary codec, SHA‑512 hasher, and builders.

- **`libvctrl_sha512` (v2.0.0)** – Stable. Pure‑Rust SHA‑512, HMAC, HKDF.

- **`libvctrl_plumbing` / `libvctrl_porcelain`** – Placeholder crates, not yet implemented.

- **`libvctrl`** (facade) – Re‑exports the above crates into a single, convenient namespace.

## Documentation

Full API documentation is available at [docs.rs](https://docs.rs) for each crate.  
For local documentation, run:

```bash
cargo doc --no-deps --open
```

## License

MIT – see the [LICENSE](LICENSE) file.
