//! SHA-384 hash function implementation.
//!
//! # Purpose
//!
//! This module provides the SHA-384 algorithm as defined in FIPS 180-4.
//! SHA-384 is structurally identical to SHA-512 but produces a 384-bit
//! (48-byte) digest instead of 512 bits, and uses a different initial
//! vector (IV). The algorithm operates on 1024-bit message blocks and uses
//! the same 64-bit word operations as SHA-512.
//!
//! # Availability
//!
//! This module is only compiled when the `sha384` feature is enabled. It is
//! part of the default feature set. If the feature is disabled, the module
//! and its re-exports are absent from the crate.
//!
//! # Design Rationale
//!
//! SHA-384 is implemented as a thin wrapper around the core SHA-512
//! [`crate::sha512::Hash`] and [`crate::sha512::State`] types. This reuse
//! avoids code duplication because the message schedule, compression
//! function, and block processing are identical. The differences are:
//!
//! - A distinct initial vector (`new_state()`) replaces the SHA-512 IV.
//! - The final digest is truncated to the first 48 bytes of the 64-byte
//!   SHA-512 output.
//!
//! This approach follows the official specification and guarantees that
//! SHA-384 results are exactly the leftmost 384 bits of the SHA-512 hash
//! computed with the SHA-384 IV.
//!
//! # HMAC and HKDF
//!
//! The module also instantiates HMAC-SHA-384 and HKDF-SHA-384 via the
//! crate's [`impl_hmac!`] and [`impl_hkdf!`] macros, providing ready-to-use
//! keyed-hash and key-derivation functions with 48-byte output and 128-byte
//! block size.
//!
//! # Security Considerations
//!
//! - **No unsafe code**: This module contains no `unsafe` blocks.
//! - **Zeroization**: The [`Hash::zeroize`] method clears internal state to
//!   reduce the lifetime of sensitive data in memory.
//! - **Verification**: The hash verification uses a non-short-circuiting
//!   comparison to reduce timing side-channel leakage (see
//!   [`crate::utils::verify`]).
//!
//! # Internal Mechanism
//!
//! The internal [`Hash`] struct owns a full [`Sha512Hash`] instance, but
//! initialised with the SHA-384 IV. The `new_state()` function creates the
//! IV by loading the 64-byte constant big-endian and converting it to eight
//! 64-bit words. During `finalize`, the inner SHA-512 hasher produces a
//! 64-byte digest; the first 48 bytes are copied to the output. This
//! guarantees that SHA-384 and SHA-512 share the same core logic, reducing
//! the amount of code to audit.
//!
//! # Examples
//!
//! Computing a SHA-384 hash in one shot:
//!
//! ```
//! # use libvctrl_sha512::sha384::Hash;
//! let digest = Hash::hash(b"hello world");
//! assert_eq!(digest.len(), 48);
//! ```
//!
//! Verifying against a known test vector:
//!
//! ```
//! # use libvctrl_sha512::sha384::Hash;
//! let digest = Hash::hash(b"abc");
//! let expected: [u8; 48] = [
//!     0xcb, 0x00, 0x75, 0x3f, 0x45, 0xa3, 0x5e, 0x8b,
//!     0xb5, 0xa0, 0x3d, 0x69, 0x9a, 0xc6, 0x50, 0x07,
//!     0x27, 0x2c, 0x32, 0xab, 0x0e, 0xde, 0xd1, 0x63,
//!     0x1a, 0x8b, 0x60, 0x5a, 0x43, 0xff, 0x5b, 0xed,
//!     0x80, 0x86, 0x07, 0x2b, 0xa1, 0xe7, 0xcc, 0x23,
//!     0x58, 0xba, 0xec, 0xa1, 0x34, 0xc8, 0x25, 0xa7,
//! ];
//! assert_eq!(digest, expected);
//! ```

use crate::sha512::{Hash as Sha512Hash, State};
use crate::utils::load_be;

/// Constructs the SHA-384 initial state vector as defined in FIPS 180-4.
///
/// # Purpose
///
/// This function builds the 8×64-bit IV by loading the constant big-endian
/// bytes specified for SHA-384. The values are the first 64 bits of the
/// fractional parts of the square roots of the 9th through 16th primes,
/// which differ from those used for SHA-512.
///
/// # Design Rationale
///
/// Using a distinct IV is what makes SHA-384 a separate function from
/// SHA-512 while sharing the same compression logic. This design ensures
/// the two algorithms produce unrelated outputs even for identical inputs.
///
/// # How It Works
///
/// The constant 64-byte IV is stored as a byte array. The function loops
/// over eight chunks of 8 bytes, converts each chunk from big-endian to a
/// `u64`, and assembles them into a [`State`] tuple. This state is then
/// used to initialise the inner [`Sha512Hash`] when a new SHA-384 hasher is
/// created.
///
/// # Examples
///
/// The function is private and not directly callable by users. It is shown
/// here for completeness of the module documentation.
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

/// SHA-384 hasher.
///
/// # Purpose
///
/// Wraps the full SHA-512 hasher ([`Sha512Hash`]) but initialises it with
/// the SHA-384 IV and truncates the final digest to 48 bytes. All update
/// operations are delegated to the inner hasher, so the performance
/// characteristics are identical to SHA-512.
///
/// # Design Rationale
///
/// The wrapper pattern avoids duplicating the SHA-512 compression logic.
/// It also makes the implementation easier to audit because the only
/// differences are the IV and the final truncation. The struct is
/// [`Clone`] but not [`Copy`] because the inner hasher owns its state and
/// buffer.
///
/// # Memory Layout
///
/// The struct contains one `Sha512Hash`, which in turn holds:
///
/// - An 8-element `u64` state (64 bytes).
/// - A 128-byte message buffer.
/// - A `usize` buffer index.
/// - A `u128` message length.
///
/// Total size is about 192 bytes. No heap allocation is performed.
///
/// # Security Considerations
///
/// Like SHA-512, the hasher supports zeroization of internal state via
/// [`Hash::zeroize`]. For verification, use [`Hash::verify`] to compare
/// digests with reduced timing side-channel leakage.
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
/// let result = h.finalize();
/// assert_eq!(result.len(), 48);
/// ```
///
/// One-shot hashing with a known answer:
///
/// ```
/// # use libvctrl_sha512::sha384::Hash;
/// let digest = Hash::hash(b"abc");
/// assert_eq!(digest, [
///     0xcb, 0x00, 0x75, 0x3f, 0x45, 0xa3, 0x5e, 0x8b,
///     0xb5, 0xa0, 0x3d, 0x69, 0x9a, 0xc6, 0x50, 0x07,
///     0x27, 0x2c, 0x32, 0xab, 0x0e, 0xde, 0xd1, 0x63,
///     0x1a, 0x8b, 0x60, 0x5a, 0x43, 0xff, 0x5b, 0xed,
///     0x80, 0x86, 0x07, 0x2b, 0xa1, 0xe7, 0xcc, 0x23,
///     0x58, 0xba, 0xec, 0xa1, 0x34, 0xc8, 0x25, 0xa7,
/// ]);
/// ```
#[derive(Clone)]
pub struct Hash(Sha512Hash);

impl Hash {
    /// Creates a new SHA-384 hasher initialised with the SHA-384 IV.
    ///
    /// # Purpose
    ///
    /// All internal buffers are zeroed and the message length is set to
    /// zero. The resulting hasher is ready to accept data via
    /// [`update`](Hash::update).
    ///
    /// # Design Rationale
    ///
    /// The constructor builds the inner SHA-512 hasher with the custom IV
    /// produced by [`new_state`]. This is the only place where the IV is
    /// applied.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_sha512::sha384::Hash;
    /// let hasher = Hash::new();
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

    /// Internal update function exposed to sibling modules.
    ///
    /// # Purpose
    ///
    /// This is the same as [`update`](Hash::update) but with `pub(crate)`
    /// visibility, allowing the crate's HMAC and HKDF implementations to
    /// feed data without calling through the public API. This avoids
    /// unnecessary bounds checks or overhead.
    ///
    /// # Examples
    ///
    /// This function is not part of the public API; it is used internally
    /// by the macro-generated HMAC and HKDF code.
    pub(crate) fn update_inner<T: AsRef<[u8]>>(&mut self, input: T) {
        self.0.update_inner(input);
    }

    /// Feeds data into the SHA-384 hasher.
    ///
    /// # Purpose
    ///
    /// Data is buffered in 128-byte blocks and compressed using the SHA-512
    /// compression function. This method can be called any number of times
    /// before finalising.
    ///
    /// # Design Rationale
    ///
    /// The method delegates to the inner SHA-512 hasher's `update_inner`,
    /// which performs the actual buffering and compression. The public API
    /// uses `impl AsRef<[u8]>` to accept byte slices, arrays, or vectors.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_sha512::sha384::Hash;
    /// let mut h = Hash::new();
    /// h.update(b"some data");
    /// ```
    pub fn update<T: AsRef<[u8]>>(&mut self, input: T) {
        self.update_inner(input);
    }

    /// Finalises the hash and returns the 48-byte SHA-384 digest.
    ///
    /// # Purpose
    ///
    /// Applies the SHA-512 padding (1 bit, zeros, 128-bit length) and
    /// compression, then truncates the 64-byte intermediate result to the
    /// first 48 bytes. This consumes the hasher.
    ///
    /// # Design Rationale
    ///
    /// The inner SHA-512 hasher produces a 64-byte digest. SHA-384 is
    /// defined as the leftmost 48 bytes of that digest when computed with
    /// the SHA-384 IV. Therefore, the truncation is simple and correct.
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
    pub fn finalize(self) -> [u8; 48] {
        let mut out = [0u8; 48];
        out.copy_from_slice(&self.0.finalize()[..48]);
        out
    }

    /// One-shot SHA-384 hash computation.
    ///
    /// # Purpose
    ///
    /// Creates a new hasher, feeds the entire input, and returns the final
    /// 48-byte digest. This is a convenience shortcut.
    ///
    /// # Design Rationale
    ///
    /// This method simplifies the most common use case: hashing a complete
    /// message in one call. It is equivalent to calling
    /// [`Hash::new`], [`Hash::update`], and [`Hash::finalize`] in sequence.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use libvctrl_sha512::sha384::Hash;
    /// let digest = Hash::hash(b"quick hash");
    /// assert_eq!(digest.len(), 48);
    /// ```
    pub fn hash<T: AsRef<[u8]>>(input: T) -> [u8; 48] {
        let mut h = Self::new();
        h.update(input);
        h.finalize()
    }

    /// Overwrites the internal state with zeros and inserts a compiler fence.
    ///
    /// # Purpose
    ///
    /// Delegates to [`Sha512Hash::zeroize`] on the inner hasher. This is a
    /// best-effort measure to clear sensitive intermediate data from memory.
    ///
    /// # Security Note
    ///
    /// This method does not guarantee complete erasure on all targets due
    /// to possible register spilling or compiler optimizations. It is
    /// intended to reduce the window for memory disclosure attacks, but for
    /// high-security contexts, additional OS-level memory protection should
    /// be used.
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
    /// Returns a new SHA-384 hasher, identical to [`Hash::new`].
    fn default() -> Self {
        Self::new()
    }
}

// The following macro invocations generate HMAC-SHA-384 and HKDF-SHA-384
// structures directly inside this module. See the crate-level documentation for
// the `impl_hmac!` and `impl_hkdf!` macros for details on usage.
impl_hmac!(Hash, 48, 128);
impl_hkdf!(Hash, 48, 128);
