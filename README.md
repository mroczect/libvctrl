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
| [`libvctrl_core`](libvctrl_core/)           | 1.1.0   | Reference implementations (memory store, SHA‑512 hasher, binary codec). |
| [`libvctrl_plumbing`](libvctrl_plumbing/)   | 0.1.0   | Atomic version control operations (generic over contracts).             |
| [`libvctrl_porcelain`](libvctrl_porcelain/) | 0.1.0   | High‑level convenience API.                                             |
| [`libvctrl_sha512`](libvctrl_sha512/)       | 2.0.0   | SHA‑512 / HMAC / HKDF implementations used by `libvctrl_core`.          |
