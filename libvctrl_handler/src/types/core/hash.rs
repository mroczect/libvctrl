//! Content-addressable hash type.
//!
//! # Purpose
//!
//! This module defines [`Hash`], a fixed-size cryptographic digest that
//! uniquely identifies objects in the version-control system. The hash is
//! stored as an owned array of bytes (`[u8; HASH_LENGTH]`) and provides
//! fallible construction, byte-level access, and human-readable formatting.
//!
//! # Design Rationale
//!
//! Content addressing is the foundation of the version control object
//! model. Every object is identified by the cryptographic hash of its
//! serialized bytes. This provides:
//!
//! - **Integrity**: Any change to an object's bytes produces a different
//!   hash, making corruption or tampering detectable.
//! - **Deduplication**: Identical objects produce identical hashes, allowing
//!   storage backends to store each unique object only once.
//! - **Immutability**: Since the hash depends on the content, objects
//!   cannot be mutated without changing their identity.
//!
//! The [`Hash`] type is intentionally simple. It is a thin wrapper around a
//! fixed-size byte array, offering no cryptographic operations itself.
//! Hashing is performed by implementations of the [`Hasher`](crate::Hasher)
//! trait; [`Hash`] merely represents the result.
//!
//! # Why a fixed-size array?
//!
//! The internal storage is `[u8; HASH_LENGTH]` rather than `Vec<u8>` for
//! several reasons:
//!
//! - **Stack allocation**: No heap allocation is needed, making hashes
//!   trivially copyable and cache-friendly.
//! - **Constant size**: The size is known at compile time, which enables
//!   optimizations and simplifies embedding in other types.
//! - **No lifetime parameters**: The struct owns its data, avoiding borrow
//!   checker complexity when storing hashes in collections or other
//!   objects.
//!
//! # Memory Layout
//!
//! A [`Hash`] occupies exactly 64 bytes on the stack (assuming
//! [`HASH_LENGTH`](crate::constants::HASH_LENGTH) is 64). It contains no
//! pointers and no heap references. The type derives [`Copy`], so assigning
//! or passing a hash by value is a cheap bitwise copy.
//!
//! # Derived Traits
//!
//! The struct derives several standard traits:
//!
//! - [`Clone`] and [`Copy`]: enables cheap duplication.
//! - [`PartialEq`] and [`Eq`]: allows equality comparisons.
//! - [`Hash`]: permits use as a key in hash maps and sets.
//! - [`PartialOrd`] and [`Ord`]: enables sorting of hashes, which is useful
//!   for deterministic iteration over object collections.
//!
//! # Relationship to Other Types
//!
//! - [`Blob`](crate::Blob), [`Tree`](crate::Tree),
//!   [`Commit`](crate::Commit), and [`Tag`](crate::Tag) all use [`Hash`]
//!   to reference other objects.
//! - [`TreeEntry`](crate::TreeEntry) stores the hash of the object it
//!   points to.
//! - [`Hasher`](crate::Hasher) produces [`Hash`] values from raw bytes.
//! - [`ObjectStore`](crate::ObjectStore) uses [`Hash`] as the primary key
//!   for storing and retrieving objects.
//!
//! # Examples
//!
//! Creating a hash from a correctly-sized slice:
//!
//! ```
//! use libvctrl_handler::types::core::Hash;
//! use libvctrl_handler::constants::HASH_LENGTH;
//!
//! let bytes = [0xab; HASH_LENGTH];
//! let hash = Hash::from_bytes(&bytes).unwrap();
//! assert_eq!(hash.as_bytes(), &bytes);
//! ```
//!
//! An incorrectly-sized slice produces an error:
//!
//! ```
//! use libvctrl_handler::types::core::Hash;
//! let short = [0u8; 10];
//! assert!(Hash::from_bytes(&short).is_err());
//! ```

use crate::constants::HASH_LENGTH;
use crate::errors::VctrlError;
use std::fmt;

/// A fixed-size hash value used for content addressing.
///
/// # Overview
///
/// `Hash` is a newtype over a fixed-size byte array of length
/// [`HASH_LENGTH`](crate::constants::HASH_LENGTH). It represents a
/// cryptographic digest that uniquely identifies a version control object.
/// The struct is deliberately minimal, exposing only byte-level access and
/// formatting; all actual hashing is the responsibility of the
/// [`Hasher`](crate::Hasher) trait.
///
/// # Design Rationale
///
/// Internally the hash is stored as a byte array of length
/// [`HASH_LENGTH`](crate::constants::HASH_LENGTH). This design ensures:
///
/// - **Stack allocation**: No heap overhead, trivially `Copy`.
/// - **Efficient comparison**: Equality checks are constant-time and
///   inlined by the compiler.
/// - **No lifetime parameters**: The struct owns its data, simplifying
///   storage in other types like [`Commit`](crate::Commit) and
///   [`Tree`](crate::Tree).
///
/// The public fields are intentionally hidden behind the private array to
/// enforce the length invariant. The only way to construct a `Hash` is via
/// [`Hash::from_bytes`], which validates the input length.
///
/// # Examples
///
/// Creating a hash from a correctly-sized slice:
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
/// An incorrectly-sized slice produces an error:
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
    /// The slice must have exactly [`HASH_LENGTH`](crate::constants::HASH_LENGTH)
    /// bytes, otherwise a [`VctrlError::InvalidHashLength`] is returned.
    ///
    /// # Why not `new`?
    ///
    /// The constructor is fallible because the length constraint is a
    /// domain invariant. Forcing callers to handle the error at creation
    /// time prevents invalid hashes from ever existing. If the constructor
    /// were infallible and simply truncated or padded input, callers might
    /// accidentally produce hashes that do not match the actual digest
    /// length, leading to subtle bugs.
    ///
    /// # How It Works Internally
    ///
    /// The method first checks the input length. If it is not exactly
    /// [`HASH_LENGTH`](crate::constants::HASH_LENGTH), it returns
    /// [`VctrlError::InvalidHashLength`] with the actual length. If the
    /// length is valid, the bytes are copied into a stack-allocated array
    /// using a manual `while` loop. A `while` loop is used instead of a
    /// `for` loop because this is a `const fn` and the loop is guaranteed
    /// to terminate within the compile-time evaluation context.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::InvalidHashLength`] if
    /// `bytes.len() != HASH_LENGTH`.
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
    /// assert_eq!(hash.as_bytes(), &data);
    /// ```
    ///
    /// Failure due to incorrect length:
    ///
    /// ```
    /// use libvctrl_handler::types::core::Hash;
    ///
    /// let too_short = [0u8; 32];
    /// let err = Hash::from_bytes(&too_short).unwrap_err();
    /// assert!(matches!(err, libvctrl_handler::VctrlError::InvalidHashLength(32)));
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

    /// Returns the raw bytes of the hash as a fixed-size array reference.
    ///
    /// # Returns
    ///
    /// A reference to the internal byte array of length
    /// [`HASH_LENGTH`](crate::constants::HASH_LENGTH).
    ///
    /// # Why a fixed-size array reference?
    ///
    /// Returning `&[u8; HASH_LENGTH]` rather than `&[u8]` communicates the
    /// exact length to the compiler and callers, enabling additional
    /// compile-time checks. The caller can always coerce the array reference
    /// to a slice if needed.
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
    /// assert_eq!(bytes.len(), HASH_LENGTH);
    /// ```
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; HASH_LENGTH] {
        &self.0
    }
}

/// Debug representation shows the first 8 bytes in hexadecimal, followed by
/// three dots to keep output compact while still useful for debugging.
///
/// # Purpose
///
/// A full 64-byte hash rendered as 128 hex characters is unwieldy for
/// debugging. This implementation prints only the first 8 bytes (16 hex
/// characters) followed by `…`, providing enough information to identify
/// the hash in most practical debugging scenarios without flooding log
/// output.
///
/// # How It Works Internally
///
/// The implementation iterates over the first 8 bytes of the internal array
/// and writes each byte as two lowercase hexadecimal digits. After the
/// eighth byte, it writes three dots. The exact output format is:
///
/// ```text
/// Hash(0102030405060708…)
/// ```
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
/// # Purpose
///
/// This implementation provides the complete 128-character lowercase
/// hexadecimal representation of the hash. It is suitable for user-facing
/// output, logging, serialization to text formats, and debugging when the
/// full hash is required.
///
/// # How It Works Internally
///
/// The implementation iterates over every byte in the internal array and
/// writes it as two lowercase hexadecimal digits using the `{:02x}` format
/// specifier. The resulting string has exactly
/// [`HASH_LENGTH`](crate::constants::HASH_LENGTH) * 2 characters.
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
