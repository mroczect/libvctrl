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
//! to be `Clone`, I/O errors are wrapped in an [`std::sync::Arc`]. This provides thread-safe
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

use crate::types::Hash;
use std::fmt;
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
    /// The length of a hash did not match the expected length.
    ///
    /// Contains the invalid length provided.
    InvalidHashLength(usize),
    /// A name was invalid (empty, too long, or contained control characters).
    ///
    /// Contains the invalid name string.
    InvalidName(String),
    /// An email address was invalid.
    ///
    /// Contains a description of why the email was invalid.
    InvalidEmail(String),
    /// An object with the given hash was not found.
    ///
    /// Contains the [`Hash`] that was queried but missing.
    ObjectNotFound(Hash),
    /// A reference with the given name was not found.
    ///
    /// Contains the reference name that was missing.
    RefNotFound(String),
    /// Data was corrupted or malformed.
    ///
    /// Contains a message describing the corruption.
    CorruptedData(String),
    /// An I/O error occurred.
    ///
    /// This variant wraps the underlying [`std::io::Error`] in an [`std::sync::Arc`]
    /// to enable `Clone` semantics, as raw I/O errors are not cloneable.
    IoError(Arc<std::io::Error>),
    /// A serialization/deserialization error occurred.
    ///
    /// Contains a message detailing the serialization failure.
    SerializationError(String),
    /// Any other error not covered by the above variants.
    ///
    /// Contains a custom error message.
    Other(String),
    /// The tree structure is invalid (e.g., unsorted entries, duplicates).
    ///
    /// Contains a message explaining the structural violation.
    InvalidTreeStructure(String),
    /// The timezone offset is out of the valid range (-1440 to 1440).
    ///
    /// Contains the invalid timezone offset in minutes.
    InvalidTimezoneOffset(i16),
    /// A commit contains duplicate parent hashes.
    DuplicateParent,
    /// A size or count limit was exceeded.
    ///
    /// Contains a message specifying what limit was exceeded.
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
    /// Provides access to the underlying error source.
    ///
    /// Currently, only [`VctrlError::IoError`] exposes a source (the underlying
    /// [`std::io::Error`]). All other variants return `None` as they encapsulate
    /// standalone error conditions without chained contexts.
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

impl VctrlError {
    /// Creates a [`VctrlError::IoError`] from a [`std::io::Error`].
    ///
    /// This is the canonical way to convert I/O errors within the crate,
    /// ensuring the `Arc` wrapping is applied consistently.
    ///
    /// # How it works
    /// It wraps the provided error in an [`std::sync::Arc`], allowing the resulting
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
    pub fn from_io(e: std::io::Error) -> Self {
        Self::IoError(Arc::new(e))
    }
}
