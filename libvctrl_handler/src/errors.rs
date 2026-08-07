//! The unified error type for the entire `libvctrl` ecosystem.
//!
//! Every fallible operation must return [`Result<T, VctrlError>`](Result).
//! This guarantees that errors are explicit and cannot be silently ignored.

use crate::types::Hash;
use std::fmt;

/// Represents every possible error that can occur within `libvctrl`.
///
/// # Design
/// This enum is exhaustive and covers validation failures, storage errors,
/// data corruption, I/O issues, and more. Variants like [`IoError`](Self::IoError)
/// store the message as a `String` to keep the type object‑safe and
/// independent of platform‑specific error types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VctrlError {
    /// The given hash does not have the required length ([`HASH_LENGTH`](crate::HASH_LENGTH)).
    InvalidHashLength(usize),
    /// The provided name is invalid (empty, too long, or contains forbidden characters).
    InvalidName(String),
    /// No object with the given hash exists in the object store.
    ObjectNotFound(Hash),
    /// No reference with the given name exists in the reference store.
    RefNotFound(String),
    /// Data read from storage is corrupted or does not conform to the expected format.
    CorruptedData(String),
    /// An I/O error occurred. The string describes the problem.
    IoError(String),
    /// Serialization or deserialization failed.
    SerializationError(String),
    /// A fallback variant for errors that do not fit the other categories.
    Other(String),
}

impl fmt::Display for VctrlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHashLength(len) => write!(
                f,
                "Invalid hash length: expected {} bytes, got {len}",
                crate::constants::HASH_LENGTH,
            ),
            Self::InvalidName(name) => write!(f, "Invalid name: '{name}'"),
            Self::ObjectNotFound(hash) => write!(f, "Object not found: {hash}"),
            Self::RefNotFound(name) => write!(f, "Reference not found: '{name}'"),
            Self::CorruptedData(msg) => write!(f, "Corrupted data: {msg}"),
            Self::IoError(msg) => write!(f, "I/O error: {msg}"),
            Self::SerializationError(msg) => write!(f, "Serialization error: {msg}"),
            Self::Other(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for VctrlError {}
