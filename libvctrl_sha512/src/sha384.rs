//! # SHA-384 Hash
//!
//! This module provides the SHA-384 cryptographic hash function as specified
//! in FIPS 180-4. SHA-384 is a variant of SHA-512 that uses a different
//! initialization vector and truncates the final digest to 48 bytes.
//!
//! ## Design rationale
//!
//! SHA-384 shares the same compression function and message schedule as
//! SHA-512. Instead of duplicating the core algorithm, this module wraps
//! [`crate::sha512::Hash`] and overrides only the initialization vector and
//! output length. This reduces code size, simplifies auditing, and guarantees
//! consistency between the two hash functions.
//!
//! ## How it works
//!
//! The [`Hash`] struct holds an internal [`crate::sha512::Hash`] instance with
//! a custom state. During finalization, the full 64-byte SHA-512 digest is
//! computed and then truncated to the first 48 bytes.
//!
//! The module also invokes the [`impl_hmac!`] and [`impl_hkdf!`] macros to
//! generate HMAC-SHA-384 and HKDF-SHA-384 implementations.

use crate::sha512::{Hash as Sha512Hash, State};
use crate::utils::load_be;

/// Creates a SHA-384 initialization vector.
///
/// This internal helper constructs a [`State`] from the SHA-384 initial
/// hash values defined in FIPS 180-4. It returns a state that will be used
/// as the starting point for SHA-384 compression.
#[inline]
fn new_state() -> State {
    const IV: [u8; 64] = [
        0xcb, 0xbb, 0x9d, 0x5d, 0xc1, 0x05, 0x9e, 0xd8, 0x62, 0x9a, 0x29, 0x2a, 0x36, 0x7c, 0xd5,
        0x07, 0x91, 0x59, 0x01, 0x5a, 0x30, 0x70, 0xdd, 0x17, 0x15, 0x2f, 0xec, 0xd8, 0xf7, 0x0e,
        0x59, 0x39, 0x67, 0x33, 0x26, 0x67, 0xff, 0xc0, 0x0b, 0x31, 0x8e, 0xb4, 0x4a, 0x87, 0x68,
        0x58, 0x15, 0x11, 0xdb, 0x0c, 0x2e, 0x0d, 0x64, 0xf9, 0x8f, 0xa7, 0x47, 0xb5, 0x48, 0x1d,
        0xbe, 0xfa, 0x4f, 0xa4,
    ];
    let mut t = [0u64; 8];
    for (i, e) in t.iter_mut().enumerate() {
        *e = load_be(&IV, i * 8);
    }
    State(t)
}

/// SHA-384 hash context.
///
/// This struct represents an incremental SHA-384 computation. It wraps
/// [`crate::sha512::Hash`] with a SHA-384-specific initialization vector and
/// truncates the final digest to 48 bytes.
///
/// # Why this struct exists
///
/// SHA-384 is defined as a truncated SHA-512 with a different IV. By
/// embedding the SHA-512 core, this struct avoids code duplication and
/// ensures the two algorithms stay synchronized.
///
/// # How it works
///
/// The internal SHA-512 state is initialized with [`new_state`]. Updates
/// are forwarded to the inner hash. Finalization computes the full 64-byte
/// SHA-512 digest and returns only the first 48 bytes.
///
/// # Examples
///
/// Incremental hashing:
///
/// ```
/// # use libvctrl_sha512::sha384::Hash;
/// let mut h = Hash::new();
/// h.update(b"hello ");
/// h.update(b"world");
/// let digest = h.finalize();
/// assert_eq!(digest.len(), 48);
/// ```
///
/// One-shot hashing:
///
/// ```
/// # use libvctrl_sha512::sha384::Hash;
/// let digest = Hash::hash(b"abc");
/// assert_eq!(digest.len(), 48);
/// ```
#[derive(Clone)]
pub struct Hash(Sha512Hash);

impl Hash {
    /// Creates a new SHA-384 hash context.
    ///
    /// The context is initialized with the SHA-384 initialization vector and
    /// zero length.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_sha512::sha384::Hash;
    /// let mut h = Hash::new();
    /// h.update(b"data");
    /// let digest = h.finalize();
    /// assert_eq!(digest.len(), 48);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self(Sha512Hash {
            state: new_state(),
            r: 0,
            w: [0u8; 128],
            len: 0,
        })
    }

    /// Internal update method shared with the wrapped SHA-512 core.
    ///
    /// This method is `pub(crate)` and not part of the public API. It forwards
    /// the input to the inner SHA-512 hash.
    pub(crate) fn update_inner<T: AsRef<[u8]>>(&mut self, input: T) {
        self.0.update_inner(input);
    }

    /// Feeds data into the SHA-384 computation.
    ///
    /// This method can be called multiple times. The input is processed
    /// immediately; no internal buffering beyond the SHA-512 block size is
    /// performed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_sha512::sha384::Hash;
    /// let mut h = Hash::new();
    /// h.update(b"chunk1");
    /// h.update(b"chunk2");
    /// let digest = h.finalize();
    /// assert_eq!(digest.len(), 48);
    /// ```
    pub fn update<T: AsRef<[u8]>>(&mut self, input: T) {
        self.update_inner(input);
    }

    /// Finalizes the SHA-384 computation and returns the 48-byte digest.
    ///
    /// This consumes the context. The full 64-byte SHA-512 digest is computed
    /// and truncated to the first 48 bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_sha512::sha384::Hash;
    /// let digest = Hash::hash(b"abc");
    /// assert_eq!(digest.len(), 48);
    /// ```
    #[must_use]
    pub fn finalize(self) -> [u8; 48] {
        let mut out = [0u8; 48];
        out.copy_from_slice(&self.0.finalize()[..48]);
        out
    }

    /// One-shot SHA-384 hash computation.
    ///
    /// This convenience method creates a new context, feeds the entire input,
    /// finalizes it, and returns the digest.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_sha512::sha384::Hash;
    /// let digest = Hash::hash(b"hello");
    /// assert_eq!(digest.len(), 48);
    /// ```
    pub fn hash<T: AsRef<[u8]>>(input: T) -> [u8; 48] {
        let mut h = Self::new();
        h.update(input);
        h.finalize()
    }

    /// Zeroizes the internal state.
    ///
    /// This method clears the wrapped SHA-512 state and any buffered data,
    /// preventing sensitive information from remaining in memory.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_sha512::sha384::Hash;
    /// let mut h = Hash::new();
    /// h.update(b"secret");
    /// h.zeroize();
    /// ```
    pub fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

impl Default for Hash {
    /// Creates a default SHA-384 hash context.
    ///
    /// This is equivalent to calling [`Hash::new`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_sha512::sha384::Hash;
    /// let h = Hash::default();
    /// let digest = h.finalize();
    /// assert_eq!(digest.len(), 48);
    /// ```
    fn default() -> Self {
        Self::new()
    }
}

impl_hmac!(Hash, 48, 128);
impl_hkdf!(Hash, 48, 128);
