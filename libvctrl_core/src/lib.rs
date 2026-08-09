//! Reference implementations for the `libvctrl_handler` version control contracts.
//!
//! # Purpose
//! `libvctrl_core` provides concrete, ready-to-use implementations of the
//! abstract traits defined in [`libvctrl_handler`]. It acts as the
//! "batteries-included" layer, offering standard backends for hashing,
//! serialization, and storage, proving that the core contracts can be fully
//! realized.
//!
//! # Design rationale
//! - **Contract Fulfillment**: The crate validates the design of
//!   [`libvctrl_handler`] by building fully functional components against it.
//!   If a trait is too difficult or impossible to implement correctly, the
//!   design flaw is exposed here.
//! - **Batteries Included**: By providing standard implementations (like
//!   SHA-512 hashing and a binary wire format), downstream applications can
//!   bootstrap a functional version control system immediately without
//!   writing boilerplate logic.
//! - **Strict Safety and Linting**: Just like the handler crate, this crate
//!   forbids `unsafe` code and enforces strict Clippy lints (`pedantic`,
//!   `nursery`). This guarantees that the reference implementations are of
//!   the highest quality and serve as safe examples for future backend
//!   developers.
//!
//! # Internal mechanism
//! The crate is divided by domain responsibility:
//! - [`codec`]: Handles encoding and decoding objects to/from binary.
//! - [`hash`]: Provides cryptographic hashing (SHA-512).
//! - [`object`]: Offers ergonomic builder patterns for constructing objects.
//! - [`store`]: Implements ephemeral in-memory storage for objects and refs.
//! - [`validate`]: Supplies security and structural validation utilities.
//!
//! # Examples
//!
//! Integrating multiple components to hash, encode, and store an object:
//!
//! ```
//! use libvctrl_handler::{Blob, Encoder, Hasher, ObjectStore};
//! use libvctrl_core::codec::BinaryEncoder;
//! use libvctrl_core::hash::Sha512Hasher;
//! use libvctrl_core::store::MemoryStore;
//!
//! let blob = Blob::new(b"my content".to_vec());
//!
//! // 1. Encode the blob into bytes
//! let encoder = BinaryEncoder;
//! let encoded_bytes = encoder.encode_blob(&blob).unwrap();
//!
//! // 2. Hash the encoded bytes to get an address
//! let hasher = Sha512Hasher;
//! let hash = hasher.hash(&encoded_bytes);
//!
//! // 3. Store the encoded bytes
//! let mut store = MemoryStore::new();
//! store.put(&hash, &encoded_bytes).unwrap();
//!
//! assert!(store.exists(&hash).unwrap());
//! ```

#![forbid(unsafe_code)]
#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub,
    unused_qualifications
)]

/// Binary serialization and deserialization implementations.
///
/// # Purpose
/// Provides the [`BinaryEncoder`](crate::codec::BinaryEncoder) and
/// [`BinaryDecoder`](crate::codec::BinaryDecoder) which translate in-memory
/// objects into a compact, deterministic byte format.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::{Blob, Encoder, Decoder};
/// use libvctrl_core::codec::{BinaryEncoder, BinaryDecoder};
///
/// let blob = Blob::new(b"data".to_vec());
/// let bytes = BinaryEncoder.encode_blob(&blob).unwrap();
/// let decoded = BinaryDecoder.decode_blob(&bytes).unwrap();
/// assert_eq!(decoded, blob);
/// ```
pub mod codec;

/// Cryptographic hashing implementations.
///
/// # Purpose
/// Provides concrete [`Hasher`](libvctrl_handler::Hasher) implementations,
/// such as [`Sha512Hasher`](crate::hash::Sha512Hasher), for content addressing.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::Hasher;
/// use libvctrl_core::hash::Sha512Hasher;
///
/// let hasher = Sha512Hasher;
/// let hash = hasher.hash(b"data");
/// assert_eq!(hash.as_bytes().len(), 64);
/// ```
pub mod hash;

/// Builder patterns for constructing version control objects.
///
/// # Purpose
/// Provides fluent APIs like [`CommitBuilder`](crate::object::CommitBuilder)
/// to ergonomically assemble complex objects step-by-step.
///
/// # Examples
///
/// ```
/// use libvctrl_core::object::BlobBuilder;
///
/// let blob = BlobBuilder::new()
///     .with_data(b"hello".to_vec())
///     .build();
/// assert_eq!(blob.size(), 5);
/// ```
pub mod object;

/// Storage backend implementations.
///
/// # Purpose
/// Provides concrete [`ObjectStore`](libvctrl_handler::ObjectStore) and
/// [`RefStore`](libvctrl_handler::RefStore) implementations, such as
/// [`MemoryStore`](crate::store::MemoryStore), for persisting data in RAM.
///
/// # Examples
///
/// ```
/// use libvctrl_core::store::MemoryStore;
/// use libvctrl_handler::{Hash, ObjectStore};
///
/// let mut store = MemoryStore::new();
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// store.put(&hash, b"data").unwrap();
/// assert!(store.exists(&hash).unwrap());
/// ```
pub mod store;

/// Validation utilities for structural integrity and security.
///
/// # Purpose
/// Provides helper functions to validate raw inputs (like names and hashes)
/// before they are turned into strongly-typed objects, preventing path
/// traversal and resource exhaustion.
///
/// # Examples
///
/// ```
/// use libvctrl_core::validate::name::validate_name;
///
/// assert!(validate_name("valid_name").is_ok());
/// assert!(validate_name("../invalid").is_err());
/// ```
pub mod validate;
