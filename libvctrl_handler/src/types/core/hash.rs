//! Content-addressable hash type.
//!
//! This module defines [`Hash`], a fixed‑size cryptographic digest that
//! uniquely identifies objects in the version‑control system. The hash
//! is stored as an owned array of bytes (`[u8; HASH_LENGTH]`) and provides
//! fallible construction, byte‑level access, and human‑readable formatting.

use crate::constants::HASH_LENGTH;
use crate::errors::VctrlError;
use std::fmt;

/// A fixed‑size hash value used for content addressing.
///
/// Internally the hash is stored as a byte array of length
/// [`HASH_LENGTH`]. This design ensures:
///
/// - **Stack allocation** – no heap overhead, trivially `Copy`.
/// - **Efficient comparison** – equality checks are constant‑time and
///   inlined by the compiler.
/// - **No lifetime parameters** – the struct owns its data, simplifying
///   storage in other types like [`Commit`] and [`Tree`].
///
/// # Examples
///
/// Creating a hash from a correctly‑sized slice:
///
/// ```
/// use libvctrl_handler::types::core::Hash;
/// use libvctrl_handler::constants::HASH_LENGTH;
///
/// let bytes = [0xab; HASH_LENGTH];
/// let hash = Hash::from_bytes(&bytes).unwrap();
/// assert_eq!(hash.as_bytes(), &bytes);
/// ```
///
/// An incorrectly‑sized slice produces an error:
///
/// ```
/// use libvctrl_handler::types::core::Hash;
/// let short = [0u8; 10];
/// assert!(Hash::from_bytes(&short).is_err());
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Hash([u8; HASH_LENGTH]);

impl Hash {
    /// Attempts to create a `Hash` from a byte slice.
    ///
    /// The slice must have exactly [`HASH_LENGTH`] bytes, otherwise an
    /// [`VctrlError::InvalidHashLength`] is returned.
    ///
    /// # Why not `new`?
    ///
    /// The constructor is fallible because the length constraint is a
    /// domain invariant. Forcing callers to handle the error at creation
    /// time prevents invalid hashes from ever existing.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::InvalidHashLength`] if `bytes.len() != HASH_LENGTH`.
    ///
    /// # Examples
    ///
    /// Successful construction:
    ///
    /// ```
    /// use libvctrl_handler::types::core::Hash;
    /// use libvctrl_handler::constants::HASH_LENGTH;
    ///
    /// let data = [0xff; HASH_LENGTH];
    /// let hash = Hash::from_bytes(&data).unwrap();
    /// # let _ = hash;
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

    /// Returns the raw bytes of the hash as a fixed‑size array reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::Hash;
    /// use libvctrl_handler::constants::HASH_LENGTH;
    ///
    /// let data = [0x42; HASH_LENGTH];
    /// let hash = Hash::from_bytes(&data).unwrap();
    /// let bytes = hash.as_bytes();
    /// assert_eq!(bytes[0], 0x42);
    /// ```
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; HASH_LENGTH] {
        &self.0
    }
}

/// Debug representation shows the first 8 bytes in hexadecimal, followed by an
/// ellipsis to keep output compact while still useful for debugging.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::types::core::Hash;
/// use libvctrl_handler::constants::HASH_LENGTH;
///
/// let data = [0x1a; HASH_LENGTH];
/// let hash = Hash::from_bytes(&data).unwrap();
/// let debug_str = format!("{:?}", hash);
/// assert!(debug_str.starts_with("Hash(1a1a1a1a1a1a1a1a…"));
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

/// Full hexadecimal string representation of the hash.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::types::core::Hash;
/// use libvctrl_handler::constants::HASH_LENGTH;
///
/// let data = [0xab; HASH_LENGTH];
/// let hash = Hash::from_bytes(&data).unwrap();
/// let hex_string = format!("{}", hash);
/// assert_eq!(hex_string.len(), HASH_LENGTH * 2);
/// assert!(hex_string.chars().all(|c| c.is_ascii_hexdigit()));
/// ```
impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &byte in &self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}
