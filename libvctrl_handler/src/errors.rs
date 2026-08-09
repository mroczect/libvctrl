//! Error handling for the `libvctrl_handler` version control contracts.
//!
//! # Purpose
//! This module defines [`VctrlError`], the unified error type returned by all
//! fallible operations within the crate. It encapsulates various failure modes
//! ranging from invalid input data to storage and serialization failures.
//!
//! # Design rationale
//! - **Simplicity and Cloning**: The error variants store `String` rather than
//!   boxed trait objects (`Box<dyn std::error::Error>`). This ensures that
//!   [`VctrlError`] implements [`Clone`], [`PartialEq`], and [`Eq`], which is
//!   crucial for testing assertions and state comparisons.
//! - **Forward Compatibility**: The enum is marked `#[non_exhaustive]`. This
//!   prevents downstream crates from exhaustively matching against it,
//!   allowing new error variants to be added in future minor versions without
//!   breaking the API.
//! - **`no_std` Readiness**: By avoiding complex heap-allocated error chains
//!   and relying on `String`, the design keeps the door open for future
//!   `#![no_std]` compatibility.
//!
//! # Internal mechanism
//! The [`std::error::Error`] trait is implemented explicitly. The `source`
//! method always returns `None` because the variant payloads are plain data
//! types (like `String` or [`Hash`](crate::Hash)), not wrapped causal errors.

use crate::types::Hash;
use std::fmt;

/// The unified error type returned by all fallible operations in the
/// `libvctrl_handler` crate.
///
/// # Design rationale
/// This enum is marked `#[non_exhaustive]` to ensure that adding new error
/// variants in the future is not considered a breaking change. Callers must
/// include a catch-all `_` arm when matching on it.
///
/// # Internal mechanism
/// Variants store `String` for messages to ensure the error type remains
/// `Clone` and `PartialEq`, unlike `Box<dyn Error>`.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::{VctrlError, Hash};
///
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// let err = VctrlError::ObjectNotFound(hash);
///
/// assert!(err.to_string().starts_with("Object not found:"));
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VctrlError {
    /// Occurs when constructing a [`Hash`](crate::Hash) from a byte slice
    /// whose length does not equal [`HASH_LENGTH`](crate::HASH_LENGTH).
    ///
    /// The payload is the invalid byte length that was provided.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::VctrlError;
    ///
    /// let err = VctrlError::InvalidHashLength(32);
    /// assert!(err.to_string().contains("got 32"));
    /// ```
    InvalidHashLength(usize),

    /// Occurs when a name (e.g., branch, tag, file entry) fails validation
    /// due to being empty or exceeding
    /// [`MAX_NAME_LENGTH`](crate::MAX_NAME_LENGTH).
    ///
    /// The payload is a descriptive message explaining the validation failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::VctrlError;
    ///
    /// let err = VctrlError::InvalidName("too long".to_string());
    /// assert_eq!(format!("{err}"), "Invalid name: 'too long'");
    /// ```
    InvalidName(String),

    /// Occurs when an object is requested from the
    /// [`ObjectStore`](crate::ObjectStore) but cannot be found.
    ///
    /// The payload is the [`Hash`](crate::Hash) of the missing object.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::{VctrlError, Hash};
    ///
    /// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let err = VctrlError::ObjectNotFound(hash);
    /// assert!(matches!(err, VctrlError::ObjectNotFound(_)));
    /// ```
    ObjectNotFound(Hash),

    /// Occurs when a reference is requested from the
    /// [`RefStore`](crate::RefStore) but cannot be found.
    ///
    /// The payload is the name of the missing reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::VctrlError;
    ///
    /// let err = VctrlError::RefNotFound("main".to_string());
    /// assert_eq!(format!("{err}"), "Reference not found: 'main'");
    /// ```
    RefNotFound(String),

    /// Occurs when serialized data fails to decode or violates structural
    /// invariants.
    ///
    /// The payload is a descriptive message detailing the corruption.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::VctrlError;
    ///
    /// let err = VctrlError::CorruptedData("bad header".to_string());
    /// assert_eq!(format!("{err}"), "Corrupted data: bad header");
    /// ```
    CorruptedData(String),

    /// Wraps I/O errors from the underlying storage or network transport.
    ///
    /// The payload is the string representation of the underlying I/O error.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::VctrlError;
    ///
    /// let err = VctrlError::IoError("disk full".to_string());
    /// assert_eq!(format!("{err}"), "I/O error: disk full");
    /// ```
    IoError(String),

    /// Wraps errors from the serialization or deserialization layer (e.g.,
    /// encoding commits or trees).
    ///
    /// The payload is the string representation of the serialization error.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::VctrlError;
    ///
    /// let err = VctrlError::SerializationError("unexpected EOF".to_string());
    /// assert_eq!(format!("{err}"), "Serialization error: unexpected EOF");
    /// ```
    SerializationError(String),

    /// A catch-all variant for unexpected or miscellaneous errors not covered
    /// by the other variants.
    ///
    /// This is typically constructed via the `vctrl_error_other!` macro.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::VctrlError;
    ///
    /// let err = VctrlError::Other("something else".to_string());
    /// assert_eq!(format!("{err}"), "something else");
    /// ```
    Other(String),
}

/// Formats the error using the given formatter.
///
/// # Design rationale
/// This implementation provides human-readable, context-rich error messages.
/// For example, [`InvalidHashLength`](VctrlError::InvalidHashLength) dynamically
/// references [`HASH_LENGTH`](crate::HASH_LENGTH) so the message is always
/// accurate even if the constant changes.
///
/// # Internal mechanism
/// It matches on `Self` and uses the `write!` macro to write the formatted
/// string directly to the formatter, avoiding intermediate allocations.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::VctrlError;
/// use std::fmt::Display;
///
/// let err = VctrlError::Other("test".to_string());
/// let s = format!("{err}");
/// assert_eq!(s, "test");
/// ```
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

/// Implementation of the standard library's [`std::error::Error`] trait.
///
/// # Design rationale
/// Implementing this trait ensures that `VctrlError` integrates seamlessly
/// with the broader Rust error handling ecosystem, allowing it to be used
/// with crates like `anyhow` or `eyre`.
///
/// # Internal mechanism
/// The `source` method explicitly returns `None` for all variants. Because
/// the variant payloads are plain `String`s or value types (not wrapped
/// causal errors), there is no underlying error source to expose.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::VctrlError;
/// use std::error::Error;
///
/// let err = VctrlError::IoError("disk full".to_string());
/// // Verifies it implements std::error::Error
/// fn assert_error<T: Error + ?Sized>(_: &T) {}
/// assert_error(&err);
/// assert!(err.source().is_none());
/// ```
impl std::error::Error for VctrlError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
