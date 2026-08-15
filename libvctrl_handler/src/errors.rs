//! Error types for the crate.

use crate::types::Hash;
use std::fmt;
use std::sync::Arc;

/// The main error type for all operations in this crate.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum VctrlError {
    /// The length of a hash did not match the expected length.
    InvalidHashLength(usize),
    /// A name was invalid (empty, too long, or contained control characters).
    InvalidName(String),
    /// An email address was invalid.
    InvalidEmail(String),
    /// An object with the given hash was not found.
    ObjectNotFound(Hash),
    /// A reference with the given name was not found.
    RefNotFound(String),
    /// Data was corrupted or malformed.
    CorruptedData(String),
    /// An I/O error occurred.
    IoError(Arc<std::io::Error>),
    /// A serialization/deserialization error occurred.
    SerializationError(String),
    /// Any other error not covered by the above variants.
    Other(String),
    /// The tree structure is invalid (e.g., unsorted entries, duplicates).
    InvalidTreeStructure(String),
    /// The timezone offset is out of the valid range (-1440 to 1440).
    InvalidTimezoneOffset(i16),
    /// A commit contains duplicate parent hashes.
    DuplicateParent,
    /// A size or count limit was exceeded.
    ExceededMaxSize(String),
    /// An invalid blame range was specified (e.g., zero line count).
    InvalidBlameRange,
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
            Self::InvalidEmail(msg) => write!(f, "Invalid email: '{msg}'"),
            Self::ObjectNotFound(hash) => write!(f, "Object not found: {hash}"),
            Self::RefNotFound(name) => write!(f, "Reference not found: '{name}'"),
            Self::CorruptedData(msg) => write!(f, "Corrupted data: {msg}"),
            Self::IoError(err) => write!(f, "I/O error: {}", err.as_ref()),
            Self::SerializationError(msg) => write!(f, "Serialization error: {msg}"),
            Self::Other(msg) => write!(f, "{msg}"),
            Self::InvalidTreeStructure(msg) => write!(f, "Invalid tree structure: {msg}"),
            Self::InvalidTimezoneOffset(offset) => write!(f, "Invalid timezone offset: {offset}"),
            Self::DuplicateParent => write!(f, "Duplicate parent in commit"),
            Self::ExceededMaxSize(msg) => write!(f, "Exceeded max size: {msg}"),
            Self::InvalidBlameRange => write!(f, "Invalid blame range"),
        }
    }
}

impl std::error::Error for VctrlError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IoError(e) => Some(e.as_ref()),
            _ => None,
        }
    }
}

impl PartialEq for VctrlError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::InvalidHashLength(a), Self::InvalidHashLength(b)) => a == b,

            // Gabungkan semua varian dengan payload String
            (
                Self::InvalidName(a)
                | Self::InvalidEmail(a)
                | Self::RefNotFound(a)
                | Self::CorruptedData(a)
                | Self::SerializationError(a)
                | Self::Other(a)
                | Self::InvalidTreeStructure(a)
                | Self::ExceededMaxSize(a),
                Self::InvalidName(b)
                | Self::InvalidEmail(b)
                | Self::RefNotFound(b)
                | Self::CorruptedData(b)
                | Self::SerializationError(b)
                | Self::Other(b)
                | Self::InvalidTreeStructure(b)
                | Self::ExceededMaxSize(b),
            ) => a == b,

            (Self::ObjectNotFound(a), Self::ObjectNotFound(b)) => a == b,
            (Self::IoError(a), Self::IoError(b)) => {
                a.as_ref().kind() == b.as_ref().kind()
                    && a.as_ref().to_string() == b.as_ref().to_string()
            }
            (Self::InvalidTimezoneOffset(a), Self::InvalidTimezoneOffset(b)) => a == b,

            // Gabungkan varian unit
            (Self::DuplicateParent, Self::DuplicateParent)
            | (Self::InvalidBlameRange, Self::InvalidBlameRange) => true,

            _ => false,
        }
    }
}

impl Eq for VctrlError {}
