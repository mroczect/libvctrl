# libvctrl

A precision toolkit for building custom version control systems.

## Philosophy

- **Mechanism, not policy** – no assumptions about branches, workflows, or defaults.
- **Unbounded flexibility, high discipline** – everything is generic and replaceable,
  but every input is strictly validated.

## Crates

| Crate                                       | Description                                                             |
| ------------------------------------------- | ----------------------------------------------------------------------- |
| [`libvctrl_handler`](libvctrl_handler/)     | Fundamental contracts – traits, types, errors. No implementations.      |
| [`libvctrl_core`](libvctrl_core/)           | Reference implementations (memory store, SHA-512 hasher, binary codec). |
| [`libvctrl_plumbing`](libvctrl_plumbing/)   | Atomic version control operations (generic over contracts).             |
| [`libvctrl_porcelain`](libvctrl_porcelain/) | High-level convenience API.                                             |

## License

MIT
