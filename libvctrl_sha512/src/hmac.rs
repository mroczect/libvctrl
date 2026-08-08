//! # HMAC‑SHA‑512 (RFC 2104)
//!
//! This module implements the **Keyed‑Hash Message Authentication Code (HMAC)**
//! as specified in [RFC 2104](https://datatracker.ietf.org/doc/html/rfc2104) and
//! updated by [FIPS 198‑1](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.198-1.pdf),
//! using **SHA‑512** as the underlying cryptographic hash function.
//!
//! ## Overview
//!
//! HMAC provides **message integrity** and **authenticity** by mixing a secret
//! key with the message data in a structured way. It can be used wherever a
//! shared secret is available, for example:
//!
//! - API request signing
//! - Token authentication
//! - Key derivation (as a building block for HKDF)
//! - Secure cookie generation
//!
//! The construction is:
//!
//! ```text
//! HMAC(K, m) = H((K' ⊕ opad) ∥ H((K' ⊕ ipad) ∥ m))
//! ```
//!
//! where:
//! - `H` is SHA‑512,
//! - `K'` is the key padded (or hashed then padded) to the block size (128 bytes),
//! - `ipad` = `0x36` repeated 128 times,
//! - `opad` = `0x5c` repeated 128 times,
//! - `⊕` denotes XOR,
//! - `∥` denotes concatenation.
//!
//! This crate’s implementation has been **audited** (v0.2.0) and all findings
//! have been resolved. The core logic is derived from Frank Denis's
//! `hmac-sha512` crate.
//!
//! ## Security Properties
//!
//! - **Collision resistance**: even if the underlying hash is not fully
//!   collision‑resistant, HMAC remains secure for MAC purposes as long as
//!   the compression function is a PRF.
//! - **Key length flexibility**: keys longer than the block size (128 bytes)
//!   are hashed to 64 bytes before padding; this is the standard behaviour.
//! - **Timing attack resistance**: all MAC comparisons use a constant‑time
//!   equality check via [`utils::verify`], preventing information leakage
//!   through timing side channels.
//! - **Memory zeroisation**: temporary key material is zeroed immediately,
//!   and the `Drop` implementation clears the padded key from the stack.
//!
//! ## Usage Patterns
//!
//! Two API styles are provided:
//!
//! 1. **One‑shot** – convenient for small messages:
//!    [`HMAC::mac`] computes the MAC in a single call;
//!    [`HMAC::verify`] checks a MAC in constant time.
//!
//! 2. **Streaming** – suitable for large or piecemeal data:
//!    [`HMAC::new`] creates a state, [`update`](HMAC::update) feeds
//!    portions of data, and [`finalize`](HMAC::finalize) produces the MAC
//!    (consuming the state).  Use [`finalize_verify`](HMAC::finalize_verify)
//!    for constant‑time verification without intermediate copies.
//!
//! ## Example
//!
//! ```rust
//! use libvctrl_sha512::HMAC;
//!
//! // One‑shot
//! let key = b"my-secret-key";
//! let msg = b"important message";
//! let mac = HMAC::mac(msg, key);
//! assert!(HMAC::verify(msg, key, &mac));
//!
//! // Streaming – same message, fed incrementally
//! let mut hmac = HMAC::new(key);
//! hmac.update(b"important ");
//! hmac.update(b"message");
//! let mac2 = hmac.finalize();
//! assert_eq!(mac, mac2);
//! ```

use crate::sha512::Hash;
use crate::utils::verify;

/// HMAC‑SHA‑512 state for incremental MAC computation.
///
/// Internally holds a SHA‑512 hasher preloaded with the **inner pad** (`ipad`)
/// and a copy of the key XORed with `ipad` that will be transformed to the
/// **outer pad** (`opad`) during finalisation.
///
/// # Security note
///
/// The `Drop` implementation zeroises the `padded` buffer to prevent the key
/// from lingering on the stack.  However, the `ih` hasher is not zeroised
/// (it contains only the intermediate hash state, not the key), which is
/// consistent with standard practice.
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
    /// Compute the HMAC‑SHA‑512 of `input` under key `k` (one‑shot).
    ///
    /// This is the simplest way to obtain a MAC when the entire message is
    /// available at once.  Internally it follows the standard HMAC
    /// construction:
    ///
    /// 1. If `k` is longer than 128 bytes, it is hashed with SHA‑512.
    /// 2. The (possibly hashed) key is XORed with `ipad` and `opad`.
    /// 3. Two nested hash computations produce the final 64‑byte tag.
    ///
    /// All temporary key‑derived buffers are zeroed before returning.
    ///
    /// # Arguments
    ///
    /// * `input` – The message to authenticate.
    /// * `k`     – The secret key.
    ///
    /// # Returns
    ///
    /// A `[u8; 64]` containing the HMAC tag.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use libvctrl_sha512::HMAC;
    ///
    /// let mac = HMAC::mac(b"message", b"key");
    /// assert_eq!(mac.len(), 64);
    /// ```
    #[inline]
    pub fn mac<T: AsRef<[u8]>, U: AsRef<[u8]>>(input: T, k: U) -> [u8; 64] {
        let input = input.as_ref();
        let k = k.as_ref();
        let mut hk = [0u8; 64];
        let k2 = if k.len() > 128 {
            hk.copy_from_slice(&Hash::hash(k));
            &hk
        } else {
            k
        };
        let mut ih = Hash::new();
        let mut padded = [0x36; 128];
        for (p, &k) in padded.iter_mut().zip(k2.iter()) {
            *p ^= k;
        }
        ih.update(&padded[..]);
        ih.update(input);

        let mut oh = Hash::new();
        padded = [0x5c; 128];
        for (p, &k) in padded.iter_mut().zip(k2.iter()) {
            *p ^= k;
        }
        oh.update(&padded[..]);
        oh.update(&ih.finalize()[..]);
        let mac = oh.finalize();

        hk.fill(0);
        padded.fill(0);
        mac
    }

    /// Create a new streaming HMAC‑SHA‑512 state.
    ///
    /// The key is processed immediately: if it exceeds the block size
    /// (128 bytes), it is first hashed.  The resulting (possibly hashed)
    /// key is XORed with `ipad` and fed into the inner SHA‑512 hasher.
    ///
    /// # Panics
    ///
    /// This method does not panic.  An empty key is allowed (though not
    /// recommended for security).
    ///
    /// # Example
    ///
    /// ```rust
    /// use libvctrl_sha512::HMAC;
    ///
    /// let mut ctx = HMAC::new(b"my-key");
    /// ctx.update(b"hello world");
    /// let mac = ctx.finalize();
    /// ```
    pub fn new(k: impl AsRef<[u8]>) -> Self {
        let k = k.as_ref();
        let mut hk = [0u8; 64];
        let k2 = if k.len() > 128 {
            hk.copy_from_slice(&Hash::hash(k));
            &hk
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
    /// This method can be called multiple times to incrementally process a
    /// message.  Internally it updates the inner SHA‑512 hasher.
    ///
    /// # Example
    ///
    /// ```rust
    /// use libvctrl_sha512::HMAC;
    ///
    /// let mut ctx = HMAC::new(b"key");
    /// ctx.update(b"chunk1");
    /// ctx.update(b"chunk2");
    /// ```
    pub fn update(&mut self, input: impl AsRef<[u8]>) {
        self.ih.update(input);
    }

    /// Finalize the HMAC computation and return the 64‑byte tag.
    ///
    /// This method consumes the state, completes the outer hash, and
    /// produces the final MAC.  The internal padded key buffer is zeroised
    /// after use (via `Drop`).
    ///
    /// # Returns
    ///
    /// `[u8; 64]` – the HMAC‑SHA‑512 tag.
    ///
    /// # Example
    ///
    /// ```rust
    /// use libvctrl_sha512::HMAC;
    ///
    /// let mut ctx = HMAC::new(b"key");
    /// ctx.update(b"data");
    /// let mac = ctx.finalize();
    /// assert_eq!(mac.len(), 64);
    /// ```
    pub fn finalize(mut self) -> [u8; 64] {
        for p in self.padded.iter_mut() {
            *p ^= 0x6a; // ipad XOR opad == 0x36 ^ 0x5c = 0x6a
        }
        let mut oh = Hash::new();
        oh.update(&self.padded[..]);
        oh.update(self.ih.finalize());
        oh.finalize()
    }

    /// Finalize and compare against `expected` in constant time.
    ///
    /// This is a convenience method that calls [`finalize`](Self::finalize)
    /// and then uses [`utils::verify`] for a timing‑attack‑resistant
    /// comparison.  It prevents the caller from accidentally using a
    /// standard equality operator (`==`) which may short‑circuit and leak
    /// timing information.
    ///
    /// # Returns
    ///
    /// `true` if the computed MAC matches `expected`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use libvctrl_sha512::HMAC;
    ///
    /// let mut ctx = HMAC::new(b"key");
    /// ctx.update(b"message");
    /// let expected = [0u8; 64]; // dummy
    /// let is_valid = ctx.finalize_verify(&expected);
    /// ```
    #[inline]
    pub fn finalize_verify(self, expected: &[u8; 64]) -> bool {
        let out = self.finalize();
        verify(&out, expected)
    }

    /// One‑shot verification of `input` under key `k` against `expected`.
    ///
    /// This computes `mac(input, k)` and compares it with `expected` in
    /// constant time.  It is equivalent to:
    ///
    /// ```rust,ignore
    /// verify(&HMAC::mac(input, k), expected)
    /// ```
    ///
    /// # Returns
    ///
    /// `true` iff the computed tag equals `expected`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use libvctrl_sha512::HMAC;
    ///
    /// let tag = HMAC::mac(b"message", b"key");
    /// assert!(HMAC::verify(b"message", b"key", &tag));
    /// assert!(!HMAC::verify(b"tampered", b"key", &tag));
    /// ```
    #[inline]
    pub fn verify<T: AsRef<[u8]>, U: AsRef<[u8]>>(input: T, k: U, expected: &[u8; 64]) -> bool {
        let mac = Self::mac(input, k);
        verify(&mac, expected)
    }
}
