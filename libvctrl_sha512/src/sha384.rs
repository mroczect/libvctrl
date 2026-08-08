//! # SHA‑384, HMAC‑SHA‑384 & HKDF‑SHA‑384 (FIPS 180‑4 / RFC 2104 / RFC 5869)
//!
//! This module provides **SHA‑384** and its derived constructions, available
//! when the crate’s `sha384` feature is enabled (which is the default).
//!
//! SHA‑384 is a member of the SHA‑2 family. It is **structurally identical**
//! to SHA‑512 but differs in two ways:
//!
//! 1. It uses a **different initialisation vector (IV)**.
//! 2. The final 512‑bit output is **truncated to 384 bits** (48 bytes).
//!
//! Because the compression function is identical, this module reuses the
//! [`State`] and [`W`] internals from the parent `sha512` module, only
//! replacing the IV and output length.
//!
//! ## Security
//!
//! SHA‑384 provides **192‑bit security** against collision attacks and
//! **384‑bit preimage resistance**. It is often preferred over SHA‑512
//! when a shorter digest suffices, and its length extension attack surface
//! is slightly smaller due to the truncation.
//!
//! The HMAC and HKDF variants follow the same standards as their SHA‑512
//! counterparts (RFC 2104 and RFC 5869), producing 48‑byte tags / PRKs.
//!
//! ## Usage
//!
//! Enable the `sha384` feature (on by default) and import the types from
//! the crate root:
//!
//! ```rust,ignore
//! use libvctrl_sha512::sha384::{Hash, HMAC, HKDF};
//! ```
//!
//! All APIs mirror the SHA‑512 versions exactly, except for the output size
//! (48 bytes instead of 64).
//!
//! ## Examples
//!
//! ### SHA‑384 hashing
//!
//! ```rust
//! use libvctrl_sha512::sha384::Hash;
//!
//! let digest = Hash::hash(b"hello world");
//! assert_eq!(digest.len(), 48);
//! ```
//!
//! ### HMAC‑SHA‑384 streaming
//!
//! ```rust
//! use libvctrl_sha512::sha384::HMAC;
//!
//! let mut ctx = HMAC::new(b"key");
//! ctx.update(b"data");
//! let mac = ctx.finalize();
//! assert_eq!(mac.len(), 48);
//! ```
//!
//! ### HKDF‑SHA‑384 key derivation
//!
//! ```rust
//! use libvctrl_sha512::sha384::HKDF;
//!
//! let prk = HKDF::extract(b"salt", b"ikm");
//! let mut key = [0u8; 32];
//! HKDF::expand(&mut key, prk, b"info");
//! ```

use crate::sha512::{Hash as Sha512Hash, State};
use crate::utils::load_be;
use crate::utils::verify;

/// Initialise a SHA‑384 working state with the standard IV.
///
/// SHA‑384 uses a different set of initial hash values (H⁰ … H⁷) compared
/// to SHA‑512.  This function creates a [`State`] with those values.
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

/// SHA‑384 hash state.
///
/// This is a thin wrapper around the SHA‑512 [`Sha512Hash`] type.  It uses
/// the SHA‑384 IV and truncates the final output to 48 bytes.  All other
/// aspects (message scheduling, compression) are identical to SHA‑512.
///
/// # Examples
///
/// ```rust
/// use libvctrl_sha512::sha384::Hash;
///
/// let mut h = Hash::new();
/// h.update(b"data");
/// let digest = h.finalize();
/// assert_eq!(digest.len(), 48);
/// ```
#[derive(Copy, Clone)]
pub struct Hash(Sha512Hash);

impl Hash {
    /// Create a new SHA‑384 hasher.
    ///
    /// The initial state is set to the SHA‑384 IV, the internal buffer and
    /// length counter are zeroed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use libvctrl_sha512::sha384::Hash;
    ///
    /// let hasher = Hash::new();
    /// ```
    pub fn new() -> Self {
        Hash(Sha512Hash {
            state: new_state(),
            r: 0,
            w: [0u8; 128],
            len: 0,
        })
    }

    /// Feed additional data into the hasher (crate‑internal).
    ///
    /// This is `pub(crate)` so that the `sha384` module can use the same
    /// buffering logic as SHA‑512.
    pub(crate) fn _update<T: AsRef<[u8]>>(&mut self, input: T) {
        self.0.update(input)
    }

    /// Feed additional data into the hasher.
    ///
    /// This method can be called multiple times before finalisation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use libvctrl_sha512::sha384::Hash;
    ///
    /// let mut h = Hash::new();
    /// h.update(b"hello ");
    /// h.update(b"world");
    /// ```
    pub fn update<T: AsRef<[u8]>>(&mut self, input: T) {
        self._update(input)
    }

    /// Finalize the hash and produce the 48‑byte SHA‑384 digest.
    ///
    /// This consumes the hasher, computes the full SHA‑512 digest internally,
    /// and truncates the result to the first 48 bytes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use libvctrl_sha512::sha384::Hash;
    ///
    /// let digest = Hash::new().finalize();
    /// assert_eq!(digest.len(), 48);
    /// ```
    pub fn finalize(self) -> [u8; 48] {
        let mut out = [0u8; 48];
        out.copy_from_slice(&self.0.finalize()[..48]);
        out
    }

    /// One‑shot hashing: compute the SHA‑384 digest of `input`.
    ///
    /// This is equivalent to creating a fresh `Hash`, calling
    /// [`update`](Hash::update) once, and then [`finalize`](Hash::finalize).
    ///
    /// # Example
    ///
    /// ```rust
    /// use libvctrl_sha512::sha384::Hash;
    ///
    /// let digest = Hash::hash(b"hello world");
    /// assert_eq!(digest.len(), 48);
    /// ```
    pub fn hash<T: AsRef<[u8]>>(input: T) -> [u8; 48] {
        let mut h = Self::new();
        h.update(input);
        h.finalize()
    }
}

impl Default for Hash {
    /// Returns a new `Hash` instance with the SHA‑384 IV.
    fn default() -> Self {
        Self::new()
    }
}

/// HMAC‑SHA‑384 state for incremental MAC computation.
///
/// Internally holds a SHA‑384 hasher preloaded with the **inner pad** (`ipad`)
/// and a copy of the key XORed with `ipad` that will be transformed to the
/// **outer pad** (`opad`) during finalisation.
///
/// # Security note
///
/// The `Drop` implementation zeroises the `padded` buffer to prevent the key
/// from lingering on the stack.  The inner hasher is not zeroised (it contains
/// only the intermediate hash state, not the key), which is consistent with
/// standard practice.
pub struct HMAC {
    ih: Hash,
    padded: [u8; 128],
}

impl Drop for HMAC {
    fn drop(&mut self) {
        self.padded.fill(0);
    }
}

impl HMAC {
    /// Compute the HMAC‑SHA‑384 of `input` under key `k` (one‑shot).
    ///
    /// # Arguments
    ///
    /// * `input` – The message to authenticate.
    /// * `k`     – The secret key.
    ///
    /// # Returns
    ///
    /// A `[u8; 48]` containing the HMAC tag.
    ///
    /// # Example
    ///
    /// ```rust
    /// use libvctrl_sha512::sha384::HMAC;
    ///
    /// let mac = HMAC::mac(b"message", b"key");
    /// assert_eq!(mac.len(), 48);
    /// ```
    pub fn mac<T: AsRef<[u8]>, U: AsRef<[u8]>>(input: T, k: U) -> [u8; 48] {
        let mut hmac = Self::new(k);
        hmac.update(input);
        hmac.finalize()
    }

    /// Create a new streaming HMAC‑SHA‑384 state.
    ///
    /// The key is processed immediately: if it exceeds the block size
    /// (128 bytes), it is first hashed with SHA‑384.  The resulting key is
    /// XORed with `ipad` and fed into the inner hasher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use libvctrl_sha512::sha384::HMAC;
    ///
    /// let mut ctx = HMAC::new(b"key");
    /// ctx.update(b"data");
    /// let mac = ctx.finalize();
    /// ```
    pub fn new(k: impl AsRef<[u8]>) -> Self {
        let k = k.as_ref();
        let mut hk = [0u8; 48];
        let k2 = if k.len() > 128 {
            hk.copy_from_slice(&Hash::hash(k));
            &hk[..]
        } else {
            k
        };
        let mut padded = [0x36; 128];
        for (p, &k) in padded.iter_mut().zip(k2.iter()) {
            *p ^= k;
        }
        let mut ih = Hash::new();
        ih.update(&padded[..]);
        HMAC { ih, padded }
    }

    /// Feed additional data into the HMAC computation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use libvctrl_sha512::sha384::HMAC;
    ///
    /// let mut ctx = HMAC::new(b"key");
    /// ctx.update(b"chunk1");
    /// ctx.update(b"chunk2");
    /// ```
    pub fn update(&mut self, input: impl AsRef<[u8]>) {
        self.ih.update(input);
    }

    /// Finalize the HMAC computation and return the 48‑byte tag.
    ///
    /// This method consumes the state, completes the outer hash, and produces
    /// the final MAC.
    ///
    /// # Example
    ///
    /// ```rust
    /// use libvctrl_sha512::sha384::HMAC;
    ///
    /// let mut ctx = HMAC::new(b"key");
    /// ctx.update(b"data");
    /// let mac = ctx.finalize();
    /// assert_eq!(mac.len(), 48);
    /// ```
    pub fn finalize(mut self) -> [u8; 48] {
        for p in self.padded.iter_mut() {
            *p ^= 0x6a; // ipad XOR opad == 0x36 ^ 0x5c = 0x6a
        }
        let mut oh = Hash::new();
        oh.update(&self.padded[..]);
        oh.update(&self.ih.finalize()[..]);
        oh.finalize()
    }

    /// Finalize and compare against `expected` in constant time.
    ///
    /// Uses [`utils::verify`](crate::utils::verify) for a timing‑attack
    /// resistant comparison.
    ///
    /// # Returns
    ///
    /// `true` if the computed MAC matches `expected`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use libvctrl_sha512::sha384::HMAC;
    ///
    /// let mut ctx = HMAC::new(b"key");
    /// ctx.update(b"message");
    /// let expected = [0u8; 48]; // dummy
    /// let is_valid = ctx.finalize_verify(&expected);
    /// ```
    #[inline]
    pub fn finalize_verify(self, expected: &[u8; 48]) -> bool {
        let out = self.finalize();
        verify(&out, expected)
    }

    /// One‑shot verification of `input` under key `k` against `expected`.
    ///
    /// # Returns
    ///
    /// `true` iff the computed tag equals `expected`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use libvctrl_sha512::sha384::HMAC;
    ///
    /// let tag = HMAC::mac(b"message", b"key");
    /// assert!(HMAC::verify(b"message", b"key", &tag));
    /// assert!(!HMAC::verify(b"tampered", b"key", &tag));
    /// ```
    #[inline]
    pub fn verify<T: AsRef<[u8]>, U: AsRef<[u8]>>(input: T, k: U, expected: &[u8; 48]) -> bool {
        let mac = Self::mac(input, k);
        verify(&mac, expected)
    }
}

/// HKDF‑SHA‑384 key derivation (RFC 5869).
///
/// This is a stateless implementation of the extract‑then‑expand paradigm
/// for SHA‑384.  The output sizes are 48 bytes for the PRK and arbitrary
/// (up to 255×48 bytes) for the OKM.
///
/// # Example
///
/// ```rust
/// use libvctrl_sha512::sha384::HKDF;
///
/// let prk = HKDF::extract(b"salt", b"ikm");
/// assert_eq!(prk.len(), 48);
///
/// let mut key = [0u8; 32];
/// HKDF::expand(&mut key, prk, b"info");
/// ```
pub struct HKDF;

impl HKDF {
    /// Perform the HKDF‑Extract step (HMAC‑SHA‑384 with the salt as key).
    ///
    /// Returns a 48‑byte pseudorandom key (PRK).
    ///
    /// # Example
    ///
    /// ```rust
    /// use libvctrl_sha512::sha384::HKDF;
    ///
    /// let prk = HKDF::extract(b"random-salt", b"input-key-material");
    /// assert_eq!(prk.len(), 48);
    /// ```
    #[inline]
    pub fn extract(salt: impl AsRef<[u8]>, ikm: impl AsRef<[u8]>) -> [u8; 48] {
        HMAC::mac(ikm, salt)
    }

    /// Perform the HKDF‑Expand step.
    ///
    /// Expands the PRK into `out.len()` bytes of output keying material
    /// using the given `info` string.
    ///
    /// # Panics
    ///
    /// - Panics if `prk.len()` is not exactly 48.
    /// - Panics if `out.len()` exceeds `255 * 48` (12 240 bytes), the
    ///   RFC 5869 limit.
    ///
    /// # Example
    ///
    /// ```rust
    /// use libvctrl_sha512::sha384::HKDF;
    ///
    /// let prk = HKDF::extract(b"salt", b"ikm");
    /// let mut okm = [0u8; 42];
    /// HKDF::expand(&mut okm, prk, b"context");
    /// ```
    #[inline]
    pub fn expand(out: &mut [u8], prk: impl AsRef<[u8]>, info: impl AsRef<[u8]>) {
        assert_eq!(prk.as_ref().len(), 48, "HKDF-SHA384 expects a 48‑byte PRK");
        let info = info.as_ref();
        let mut counter: u8 = 1;
        assert!(
            out.len() < 0xff * 48,
            "Requested output exceeds RFC 5869 limit"
        );
        let mut i = 0;
        while i < out.len() {
            let mut hmac = HMAC::new(&prk);
            if i != 0 {
                hmac.update(&out[i - 48..][..48]);
            }
            hmac.update(info);
            hmac.update([counter]);
            let left = core::cmp::min(48, out.len() - i);
            out[i..][..left].copy_from_slice(&hmac.finalize()[..left]);
            counter += 1;
            i += 48;
        }
    }
}
