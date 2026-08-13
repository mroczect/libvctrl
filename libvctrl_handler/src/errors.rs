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
/// # Purpose
///
/// `VctrlError` consolidates every possible failure mode in the crate into a
/// single, matchable enum. This allows consumers to handle specific error
/// conditions with exhaustive pattern matching, or to propagate the entire
/// error upward without losing contextual information.
///
/// # Design Rationale
///
/// The variants are deliberately designed around the crate's core domains:
///
/// - **Validation failures** ([`InvalidName`](Self::InvalidName),
///   [`InvalidEmail`](Self::InvalidEmail),
///   [`InvalidHashLength`](Self::InvalidHashLength)) capture malformed inputs
///   before they enter the storage layer.
/// - **Storage failures** ([`ObjectNotFound`](Self::ObjectNotFound),
///   [`RefNotFound`](Self::RefNotFound)) cover lookups that miss.
/// - **Serialization failures** ([`CorruptedData`](Self::CorruptedData),
///   [`SerializationError`](Self::SerializationError)) handle byte-level and
///   encoding/decoding problems.
/// - **Infrastructure failures** ([`IoError`](Self::IoError),
///   [`Other`](Self::Other)) wrap lower-level I/O and catch-all conditions.
///
/// The manual [`Clone`] implementation is required because
/// [`std::io::Error`] does not implement [`Clone`]. Instead of cloning the
/// exact OS-level error object, we reconstruct a semantically equivalent
/// [`std::io::Error`] from its [`std::io::ErrorKind`] and message string.
///
/// # Internal Mechanism
///
/// Internally, the enum is a tagged union. The [`Display`] implementation
/// converts each variant into a human-readable string, while
/// [`std::error::Error::source`] exposes only the wrapped I/O error as the
/// root cause. For equality, a helper function generated by the
/// `string_payload_variants!` macro extracts the string payload from all
/// string-bearing variants. This enables comparing errors without requiring
/// byte-for-byte fidelity of OS error codes.
///
/// # Examples
///
/// Constructing and inspecting each major category:
///
/// ```
/// use libvctrl_handler::{Hash, VctrlError};
///
/// let hash_error = VctrlError::InvalidHashLength(10);
/// assert!(hash_error.to_string().starts_with("Invalid hash length:"));
///
/// let name_error = VctrlError::InvalidName("".to_string());
/// assert!(name_error.to_string().starts_with("Invalid name:"));
///
/// let email_error = VctrlError::InvalidEmail("not-an-email".to_string());
/// assert!(email_error.to_string().starts_with("Invalid email:"));
///
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// let object_error = VctrlError::ObjectNotFound(hash);
/// assert!(object_error.to_string().starts_with("Object not found:"));
///
/// let io_error = VctrlError::IoError(std::io::Error::new(
///     std::io::ErrorKind::PermissionDenied,
///     "read-only file system",
/// ));
/// assert!(io_error.to_string().starts_with("I/O error:"));
/// ```
#[non_exhaustive]
#[derive(Debug)]
pub enum VctrlError {
    /// Occurs when constructing a [`Hash`](crate::Hash) from a byte slice
    /// whose length does not equal
    /// [`HASH_LENGTH`](crate::constants::HASH_LENGTH).
    ///
    /// # Purpose
    ///
    /// This variant protects the [`Hash`] invariant that exactly 64 bytes are
    /// required. Any attempt to create a [`Hash`] from a slice of incorrect
    /// length yields this error.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::VctrlError;
    ///
    /// let err = VctrlError::InvalidHashLength(10);
    /// assert!(err.to_string().contains("expected 64 bytes, got 10"));
    /// ```
    InvalidHashLength(usize),

    /// Occurs when a name (e.g., branch, tag, file entry) fails validation
    /// due to being empty or exceeding
    /// [`MAX_NAME_LENGTH`](crate::constants::MAX_NAME_LENGTH).
    ///
    /// # Purpose
    ///
    /// Names are validated by internal constructors such as
    /// [`UserID::new`](crate::UserID::new) and
    /// [`Tag::new`](crate::Tag::new). This variant prevents empty or
    /// excessively long names from entering the system.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::VctrlError;
    ///
    /// let err = VctrlError::InvalidName("".to_string());
    /// assert!(err.to_string().starts_with("Invalid name:"));
    /// ```
    InvalidName(String),

    /// Occurs when an email address is empty or fails validation.
    ///
    /// # Purpose
    ///
    /// This variant was added to enforce a minimum email format check. Unlike
    /// [`InvalidName`](Self::InvalidName), it focuses specifically on email
    /// fields, enabling callers to provide targeted feedback.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::VctrlError;
    ///
    /// let err = VctrlError::InvalidEmail("bad-email".to_string());
    /// assert!(err.to_string().starts_with("Invalid email:"));
    /// ```
    InvalidEmail(String),

    /// Occurs when an object is requested from the
    /// [`ObjectStore`](crate::ObjectStore) but cannot be found.
    ///
    /// # Purpose
    ///
    /// This variant signals a content-addressed lookup miss. It carries the
    /// [`Hash`] of the requested object so that callers can inspect or
    /// re-request it if necessary.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::{Hash, VctrlError};
    ///
    /// let hash = Hash::from_bytes(&[0xAB; 64]).unwrap();
    /// let err = VctrlError::ObjectNotFound(hash);
    /// assert!(err.to_string().starts_with("Object not found:"));
    /// ```
    ObjectNotFound(Hash),

    /// Occurs when a reference is requested from the
    /// [`RefStore`](crate::RefStore) but cannot be found.
    ///
    /// # Purpose
    ///
    /// This variant handles missing branch or tag names. The payload is the
    /// name of the missing reference, allowing callers to present it in user
    /// interfaces or fallback logic.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::VctrlError;
    ///
    /// let err = VctrlError::RefNotFound("main".to_string());
    /// assert!(err.to_string().starts_with("Reference not found:"));
    /// ```
    RefNotFound(String),

    /// Occurs when serialized data fails to decode or violates structural
    /// invariants.
    ///
    /// # Purpose
    ///
    /// This variant is returned by [`Decoder`](crate::Decoder) implementations
    /// when byte representations are malformed or logically inconsistent.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::VctrlError;
    ///
    /// let err = VctrlError::CorruptedData("bad header".to_string());
    /// assert!(err.to_string().starts_with("Corrupted data:"));
    /// ```
    CorruptedData(String),

    /// Wraps an I/O error from the underlying storage or network transport.
    ///
    /// # Purpose
    ///
    /// This variant preserves the original [`std::io::Error`] so that callers
    /// can inspect [`std::io::ErrorKind`] and other I/O-specific details.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::VctrlError;
    ///
    /// let io = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
    /// let err = VctrlError::IoError(io);
    /// assert!(err.to_string().starts_with("I/O error:"));
    /// ```
    IoError(std::io::Error),

    /// Wraps errors from the serialization or deserialization layer
    /// (e.g., encoding commits or trees).
    ///
    /// # Purpose
    ///
    /// This variant is used by [`Encoder`](crate::Encoder) implementations when
    /// an object cannot be transformed into its byte representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::VctrlError;
    ///
    /// let err = VctrlError::SerializationError("encoding failed".to_string());
    /// assert!(err.to_string().starts_with("Serialization error:"));
    /// ```
    SerializationError(String),

    /// A catch-all variant for unexpected or miscellaneous errors not covered
    /// by the other variants.
    ///
    /// This is typically constructed via the [`vctrl_error_other!`](crate::vctrl_error_other)
    /// macro, which mimics `format!` syntax.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::{vctrl_error_other, VctrlError};
    ///
    /// let err: VctrlError = vctrl_error_other!("code {}", 500);
    /// assert_eq!(err.to_string(), "code 500");
    /// ```
    Other(String),
}

/// Manual [`Clone`] implementation for [`VctrlError`].
///
/// # Why manual
///
/// [`std::io::Error`] does not implement [`Clone`]. To make
/// [`VctrlError`] clonable, the [`IoError`](VctrlError::IoError) variant is
/// reconstructed from the original error's [`std::io::ErrorKind`] and display
/// message. This preserves enough semantic information for testing and
/// comparison while avoiding the need for byte-for-byte OS error fidelity.
///
/// # How it works
///
/// All variants except [`IoError`](VctrlError::IoError) are cloned by cloning
/// their primitive or owned payload. For [`IoError`](VctrlError::IoError),
/// a fresh [`std::io::Error`] is created with the same kind and message.
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

/// Human-readable formatting for [`VctrlError`].
///
/// # Purpose
///
/// Each variant is rendered as a descriptive string that includes the
/// offending value where applicable. This is the default string returned by
/// `to_string()`.
///
/// # How it works
///
/// The implementation matches on each variant and uses [`write!`] to build
/// the final message. For example, [`InvalidHashLength`](VctrlError::InvalidHashLength)
/// includes both the expected and actual length.
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

/// Integration with the standard error ecosystem.
///
/// # Purpose
///
/// Implementing [`std::error::Error`] allows [`VctrlError`] to be used with
/// error-reporting crates such as `anyhow` and `eyre`, and to be propagated
/// through [`Box<dyn Error>`](std::error::Error).
///
/// # How it works
///
/// The [`source`](std::error::Error::source) method returns `Some` only for
/// the [`IoError`](VctrlError::IoError) variant, because that is the only
/// variant that wraps an underlying standard library error. All other
/// variants are leaf errors.
impl std::error::Error for VctrlError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IoError(e) => Some(e),
            _ => None,
        }
    }
}

/// Value equality for [`VctrlError`].
///
/// # Purpose
///
/// Enables `assert_eq!` and other comparison operations in tests and
/// application logic.
///
/// # How it works
///
/// - [`InvalidHashLength`](VctrlError::InvalidHashLength) is compared by
///   numeric value.
/// - [`ObjectNotFound`](VctrlError::ObjectNotFound) is compared by [`Hash`].
/// - [`IoError`](VctrlError::IoError) is compared by [`std::io::ErrorKind`]
///   and its display string, rather than exact OS error identity.
/// - All string-bearing variants are compared by their string payload.
///
/// The `string_payload_variants!` macro generates a private helper function
/// inside this method that extracts string slices from the relevant variants,
/// avoiding repetitive `match` arms.
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

/// Marker trait indicating that [`VctrlError`] has a total equality relation.
///
/// This is valid because all payload types used in the variants implement
/// [`Eq`].
impl Eq for VctrlError {}
