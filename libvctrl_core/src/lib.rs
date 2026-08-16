//! Reference implementations of the libvctrl contracts6 contracts (in-memory store, SHA-512 hasher, binary codec).

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
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::missing_safety_doc,
    clippy::must_use_candidate
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
