//! Cryptographic hash function implementations.
//!
//! This module provides concrete implementations of the [`Hasher`] trait
//! from `libvctrl_handler`. Each hasher is a stateless value that produces
//! a 64‑byte [`Hash`] for any input data.
//!
//! # Why a separate module for hashing?
//!
//! Hashing is the **foundation** of content‑addressable storage. Every object
//! in a `libvctrl` repository is identified by its hash. This means the
//! choice of hash function has profound implications for security,
//! performance, and interoperability. By isolating the hasher in its own
//! module, we make it easy to:
//!
//! - Swap implementations (e.g., SHA‑256, BLAKE3) without touching other code.
//! - Benchmark different algorithms on a per‑application basis.
//! - Audit the hashing code independently of the rest of the system.
//!
//! # Available hashers
//!
//! | Hasher | Algorithm | Output size | Crate |
//! |---|---|---|---|
//! | [`Sha512Hasher`] | SHA‑512 | 64 bytes | `libvctrl_sha512` |
//!
//! Additional hashers (SHA‑256, BLAKE3, etc.) may be added in the future as
//! separate crates.
//!
//! # Writing your own hasher
//!
//! To provide a custom hash function, implement the [`Hasher`] trait:
//!
//! ```rust
//! use libvctrl_handler::{Hash, Hasher, HASH_LENGTH};
//!
//! struct MyHasher;
//!
//! impl Hasher for MyHasher {
//!     fn hash(&self, data: &[u8]) -> Hash {
//!         // Compute the digest and convert it to a Hash.
//!         // The digest MUST be exactly HASH_LENGTH bytes.
//!         let digest = my_hash_function(data);
//!         Hash::from_bytes(&digest).expect("must produce 64 bytes")
//!     }
//! }
//! ```
//!
//! # Security considerations
//!
//! - **Collision resistance** – The hasher must make it computationally
//!   infeasible to find two different inputs with the same hash.
//! - **Preimage resistance** – Given a hash, it must be infeasible to find
//!   an input that hashes to it.
//! - **Determinism** – The same input must always produce the same hash.
//!
//! The provided [`Sha512Hasher`] meets all these requirements and has been
//! audited as part of the `libvctrl_sha512` crate.

pub mod sha512;
pub use sha512::Sha512Hasher;
