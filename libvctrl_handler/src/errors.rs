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

use crate::types::Hash;
use std::fmt;

/// The unified error type returned by all fallible operations in the
/// `libvctrl_handler` crate.
///
/// # Why this exists
///
/// A single, rich error enum allows callers to handle different failure
/// scenarios with pattern matching while still being able to propagate
/// errors generically via [`std::error::Error`] trait objects. Marking the
/// enum `#[non_exhaustive]` ensures library evolution without breaking
/// downstream code.
///
/// # How it works internally
///
/// Each variant captures the minimal data necessary to describe the failure.
/// The [`Display`] implementation turns that data into a human-readable
/// message. The [`std::error::Error`] implementation exposes the underlying
/// I/O error as a source when applicable. The manual [`Clone`] and
/// [`PartialEq`] implementations handle the non-clonable
/// [`std::io::Error`] by reconstructing it from its public components.
///
/// # Examples
///
/// Basic construction and matching:
///
/// ```
/// use libvctrl_handler::{VctrlError, Hash};
///
/// let hash = Hash::from_bytes(&[0xAA; 64]).unwrap();
/// let err = VctrlError::ObjectNotFound(hash);
///
/// match &err {
///     VctrlError::ObjectNotFound(h) => println!("Missing: {}", h),
///     _ => unreachable!(),
/// }
/// ```
///
/// Cloning and comparing I/O errors:
///
/// ```
/// use libvctrl_handler::VctrlError;
///
/// let io = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
/// let err1 = VctrlError::IoError(io);
/// let err2 = err1.clone();
/// assert_eq!(err1, err2);
/// ```
#[non_exhaustive]
#[derive(Debug)]
pub enum VctrlError {
    /// Occurs when constructing a [`Hash`](crate::Hash) from a byte slice
    /// whose length does not equal [`HASH_LENGTH`](crate::constants::HASH_LENGTH).
    ///
    /// The payload is the actual byte length that was provided. This allows
    /// callers to adjust buffer sizes dynamically or return a more specific
    /// diagnostic to the user.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::VctrlError;
    /// use std::error::Error;
    ///
    /// let err = VctrlError::InvalidHashLength(15);
    /// assert!(err.to_string().contains("15"));
    /// ```
    InvalidHashLength(usize),

    /// Occurs when a name (e.g., branch, tag, file entry) fails validation
    /// due to being empty or exceeding [`MAX_NAME_LENGTH`](crate::constants::MAX_NAME_LENGTH).
    ///
    /// The payload is the rejected name as a [`String`] so that upper layers
    /// can log it or present it in a user interface without re-parsing an
    /// error message.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::VctrlError;
    /// use std::error::Error;
    ///
    /// let err = VctrlError::InvalidName("".into());
    /// assert!(err.to_string().starts_with("Invalid name:"));
    /// ```
    InvalidName(String),

    /// Occurs when an object is requested from the [`ObjectStore`](crate::traits::ObjectStore)
    /// but cannot be found.
    ///
    /// The contained [`Hash`] identifies the missing object, making it easy to
    /// construct follow-up queries or diagnostics.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::{VctrlError, Hash};
    ///
    /// let hash = Hash::from_bytes(&[0x11; 64]).unwrap();
    /// let err = VctrlError::ObjectNotFound(hash);
    /// assert!(err.to_string().contains(&hash.to_string()));
    /// ```
    ObjectNotFound(Hash),

    /// Occurs when a reference is requested from the [`RefStore`](crate::traits::RefStore)
    /// but cannot be found.
    ///
    /// The payload is the reference name, which can be used to suggest
    /// similar names via a fuzzy-matching UI.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::VctrlError;
    /// use std::error::Error;
    ///
    /// let err = VctrlError::RefNotFound("main".into());
    /// assert!(err.to_string().contains("main"));
    /// ```
    RefNotFound(String),

    /// Occurs when serialized data fails to decode or violates structural
    /// invariants.
    ///
    /// The [`String`] message explains what went wrong (e.g., unexpected
    /// byte sequence, checksum mismatch). This variant is deliberately
    /// opaque to avoid leaking internal parsing details.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::VctrlError;
    /// use std::error::Error;
    ///
    /// let err = VctrlError::CorruptedData("invalid tree entry".into());
    /// assert!(err.to_string().starts_with("Corrupted data:"));
    /// ```
    CorruptedData(String),

    /// Wraps an I/O error from the underlying storage or network transport.
    ///
    /// The payload is the original [`std::io::Error`], preserving its
    /// [`ErrorKind`](std::io::ErrorKind) and the full error chain. This is the
    /// only variant that returns [`Some`] from [`std::error::Error::source`],
    /// enabling consumers to inspect the root cause with crates like `anyhow`.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::VctrlError;
    /// use std::error::Error;
    ///
    /// let io = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "pipe closed");
    /// let err = VctrlError::IoError(io);
    /// let source = err.source().expect("IoError must have a source");
    /// assert_eq!(source.to_string(), "pipe closed");
    /// ```
    IoError(std::io::Error),

    /// Wraps errors from the serialization or deserialization layer
    /// (e.g., encoding commits or trees).
    ///
    /// This is distinct from [`CorruptedData`](VctrlError::CorruptedData)
    /// because it indicates a problem with the *process* of
    /// serialization/deserialization rather than the *content* itself.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::VctrlError;
    /// use std::error::Error;
    ///
    /// let err = VctrlError::SerializationError("failed to encode commit".into());
    /// assert!(err.to_string().contains("Serialization error:"));
    /// ```
    SerializationError(String),

    /// A catch-all variant for unexpected or miscellaneous errors not covered
    /// by the other variants. This is typically constructed via the
    /// `vctrl_error_other!` macro.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::VctrlError;
    /// use std::error::Error;
    ///
    /// let err = VctrlError::Other("something unexpected happened".into());
    /// assert_eq!(err.to_string(), "something unexpected happened");
    /// ```
    Other(String),
}

// ---------------------------------------------------------------------------
// Manual Clone implementation because std::io::Error is not Clone.
// We reconstruct the I/O error from its kind and message, which preserves
// enough information for error display and kind matching.
// ---------------------------------------------------------------------------
impl Clone for VctrlError {
    /// Returns a deep copy of the error.
    ///
    /// # Why manual clone
    ///
    /// [`std::io::Error`] does not implement [`Clone`], so `#[derive(Clone)]`
    /// would fail for the [`IoError`](VctrlError::IoError) variant. The manual
    /// implementation reconstructs an I/O error with the same
    /// [`std::io::ErrorKind`] and textual message, which is sufficient for
    /// display and kind-based matching. This preserves the ability to clone
    /// errors in test assertions and when retrying fallible operations.
    ///
    /// # How it works
    ///
    /// For the [`IoError`](VctrlError::IoError) variant,
    /// [`std::io::Error::new`] is called with the original kind (obtained via
    /// [`std::io::Error::kind`]) and the original message (obtained via
    /// [`std::fmt::Display`]). The resulting error is semantically equivalent
    /// for all practical purposes, though the internal representation may
    /// differ.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::VctrlError;
    ///
    /// let io = std::io::Error::new(std::io::ErrorKind::AddrInUse, "port 8080");
    /// let err = VctrlError::IoError(io);
    /// let cloned = err.clone();
    ///
    /// // The clone retains the error kind and message.
    /// if let VctrlError::IoError(e) = &cloned {
    ///     assert_eq!(e.kind(), std::io::ErrorKind::AddrInUse);
    ///     assert_eq!(e.to_string(), "port 8080");
    /// }
    /// ```
    fn clone(&self) -> Self {
        match self {
            Self::InvalidHashLength(v) => Self::InvalidHashLength(*v),
            Self::InvalidName(v) => Self::InvalidName(v.clone()),
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
    /// Formats the error for human consumption.
    ///
    /// Each variant produces a message prefixed with a category (e.g.,
    /// `"I/O error: ..."`) and includes the specific payload (hash, name,
    /// length, etc.). This design makes log output easily searchable.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::VctrlError;
    ///
    /// let err = VctrlError::InvalidName("HEAD".into());
    /// let msg = err.to_string();
    /// assert!(msg.contains("Invalid name:") && msg.contains("HEAD"));
    /// ```
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
            Self::IoError(err) => write!(f, "I/O error: {err}"),
            Self::SerializationError(msg) => write!(f, "Serialization error: {msg}"),
            Self::Other(msg) => write!(f, "{msg}"),
        }
    }
}

// ---------------------------------------------------------------------------
// std::error::Error implementation – now with working source()
// ---------------------------------------------------------------------------
impl std::error::Error for VctrlError {
    /// Provides access to the lower-level error that caused this error.
    ///
    /// # Design decision
    ///
    /// Only the [`IoError`](VctrlError::IoError) variant returns a source,
    /// because it is the only variant that wraps an error from another crate
    /// or the OS. All other variants are considered terminal failures within
    /// the application logic, so they return [`None`].
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::VctrlError;
    /// use std::error::Error;
    ///
    /// let io = std::io::Error::new(std::io::ErrorKind::TimedOut, "connect timeout");
    /// let err = VctrlError::IoError(io);
    /// assert!(err.source().is_some());
    ///
    /// let err = VctrlError::InvalidName("bad".into());
    /// assert!(err.source().is_none());
    /// ```
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
    /// Compares two errors for equality.
    ///
    /// # Why manual implementation
    ///
    /// Deriving [`PartialEq`] would compare [`std::io::Error`] values by
    /// address or raw OS code, which is not stable across clones or
    /// test environments. The manual implementation compares I/O errors by
    /// their [`std::io::ErrorKind`] and their display message, providing a
    /// semantic equality that works with the custom [`Clone`] implementation.
    ///
    /// For all other variants, the comparison is either a direct value
    /// comparison (e.g., `usize`) or a string comparison of the payload.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::VctrlError;
    ///
    /// let a = VctrlError::InvalidHashLength(64);
    /// let b = VctrlError::InvalidHashLength(64);
    /// assert_eq!(a, b);
    ///
    /// let c = VctrlError::Other("boom".into());
    /// let d = VctrlError::Other("boom".into());
    /// assert_eq!(c, d);
    /// ```
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::InvalidHashLength(a), Self::InvalidHashLength(b)) => a == b,
            (Self::ObjectNotFound(a), Self::ObjectNotFound(b)) => a == b,
            (Self::IoError(a), Self::IoError(b)) => {
                a.kind() == b.kind() && a.to_string() == b.to_string()
            }
            _ => {
                const fn string_payload(v: &VctrlError) -> Option<&str> {
                    match v {
                        VctrlError::InvalidName(s)
                        | VctrlError::RefNotFound(s)
                        | VctrlError::CorruptedData(s)
                        | VctrlError::SerializationError(s)
                        | VctrlError::Other(s) => Some(s.as_str()),
                        _ => None,
                    }
                }
                match (string_payload(self), string_payload(other)) {
                    (Some(s1), Some(s2)) => s1 == s2,
                    _ => false,
                }
            }
        }
    }
}

impl Eq for VctrlError {}
