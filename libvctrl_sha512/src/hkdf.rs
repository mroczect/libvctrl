//! # HKDF-SHA512 (RFC 5869)
//!
//! HMAC-based Key Derivation Function (HKDF) using SHA‑512 as the underlying hash.
//! This is a key derivation function that transforms any initial keying material
//! (IKM) into one or more cryptographically strong secret keys.
//!
//! ## Overview
//!
//! HKDF follows the **extract‑then‑expand** paradigm:
//!
//! 1. **Extract** – takes the input keying material (IKM) and an optional salt,
//!    produces a pseudorandom key (PRK) of fixed length (64 bytes for SHA‑512).
//! 2. **Expand** – takes the PRK, optional context information (info), and
//!    desired output length, produces the output keying material (OKM).
//!
//! This separation allows:
//! - **Extract** – to "compress" potentially weak or non‑uniform IKM into a strong key.
//! - **Expand** – to generate multiple keys from the same PRK, for different contexts.
//!
//! ## Security Properties
//!
//! - **PRF‑security**: the PRK is computationally indistinguishable from random,
//!   provided the salt is random or the IKM has sufficient entropy.
//! - **Key independence**: outputs for different `info` values are independent.
//! - **Domain separation**: using distinct `info` avoids key reuse across applications.
//! - **Constant‑time**: all operations run in constant time to resist timing attacks.
//!
//! ## Usage Examples
//!
//! ### Basic key derivation
//! ```
//! use libvctrl_sha512::HKDF;
//!
//! // Input keying material (IKM) – can be a password, shared secret, etc.
//! let ikm = b"my-secret-input-material";
//!
//! // Salt (optional) – should be random but can be static; if omitted, empty salt is used.
//! let salt = b"random-salt-value";
//!
//! // Extract phase: derive a pseudorandom key (PRK)
//! let prk = HKDF::extract(salt, ikm);
//!
//! // Info (optional) – context string to separate different uses.
//! let info = b"encryption-key-v1";
//!
//! // Expand phase: generate 32 bytes of output keying material (OKM)
//! let mut okm = [0u8; 32];
//! HKDF::expand(&mut okm, prk, info);
//!
//! // Use `okm` as your AES‑256 key, etc.
//! ```
//!
//! ### Generating multiple keys from one PRK
//! ```
//! use libvctrl_sha512::HKDF;
//!
//! let ikm = b"master-key";
//! let salt = b"app-salt";
//! let prk = HKDF::extract(salt, ikm);
//!
//! let mut enc_key = [0u8; 32];
//! let mut mac_key = [0u8; 64];
//!
//! // Derive separate keys for encryption and MAC
//! HKDF::expand(&mut enc_key, prk, b"encryption");
//! HKDF::expand(&mut mac_key, prk, b"mac");
//! ```
//!
//! ### With empty salt and info
//! ```
//! use libvctrl_sha512::HKDF;
//!
//! let ikm = b"shared-secret";
//! let prk = HKDF::extract([], ikm);
//! let mut okm = [0u8; 48];
//! HKDF::expand(&mut okm, prk, []);
//! // OKM is now 48 bytes of derived key material.
//! ```
//!
//! ## Panics
//!
//! The `expand` function panics if the requested output length exceeds
//! `255 * 64 = 16320` bytes, as mandated by RFC 5869.
//!
//! ## When to use HKDF
//!
//! - Deriving keys from Diffie‑Hellman shared secrets.
//! - Deriving multiple keys from a master secret.
//! - Key stretching (though for passwords, consider Argon2 or PBKDF2).
//! - Domain‑specific key derivation (e.g., separate keys for encryption and signing).
//!
//! ## Security Notes
//!
//! - The **salt** should be random and non‑secret; using a fixed salt is allowed
//!   but reduces security when IKM is weak.
//! - The **info** string should be unique per context; it provides domain separation.
//! - Never reuse a PRK with different `info` values that could collide.
//! - HKDF does **not** perform key stretching; for low‑entropy inputs (passwords),
//!   combine with a KDF like Argon2.
//!
//! ## References
//!
//! - [RFC 5869 – HMAC‑based Extract‑and‑Expand Key Derivation Function (HKDF)](https://datatracker.ietf.org/doc/html/rfc5869)
//! - [NIST SP 800‑56C](https://csrc.nist.gov/publications/detail/sp/800-56c/rev-2/final)

use crate::hmac::HMAC;

/// HKDF‑SHA512 implementation.
///
/// This struct holds no state; all methods are stateless.
/// It follows the HKDF specification (RFC 5869) using SHA‑512 as the hash function.
pub struct HKDF;

impl HKDF {
    /// The HKDF‑Extract function.
    ///
    /// This step takes the input keying material (IKM) and an optional salt,
    /// and produces a pseudorandom key (PRK) of 64 bytes.
    ///
    /// # Arguments
    ///
    /// * `salt` – An optional salt value. This should be a non‑secret random string,
    ///   but can be empty or static. It is used to "strengthen" the IKM.
    /// * `ikm`  – The input keying material. This is the secret value to be derived.
    ///   It can be any byte sequence (e.g., DH shared secret, master key).
    ///
    /// # Returns
    ///
    /// A 64‑byte pseudorandom key (PRK).
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_sha512::HKDF;
    ///
    /// let ikm = b"my-shared-secret";
    /// let salt = b"random-salt";
    /// let prk = HKDF::extract(salt, ikm);
    /// assert_eq!(prk.len(), 64);
    /// ```
    ///
    /// # Security
    ///
    /// - Use a random salt when possible to achieve maximum security.
    /// - If the IKM is already uniformly random, the salt can be omitted.
    /// - The salt is not secret and can be stored alongside the ciphertext.
    #[inline]
    pub fn extract(salt: impl AsRef<[u8]>, ikm: impl AsRef<[u8]>) -> [u8; 64] {
        HMAC::mac(ikm, salt)
    }

    /// The HKDF‑Expand function.
    ///
    /// This step takes the pseudorandom key (PRK) from the extract step and
    /// produces an arbitrary‑length output keying material (OKM) using optional
    /// context information (`info`).
    ///
    /// # Arguments
    ///
    /// * `out` – A mutable slice that will be filled with the derived key material.
    ///   The length of `out` determines how many bytes are produced.
    /// * `prk` – The pseudorandom key (64 bytes) from the extract step.
    /// * `info` – Optional context and application‑specific information.
    ///   This provides domain separation. It can be empty.
    ///
    /// # Panics
    ///
    /// Panics if the requested output length (i.e., `out.len()`) exceeds
    /// `255 * 64 = 16320` bytes, as per the RFC 5869 limit.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_sha512::HKDF;
    ///
    /// let prk = HKDF::extract(b"salt", b"ikm");
    ///
    /// // Generate 32 bytes for AES‑256 key
    /// let mut key = [0u8; 32];
    /// HKDF::expand(&mut key, prk, b"aes-key");
    ///
    /// // Generate 64 bytes for HMAC‑SHA512 key
    /// let mut mac_key = [0u8; 64];
    /// HKDF::expand(&mut mac_key, prk, b"hmac-key");
    /// ```
    ///
    /// # Security
    ///
    /// - The `info` string should be unique per purpose to ensure domain separation.
    /// - For multiple keys, use distinct `info` values (never reuse the same `info`
    ///   for different purposes).
    /// - The output is deterministic given the same `prk`, `info`, and output length.
    ///
    /// # Performance
    ///
    /// Each 64‑byte block requires one HMAC‑SHA512 computation. The function is
    /// O(n) where n is `out.len()`.
    #[inline]
    pub fn expand(out: &mut [u8], prk: impl AsRef<[u8]>, info: impl AsRef<[u8]>) {
        let info = info.as_ref();
        let mut counter: u8 = 1;
        // RFC 5869: L <= 255 * HashLen
        assert!(
            out.len() < 0xff * 64,
            "Requested output length exceeds RFC 5869 limit (16320 bytes)"
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
