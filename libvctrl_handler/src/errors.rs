//! Error types used throughout the crate.
//!
//! # Architecture
//! This module centralizes all error handling into a single, comprehensive [`VctrlError`] enum.
//! By using a unified error type, the crate ensures that consumers can handle failures
//! uniformly using the `?` operator across different subsystems (I/O, validation, parsing)
//! without needing to manually box or wrap disparate error types.
//!
//! # Design Rationale: `Arc<std::io::Error>`
//! The standard library's [`std::io::Error`] does not implement the `Clone` trait because
//! it may contain custom payloads that are not safely cloneable. To allow [`VctrlError`]
//! to be `Clone`, I/O errors are wrapped in an `Arc`. This provides thread-safe
//! reference counting, allowing the error to be cloned cheaply (a single atomic increment)
//! and shared across threads if necessary, while maintaining the original error's context.
//!
//! # Custom `PartialEq` Implementation
//! Because [`std::io::Error`] lacks a `PartialEq` implementation, a manual comparison is
//! provided for [`VctrlError::IoError`]. Two I/O errors are considered equal if their
//! [`std::io::Error::kind()`] and their string representations match. This heuristic
//! allows for predictable testing and equality checks without discarding the error details.
//!
//! # Examples
//! *Note: The following examples assume this crate is named `libvctrl_handler`.*
//!
//! Handling errors from I/O operations:
//!
//! ```
//! # use libvctrl_handler::VctrlError;
//! use std::io::{self, ErrorKind};
//!
//! let io_err = io::Error::new(ErrorKind::NotFound, "file missing");
//! let vctrl_err = VctrlError::from_io(io_err);
//!
//! assert!(matches!(vctrl_err, VctrlError::IoError(_)));
//! ```

use crate::constants::HASH_LENGTH;
use crate::types::Hash;
#[allow(clippy::std_instead_of_core)]
use std::error::Error;
#[allow(clippy::std_instead_of_alloc)]
use std::fmt;
use std::io;
#[allow(clippy::std_instead_of_alloc)]
use std::sync::Arc;

/// The main error type for all operations in this crate.
///
/// This enum is marked as `#[non_exhaustive]` to allow for the addition of new error
/// variants in future versions without causing breaking API changes. Consumers must
/// include a `_` catch-all arm when matching against this enum to ensure forward compatibility.
///
/// # Examples
///
/// ```
/// # use libvctrl_handler::VctrlError;
/// let err = VctrlError::InvalidName("bad name".to_string());
/// assert_eq!(err.to_string(), "Invalid name: 'bad name'");
/// ```
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum VctrlError {
    /// Data was corrupted or malformed.
    CorruptedData(String),
    /// A commit contains duplicate parent hashes.
    DuplicateParent,
    /// A size or count limit was exceeded.
    ExceededMaxSize(String),
    /// An invalid blame range was specified (e.g., zero line count).
    InvalidBlameRange,
    /// An email address was invalid.
    InvalidEmail(String),
    /// The length of a hash did not match the expected length.
    InvalidHashLength(usize),
    /// A name was invalid (empty, too long, or contained control characters).
    InvalidName(String),
    /// The timezone offset is out of the valid range (-1440 to 1440).
    InvalidTimezoneOffset(i16),
    /// The tree structure is invalid (e.g., unsorted entries, duplicates).
    InvalidTreeStructure(String),
    /// An I/O error occurred.
    IoError(Arc<io::Error>),
    /// An object with the given hash was not found.
    ObjectNotFound(Hash),
    /// Any other error not covered by the above variants.
    Other(String),
    /// A reference with the given name was not found.
    RefNotFound(String),
    /// A serialization/deserialization error occurred.
    SerializationError(String),
}

impl fmt::Display for VctrlError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::CorruptedData(ref msg) => return write!(f, "Corrupted data: {msg}"),
            Self::DuplicateParent => return write!(f, "Duplicate parent in commit"),
            Self::ExceededMaxSize(ref msg) => return write!(f, "Exceeded max size: {msg}"),
            Self::InvalidBlameRange => return write!(f, "Invalid blame range"),
            Self::InvalidEmail(ref msg) => return write!(f, "Invalid email: '{msg}'"),
            Self::InvalidHashLength(len) => {
                return write!(
                    f,
                    "Invalid hash length: expected {} bytes, got {len}",
                    HASH_LENGTH,
                );
            }
            Self::InvalidName(ref name) => return write!(f, "Invalid name: '{name}'"),
            Self::InvalidTimezoneOffset(offset) => {
                return write!(f, "Invalid timezone offset: {offset}");
            }
            Self::InvalidTreeStructure(ref msg) => {
                return write!(f, "Invalid tree structure: {msg}");
            }
            Self::IoError(ref err) => return write!(f, "I/O error: {}", err.as_ref()),
            Self::ObjectNotFound(hash) => return write!(f, "Object not found: {hash}"),
            Self::Other(ref msg) => return write!(f, "{msg}"),
            Self::RefNotFound(ref name) => return write!(f, "Reference not found: '{name}'"),
            Self::SerializationError(ref msg) => return write!(f, "Serialization error: {msg}"),
        }
    }
}

#[allow(clippy::missing_trait_methods)]
impl Error for VctrlError {
    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            Self::IoError(ref err) => return Some(err.as_ref()),
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
            | Self::SerializationError(_) => return None,
        }
    }
}

#[allow(clippy::missing_trait_methods)]
impl PartialEq for VctrlError {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::CorruptedData(a), Self::CorruptedData(b)) => return a == b,
            (Self::DuplicateParent, Self::DuplicateParent) => return true,
            (Self::ExceededMaxSize(a), Self::ExceededMaxSize(b)) => return a == b,
            (Self::InvalidBlameRange, Self::InvalidBlameRange) => return true,
            (Self::InvalidEmail(a), Self::InvalidEmail(b)) => return a == b,
            (Self::InvalidHashLength(a), Self::InvalidHashLength(b)) => return a == b,
            (Self::InvalidName(a), Self::InvalidName(b)) => return a == b,
            (Self::InvalidTimezoneOffset(a), Self::InvalidTimezoneOffset(b)) => return a == b,
            (Self::InvalidTreeStructure(a), Self::InvalidTreeStructure(b)) => return a == b,
            (Self::IoError(a), Self::IoError(b)) => {
                return a.as_ref().kind() == b.as_ref().kind()
                    && a.as_ref().to_string() == b.as_ref().to_string();
            }
            (Self::ObjectNotFound(a), Self::ObjectNotFound(b)) => return a == b,
            (Self::Other(a), Self::Other(b)) => return a == b,
            (Self::RefNotFound(a), Self::RefNotFound(b)) => return a == b,
            (Self::SerializationError(a), Self::SerializationError(b)) => return a == b,
            _ => return false,
        }
    }
}

impl Eq for VctrlError {}

impl VctrlError {
    /// Creates a [`VctrlError::IoError`] from a [`std::io::Error`].
    ///
    /// This is the canonical way to convert I/O errors within the crate,
    /// ensuring the `Arc` wrapping is applied consistently.
    ///
    /// # How it works
    /// It wraps the provided error in an `Arc`, allowing the resulting
    /// [`VctrlError`] to be cloned and shared across threads cheaply, despite
    /// [`std::io::Error`] not natively implementing `Clone`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::VctrlError;
    /// use std::io::{self, ErrorKind};
    ///
    /// let io_err = io::Error::new(ErrorKind::PermissionDenied, "access denied");
    /// let vctrl_err = VctrlError::from_io(io_err);
    ///
    /// let cloned_err = vctrl_err.clone();
    /// assert_eq!(vctrl_err, cloned_err);
    /// ```
    #[must_use]
    #[inline]
    pub fn from_io(err: io::Error) -> Self {
        return Self::IoError(Arc::new(err));
    }
}
