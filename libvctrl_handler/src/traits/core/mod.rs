//! Core traits for repository operations.

/// Blame computation trait.
pub mod blame;
/// Configuration store trait.
pub mod config;
/// Object decoder trait.
pub mod decoder;
/// Tree differencing trait.
pub mod diff;
/// Object encoder trait.
pub mod encoder;
/// Hashing trait.
pub mod hasher;
/// Index (staging area) trait.
pub mod index;
/// Object storage trait.
pub mod object_store;
/// Pack file reader/writer traits.
pub mod pack;
/// Reference store trait.
pub mod ref_store;
/// Reflog store trait.
pub mod reflog;
/// Remote repository trait.
pub mod remote;
/// Revision walking trait.
pub mod revwalk;
/// Signing trait.
pub mod signer;
/// Transport trait.
pub mod transport;
/// Verification trait.
pub mod verifier;
