//! # HMAC-SHA512
//!
//! This module provides HMAC (Hash-based Message Authentication Code) using SHA-512
//! as the underlying hash function. HMAC is a keyed-hash message authentication
//! code that can be used to verify both the integrity and authenticity of a message.
//!
//! ## Overview
//!
//! HMAC is defined in RFC 2104 and uses a secret key together with a hash function
//! to produce a fixed-size output (MAC). This implementation uses SHA-512, which
//! produces a 64-byte MAC.
//!
//! The HMAC construction is:
//! ```text
//! HMAC(K, m) = H((K ⊕ opad) || H((K ⊕ ipad) || m))
//! ```
//! where:
//! - `K` is the secret key (padded to the block size of the hash, 128 bytes).
//! - `m` is the message.
//! - `opad` and `ipad` are fixed constants (`0x5c` and `0x36`).
//! - `||` denotes concatenation.
//!
//! ## Security Properties
//!
//! - **Resistance to forgery**: Without the key, an attacker cannot produce a valid MAC.
//! - **Collision resistance**: Inherits from SHA-512; finding collisions is infeasible.
//! - **Constant‑time verification**: The `verify` and `finalize_verify` functions
//!   compare the computed MAC with the expected value in constant time,
//!   mitigating timing side‑channel attacks.
//!
//! ## Key Handling
//!
//! - Keys longer than the block size (128 bytes) are hashed (using SHA-512) first,
//!   then used as the effective key.
//! - Keys shorter than the block size are padded with zeros to 128 bytes.
//! - Keys should be generated using a cryptographically secure random number generator.
//!
//! ## Usage Examples
//!
//! ### One-shot HMAC (simple, all-in-one)
//! ```
//! use libvctrl_sha512::HMAC;
//!
//! let key = b"my-secret-key";
//! let message = b"Hello, world!";
//! let mac = HMAC::mac(message, key);
//! // `mac` is a 64-byte array
//! ```
//!
//! ### Streaming HMAC (incremental processing)
//! ```
//! use libvctrl_sha512::HMAC;
//!
//! let key = b"my-secret-key";
//! let mut hmac = HMAC::new(key);
//! hmac.update(b"Hello, ");
//! hmac.update(b"world!");
//! let mac = hmac.finalize();
//! ```
//!
//! ### Verification (constant-time)
//! ```
//! use libvctrl_sha512::HMAC;
//!
//! let key = b"secret";
//! let message = b"important data";
//! let expected = HMAC::mac(message, key);
//!
//! // Verify in one shot
//! assert!(HMAC::verify(message, key, &expected));
//!
//! // Or stream and verify
//! let mut hmac = HMAC::new(key);
//! hmac.update(message);
//! assert!(hmac.finalize_verify(&expected));
//! ```
//!
//! ## Performance Considerations
//!
//! - The one‑shot `mac` function performs two hash computations (inner and outer),
//!   so it is efficient for short messages.
//! - The streaming API allows processing large messages without holding the entire
//!   message in memory.
//! - HMAC is generally fast; for very high throughput, consider hardware‑accelerated
//!   variants if available, but this implementation is purely software.
//!
//! ## References
//!
//! - [RFC 2104 – HMAC: Keyed-Hashing for Message Authentication](https://datatracker.ietf.org/doc/html/rfc2104)
//! - [FIPS 198-1 – The Keyed-Hash Message Authentication Code (HMAC)](https://csrc.nist.gov/publications/detail/fips/198/1/final)
//! - [NIST SP 800-107 – Recommendation for Applications Using Approved Hash Algorithms](https://csrc.nist.gov/publications/detail/sp/800-107/rev-1/final)

use crate::sha512::Hash;
use crate::utils::verify;

/// An HMAC‑SHA512 instance that can process data incrementally.
///
/// This struct holds the inner hash state (`ih`) and the padded key (`padded`).
/// After instantiation with `HMAC::new(key)`, you can call `update` multiple times
/// and finally `finalize` to obtain the MAC.
///
/// # Example
/// ```
/// use libvctrl_sha512::HMAC;
///
/// let key = b"my-key";
/// let mut hmac = HMAC::new(key);
/// hmac.update(b"first part ");
/// hmac.update(b"second part");
/// let mac = hmac.finalize();
/// ```
///
/// # Note
/// This struct consumes itself on `finalize`, so it cannot be reused. To compute
/// another MAC, create a new instance.
pub struct HMAC {
    /// Inner hash state (already processed with `ipad` key).
    ih: Hash,
    /// Padded and XORed key (used later for the outer hash).
    padded: [u8; 128],
}

impl HMAC {
    /// Computes the HMAC‑SHA512 of a message in **one shot**.
    ///
    /// This is the simplest way to compute a MAC. It takes the message and key,
    /// processes them, and returns the 64-byte MAC.
    ///
    /// # Arguments
    ///
    /// * `input` – The message to authenticate. Can be any type that implements
    ///   `AsRef<[u8]>` (e.g., `&[u8]`, `&str`, `Vec<u8>`).
    /// * `k`     – The secret key. Same flexibility as `input`.
    ///
    /// # Returns
    ///
    /// A fixed-size `[u8; 64]` array containing the HMAC-SHA512.
    ///
    /// # Example
    /// ```
    /// use libvctrl_sha512::HMAC;
    ///
    /// let mac = HMAC::mac(b"Hello, world!", b"secret");
    /// assert_eq!(mac.len(), 64);
    /// ```
    ///
    /// # Security
    ///
    /// This function does **not** verify the MAC; it only computes it. For
    /// verification, use `verify` or `finalize_verify`.
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
        oh.finalize()
    }

    /// Creates a new streaming HMAC instance with the given key.
    ///
    /// This initializes the inner hash state with the `ipad` (0x36) and stores the
    /// padded key for later use in the outer hash.
    ///
    /// # Arguments
    ///
    /// * `k` – The secret key. Any type that can be borrowed as `&[u8]`.
    ///
    /// # Returns
    ///
    /// A new `HMAC` struct ready to absorb data via `update`.
    ///
    /// # Example
    /// ```
    /// use libvctrl_sha512::HMAC;
    ///
    /// let hmac = HMAC::new(b"my-key");
    /// // ... call update and finalize
    /// ```
    ///
    /// # Key Handling
    ///
    /// If the key is longer than 128 bytes, it is first hashed using SHA-512 to
    /// reduce it to 64 bytes, then padded to 128 bytes. This follows the HMAC
    /// specification.
    #[inline]
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

    /// Absorbs more data into the HMAC state.
    ///
    /// This method can be called multiple times to process a message in chunks.
    /// The data is fed into the inner hash (which already incorporates the `ipad` key).
    ///
    /// # Arguments
    ///
    /// * `input` – The chunk of data to authenticate.
    ///
    /// # Example
    /// ```
    /// use libvctrl_sha512::HMAC;
    ///
    /// let mut hmac = HMAC::new(b"key");
    /// hmac.update(b"Hello, ");
    /// hmac.update(b"world!");
    /// // finalize later
    /// ```
    #[inline]
    pub fn update(&mut self, input: impl AsRef<[u8]>) {
        self.ih.update(input);
    }

    /// Finalizes the HMAC computation and returns the MAC.
    ///
    /// This consumes the `HMAC` instance, computes the outer hash using the
    /// stored `padded` key (with `opad` XOR), and returns the final 64-byte MAC.
    ///
    /// # Returns
    ///
    /// A `[u8; 64]` array containing the HMAC-SHA512.
    ///
    /// # Example
    /// ```
    /// use libvctrl_sha512::HMAC;
    ///
    /// let mut hmac = HMAC::new(b"key");
    /// hmac.update(b"message");
    /// let mac = hmac.finalize();
    /// ```
    ///
    /// # Note
    /// After calling `finalize`, the instance is consumed and cannot be reused.
    #[inline]
    pub fn finalize(mut self) -> [u8; 64] {
        for p in self.padded.iter_mut() {
            *p ^= 0x6a;
        }
        let mut oh = Hash::new();
        oh.update(&self.padded[..]);
        oh.update(self.ih.finalize());
        oh.finalize()
    }

    /// Finalizes the HMAC and verifies it against an expected MAC.
    ///
    /// This is a convenience method that combines `finalize` with a constant-time
    /// comparison. It consumes the `HMAC` instance and returns `true` if the
    /// computed MAC matches the expected value, `false` otherwise.
    ///
    /// # Arguments
    ///
    /// * `expected` – The expected MAC (64 bytes).
    ///
    /// # Returns
    ///
    /// `true` if the MACs match, `false` otherwise.
    ///
    /// # Security
    ///
    /// The comparison is performed in constant time, so it is safe against
    /// timing attacks.
    ///
    /// # Example
    /// ```
    /// use libvctrl_sha512::HMAC;
    ///
    /// let key = b"secret";
    /// let msg = b"hello";
    /// let expected = HMAC::mac(msg, key);
    ///
    /// let mut hmac = HMAC::new(key);
    /// hmac.update(msg);
    /// assert!(hmac.finalize_verify(&expected));
    /// ```
    #[inline]
    pub fn finalize_verify(self, expected: &[u8; 64]) -> bool {
        let out = self.finalize();
        verify(&out, expected)
    }

    /// Verifies a message's HMAC in **one shot** without streaming.
    ///
    /// This is a convenience function that computes the MAC of the given message
    /// and key, and compares it to the expected value in constant time.
    ///
    /// # Arguments
    ///
    /// * `input`    – The message to verify.
    /// * `k`        – The secret key.
    /// * `expected` – The expected MAC (64 bytes).
    ///
    /// # Returns
    ///
    /// `true` if the computed MAC equals `expected`, `false` otherwise.
    ///
    /// # Security
    ///
    /// The comparison is constant-time.
    ///
    /// # Example
    /// ```
    /// use libvctrl_sha512::HMAC;
    ///
    /// let key = b"secret";
    /// let msg = b"important data";
    /// let expected = HMAC::mac(msg, key);
    ///
    /// assert!(HMAC::verify(msg, key, &expected));
    ///
    /// // Tampering detection
    /// let wrong = [0u8; 64];
    /// assert!(!HMAC::verify(msg, key, &wrong));
    /// ```
    #[inline]
    pub fn verify<T: AsRef<[u8]>, U: AsRef<[u8]>>(input: T, k: U, expected: &[u8; 64]) -> bool {
        let mac = Self::mac(input, k);
        verify(&mac, expected)
    }
}
