//! Error handling for the `libvctrl_handler` version control contracts.
//!
//! # Purpose
//!
//! This module defines [`VctrlError`], the unified error type returned by all
//! fallible operations within the crate. It encapsulates various failure modes
//! ranging from invalid input data to storage and serialization failures. Every
//! public API that can fail returns a [`Result<T, VctrlError>`], enabling
//! callers to match on specific variants or propagate errors upward.
//!
//! # Design Rationale
//!
//! - **Error chain preservation**: The [`IoError`](VctrlError::IoError) variant
//!   stores the original [`std::io::Error`], and the implementation of
//!   [`std::error::Error::source`] returns it, preserving the causal chain.
//!   This enables full interoperability with error-reporting crates like
//!   `anyhow` and `eyre`, and allows programmatic matching on
//!   [`std::io::ErrorKind`].
//! - **Cloning capability**: A manual [`Clone`] implementation reconstructs the
//!   I/O error from its kind and message, ensuring the error type remains
//!   clonable for testing and state comparison without requiring
//!   `std::io::Error` itself to be [`Clone`].
//! - **Forward Compatibility**: The enum is marked `#[non_exhaustive]`. This
//!   prevents downstream crates from exhaustively matching against it,
//!   allowing new error variants to be added in future minor versions without
//!   breaking the API.
//! - **`no_std` Readiness**: By avoiding heap-allocated trait objects for
//!   non-I/O variants and relying on plain data (e.g., [`String`] for
//!   messages), the design keeps the door open for future `#![no_std]`
//!   compatibility (provided an allocator for [`String`] is available).
//!
//! # Internal Mechanism
//!
//! [`VctrlError`] is a plain enum. The [`Display`] implementation formats each
//! variant into a human-readable message, often including the offending value
//! (e.g., hash, name). The [`std::error::Error`] implementation delegates
//! `source()` exclusively to the [`IoError`](VctrlError::IoError) variant,
//! because only I/O errors carry an underlying cause worth propagating. For
//! comparison purposes, a manual [`PartialEq`] implementation treats
//! [`IoError`](VctrlError::IoError) instances as equal if their error kind and
//! display message match, while all string-bearing variants are compared by
//! their payload. This design ensures that errors can be compared in tests
//! without requiring a byte-for-byte match on potentially non-deterministic
//! OS error codes.
//!
//! # Examples
//!
//! Constructing and displaying a few common errors:
//!
//! ```
//! use libvctrl_handler::{VctrlError, Hash};
//!
//! // Invalid hash length
//! let err = VctrlError::InvalidHashLength(32);
//! assert!(err.to_string().starts_with("Invalid hash length:"));
//!
//! // Object not found
//! let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
//! let err = VctrlError::ObjectNotFound(hash);
//! assert!(err.to_string().starts_with("Object not found:"));
//!
//! // I/O error with source
//! let io = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
//! use std::error::Error;
//!
//! let err = VctrlError::IoError(io);
//! assert!(err.to_string().contains("I/O error"));
//! assert!(err.source().is_some());
//! ```

use crate::string_payload_variants;
use crate::types::Hash;
use std::fmt;

/// The unified error type returned by all fallible operations in the
/// `libvctrl_handler` crate.
///
/// ... (documentation unchanged) ...
#[non_exhaustive]
#[derive(Debug)]
pub enum VctrlError {
    /// Occurs when constructing a [`Hash`](crate::Hash) from a byte slice
    /// whose length does not equal [`HASH_LENGTH`](crate::constants::HASH_LENGTH).
    InvalidHashLength(usize),

    /// Occurs when a name (e.g., branch, tag, file entry) fails validation
    /// due to being empty or exceeding [`MAX_NAME_LENGTH`](crate::constants::MAX_NAME_LENGTH).
    InvalidName(String),

    /// *** NEW *** Occurs when an email address is empty or fails validation.
    InvalidEmail(String),

    /// Occurs when an object is requested from the [`ObjectStore`](crate::traits::ObjectStore)
    /// but cannot be found.
    ObjectNotFound(Hash),

    /// Occurs when a reference is requested from the [`RefStore`](crate::traits::RefStore)
    /// but cannot be found.
    RefNotFound(String),

    /// Occurs when serialized data fails to decode or violates structural
    /// invariants.
    CorruptedData(String),

    /// Wraps an I/O error from the underlying storage or network transport.
    IoError(std::io::Error),

    /// Wraps errors from the serialization or deserialization layer
    /// (e.g., encoding commits or trees).
    SerializationError(String),

    /// A catch-all variant for unexpected or miscellaneous errors not covered
    /// by the other variants. This is typically constructed via the
    /// `vctrl_error_other!` macro.
    Other(String),
}

// ---------------------------------------------------------------------------
// Manual Clone implementation
// ---------------------------------------------------------------------------
impl Clone for VctrlError {
    fn clone(&self) -> Self {
        match self {
            Self::InvalidHashLength(v) => Self::InvalidHashLength(*v),
            Self::InvalidName(v) => Self::InvalidName(v.clone()),
            Self::InvalidEmail(v) => Self::InvalidEmail(v.clone()), // NEW
            Self::ObjectNotFound(v) => Self::ObjectNotFound(*v),
            Self::RefNotFound(v) => Self::RefNotFound(v.clone()),
            Self::CorruptedData(v) => Self::CorruptedData(v.clone()),
            Self::IoError(e) => Self::IoError(std::io::Error::new(e.kind(), e.to_string())),
            Self::SerializationError(v) => Self::SerializationError(v.clone()),
            Self::Other(v) => Self::Other(v.clone()),
        }
    }
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------
impl fmt::Display for VctrlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHashLength(len) => write!(
                f,
                "Invalid hash length: expected {} bytes, got {len}",
                crate::constants::HASH_LENGTH,
            ),
            Self::InvalidName(name) => write!(f, "Invalid name: '{name}'"),
            Self::InvalidEmail(msg) => write!(f, "Invalid email: '{msg}'"), // NEW
            Self::ObjectNotFound(hash) => write!(f, "Object not found: {hash}"),
            Self::RefNotFound(name) => write!(f, "Reference not found: '{name}'"),
            Self::CorruptedData(msg) => write!(f, "Corrupted data: {msg}"),
            Self::IoError(err) => write!(f, "I/O error: {err}"),
            Self::SerializationError(msg) => write!(f, "Serialization error: {msg}"),
            Self::Other(msg) => write!(f, "{msg}"),
        }
    }
}

// ---------------------------------------------------------------------------
// std::error::Error implementation
// ---------------------------------------------------------------------------
impl std::error::Error for VctrlError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IoError(e) => Some(e),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// PartialEq + Eq – compare kinds and string representations for IoError
// ---------------------------------------------------------------------------
impl PartialEq for VctrlError {
    fn eq(&self, other: &Self) -> bool {
        // Use the macro to generate string_payload function covering all string variants
        string_payload_variants!(
            InvalidName,
            InvalidEmail, // NEW
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
