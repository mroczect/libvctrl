//! # SHA-384, HMAC-SHA384, and HKDF-SHA384
//!
//! This module provides cryptographic primitives based on SHA-384, which is a
//! variant of SHA-512 with different initial values and a truncated output (48 bytes).
//! SHA-384 is specified in FIPS 180-4 and offers stronger resistance against
//! length-extension attacks than SHA-256, while being more efficient than SHA-512
//! on 64‑bit platforms (though it uses the same compression function).
//!
//! ## Overview
//!
//! This module includes:
//! - **SHA‑384 hash** (`Hash`) – computes a 48‑byte digest.
//! - **HMAC‑SHA384** (`HMAC`) – keyed message authentication code using SHA‑384.
//! - **HKDF‑SHA384** (`HKDF`) – key derivation function (RFC 5869) using SHA‑384.
//!
//! All types mirror the SHA‑512 versions but with a smaller output size (48 bytes
//! instead of 64). The internal block size remains 128 bytes.
//!
//! ## When to use SHA‑384
//!
//! - Use SHA‑384 when you need a strong hash with a 384‑bit digest.
//! - It is commonly used in TLS/SSL, digital signatures, and certificate
//!   verification.
//! - For new applications that require a 384‑bit hash, SHA‑384 is a solid
//!   choice.
//!
//! ## Features
//!
//! This module is only available when the `sha384` feature is enabled (it is
//! enabled by default). To disable it, add `default-features = false` to your
//! `Cargo.toml` dependency.
//!
//! ## Examples
//!
//! ### SHA‑384 hashing
//! ```
//! use libvctrl_sha512::sha384::Hash;
//!
//! let digest = Hash::hash(b"hello world");
//! assert_eq!(digest.len(), 48);
//!
//! // Streaming
//! let mut hasher = Hash::new();
//! hasher.update(b"hello ");
//! hasher.update(b"world");
//! let digest2 = hasher.finalize();
//! assert_eq!(digest, digest2);
//! ```
//!
//! ### HMAC‑SHA384
//! ```
//! use libvctrl_sha512::sha384::HMAC;
//!
//! let key = b"my-secret-key";
//! let msg = b"important data";
//! let mac = HMAC::mac(msg, key);
//! assert_eq!(mac.len(), 48);
//!
//! // Verify
//! let expected = HMAC::mac(msg, key);
//! assert!(HMAC::verify(msg, key, &expected));
//! ```
//!
//! ### HKDF‑SHA384
//! ```
//! use libvctrl_sha512::sha384::HKDF;
//!
//! let ikm = b"shared-secret";
//! let salt = b"random-salt";
//! let info = b"session-encryption";
//! let prk = HKDF::extract(salt, ikm);
//!
//! let mut okm = [0u8; 32];
//! HKDF::expand(&mut okm, prk, info);
//! // `okm` is a 32‑byte derived key.
//! ```
//!
//! ## Security Notes
//!
//! - SHA‑384 is considered secure and is not known to be broken.
//! - The HMAC implementation uses constant‑time verification to resist timing
//!   attacks.
//! - HKDF provides domain separation via the `info` parameter; always use distinct
//!   `info` strings for different purposes.
//!
//! ## References
//!
//! - [FIPS 180-4: Secure Hash Standard (SHS)](https://csrc.nist.gov/publications/detail/fips/180/4/final)
//! - [RFC 2104 – HMAC](https://datatracker.ietf.org/doc/html/rfc2104)
//! - [RFC 5869 – HKDF](https://datatracker.ietf.org/doc/html/rfc5869)

use crate::sha512::{Hash as Sha512Hash, State};
use crate::utils::load_be;
use crate::utils::verify;

/// Initial state (IV) for SHA‑384, as defined in FIPS 180‑4.
#[inline]
fn new_state() -> State {
    const IV: [u8; 64] = [
        0xcb, 0xbb, 0x9d, 0x5d, 0xc1, 0x05, 0x9e, 0xd8,
        0x62, 0x9a, 0x29, 0x2a, 0x36, 0x7c, 0xd5, 0x07,
        0x91, 0x59, 0x01, 0x5a, 0x30, 0x70, 0xdd, 0x17,
        0x15, 0x2f, 0xec, 0xd8, 0xf7, 0x0e, 0x59, 0x39,
        0x67, 0x33, 0x26, 0x67, 0xff, 0xc0, 0x0b, 0x31,
        0x8e, 0xb4, 0x4a, 0x87, 0x68, 0x58, 0x15, 0x11,
        0xdb, 0x0c, 0x2e, 0x0d, 0x64, 0xf9, 0x8f, 0xa7,
        0x47, 0xb5, 0x48, 0x1d, 0xbe, 0xfa, 0x4f, 0xa4,
    ];
    let mut t = [0u64; 8];
    for (i, e) in t.iter_mut().enumerate() {
        *e = load_be(&IV, i * 8);
    }
    State(t)
}

/// SHA‑384 hasher with streaming support.
///
/// This struct wraps the SHA‑512 hasher but uses the SHA‑384 initial vector and
/// truncates the output to 48 bytes. It implements the same API as `Sha512Hash`.
///
/// # Example
/// ```
/// use libvctrl_sha512::sha384::Hash;
///
/// let mut hasher = Hash::new();
/// hasher.update(b"Hello, ");
/// hasher.update(b"world!");
/// let digest = hasher.finalize();
/// assert_eq!(digest.len(), 48);
/// ```
#[derive(Copy, Clone)]
pub struct Hash(Sha512Hash);

impl Hash {
    /// Creates a new SHA‑384 hasher with the default initial state.
    ///
    /// # Example
    /// ```
    /// use libvctrl_sha512::sha384::Hash;
    /// let hasher = Hash::new();
    /// // Hasher is ready to absorb data.
    /// ```
    #[inline]
    pub fn new() -> Self {
        Hash(Sha512Hash {
            state: new_state(),
            r: 0,
            w: [0u8; 128],
            len: 0,
        })
    }

    /// Internal update function (used by the digest trait implementations).
    ///
    /// This is not intended for direct public use; use `update` instead.
    #[doc(hidden)]
    pub(crate) fn _update<T: AsRef<[u8]>>(&mut self, input: T) {
        self.0.update(input)
    }

    /// Absorbs more data into the hash state.
    ///
    /// This method can be called multiple times to process a message in chunks.
    ///
    /// # Arguments
    /// * `input` – The chunk of data to hash.
    ///
    /// # Example
    /// ```
    /// use libvctrl_sha512::sha384::Hash;
    ///
    /// let mut hasher = Hash::new();
    /// hasher.update(b"first part ");
    /// hasher.update(b"second part");
    /// let digest = hasher.finalize();
    /// ```
    #[inline]
    pub fn update<T: AsRef<[u8]>>(&mut self, input: T) {
        self._update(input)
    }

    /// Finalizes the hash computation and returns the 48‑byte digest.
    ///
    /// Consumes the hasher and returns the SHA‑384 hash of all absorbed data.
    ///
    /// # Returns
    /// A `[u8; 48]` array containing the digest.
    ///
    /// # Example
    /// ```
    /// use libvctrl_sha512::sha384::Hash;
    /// let digest = Hash::new().finalize(); // hash of empty input
    /// assert_eq!(digest.len(), 48);
    /// ```
    #[inline]
    pub fn finalize(self) -> [u8; 48] {
        let mut out = [0u8; 48];
        out.copy_from_slice(&self.0.finalize()[..48]);
        out
    }

    /// One‑shot SHA‑384 hash of the given input.
    ///
    /// This is a convenience function that creates a new hasher, updates it with
    /// the input, and finalizes it in one step.
    ///
    /// # Arguments
    /// * `input` – The data to hash (any `AsRef<[u8]>`).
    ///
    /// # Returns
    /// The 48‑byte hash digest.
    ///
    /// # Example
    /// ```
    /// use libvctrl_sha512::sha384::Hash;
    /// let digest = Hash::hash(b"hello");
    /// assert_eq!(digest.len(), 48);
    /// ```
    #[inline]
    pub fn hash<T: AsRef<[u8]>>(input: T) -> [u8; 48] {
        let mut h = Self::new();
        h.update(input);
        h.finalize()
    }
}

impl Default for Hash {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// HMAC‑SHA384 instance for incremental message authentication.
///
/// This is analogous to the HMAC‑SHA512 struct but produces 48‑byte outputs.
/// It can be used to authenticate messages of arbitrary length in chunks.
///
/// # Example
/// ```
/// use libvctrl_sha512::sha384::HMAC;
///
/// let key = b"my-key";
/// let mut hmac = HMAC::new(key);
/// hmac.update(b"first part ");
/// hmac.update(b"second part");
/// let mac = hmac.finalize();
/// assert_eq!(mac.len(), 48);
/// ```
pub struct HMAC {
    ih: Hash,
    padded: [u8; 128],
}

impl HMAC {
    /// Computes the HMAC‑SHA384 of a message in one shot.
    ///
    /// # Arguments
    /// * `input` – The message to authenticate.
    /// * `k`     – The secret key.
    ///
    /// # Returns
    /// A 48‑byte MAC.
    ///
    /// # Example
    /// ```
    /// use libvctrl_sha512::sha384::HMAC;
    /// let mac = HMAC::mac(b"message", b"key");
    /// assert_eq!(mac.len(), 48);
    /// ```
    #[inline]
    pub fn mac<T: AsRef<[u8]>, U: AsRef<[u8]>>(input: T, k: U) -> [u8; 48] {
        let mut hmac = Self::new(k);
        hmac.update(input);
        hmac.finalize()
    }

    /// Creates a new streaming HMAC‑SHA384 instance with the given key.
    ///
    /// Keys longer than the block size (128 bytes) are hashed first.
    /// Keys shorter are zero‑padded.
    ///
    /// # Arguments
    /// * `k` – The secret key.
    #[inline]
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

    /// Absorbs more data into the HMAC state.
    ///
    /// # Arguments
    /// * `input` – The chunk of data to authenticate.
    #[inline]
    pub fn update(&mut self, input: impl AsRef<[u8]>) {
        self.ih.update(input);
    }

    /// Finalizes the HMAC computation and returns the 48‑byte MAC.
    ///
    /// This consumes the instance.
    #[inline]
    pub fn finalize(mut self) -> [u8; 48] {
        for p in self.padded.iter_mut() {
            *p ^= 0x6a;
        }
        let mut oh = Hash::new();
        oh.update(&self.padded[..]);
        oh.update(&self.ih.finalize()[..]);
        oh.finalize()
    }

    /// Finalizes and verifies the computed MAC against an expected value.
    ///
    /// Comparison is constant‑time.
    ///
    /// # Arguments
    /// * `expected` – The expected 48‑byte MAC.
    ///
    /// # Returns
    /// `true` if the MAC matches, `false` otherwise.
    #[inline]
    pub fn finalize_verify(self, expected: &[u8; 48]) -> bool {
        let out = self.finalize();
        verify(&out, expected)
    }

    /// One‑shot verification of a message's HMAC.
    ///
    /// # Arguments
    /// * `input`    – The message.
    /// * `k`        – The key.
    /// * `expected` – The expected MAC.
    ///
    /// # Returns
    /// `true` if the MAC matches, `false` otherwise.
    #[inline]
    pub fn verify<T: AsRef<[u8]>, U: AsRef<[u8]>>(input: T, k: U, expected: &[u8; 48]) -> bool {
        let mac = Self::mac(input, k);
        verify(&mac, expected)
    }
}

/// HKDF‑SHA384 implementation.
///
/// This provides the Extract‑and‑Expand functionality (RFC 5869) using SHA‑384.
/// It produces a 48‑byte pseudorandom key (PRK) from the extract step and can
/// generate arbitrary‑length output keying material (OKM) from the expand step.
///
/// # Example
/// ```
/// use libvctrl_sha512::sha384::HKDF;
///
/// let ikm = b"shared-secret";
/// let salt = b"random-salt";
/// let prk = HKDF::extract(salt, ikm);
///
/// let mut okm = [0u8; 32];
/// HKDF::expand(&mut okm, prk, b"encryption");
/// ```
pub struct HKDF;

impl HKDF {
    /// HKDF‑Extract: produces a 48‑byte pseudorandom key from the input keying material.
    ///
    /// # Arguments
    /// * `salt` – Optional salt value (can be empty).
    /// * `ikm`  – Input keying material.
    ///
    /// # Returns
    /// A 48‑byte PRK.
    ///
    /// # Security
    /// Use a random salt when possible for maximum security.
    #[inline]
    pub fn extract(salt: impl AsRef<[u8]>, ikm: impl AsRef<[u8]>) -> [u8; 48] {
        HMAC::mac(ikm, salt)
    }

    /// HKDF‑Expand: derives output keying material of the requested length.
    ///
    /// # Arguments
    /// * `out` – Mutable slice to fill with derived key material.
    /// * `prk` – Pseudorandom key (from the extract step).
    /// * `info` – Optional context information (can be empty).
    ///
    /// # Panics
    /// Panics if the requested output length exceeds `255 * 48 = 12240` bytes.
    ///
    /// # Security
    /// Use distinct `info` strings for different contexts to ensure domain separation.
    #[inline]
    pub fn expand(out: &mut [u8], prk: impl AsRef<[u8]>, info: impl AsRef<[u8]>) {
        let info = info.as_ref();
        let mut counter: u8 = 1;
        assert!(out.len() < 0xff * 48, "Requested output length exceeds RFC 5869 limit (12240 bytes)");
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
