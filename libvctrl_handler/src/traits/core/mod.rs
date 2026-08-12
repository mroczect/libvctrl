//! Core traits for the version control handler.
//!
//! Each submodule defines a single trait, keeping concerns separated.
//! All traits are re-exported at the crate root for convenience.

/// Defines the `Decoder` trait for deserializing objects.
pub mod decoder;

/// Defines the `Encoder` trait for serializing objects.
pub mod encoder;

/// Defines the `Hasher` trait for content addressing.
pub mod hasher;

/// Defines the `ObjectStore` trait for content-addressable storage.
pub mod object_store;

/// Defines the `RefStore` trait for named references.
pub mod ref_store;

/// Defines the `Signer` trait for cryptographic signatures.
pub mod signer;

/// Defines the `Transport` trait for remote synchronization.
pub mod transport;

/// Defines the `Verifier` trait for signature verification.
pub mod verifier;
