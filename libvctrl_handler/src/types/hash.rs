//! # Cryptographic Hash – The Foundation of Content Addressability
//!
//! The `Hash` type is the heart of `libvctrl`’s content‑addressable storage.
//! Every object (blob, tree, commit, tag) is identified by a 64‑byte SHA‑512 hash
//! of its content. This design provides **integrity** (any change to content
//! changes its hash) and **deduplication** (identical content yields the same
//! hash).
//!
//! ## Why SHA‑512?
//!
//! - **Security**: 256‑bit collision resistance and 512‑bit preimage resistance.
//!   Even against quantum computers, effective security exceeds 128 bits.
//! - **Performance**: Designed for 64‑bit architectures, SHA‑512 often outperforms
//!   SHA‑256 on modern hardware (x86‑64, ARM64).
//! - **Simplicity**: A fixed 64‑byte length means no dynamic allocation and no
//!   generics – `Hash` is a simple `[u8; 64]` newtype.
//!
//! ## Validation
//!
//! The constructor [`from_bytes`](Hash::from_bytes) enforces that the input slice
//! is exactly [`HASH_LENGTH`](crate::HASH_LENGTH) (64) bytes. Any other length
//! returns [`VctrlError::InvalidHashLength`]. This strictness ensures that
//! hashes are always well‑formed and comparable.
//!
//! ## Display vs Debug
//!
//! - [`Display`] produces the full 128‑character hex string (for logs, user output).
//! - [`Debug`] shows only the first 8 bytes followed by an ellipsis, making it
//!   concise in debugging output.
//!
//! ## Usage in the Ecosystem
//!
//! - Stored in [`TreeEntry`](crate::TreeEntry) and [`Commit`](crate::Commit).
//! - Looked up in [`ObjectStore`](crate::ObjectStore) and [`RefStore`](crate::RefStore).
//! - Used as keys in maps and sets (implements `Hash`, `PartialEq`, `Eq`, `Ord`).
//!
//! ## Example: Creating and Displaying a Hash
//!
//! ```rust
//! use libvctrl_handler::{Hash, HASH_LENGTH};
//!
//! // Create a hash from a 64‑byte array
//! let bytes = [0xAA; HASH_LENGTH];
//! let hash = Hash::from_bytes(&bytes).unwrap();
//!
//! // Display full hex
//! assert_eq!(format!("{}", hash), "aa".repeat(HASH_LENGTH));
//!
//! // Debug shows first 8 bytes only
//! assert_eq!(format!("{:?}", hash), "Hash(aaaaaaaa…)");
//! ```
//!
//! ## Safety and Performance
//!
//! `Hash` is `Copy` and `Send + Sync`, making it cheap to pass around and safe
//! across threads. The `const fn` constructor enables compile‑time hash creation
//! in `const` contexts.

use crate::constants::HASH_LENGTH;
use crate::errors::VctrlError;
use std::fmt;

/// A content hash – a fixed‑size array of 64 bytes (SHA‑512).
///
/// This is the fundamental identifier for all objects in the system.
/// A `Hash` is **always** 64 bytes; any attempt to create one with
/// a different length will fail with [`VctrlError::InvalidHashLength`].
///
/// # Construction
///
/// Use [`Hash::from_bytes`] to convert a byte slice. This function validates
/// the length and returns `Err` if it does not match [`HASH_LENGTH`].
///
/// ```rust
/// use libvctrl_handler::{Hash, HASH_LENGTH};
///
/// // Correct length → succeeds.
/// let h = Hash::from_bytes(&[0x00; HASH_LENGTH]).unwrap();
///
/// // Wrong length → fails.
/// assert!(Hash::from_bytes(&[0; 10]).is_err());
/// ```
///
/// # Display and Debug
///
/// - [`Display`] prints the full 64‑byte hex string (128 characters).
/// - [`Debug`] prints only the first 8 bytes followed by `…` for brevity.
///
/// ```rust
/// use libvctrl_handler::{Hash, HASH_LENGTH};
///
/// let h = Hash::from_bytes(&[0xAB; HASH_LENGTH]).unwrap();
///
/// // Display: "abababababababababab..."
/// // Debug:  "Hash(abababababababab…)"
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Hash([u8; HASH_LENGTH]);

impl Hash {
    /// Creates a `Hash` from a byte slice.
    ///
    /// This is the **only** way to construct a `Hash` from raw bytes. It ensures
    /// that the length is correct, preserving the invariant that every `Hash`
    /// is exactly 64 bytes.
    ///
    /// # Errors
    /// Returns [`VctrlError::InvalidHashLength`] if `bytes.len()` ≠ [`HASH_LENGTH`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # use libvctrl_handler::*;
    /// let data = [0xAA; HASH_LENGTH];
    /// let hash = Hash::from_bytes(&data).unwrap();
    /// assert_eq!(hash.as_bytes(), &data);
    /// ```
    ///
    /// # Performance
    /// The function is `const fn`, so it can be used in compile‑time contexts.
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

    /// Returns a reference to the underlying 64‑byte array.
    ///
    /// This is the primary method to access the raw bytes, e.g., for passing
    /// to a hash function (though a `Hash` is typically produced by a hasher,
    /// not consumed by one) or for encoding.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; HASH_LENGTH] {
        &self.0
    }
}

impl fmt::Debug for Hash {
    /// Formats the hash for debugging purposes.
    ///
    /// The output is `Hash(########…)` where `########` is the first 8 bytes
    /// of the hash in hex. This keeps debug output concise while still
    /// providing enough information to distinguish hashes in logs.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash(")?;
        for &byte in self.0.iter().take(8) {
            write!(f, "{byte:02x}")?;
        }
        write!(f, "…)")
    }
}

impl fmt::Display for Hash {
    /// Formats the hash as a full 128‑character lowercase hex string.
    ///
    /// This is the canonical textual representation of a hash, used in
    /// user‑facing outputs such as commit logs, reference names, and error
    /// messages.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &byte in &self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}
