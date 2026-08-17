//! # libvctrl_core
//!
//! Reference implementations for the contracts defined by
//! [`libvctrl_handler`](https://docs.rs/libvctrl_handler).
//!
//! This crate provides production-ready, safe implementations of hashing,
//! binary serialization, in-memory storage, reference management, and builder
//! utilities. It is the first concrete consumer of the `libvctrl_handler`
//! traits and serves as a quality exemplar for downstream custom backends.
//!
//! ## Architecture
//!
//! The crate is organized by domain responsibility:
//!
//! - [`codec`](crate::codec) — deterministic binary encoding and decoding.
//! - [`hash`](crate::hash) — SHA-512 content addressing.
//! - [`object`](crate::object) — ergonomic builder patterns.
//! - [`store`](crate::store) — in-memory object and reference stores.
//!
//! Each module depends only on the public contracts exposed by
//! `libvctrl_handler`, plus the SHA-512 implementation from
//! `libvctrl_sha512`. No module contains unsafe code.
//!
//! ## Safety and quality
//!
//! The crate forbids unsafe code and denies a strict set of Clippy and
//! rustc lints. Every public item is documented and has doctests where
//! applicable. The binary decoder is especially defensive: it bounds all
//! input reads, verifies version bytes, validates UTF-8, and re-checks system
//! limits before constructing any object.
//!
//! ## Example
//!
//! A common workflow encodes an object, hashes it, stores it, and retrieves
//! it through the in-memory store:
//!
//! ```
//! # use libvctrl_handler::{Blob, Encoder, Hasher, ObjectStore};
//! # use libvctrl_core::codec::BinaryEncoder;
//! # use libvctrl_core::hash::Sha512Hasher;
//! # use libvctrl_core::store::MemoryStore;
//! # use std::io::Read;
//! let blob = Blob::new(b"my content".to_vec()).unwrap();
//!
//! let mut encoded = Vec::new();
//! BinaryEncoder.encode_blob(&blob, &mut encoded).unwrap();
//!
//! let hash = Sha512Hasher.hash(&mut encoded.as_slice()).unwrap();
//!
//! let mut store = MemoryStore::new();
//! store.put(&hash, &encoded).unwrap();
//!
//! let mut reader = store.get(&hash).unwrap();
//! let mut decoded = Vec::new();
//! reader.read_to_end(&mut decoded).unwrap();
//!
//! assert_eq!(decoded, encoded);
//! ```

#[cfg(test)]
use proptest as _;

/// Binary codec for encoding and decoding objects.
///
/// This module contains the reference binary serialization format. The
/// encoder and decoder are separated to isolate trusted production of bytes
/// from untrusted parsing. See [`crate::codec`] for the module-level details.
pub mod codec;

/// Hashing algorithms.
///
/// This module bridges the pure SHA-512 implementation from
/// `libvctrl_sha512` to the [`Hasher`](libvctrl_handler::Hasher) trait.
/// The result is a content-addressing primitive that produces 64-byte hashes
/// matching `libvctrl_handler::HASH_LENGTH`.
pub mod hash;

/// Object builders for ergonomic construction.
///
/// These builders provide fluent APIs for creating blobs, commits, tags,
/// trees, and tree entries. They defer validation until the final build step,
/// allowing fields to be supplied in any order while keeping the resulting
/// objects immutable and validated.
pub mod object;

/// In-memory object and reference stores.
///
/// These stores implement the [`ObjectStore`](libvctrl_handler::ObjectStore)
/// and [`RefStore`](libvctrl_handler::RefStore) contracts using
/// [`std::collections::HashMap`]. They are ideal for tests, prototypes, and
/// short-lived embedded use cases.
pub mod store;
