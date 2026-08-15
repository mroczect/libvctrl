use crate::string_payload_variants;
use crate::types::Hash;
use std::fmt;

#[non_exhaustive]
#[derive(Debug)]
pub enum VctrlError {
    InvalidHashLength(usize),

    InvalidName(String),

    InvalidEmail(String),

    ObjectNotFound(Hash),

    RefNotFound(String),

    CorruptedData(String),

    IoError(std::io::Error),

    SerializationError(String),

    Other(String),
}

impl Clone for VctrlError {
    fn clone(&self) -> Self {
        match self {
            Self::InvalidHashLength(v) => Self::InvalidHashLength(*v),
            Self::InvalidName(v) => Self::InvalidName(v.clone()),
            Self::InvalidEmail(v) => Self::InvalidEmail(v.clone()),
            Self::ObjectNotFound(v) => Self::ObjectNotFound(*v),
            Self::RefNotFound(v) => Self::RefNotFound(v.clone()),
            Self::CorruptedData(v) => Self::CorruptedData(v.clone()),
            Self::IoError(e) => Self::IoError(std::io::Error::new(e.kind(), e.to_string())),
            Self::SerializationError(v) => Self::SerializationError(v.clone()),
            Self::Other(v) => Self::Other(v.clone()),
        }
    }
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
            Self::IoError(err) => write!(f, "I/O error: {err}"),
            Self::SerializationError(msg) => write!(f, "Serialization error: {msg}"),
            Self::Other(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for VctrlError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl PartialEq for VctrlError {
    fn eq(&self, other: &Self) -> bool {
        string_payload_variants!(
            InvalidName,
            InvalidEmail,
            RefNotFound,
            CorruptedData,
            SerializationError,
            Other
        );

        match (self, other) {
            (Self::InvalidHashLength(a), Self::InvalidHashLength(b)) => a == b,
            (Self::ObjectNotFound(a), Self::ObjectNotFound(b)) => a == b,
            (Self::IoError(a), Self::IoError(b)) => {
                a.kind() == b.kind() && a.to_string() == b.to_string()
            }
            _ => match (string_payload(self), string_payload(other)) {
                (Some(s1), Some(s2)) => s1 == s2,
                _ => false,
            },
        }
    }
}

impl Eq for VctrlError {}
