//! Utility functions and constants used by the SHA-512, HMAC, and HKDF
//! implementations.
//!
//! # Why this module exists
//!
//! This module centralizes low-level helpers that are shared across multiple
//! hash and MAC constructs:
//!
//! - Byte-order conversion between big-endian and native representation.
//! - Constant-time comparison of byte slices, mitigating timing side-channel
//!   attacks during MAC verification.
//! - Common constants such as the SHA-512 block size and output size.
//!
//! By keeping these utilities in one place, the rest of the crate remains
//! focused on algorithm-specific logic without duplicating foundational code.
//!
//! # How it works
//!
//! The [`load_be`] and [`store_be`] functions convert between byte arrays and
//! 64-bit integers using big-endian order, as required by FIPS 180-4.
//! [`verify`] compares two byte slices of equal length using an XOR
//! accumulation loop and `core::hint::black_box` to prevent the compiler from
//! short-circuiting or optimizing away the comparison. This ensures that
//! verification time does not leak information about the compared values.

/// The SHA-512 block size in bytes.
///
/// Each compression round processes exactly 128 bytes (1024 bits). This
/// constant is used for padding, buffering, and HMAC key preparation.
///
/// # Examples
///
/// ```
/// use libvctrl_sha512::utils::BLOCKBYTES;
/// assert_eq!(BLOCKBYTES, 128);
/// ```
pub const BLOCKBYTES: usize = 128;

/// The SHA-512 output size in bytes.
///
/// A SHA-512 digest is always 64 bytes (512 bits). This constant is used by
/// HMAC and HKDF to size output arrays and PRKs.
///
/// # Examples
///
/// ```
/// use libvctrl_sha512::utils::BYTES;
/// assert_eq!(BYTES, 64);
/// ```
pub const BYTES: usize = 64;

/// Loads a 64-bit big-endian integer from the given byte slice at the
/// specified offset.
///
/// # How it works
///
/// The function reads eight bytes starting at `offset`, converts them to a
/// `u64` using `from_be_bytes`, and returns the result. It expects the slice
/// to contain at least `offset + 8` bytes; if not, it panics.
///
/// # Panics
///
/// Panics if `base.len() < offset + 8`.
///
/// # Examples
///
/// ```
/// use libvctrl_sha512::utils::load_be;
///
/// let bytes = [0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0];
/// assert_eq!(load_be(&bytes, 0), 0x123456789abcdef0);
/// ```
#[inline]
#[must_use]
pub fn load_be(base: &[u8], offset: usize) -> u64 {
    u64::from_be_bytes(base[offset..offset + 8].try_into().unwrap())
}

/// Stores a 64-bit integer into the given byte slice at the specified offset
/// in big-endian order.
///
/// # How it works
///
/// The function converts `x` to its big-endian byte representation and writes
/// it into `base` starting at `offset`. It assumes the slice is large enough
/// to hold eight bytes at that position.
///
/// # Panics
///
/// Panics if `base.len() < offset + 8`.
///
/// # Examples
///
/// ```
/// use libvctrl_sha512::utils::{load_be, store_be};
///
/// let mut buf = [0u8; 8];
/// store_be(&mut buf, 0, 0x0102030405060708);
/// assert_eq!(load_be(&buf, 0), 0x0102030405060708);
/// ```
#[inline]
pub fn store_be(base: &mut [u8], offset: usize, x: u64) {
    base[offset..offset + 8].copy_from_slice(&x.to_be_bytes());
}

/// Compares two byte slices of equal length in constant-ish time.
///
/// # Why this exists
///
/// When verifying MACs or digests, a naive `==` comparison may return early
/// on the first differing byte, leaking information about the expected value
/// through timing. This function accumulates differences across all bytes and
/// only returns a boolean at the end, making the runtime independent of the
/// number of leading matches.
///
/// # How it works
///
/// - If the lengths differ, it returns `false` immediately (length is not
///   secret).
/// - Otherwise, it XORs each corresponding byte pair and ORs the result into
///   an accumulator.
/// - On WebAssembly targets, an additional hash-based mask is applied to
///   mitigate compiler optimizations.
/// - Finally, `core::hint::black_box` is used to force the compiler to
///   materialize the accumulator before comparison, preventing it from
///   optimizing away the loop.
///
/// # Examples
///
/// ```
/// use libvctrl_sha512::utils::verify;
///
/// let a = [0u8; 64];
/// let b = [0u8; 64];
/// assert!(verify(&a, &b));
///
/// let c = [1u8; 64];
/// assert!(!verify(&a, &c));
/// ```
#[must_use]
pub fn verify(x: &[u8], y: &[u8]) -> bool {
    if x.len() != y.len() {
        return false;
    }
    let mut v: u32 = 0;

    #[cfg(any(target_arch = "wasm32", target_arch = "wasm64"))]
    {
        let (mut h1, mut h2) = (0u32, 0u32);
        for (b1, b2) in x.iter().zip(y.iter()) {
            h1 ^= (h1 << 5).wrapping_add((h1 >> 2) ^ *b1 as u32);
            h2 ^= (h2 << 5).wrapping_add((h2 >> 2) ^ *b2 as u32);
        }
        v |= h1 ^ h2;
    }

    for (a, b) in x.iter().zip(y.iter()) {
        v |= u32::from(a ^ b);
    }

    let v = core::hint::black_box(v);
    v == 0
}
