



































use crate::constants::HASH_LENGTH;
use crate::types::Hash;
use std::error::Error;
use std::fmt;
use std::io;
use std::sync::Arc;














#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum VctrlError {
    
    CorruptedData(String),
    
    DuplicateParent,
    
    ExceededMaxSize(String),
    
    InvalidBlameRange,
    
    InvalidEmail(String),
    
    InvalidHashLength(usize),
    
    InvalidName(String),
    
    InvalidTimezoneOffset(i16),
    
    InvalidTreeStructure(String),
    
    IoError(Arc<io::Error>),
    
    ObjectNotFound(Hash),
    
    Other(String),
    
    RefNotFound(String),
    
    SerializationError(String),
}

impl fmt::Display for VctrlError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CorruptedData(msg) => write!(f, "Corrupted data: {msg}"),
            Self::DuplicateParent => write!(f, "Duplicate parent in commit"),
            Self::ExceededMaxSize(msg) => write!(f, "Exceeded max size: {msg}"),
            Self::InvalidBlameRange => write!(f, "Invalid blame range"),
            Self::InvalidEmail(msg) => write!(f, "Invalid email: '{msg}'"),
            Self::InvalidHashLength(len) => {
                write!(
                    f,
                    "Invalid hash length: expected {HASH_LENGTH} bytes, got {len}"
                )
            }
            Self::InvalidName(name) => write!(f, "Invalid name: '{name}'"),
            Self::InvalidTimezoneOffset(offset) => {
                write!(f, "Invalid timezone offset: {offset}")
            }
            Self::InvalidTreeStructure(msg) => write!(f, "Invalid tree structure: {msg}"),
            Self::IoError(err) => write!(f, "I/O error: {}", err.as_ref()),
            Self::ObjectNotFound(hash) => write!(f, "Object not found: {hash}"),
            Self::Other(msg) => write!(f, "{msg}"),
            Self::RefNotFound(name) => write!(f, "Reference not found: '{name}'"),
            Self::SerializationError(msg) => write!(f, "Serialization error: {msg}"),
        }
    }
}

impl Error for VctrlError {
    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::IoError(err) => Some(err.as_ref()),
            Self::CorruptedData(_)
            | Self::DuplicateParent
            | Self::ExceededMaxSize(_)
            | Self::InvalidBlameRange
            | Self::InvalidEmail(_)
            | Self::InvalidHashLength(_)
            | Self::InvalidName(_)
            | Self::InvalidTimezoneOffset(_)
            | Self::InvalidTreeStructure(_)
            | Self::ObjectNotFound(_)
            | Self::Other(_)
            | Self::RefNotFound(_)
            | Self::SerializationError(_) => None,
        }
    }
}

impl PartialEq for VctrlError {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::IoError(a), Self::IoError(b)) => {
                a.as_ref().kind() == b.as_ref().kind()
                    && a.as_ref().to_string() == b.as_ref().to_string()
            }
            (Self::DuplicateParent, Self::DuplicateParent)
            | (Self::InvalidBlameRange, Self::InvalidBlameRange) => true,
            (
                Self::CorruptedData(a)
                | Self::ExceededMaxSize(a)
                | Self::InvalidEmail(a)
                | Self::InvalidName(a)
                | Self::InvalidTreeStructure(a)
                | Self::Other(a)
                | Self::RefNotFound(a)
                | Self::SerializationError(a),
                Self::CorruptedData(b)
                | Self::ExceededMaxSize(b)
                | Self::InvalidEmail(b)
                | Self::InvalidName(b)
                | Self::InvalidTreeStructure(b)
                | Self::Other(b)
                | Self::RefNotFound(b)
                | Self::SerializationError(b),
            ) => a == b,
            (Self::InvalidHashLength(a), Self::InvalidHashLength(b)) => a == b,
            (Self::InvalidTimezoneOffset(a), Self::InvalidTimezoneOffset(b)) => a == b,
            (Self::ObjectNotFound(a), Self::ObjectNotFound(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for VctrlError {}

impl From<io::Error> for VctrlError {
    #[inline]
    fn from(err: io::Error) -> Self {
        Self::IoError(Arc::new(err))
    }
}

impl VctrlError {
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    #[must_use]
    #[inline]
    pub fn from_io(err: io::Error) -> Self {
        Self::IoError(Arc::new(err))
    }
}
