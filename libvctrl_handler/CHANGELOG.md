# Changelog

All notable changes to `libvctrl_handler` will be documented in this file.

## [1.0.0] – 2026-08-08

### Added
- Fundamental contracts: `ObjectStore`, `RefStore`, `Hasher`, `Encoder`, `Decoder`, `Signer`, `Verifier`, `Transport`.
- Data types: `Hash`, `Blob`, `Tree`, `TreeEntry`, `Commit`, `Tag`, `UserID`, `EntryKind`.
- Error type: `VctrlError`.
- Constants: `HASH_LENGTH`, `MAX_NAME_LENGTH`.
- Strict lints (`forbid(unsafe_code)`, `deny(missing_docs)`, `clippy::pedantic`, `clippy::nursery`, `clippy::cargo`).
- Zero external dependencies.
- Full formal documentation with pre/postconditions and examples.
- Unit tests for `Hash` and `VctrlError`.

### Changed
- All types now enforce invariants via private fields and validated constructors.
- `ObjectStore::exists` is now fallible (`Result<bool, VctrlError>`).
- All public enums are `#[non_exhaustive]`.
- Re-export only explicit, non-wildcard items.

### Fixed
- Missing `pub mod macros;` in crate root.
- Panic risk in `Hash::Debug` if `HASH_LENGTH` < 8.

## [0.1.0] – Unreleased
- Initial concept.
