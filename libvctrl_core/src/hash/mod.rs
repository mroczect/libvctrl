//! SHA-512 hasher implementation for content addressing.
//!
//! # Why this module exists
//!
//! The [`libvctrl_handler`] crate defines the [`Hasher`](libvctrl_handler::Hasher)
//! trait as the abstraction for content-addressable object hashing. This module
//! provides a concrete implementation using the SHA-512 algorithm from the
//! [`libvctrl_sha512`] crate. It bridges the raw SHA-512 digest computation to
//! the handler's [`Hash`] type, ensuring that all hashes produced by this
//! crate are compatible with the rest of the VCS ecosystem.
//!
//! # How it works
//!
//! The [`Sha512Hasher`] is a zero-sized struct. It holds no state because
//! hashing is stateless across invocations. The [`hash`](Sha512Hasher::hash)
//! method reads from a generic [`Read`](std::io::Read) stream in fixed-size
//! chunks, feeds each chunk into the underlying [`Sha512Hash`] engine, and
//! finalizes the digest into a 64-byte [`Hash`]. The result length always
//! matches [`HASH_LENGTH`](libvctrl_handler::HASH_LENGTH), so conversion
//! cannot fail.
//!
//! # Examples
//!
//! Hash a byte slice:
//!
//! ```
//! use libvctrl_core::hash::Sha512Hasher;
//! use libvctrl_handler::Hasher;
//!
//! let hasher = Sha512Hasher;
//! let hash = hasher.hash(b"hello world".as_ref()).unwrap();
//! assert_eq!(hash.as_bytes().len(), 64);
//! ```

/// SHA-512 hasher implementation.
///
/// This submodule contains the [`Sha512Hasher`] type, which implements the
/// [`Hasher`](libvctrl_handler::Hasher) trait using the SHA-512 algorithm.
/// The implementation is stateless, thread-safe, and suitable for both small
/// byte slices and large streaming inputs.
pub mod sha512;

/// Re-export of [`Sha512Hasher`] for convenient access at the module root.
///
/// By re-exporting, users can refer to `libvctrl_core::hash::Sha512Hasher`
/// instead of the longer `libvctrl_core::hash::sha512::Sha512Hasher`. This
/// aligns with the crate's goal of providing ergonomic, discoverable APIs.
pub use sha512::Sha512Hasher;
