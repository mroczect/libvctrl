//! Hash type.
//!
//! # Architecture
//! This module defines the [`Hash`] type, a fixed-size wrapper around a 64-byte
//! array (SHA-512). In a content-addressable storage (CAS) system, hashes are the
//! primary keys for all objects and references.
//!
//! # Design Rationale: Stack Allocation
//! By wrapping a fixed-size array `[u8; 64]` instead of using a `Vec<u8>` or `Box<[u8]>`,
//! the [`Hash`] type is inherently `Copy` and requires no heap allocation. This is a
//! critical performance optimization: hashes are created, copied, and compared millions
//! of times during graph traversal and object packing. Keeping them on the stack
//! eliminates allocator overhead and memory fragmentation.

use crate::constants::HASH_LENGTH;
use crate::errors::VctrlError;
use core::fmt;
use core::str::FromStr;

/// A fixed-size hash (64 bytes, e.g., SHA-512).
///
/// # Why this exists
/// Provides a strongly-typed, length-guaranteed representation of a cryptographic hash.
/// By encoding the length (64 bytes) directly into the type system via a constant
/// generic array, the compiler guarantees that a [`Hash`] can never accidentally hold
/// a 20-byte SHA-1 or a 32-byte SHA-256. This prevents entire classes of length-mismatch
/// bugs at compile time.
///
/// # How it works
/// The struct is a tuple wrapping `[u8; HASH_LENGTH]`. It derives `PartialEq`, `Eq`,
/// `Hash`, and `Ord`, allowing it to be used as a key in `HashMap` or `BTreeMap`. The
/// `Copy` trait is derived, meaning assigning a hash to a new variable performs a fast
/// 64-byte stack copy rather than a pointer move.
///
/// # Examples
///
/// Creating a hash from raw bytes:
///
/// ```
/// # use libvctrl_handler::types::core::hash::Hash;
/// # use libvctrl_handler::VctrlError;
/// let raw_bytes = [0_u8; 64];
/// let hash = Hash::from_bytes(&raw_bytes)?;
/// assert_eq!(hash.as_bytes(), &raw_bytes);
/// # Ok::<(), VctrlError>(())
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Hash([u8; HASH_LENGTH]);

impl Hash {
    /// Creates a hash from a byte slice.
    ///
    /// # How it works
    /// This function is `const`, meaning it can be evaluated at compile time if the
    /// input slice is a static literal. Because `for` loops over slices were not fully
    /// stable in `const fn` contexts during early Rust editions, this implementation
    /// uses a `while` loop with an index to copy bytes into a fixed-size array. If the
    /// slice length does not exactly match [`HASH_LENGTH`], an error is returned.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::InvalidHashLength`] if the slice length does not match [`HASH_LENGTH`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::types::core::hash::Hash;
    /// # use libvctrl_handler::VctrlError;
    /// let valid_hash = Hash::from_bytes(&[1u8; 64]);
    /// assert!(valid_hash.is_ok());
    ///
    /// let invalid_hash = Hash::from_bytes(&[1u8; 32]);
    /// assert!(invalid_hash.is_err());
    /// # Ok::<(), VctrlError>(())
    /// ```
    #[allow(clippy::indexing_slicing)]
    pub const fn from_bytes(bytes: &[u8]) -> Result<Self, VctrlError> {
        if bytes.len() != HASH_LENGTH {
            return Err(VctrlError::InvalidHashLength(bytes.len()));
        }
        let mut arr = [0_u8; HASH_LENGTH];
        let mut i = 0;
        while i < HASH_LENGTH {
            arr[i] = bytes[i];
            i += 1;
        }
        Ok(Self(arr))
    }

    /// Returns the raw bytes of the hash.
    ///
    /// # How it works
    /// Returns a reference to the inner fixed-size array. This avoids any slicing or
    /// copying overhead, providing direct access to the underlying 64 bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::types::core::hash::Hash;
    /// # use libvctrl_handler::VctrlError;
    /// let hash = Hash::from_bytes(&[0xAB; 64])?;
    /// assert_eq!(hash.as_bytes(), &[0xAB; 64]);
    /// # Ok::<(), VctrlError>(())
    /// ```
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; HASH_LENGTH] {
        &self.0
    }
}

impl From<[u8; HASH_LENGTH]> for Hash {
    /// Converts a raw array into a [`Hash`].
    ///
    /// # How it works
    /// This infallible conversion wraps the array directly. It is used when the caller
    /// already possesses a correctly sized array, bypassing the need for slice validation.
    fn from(arr: [u8; HASH_LENGTH]) -> Self {
        Self(arr)
    }
}

impl TryFrom<&[u8]> for Hash {
    type Error = VctrlError;

    /// Attempts to convert a byte slice into a [`Hash`].
    ///
    /// # How it works
    /// Delegates to [`Hash::from_bytes`]. This trait implementation allows ergonomic
    /// use of the `?` operator when converting from generic byte slices.
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::from_bytes(value)
    }
}

impl AsRef<[u8]> for Hash {
    /// Converts to a byte slice.
    ///
    /// # How it works
    /// Allows the [`Hash`] to be used with APIs that expect `AsRef<[u8]>`, providing
    /// interoperability with standard cryptographic and I/O crates without exposing
    /// the internal array representation.
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl FromStr for Hash {
    type Err = VctrlError;

    /// Parses a hexadecimal string into a [`Hash`].
    ///
    /// # How it works
    /// Expects a string of exactly 128 characters (64 bytes * 2 hex chars). It iterates
    /// through the string in 2-character chunks, parsing each chunk into a byte using
    /// `u8::from_str_radix`. If any character is invalid hex, or if the length is wrong,
    /// it returns an error.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::InvalidHashLength`] if the string length is not 128.
    /// Returns [`VctrlError::CorruptedData`] if the string contains non-hexadecimal characters.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::types::core::hash::Hash;
    /// # use std::str::FromStr;
    /// # use libvctrl_handler::VctrlError;
    /// let hex_str = "0".repeat(128);
    /// let hash = Hash::from_str(&hex_str)?;
    /// assert_eq!(hash.as_bytes(), &[0_u8; 64]);
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != HASH_LENGTH * 2 {
            return Err(VctrlError::InvalidHashLength(s.len()));
        }
        let mut bytes = [0_u8; HASH_LENGTH];
        for (out, chunk) in bytes.iter_mut().zip(s.as_bytes().chunks_exact(2)) {
            let hex_str = core::str::from_utf8(chunk).map_err(|e| {
                VctrlError::CorruptedData(format!("invalid hex char in hash: {s}: {e}"))
            })?;
            *out = u8::from_str_radix(hex_str, 16).map_err(|e| {
                VctrlError::CorruptedData(format!("invalid hex char in hash: {s}: {e}"))
            })?;
        }
        Ok(Self(bytes))
    }
}

impl fmt::Debug for Hash {
    /// Formats the hash for debugging purposes.
    ///
    /// # How it works
    /// To prevent flooding debug logs with 128-character strings, this implementation
    /// only prints the first 16 bytes (32 hex characters) followed by `...`. This provides
    /// enough context to distinguish between different hashes while remaining readable.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash(")?;
        for &byte in self.0.iter().take(16) {
            write!(f, "{byte:02x}")?;
        }
        write!(f, "...)")
    }
}

impl fmt::Display for Hash {
    /// Formats the hash as a full hexadecimal string.
    ///
    /// # How it works
    /// Iterates over all 64 bytes, formatting each as a two-character zero-padded
    /// hexadecimal value. This produces the canonical 128-character string representation
    /// expected by Git tools.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::types::core::hash::Hash;
    /// # use libvctrl_handler::VctrlError;
    /// use std::fmt::Display;
    /// let hash = Hash::from_bytes(&[0_u8; 64])?;
    /// assert_eq!(format!("{hash}"), "0".repeat(128));
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &byte in &self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}
