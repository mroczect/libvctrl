//! Hash type for the `libvctrl_handler` version control contracts.
//!
//! # Purpose
//! A [`Hash`](crate::Hash) is a fixed-size, 64-byte cryptographic digest that
//! uniquely identifies an object (blob, tree, commit, or tag) in the version
//! control system. It serves as the primary key for the
//! [`ObjectStore`](crate::ObjectStore) and
//! [`RefStore`](crate::RefStore).
//!
//! # Design rationale
//! The type is a tuple struct wrapping a `[u8; 64]` array. This provides
//! nominal typing: a `Hash` cannot be accidentally confused with another
//! 64-byte array (like a raw SHA-512 digest) because it is a distinct type.
//! It also allows implementing custom [`Display`](std::fmt::Display) and
//! [`Debug`](std::fmt::Debug) traits without violating the orphan rules.
//!
//! # Internal mechanism
//! The [`Hash`](crate::Hash) is `Copy` and `Clone` because 64 bytes is small
//! enough to be cheaply copied on the stack. The
//! [`from_bytes`](crate::Hash::from_bytes) constructor is a `const fn` that
//! validates the length and copies the bytes into the inner array. The `const`
//! context forces the use of a `while` loop instead of iterator methods, but
//! ensures the function can be evaluated at compile time if needed.

use crate::constants::HASH_LENGTH;
use crate::errors::VctrlError;
use std::fmt;

/// A 64-byte cryptographic hash used to identify version control objects.
///
/// # Purpose
/// This type represents the output of a 512-bit hash function (like SHA-512).
/// It is used to address and retrieve objects in the
/// [`ObjectStore`](crate::ObjectStore).
///
/// # Design rationale
/// By wrapping the byte array in a tuple struct, we prevent type confusion
/// with other 64-byte arrays. The inner array is private to ensure it can
/// only be constructed via [`Hash::from_bytes`](crate::Hash::from_bytes),
/// which enforces the length invariant.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::Hash;
///
/// let bytes = [0u8; 64];
/// let hash = Hash::from_bytes(&bytes).unwrap();
/// assert_eq!(hash.as_bytes(), &bytes);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Hash([u8; HASH_LENGTH]);

impl Hash {
    /// Creates a `Hash` from a slice of bytes.
    ///
    /// # Design rationale
    /// This is a `const fn` to allow compile-time construction of hashes. The
    /// `while` loop is used because `for` loops and slice iterators were
    /// historically not stable in `const` contexts.
    ///
    /// # Errors
    /// Returns [`VctrlError::InvalidHashLength`](crate::VctrlError::InvalidHashLength)
    /// if the length of `bytes` is not exactly
    /// [`HASH_LENGTH`](crate::HASH_LENGTH).
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::{Hash, VctrlError};
    ///
    /// let valid = Hash::from_bytes(&[0u8; 64]);
    /// assert!(valid.is_ok());
    ///
    /// let invalid = Hash::from_bytes(&[0u8; 32]);
    /// assert!(matches!(invalid, Err(VctrlError::InvalidHashLength(32))));
    /// ```
    pub const fn from_bytes(bytes: &[u8]) -> Result<Self, VctrlError> {
        if bytes.len() != HASH_LENGTH {
            return Err(VctrlError::InvalidHashLength(bytes.len()));
        }
        let mut arr = [0u8; HASH_LENGTH];
        let mut i = 0;
        while i < HASH_LENGTH {
            arr[i] = bytes[i];
            i += 1;
        }
        Ok(Self(arr))
    }

    /// Returns the hash as a fixed-size byte array reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::Hash;
    ///
    /// let bytes = [1u8; 64];
    /// let hash = Hash::from_bytes(&bytes).unwrap();
    /// assert_eq!(hash.as_bytes(), &bytes);
    /// ```
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; HASH_LENGTH] {
        &self.0
    }
}

/// Formats the hash for debugging purposes.
///
/// # Design rationale
/// The default `Debug` implementation for arrays would print all 64 bytes,
/// which clutters log output. This implementation prints only the first 8
/// bytes (16 hex characters) followed by an ellipsis, which is sufficient to
/// distinguish between different hashes in logs.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::Hash;
///
/// let bytes = [0u8; 64];
/// let hash = Hash::from_bytes(&bytes).unwrap();
/// assert_eq!(format!("{hash:?}"), "Hash(0000000000000000…)");
/// ```
impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash(")?;
        for &byte in self.0.iter().take(8) {
            write!(f, "{byte:02x}")?;
        }
        write!(f, "…)")
    }
}

/// Formats the hash as a lowercase hexadecimal string.
///
/// # Design rationale
/// Hexadecimal is the standard representation for cryptographic hashes in
/// version control systems (e.g., Git object IDs). This implementation is
/// zero-allocation and writes directly to the formatter.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::Hash;
///
/// let bytes = [0u8; 64];
/// let hash = Hash::from_bytes(&bytes).unwrap();
/// assert_eq!(format!("{hash}"), "00".repeat(64));
/// ```
impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &byte in &self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}
