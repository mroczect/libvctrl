//! # HKDF-SHA512 (RFC 5869)
//!
//! This module implements the HMAC-based Extract-and-Expand Key Derivation Function
//! (HKDF) as specified in [RFC 5869](https://datatracker.ietf.org/doc/html/rfc5869),
//! using SHA-512 as the underlying hash function.
//!
//! ## Overview
//!
//! HKDF is a simple, well-analyzed key derivation function that transforms
//! initial keying material (IKM) — which may be a non-uniformly random or
//! partially compromised secret — into one or more cryptographically strong
//! secret keys. The process consists of two stages:
//!
//! 1. **Extract**: concentrate the entropy from the IKM into a fixed-length
//!    pseudorandom key (PRK). A salt (optional but recommended) helps to
//!    randomise the extraction and can make even a weak IKM produce a strong
//!    PRK.
//!
//! 2. **Expand**: take the PRK and an optional context string (info) to
//!    produce an arbitrary amount of output keying material (OKM). The same
//!    PRK can be used with different info values to generate multiple
//!    independent keys from a single IKM.
//!
//! This separation provides both **entropy extraction** and **domain
//! separation**, two essential requirements for a robust KDF.
//!
//! ## Why SHA-512?
//!
//! SHA-512 offers a large block size (128 bytes) and a large internal state
//! (512 bits). In the context of HKDF:
//!
//! - A 64-byte hash output length allows the extract phase to accommodate
//!   high-entropy inputs (e.g., DH shared secrets) without truncation loss.
//! - The high security margin of SHA-512 (preimage resistance, collision
//!   resistance) makes the derived keys resilient even against future
//!   cryptanalytic advances.
//! - On 64-bit platforms, SHA-512 is often faster than SHA-256 because it
//!   processes twice as many bytes per round.
//!
//! ## Security Properties
//!
//! HKDF-SHA512 inherits the security properties of HMAC-SHA512:
//!
//! - **Pseudorandomness**: if the IKM has sufficient min-entropy, the PRK is
//!   computationally indistinguishable from a random string of the same
//!   length.
//! - **Independence**: different `info` strings produce independent output
//!   keys; an attacker who learns one OKM gains no information about another
//!   derived from the same PRK but a different info.
//! - **Resistance to related-key attacks**: the nested HMAC construction
//!   prevents known attacks against simple concatenation KDFs.
//!
//! ## Usage Recommendations
//!
//! - **Salt**: a random, non-secret salt should be used whenever possible.
//!   Even a salt derived from protocol constants is better than an empty salt.
//! - **Info**: always use a unique `info` string per key purpose (e.g.,
//!   `b"encryption-key"` vs `b"mac-key"`). This provides domain separation.
//! - **PRK reuse**: the PRK may be reused with many different info values to
//!   derive multiple keys without sacrificing security, **provided** the
//!   underlying IKM remains the same and the salt is fixed.
//! - **Long output**: RFC 5869 limits the total output length to
//!   `255 * HashLen` bytes (i.e., 16 320 bytes for SHA-512). This module
//!   enforces that limit with a panic.
//!
//! ## Input Validation & Panic Policy
//!
//! To prevent silent misuse, this implementation **validates** the length of
//! the PRK in `expand`:
//!
//! - For SHA-512, the PRK **must** be exactly 64 bytes. Passing a slice of
//!   any other length causes a panic with a clear error message.
//! - The maximum output length is checked against the RFC limit and also
//!   causes a panic on violation.
//!
//! While panicking on invalid input is not always idiomatic for general
//! libraries, in the context of a cryptographic library a clear panic is
//! preferable to silently producing weak or incorrect output. These checks
//! are programmer errors that should be caught during development and testing;
//! they are not expected to occur in production with correct usage.
//!
//! ## Performance
//!
//! Each call to `expand` requires one HMAC-SHA512 computation per 64-byte
//! output block. The `extract` step performs a single HMAC-SHA512 operation.
//! For typical key lengths (e.g., 32 bytes) the overhead is negligible.
//!
//! ## Design Decisions
//!
//! - **Zero-sized struct**: `HKDF` is a stateless marker struct. This avoids
//!   any allocation and makes the API clean (no need to instantiate anything).
//! - **`impl AsRef<[u8]>` parameters**: allows passing `&[u8]`, `Vec<u8>`,
//!   arrays, or string literals, making the API flexible and ergonomic.
//! - **Panics instead of `Result`**: we deliberately panic on invalid PRK
//!   length or output length because these are unrecoverable programming errors.
//!   A cryptographic library must never silently produce weak keys.
//!
//! ## Examples
//!
//! ### Basic key derivation
//! ```rust
//! use libvctrl_sha512::HKDF;
//!
//! let ikm = b"shared-secret";
//! let salt = b"random-salt";
//! let info = b"encryption-key";
//!
//! // Extract PRK
//! let prk = HKDF::extract(salt, ikm);
//! assert_eq!(prk.len(), 64);
//!
//! // Expand to a 32-byte AES key
//! let mut aes_key = [0u8; 32];
//! HKDF::expand(&mut aes_key, prk, info);
//! ```
//!
//! ### Deriving multiple keys from one IKM
//! ```rust
//! use libvctrl_sha512::HKDF;
//!
//! let ikm = b"master-secret";
//! let salt = b"protocol-v1";
//! let prk = HKDF::extract(salt, ikm);
//!
//! // Two separate keys with different contexts
//! let mut enc_key = [0u8; 32];
//! let mut mac_key = [0u8; 64];
//! HKDF::expand(&mut enc_key, prk, b"encryption");
//! HKDF::expand(&mut mac_key, prk, b"authentication");
//! ```
//!
//! ### Empty salt and info
//! Although not recommended for production, the API supports empty slices:
//! ```rust
//! use libvctrl_sha512::HKDF;
//!
//! let prk = HKDF::extract([], b"some-input");
//! let mut okm = [0u8; 16];
//! HKDF::expand(&mut okm, prk, []);
//! ```

use crate::hmac::HMAC;

/// HKDF-SHA512 implementation.
///
/// This is a zero-sized struct whose methods implement the HKDF operations.
/// Because it holds no state, all methods are stateless and can be called
/// freely. You never need to instantiate `HKDF`; just call `HKDF::extract(...)`
/// and `HKDF::expand(...)` directly.
///
/// # Example
///
/// ```rust
/// use libvctrl_sha512::HKDF;
///
/// let prk = HKDF::extract(b"salt", b"secret");
/// let mut key = [0u8; 32];
/// HKDF::expand(&mut key, prk, b"my-app-info");
/// ```
pub struct HKDF;

impl HKDF {
    /// Performs the HKDF-Extract function.
    ///
    /// Extracts a 64-byte pseudorandom key (PRK) from the given input keying
    /// material (IKM) and an optional salt.
    ///
    /// # Arguments
    ///
    /// * `salt` – An optional salt value. A non-secret, random string is
    ///   recommended for best security, but an empty slice is accepted.
    /// * `ikm`  – The input keying material. This is the secret from which
    ///   keys will be derived (e.g., a Diffie-Hellman shared secret or a
    ///   master password).
    ///
    /// # Returns
    ///
    /// A `[u8; 64]` containing the pseudorandom key (PRK).
    ///
    /// # Security
    ///
    /// The salt may be public; it helps to randomise the extraction process.
    /// When the IKM is already uniformly random, the salt can be omitted
    /// (empty slice). However, using even a fixed salt is better than none.
    ///
    /// # Example
    ///
    /// ```rust
    /// use libvctrl_sha512::HKDF;
    ///
    /// let prk = HKDF::extract(b"salt", b"my-secret");
    /// assert_eq!(prk.len(), 64);
    /// ```
    #[inline]
    pub fn extract(salt: impl AsRef<[u8]>, ikm: impl AsRef<[u8]>) -> [u8; 64] {
        HMAC::mac(ikm, salt)
    }

    /// Performs the HKDF-Expand function.
    ///
    /// Expands the given pseudorandom key (PRK) into an arbitrary-length output
    /// keying material (OKM) using an optional context/info string.
    ///
    /// # Arguments
    ///
    /// * `out` – A mutable byte slice that will be filled with the derived key
    ///   material. The length of this slice determines how many bytes are
    ///   produced.
    /// * `prk` – The pseudorandom key from the extract step. **Must be exactly
    ///   64 bytes** for SHA-512; a panic will occur otherwise.
    /// * `info` – An optional context and application-specific information
    ///   (e.g., `b"encryption-key"`). This provides domain separation. It can
    ///   be empty.
    ///
    /// # Panics
    ///
    /// - Panics if `prk.len()` is not 64.
    /// - Panics if `out.len()` exceeds `255 * 64` (16 320 bytes), the maximum
    ///   allowed by RFC 5869.
    ///
    /// # Security
    ///
    /// Always use distinct `info` values for different purposes. Never reuse
    /// the same info for two different keys unless the PRK is also different.
    /// The OKM is deterministic given the same PRK, info, and output length;
    /// the caller must ensure uniqueness where required.
    ///
    /// # Example
    ///
    /// ```rust
    /// use libvctrl_sha512::HKDF;
    ///
    /// let prk = HKDF::extract(b"salt", b"secret");
    /// let mut key = [0u8; 32];
    /// HKDF::expand(&mut key, prk, b"application-info");
    /// assert_eq!(key.len(), 32);
    /// ```
    #[inline]
    pub fn expand(out: &mut [u8], prk: impl AsRef<[u8]>, info: impl AsRef<[u8]>) {
        assert_eq!(prk.as_ref().len(), 64, "HKDF-SHA512 expects a 64‑byte PRK");
        let info = info.as_ref();
        let mut counter: u8 = 1;
        assert!(
            out.len() < 0xff * 64,
            "Requested output exceeds RFC 5869 limit"
        );
        let mut i = 0;
        while i < out.len() {
            let mut hmac = HMAC::new(&prk);
            if i != 0 {
                hmac.update(&out[i - 64..][..64]);
            }
            hmac.update(info);
            hmac.update([counter]);
            let left = core::cmp::min(64, out.len() - i);
            out[i..][..left].copy_from_slice(&hmac.finalize()[..left]);
            counter += 1;
            i += 64;
        }
    }
}
