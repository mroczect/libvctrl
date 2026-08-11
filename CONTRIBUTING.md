# Contributing to libvctrl

Thank you for your interest in contributing to **libvctrl**! This document outlines the rules, workflows, and standards we follow to keep the project consistent and maintainable. Please read it carefully before opening an issue or a pull request.

---

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Project Philosophy](#project-philosophy)
- [How Can I Contribute?](#how-can-i-contribute)
  - [Reporting Bugs](#reporting-bugs)
  - [Suggesting Features](#suggesting-features)
  - [Your First Code Contribution](#your-first-code-contribution)
- [Development Setup](#development-setup)
  - [Prerequisites](#prerequisites)
  - [Building the Workspace](#building-the-workspace)
  - [Running Tests](#running-tests)
  - [Linting & Formatting](#linting--formatting)
- [Development Workflow](#development-workflow)
  - [Branching Model](#branching-model)
  - [Commit Messages](#commit-messages)
  - [Pull Requests](#pull-requests)
  - [Code Review](#code-review)
- [Code Style](#code-style)
  - [Rust Conventions](#rust-conventions)
  - [Documentation](#documentation)
  - [Error Handling](#error-handling)
- [Testing Guidelines](#testing-guidelines)
  - [Unit Tests](#unit-tests)
  - [Integration Tests](#integration-tests)
  - [Doctests](#doctests)
  - [Property-Based Tests](#property-based-tests)
- [Release Process](#release-process)
  - [Versioning](#versioning)
  - [Publishing Crates](#publishing-crates)
- [Security](#security)
- [Recognition](#recognition)

---

## Code of Conduct

We follow the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report unacceptable behavior to the maintainers.

---

## Project Philosophy

**libvctrl** is a **precision toolkit** for building custom version control systems. It provides the fundamental contracts (types, traits) and reference implementations, enabling you to construct a VCS that fits your needs exactly – whether it's for an embedded system, a game engine, or a document store.

We value:

- **Correctness** over convenience
- **Safety** (no `unsafe` unless absolutely necessary and well‑justified)
- **Explicit** interfaces (traits are the primary abstraction)
- **Testability** (every feature must be testable with minimal setup)
- **Documentation** (every public item must have a docstring)

---

## How Can I Contribute?

### Reporting Bugs

1. Check the [issue tracker](https://github.com/mroczect/libvctrl/issues) to see if the bug has already been reported.
2. If not, [open a new issue](https://github.com/mroczect/libvctrl/issues/new) and include:
   - A clear, descriptive title.
   - Steps to reproduce the bug.
   - Expected behaviour vs. actual behaviour.
   - Rust version (`rustc --version`), platform, and relevant crate versions.
   - A minimal code example or test case, if possible.

### Suggesting Features

- Open an issue with the label `enhancement`.
- Explain the use case, why it is important, and how it fits into the project philosophy.
- If you already have a design in mind, describe it – but be open to discussion.

### Your First Code Contribution

1. Look for issues labelled `good first issue` or `help wanted`.
2. Comment on the issue to indicate you are working on it, or ask for guidance.
3. Follow the [Development Setup](#development-setup) and [Workflow](#development-workflow) sections below.

---

## Development Setup

### Prerequisites

- Rust **stable** (install via `rustup`)
- `cargo`, `rustfmt`, and `clippy` (included with `rustup`)
- `jq` and `curl` (for the publish script, optional)
- `gh` CLI (optional, for GitHub Actions)

### Building the Workspace

```bash
git clone https://github.com/mroczect/libvctrl.git
cd libvctrl
make ci          # runs format check, clippy, and all tests
```

The workspace contains several crates:

| Crate                | Description                                      |
| -------------------- | ------------------------------------------------ |
| `libvctrl_handler`   | Contracts – types, traits, errors                |
| `libvctrl_core`      | Reference implementations (store, codec, hasher) |
| `libvctrl_sha512`    | Pure-Rust SHA-512 / HMAC / HKDF                  |
| `libvctrl`           | Re-export umbrella crate                         |
| `libvctrl_plumbing`  | Plumbing commands (e.g., cat-file)               |
| `libvctrl_porcelain` | Porcelain commands (future)                      |

### Running Tests

```bash
make test-verbose   # runs all tests with RUST_BACKTRACE=1
```

Target a single crate:

```bash
cargo test -p libvctrl_handler
```

### Linting & Formatting

```bash
make fmt            # formats all crates
make handler        # checks & lints libvctrl_handler specifically
make core           # checks & lints libvctrl_core specifically
make plumbing       # checks & lints libvctrl_plumbing specifically
make ci             # run everything
```

---

## Development Workflow

### Branching Model

- `master` (or `main`) is the stable branch. All releases are tagged from here.
- Create feature branches from `master`:

```bash
git checkout -b feature/my-feature
```

- For fixes:

```bash
git checkout -b fix/my-fix
```

- Keep branches small and focused. Merge back via pull request.

### Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/) format:

```
<type>(<scope>): <short summary>

[optional body]

[optional footer(s)]
```

Examples:

- `feat(handler): add fallible Hasher trait`
- `fix(core): update Sha512Hasher to new Hasher signature`
- `docs(libvctrl): update root-level examples`

Allowed types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`, `ci`, `build`.

Scopes are optional but encouraged (crate name, module, etc.).

### Pull Requests

1. Push your branch.
2. Open a PR against `master`.
3. Fill in the PR template (title, description, related issues).
4. Ensure CI passes (format, clippy, tests).
5. Assign one or more reviewers.
6. Once approved, the PR will be merged (squash‑merged by default).

**Important:** Due to branch protection rules, merging is blocked until all required status checks are green.

### Code Review

- Be respectful and constructive.
- Focus on correctness, safety, and testability.
- If you need more time, use the "draft" state.
- Address feedback promptly.

---

## Code Style

### Rust Conventions

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).
- Use `forbid(unsafe_code)` – any `unsafe` must be explicitly justified and reviewed.
- Adhere to the strict Clippy rules enforced in each crate (see `Cargo.toml` lint sections).
- No `unwrap()` or `expect()` outside of doctests and examples unless the invariant is guaranteed by context (use `Result` propagation instead).
- Run `cargo fmt` before every commit.

### Documentation

- Every **public** item (module, struct, enum, trait, function) must have a doc comment (`///` or `//!`).
- Use `# Examples` sections that compile as doctests.
- Link to relevant types, traits, and modules with intra‑doc links (`[`Type`]`).
- Document error conditions, panics, and safety invariants.

### Error Handling

- The unified error type is `VctrlError` (defined in `libvctrl_handler`).
- All fallible functions return `Result<T, VctrlError>`.
- Do **not** panic on recoverable errors.
- When adding a new error variant that carries a `String`, update the `string_payload_variants!` macro invocation in `PartialEq`.

---

## Testing Guidelines

### Unit Tests

- Place `#[cfg(test)] mod tests { ... }` inside the same file.
- Test private functions where appropriate.
- Mock traits using simple structs rather than heavy frameworks.

### Integration Tests

- Located in `tests/` directories of each crate.
- Test public APIs end‑to‑end.
- Use the in‑memory store (`MemoryStore`) and `BinaryEncoder`/`BinaryDecoder` for fast, deterministic tests.

### Doctests

- Every code block in documentation is compiled and run.
- Ensure doctests do not panic; return `Result` or handle errors gracefully.
- Use `# ` hidden lines to set up test context without showing it in the docs.

### Property-Based Tests

- We use [`proptest`](https://crates.io/crates/proptest) for round‑trip and fuzz tests.
- See `libvctrl_core/tests/proptest_codec.rs` for examples.
- Add proptest for any new encoder/decoder or serialization logic.

---

## Release Process

### Versioning

All crates follow [Semantic Versioning](https://semver.org/):

- **Major** – breaking API change.
- **Minor** – new feature, backward‑compatible.
- **Patch** – bug fix, no API change.

Version numbers are set in each crate’s `Cargo.toml`.

### Publishing Crates

1. **Local verification**: Run `make ci` and ensure everything passes.
2. **Prepare the release**: Use the script `scripts/publish_crates.sh` (or create tags manually in the format `crate@version`, e.g., `libvctrl_handler@4.1.0`).
3. **Push the tag** – the CI workflow `.github/workflows/publish.yml` will automatically:
   - Verify the version matches `Cargo.toml`.
   - Run `cargo test` and `cargo clippy`.
   - Publish to crates.io.
   - Create a GitHub release.

**Only maintainers with publishing rights** should trigger releases.

---

## Security

- All crates use `#![forbid(unsafe_code)]`.
- Path‑traversal vulnerabilities are prevented at the handler level (`validate_tree_entry_name`).
- Do not introduce unsafe dependencies without prior discussion.
- If you discover a security issue, please **do not** open a public issue. Instead, email the maintainer directly.

---

## Recognition

All contributors will be acknowledged in the project’s documentation and release notes. Significant contributions may also be mentioned in the project’s README.

Thank you for helping make **libvctrl** better!
