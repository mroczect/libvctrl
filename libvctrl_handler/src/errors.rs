use crate::types::Hash;
use std::fmt;
use std::sync::Arc;

#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum VctrlError {
    InvalidHashLength(usize),

    InvalidName(String),

    InvalidEmail(String),

    ObjectNotFound(Hash),

    RefNotFound(String),

    CorruptedData(String),

    IoError(Arc<std::io::Error>),

    SerializationError(String),

    Other(String),

    InvalidTreeStructure(String),

    InvalidTimezoneOffset(i16),

    DuplicateParent,

    ExceededMaxSize(String),

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
    #[allow(clippy::match_same_arms)]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::InvalidHashLength(a), Self::InvalidHashLength(b)) => a == b,
            (Self::InvalidName(a), Self::InvalidName(b)) => a == b,
            (Self::InvalidEmail(a), Self::InvalidEmail(b)) => a == b,
            (Self::ObjectNotFound(a), Self::ObjectNotFound(b)) => a == b,
            (Self::RefNotFound(a), Self::RefNotFound(b)) => a == b,
            (Self::CorruptedData(a), Self::CorruptedData(b)) => a == b,
            (Self::SerializationError(a), Self::SerializationError(b)) => a == b,
            (Self::Other(a), Self::Other(b)) => a == b,
            (Self::InvalidTreeStructure(a), Self::InvalidTreeStructure(b)) => a == b,
            (Self::ExceededMaxSize(a), Self::ExceededMaxSize(b)) => a == b,
            (Self::IoError(a), Self::IoError(b)) => {
                a.as_ref().kind() == b.as_ref().kind()
                    && a.as_ref().to_string() == b.as_ref().to_string()
            }
            (Self::InvalidTimezoneOffset(a), Self::InvalidTimezoneOffset(b)) => a == b,
            (Self::DuplicateParent, Self::DuplicateParent) => true,
            (Self::InvalidBlameRange, Self::InvalidBlameRange) => true,
            _ => false,
        }
    }
}

impl Eq for VctrlError {}
