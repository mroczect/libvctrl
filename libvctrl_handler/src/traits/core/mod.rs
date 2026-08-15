//! Core traits for repository operations.

/// Decoder traits.
pub mod decoder;

/// Encoder traits.
pub mod encoder;

/// Hasher traits.
pub mod hasher;

/// Object store traits.
pub mod object_store;

/// Reference store traits.
pub mod ref_store;

/// Revision walking traits.
pub mod revwalk;

/// Signing traits.
pub mod signer;

/// Transport traits.
pub mod transport;

/// Verification traits.
pub mod verifier;

/// Blame traits.
pub mod blame;

/// Configuration traits.
pub mod config;

/// Difference traits.
pub mod diff;

/// Index traits.
pub mod index;

/// Pack file traits.
pub mod pack;

/// Reflog traits.
pub mod reflog;

/// Remote traits.
pub mod remote;
