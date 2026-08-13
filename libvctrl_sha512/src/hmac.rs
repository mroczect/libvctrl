//! HMAC-SHA-512 (Hash-based Message Authentication Code with SHA-512).
//!
//! # Purpose
//!
//! This module instantiates the crate-level [`impl_hmac!`] macro to produce a
//! fully functional HMAC implementation using SHA-512 as the underlying hash
//! function. The macro generates a public [`HMAC`] struct and its associated
//! methods, following the HMAC construction defined in [RFC 2104].
//!
//! # What is HMAC?
//!
//! HMAC is a mechanism for message authentication using cryptographic hash
//! functions. It combines a secret key with the message data to produce a
//! fixed-size authentication tag. HMAC provides two properties:
//!
//! 1. **Integrity**: Any modification to the message changes the tag.
//! 2. **Authenticity**: Without the secret key, an attacker cannot forge a
//!    valid tag.
//!
//! HMAC is widely used in APIs, file integrity checks, and key derivation.
//!
//! # Generated API
//!
//! - [`HMAC::new`] - Creates a new HMAC context from a secret key.
//! - [`HMAC::update`] - Feeds input data into the HMAC.
//! - [`HMAC::finalize`] - Produces the 64-byte authentication tag.
//! - [`HMAC::mac`] - One-shot HMAC computation.
//! - [`HMAC::verify`] / [`HMAC::finalize_verify`] - Verification in
//!   constant-ish time (see [`crate::utils::verify`]).
//!
//! # Key Handling
//!
//! Keys longer than the SHA-512 block size (128 bytes) are first hashed;
//! shorter keys are zero-padded. This matches the RFC specification.
//!
//! If the key is longer than the block size, it is hashed once and the digest
//! is used as the effective key. This ensures that the key length is always
//! exactly one block, which is a requirement of the HMAC algorithm.
//!
//! # Design Rationale
//!
//! This module does not implement HMAC manually. Instead, it invokes the
//! [`impl_hmac!`] macro with:
//!
//! - [`crate::sha512::Hash`] as the underlying hash function.
//! - `64` as the hash output length.
//! - `128` as the SHA-512 block size.
//!
//! The macro-based design allows the same HMAC logic to be reused for
//! SHA-384 when the `sha384` feature is enabled, with output length 48 and
//! block size 128. This avoids code duplication and guarantees a consistent
//! implementation across the crate.
//!
//! # Internal Mechanism
//!
//! The HMAC construction works as follows:
//!
//! 1. The key is normalized to the block size as described above.
//! 2. Two padded keys are produced:
//!    - Inner pad: `key XOR 0x36`
//!    - Outer pad: `key XOR 0x5c`
//! 3. The inner hash is computed over `inner_pad || message`.
//! 4. The final HMAC tag is computed over
//!    `outer_pad || inner_hash`.
//!
//! Formally:
//!
//! `HMAC(K, m) = H((K' XOR opad) || H((K' XOR ipad) || m))`
//!
//! where `K'` is the normalized key, `H` is SHA-512, `ipad = 0x36`, and
//! `opad = 0x5c`.
//!
//! # Security Considerations
//!
//! - **Side-channel resistance**: The verification functions use a
//!   non-short-circuiting comparison to reduce timing side-channel leakage.
//!   However, this is not a full constant-time implementation on all targets.
//!   For high-security contexts, consider a dedicated constant-time library.
//! - **Key secrecy**: The key must remain secret. The HMAC context stores the
//!   padded key internally; the `Drop` implementation zeroizes the inner hash
//!   state and the padded key buffer to reduce the lifetime of sensitive data.
//! - **No unsafe code**: The HMAC implementation in this crate uses only safe
//!   Rust. The only `unsafe` in the crate is in [`crate::utils::verify`] for
//!   `read_volatile`, which is isolated and reviewed.
//!
//! # Performance
//!
//! The HMAC construction adds only a small overhead over a direct SHA-512
//! hash: it hashes two additional blocks (inner and outer padding). The
//! one-shot [`HMAC::mac`] is optimized for common cases and does not allocate
//! heap memory.
//!
//! # Examples
//!
//! Computing an HMAC-SHA-512 tag in one shot:
//!
//! ```
//! # use libvctrl_sha512::hmac::HMAC;
//! let key = b"super secret key";
//! let message = b"important message";
//! let tag = HMAC::mac(message, key);
//! assert_eq!(tag.len(), 64);
//! ```
//!
//! Incremental usage with verification:
//!
//! ```
//! # use libvctrl_sha512::hmac::HMAC;
//! let key = b"another key";
//! let message = b"data to authenticate";
//! let expected = HMAC::mac(message, key);
//!
//! let mut hmac = HMAC::new(key);
//! hmac.update(&message[..4]);
//! hmac.update(&message[4..]);
//! assert!(hmac.finalize_verify(&expected));
//! ```
//!
//! Verifying a tag without re-computing it separately:
//!
//! ```
//! # use libvctrl_sha512::hmac::HMAC;
//! let key = b"password";
//! let message = b"data";
//! let tag = HMAC::mac(message, key);
//! assert!(HMAC::verify(message, key, &tag));
//! ```
//!
//! [RFC 2104]: https://datatracker.ietf.org/doc/html/rfc2104
use crate::sha512::Hash;

impl_hmac!(Hash, 64, 128);
