//! The unified error type for the entire `libvctrl` ecosystem.
//!
//! This module provides [`VctrlError`], the **single error type** that every
//! fallible operation in `libvctrl` must return.  By having one error type we
//! guarantee that errors are explicit, predictable, and can never be silently
//! ignored.
//!
//! # Design principles
//!
//! - **Exhaustive** – every possible failure (validation, storage, corruption,
//!   I/O, serialisation) is covered by a dedicated variant.
//! - **Object‑safe** – the error type is `Clone + Eq + 'static` and implements
//!   [`std::error::Error`]; it can be used in dynamic contexts without boxing.
//! - **No platform coupling** – I/O and transport errors are stored as a plain
//!   `String` so that the handler crate remains `#![no_std]` compatible (when
//!   built without `std`) and does not depend on `std::io::Error`.
//! - **Forward‑compatible** – the `#[non_exhaustive]` attribute and the
//!   [`Other`](VctrlError::Other) fallback variant mean that new error kinds
//!   can be added in minor releases without breaking existing code.
//!
//! # Usage
//!
//! ```rust
//! use libvctrl_handler::{Hash, VctrlError, HASH_LENGTH};
//!
//! // Construct errors directly ...
//! let bad_hash = VctrlError::InvalidHashLength(10);
//! let not_found = VctrlError::ObjectNotFound(
//!     Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap()
//! );
//!
//! // ... or use the convenience macro
//! let custom = libvctrl_handler::vctrl_error_other!("something broke: {}", 42);
//! assert_eq!(custom.to_string(), "something broke: 42");
//! ```

use crate::types::Hash;
use std::fmt;

/// Represents every possible error that can occur within `libvctrl`.
///
/// Every fallible public API in the workspace returns `Result<T, VctrlError>`.
/// This enum is **the** contract for error handling – no other error type
/// should leak across crate boundaries.
///
/// # When to use which variant
///
/// | Situation | Variant |
/// |---|---|
/// | A byte slice that should be a hash has the wrong length | [`InvalidHashLength`](Self::InvalidHashLength) |
/// | A name (file, reference, user, tag) is empty or too long | [`InvalidName`](Self::InvalidName) |
/// | An object hash is not in the store | [`ObjectNotFound`](Self::ObjectNotFound) |
/// | A reference name is not in the ref store | [`RefNotFound`](Self::RefNotFound) |
/// | Stored data is truncated, has bad magic bytes, or is otherwise unreadable | [`CorruptedData`](Self::CorruptedData) |
/// | A real I/O operation failed (disk full, permission denied, etc.) | [`IoError`](Self::IoError) |
/// | An encoder/decoder cannot process a value | [`SerializationError`](Self::SerializationError) |
/// | Any error that does not fit the above categories | [`Other`](Self::Other) |
///
/// # Display
///
/// The [`Display`](std::fmt::Display) implementation produces human‑readable
/// messages that include relevant detail (hash value, name, etc.).  These
/// messages are **not** guaranteed to be stable across versions; they are
/// intended for developers, not for programmatic matching.
///
/// # Stability
///
/// `VctrlError` is `#[non_exhaustive]`.  Pattern‑matching on its variants
/// requires a wildcard arm.  This allows us to add new error kinds without a
/// semver‑breaking change.
///
/// # Example
///
/// ```rust
/// use libvctrl_handler::{Hash, VctrlError, HASH_LENGTH};
///
/// let hash = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
/// let err = VctrlError::ObjectNotFound(hash);
/// println!("{err}");  // "Object not found: 0000000000000000..."
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VctrlError {
    /// The given hash does not have the required length
    /// ([`HASH_LENGTH`](crate::constants::HASH_LENGTH)).
    ///
    /// The contained `usize` is the invalid length that was provided.
    ///
    /// ```rust
    /// # use libvctrl_handler::VctrlError;
    /// let err = VctrlError::InvalidHashLength(10);
    /// assert_eq!(err.to_string(), "Invalid hash length: expected 64 bytes, got 10");
    /// ```
    InvalidHashLength(usize),

    /// The provided name is invalid.
    ///
    /// Reasons include:
    /// - empty string,
    /// - length exceeds [`MAX_NAME_LENGTH`](crate::constants::MAX_NAME_LENGTH),
    /// - contains forbidden characters (e.g., `'/'`).
    ///
    /// The contained `String` is the offending name.
    InvalidName(String),

    /// No object with the given hash exists in the object store.
    ///
    /// This is returned by [`ObjectStore::get`](crate::ObjectStore::get) when
    /// the requested hash is not found.
    ObjectNotFound(Hash),

    /// No reference with the given name exists in the reference store.
    ///
    /// This is returned by [`RefStore::get_ref`](crate::RefStore::get_ref).
    RefNotFound(String),

    /// Data read from storage is corrupted or does not conform to the expected
    /// format.
    ///
    /// Common causes: truncated files, incorrect magic bytes, checksum
    /// mismatch, or an unsupported format version.  Decoders should return
    /// this variant when the input is structurally invalid.
    CorruptedData(String),

    /// An I/O error occurred.
    ///
    /// The contained string describes the problem.  This variant exists so
    /// that the handler crate does not need to depend on `std::io::Error`
    /// (which is not available in `no_std` environments).  Implementations
    /// that *do* have access to `std` can convert the OS error to a string
    /// via `e.to_string()`.
    IoError(String),

    /// Serialization or deserialization failed.
    ///
    /// This is typically returned by [`Encoder`](crate::Encoder) and
    /// [`Decoder`](crate::Decoder) implementations when they encounter data
    /// that cannot be encoded or decoded (e.g., an unsupported object type,
    /// a field that exceeds a format‑specific limit, etc.).
    SerializationError(String),

    /// A fallback variant for errors that do not fit the other categories.
    ///
    /// Use the [`vctrl_error_other!`](crate::vctrl_error_other) macro for a
    /// convenient way to construct this variant with a formatted message.
    ///
    /// ```rust
    /// # use libvctrl_handler::VctrlError;
    /// let err = VctrlError::Other("custom error".into());
    /// assert_eq!(err.to_string(), "custom error");
    /// ```
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
        // VctrlError does not wrap external errors, so there is no source.
        None
    }
}
