//! # HMAC-SHA512
//!
//! This module provides an implementation of the Hash-based Message
//! Authentication Code (HMAC) as specified in RFC 2104, instantiated with
//! SHA-512 as the underlying hash function.
//!
//! ## What is HMAC?
//!
//! HMAC is a keyed hash function used for message authentication. It combines
//! a secret key with a message to produce a fixed-size authentication tag.
//! The construction is:
//!
//! ```text
//! HMAC(K, m) = H((K' XOR opad) || H((K' XOR ipad) || m))
//! ```
//!
//! Where:
//!
//! - `H` is the underlying hash function (SHA-512 here).
//! - `K'` is the key padded or hashed to the block size.
//! - `opad` is `0x5c` repeated 128 times.
//! - `ipad` is `0x36` repeated 128 times.
//!
//! ## Parameters
//!
//! This HMAC instance uses:
//!
//! - Output size: **64 bytes**
//! - Block size: **128 bytes**
//!
//! These parameters are fed into the [`impl_hmac!`] macro, which generates the
//! [`HMAC`] struct and its associated methods.
//!
//! ## Security considerations
//!
//! HMAC security depends on the secrecy and entropy of the key. A key length
//! of at least 64 bytes is recommended for 256-bit security. The
//! implementation zeroizes internal state on drop.
//!
//! # Examples
//!
//! Compute an authentication tag:
//!
//! ```
//! # use libvctrl_sha512::hmac::HMAC;
//! let tag = HMAC::mac(b"message", b"secret key");
//! assert_eq!(tag.len(), 64);
//! ```
//!
//! Verify a tag:
//!
//! ```
//! # use libvctrl_sha512::hmac::HMAC;
//! let key = b"secret key";
//! let tag = HMAC::mac(b"message", key);
//! assert!(HMAC::verify(b"message", key, &tag));
//! ```

use crate::sha512::Hash;

impl_hmac!(Hash, 64, 128);
