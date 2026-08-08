//!   `String` so that the handler crate remains `#![no_std]` compatible (when
//! - **Forward‑compatible** – the `#[non_exhaustive]` attribute and the
use crate::types::Hash;
use std::fmt;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VctrlError {
    InvalidHashLength(usize),

    InvalidName(String),

    ObjectNotFound(Hash),

    RefNotFound(String),

    CorruptedData(String),

    IoError(String),

    SerializationError(String),

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

impl std::error::Error for VctrlError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
