//! Reference implementations of the libvctrl contracts (in-memory store, SHA-512 hasher, binary codec).

#![forbid(unsafe_code)]
#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub,
    unused_crate_dependencies,
    unused_qualifications
)]

#[cfg(test)]
use proptest as _;

/// Binary codec for encoding and decoding objects.
pub mod codec;

/// Hashing algorithms.
pub mod hash;

/// Object builders for ergonomic construction.
pub mod object;

/// In-memory object and reference stores.
pub mod store;

/// Validation helpers for names and hashes.
pub mod validate;
