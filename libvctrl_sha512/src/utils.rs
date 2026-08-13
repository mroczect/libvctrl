//! Low-level utility functions and constants shared across the crate.
//!
//! # Purpose
//!
//! This module provides foundational building blocks used by the
//! cryptographic implementations in this crate. It centralizes:
//!
//! - Constants [`BLOCKBYTES`] and [`BYTES`] describing SHA-512 block and
//!   output sizes.
//! - Big-endian byte-order conversion helpers [`load_be`] and [`store_be`].
//! - A non-short-circuiting comparison function [`verify`] that aims to
//!   reduce timing side-channel leakage.
//!
//! # Design Rationale
//!
//! The utilities are deliberately small and highly optimized:
//!
//! - **Zero-cost abstraction**: All functions are marked `#[inline]` or
//!   `#[inline(always)]` to ensure they are compiled directly into the
//!   callers without function call overhead. In release builds, the
//!   endianness conversions become single `mov` and `bswap` instructions on
//!   little-endian platforms.
//! - **`no_std` compatibility**: The module uses only `core` primitives
//!   (`u64::from_be_bytes`, `u64::to_be_bytes`,
//!   `core::ptr::read_volatile`), so it works in embedded and bare-metal
//!   environments.
//! - **Centralized constants**: Defining [`BLOCKBYTES`] and [`BYTES`] once
//!   avoids magic numbers scattered throughout the crate. When the
//!   underlying hash algorithm changes (e.g., future SHA-256 support), these
//!   constants can be updated in one place.
//!
//! # Security Considerations
//!
//! The [`verify`] function is designed to resist timing side-channel
//! attacks. Standard slice equality short-circuits on the first differing
//! byte, which can leak information through timing. [`verify`] instead
//! iterates over the full input and accumulates XOR differences, returning
//! `true` only if all bytes match. On WASM targets an extra hash-based
//! mixing step is added to further obscure timing patterns. However, it is
//! not a fully constant-time implementation on all architectures; for
//! high-security applications, a dedicated constant-time library is
//! recommended.
//!
//! # Internal Mechanism
//!
//! The byte-order functions use standard library methods that are
//! well-optimized. [`load_be`] reads 8 bytes and converts them to `u64`
//! big-endian. [`store_be`] performs the inverse. The [`verify`] function
//! uses a 32-bit accumulator and XORs each byte pair; an optional WASM
//! path adds two separate hash accumulators to scramble the intermediate
//! state.
//!
//! # Examples
//!
//! Basic use of the constants and conversion functions:
//!
//! ```
//! # use libvctrl_sha512::utils::{BLOCKBYTES, BYTES, load_be, store_be};
//! assert_eq!(BLOCKBYTES, 128);
//! assert_eq!(BYTES, 64);
//!
//! let mut buf = [0u8; 8];
//! store_be(&mut buf, 0, 0x0102030405060708);
//! assert_eq!(load_be(&buf, 0), 0x0102030405060708);
//! ```

/// SHA-512 block size in bytes.
///
/// The SHA-512 algorithm processes messages in 1024-bit (128-byte) blocks.
/// This constant is used throughout the crate to dimension internal buffers
/// and to verify HKDF/HMAC input constraints.
///
/// # Design Rationale
///
/// The block size is a fundamental property of SHA-512. Exposing it as a
/// constant allows other modules (HMAC, HKDF) to refer to it without
/// hardcoding magic numbers. If the algorithm is swapped in the future, this
/// constant can be updated accordingly.
///
/// # Examples
///
/// ```
/// # use libvctrl_sha512::utils::BLOCKBYTES;
/// assert_eq!(BLOCKBYTES, 128);
/// ```
pub const BLOCKBYTES: usize = 128;

/// SHA-512 output size in bytes.
///
/// A full SHA-512 digest is 512 bits = 64 bytes. HMAC-SHA-512 and HKDF-SHA-512
/// also produce 64-byte outputs. For SHA-384, the output is truncated to 48 bytes
/// (`BYTES` still represents the underlying SHA-512 length).
///
/// # Design Rationale
///
/// The output size is used for buffer sizing and to validate HMAC/HKDF
/// constraints. Keeping it as a public constant promotes clarity and
/// prevents magic numbers.
///
/// # Examples
///
/// ```
/// # use libvctrl_sha512::utils::BYTES;
/// assert_eq!(BYTES, 64);
/// ```
pub const BYTES: usize = 64;

/// Loads a 64-bit unsigned integer from a big-endian byte slice at a given offset.
///
/// This function is the fundamental building block for reading message words and
/// state values. It uses the standard library's [`u64::from_be_bytes`], which is
/// compiled to an efficient byte-swap if the target is little-endian, and a plain
/// load on big-endian platforms.
///
/// # Why this is `#[inline]`
///
/// The function is always inlined to guarantee that the bounds check on
/// `base[offset..offset + 8]` is eliminated when the caller ensures the slice is
/// at least `offset + 8` bytes long. In release builds, this becomes a single
/// `mov` + `bswap` on x86-64.
///
/// # Panics
///
/// Panics if `base` is shorter than `offset + 8` bytes.
///
/// # Examples
///
/// ```
/// # use libvctrl_sha512::utils::load_be;
/// let data: [u8; 8] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00];
/// let value = load_be(&data, 0);
/// assert_eq!(value, 256);
/// ```
///
/// Demonstrating the roundtrip with [`store_be`]:
///
/// ```
/// # use libvctrl_sha512::utils::{load_be, store_be};
/// let original: u64 = 0x0123456789abcdef;
/// let mut buf = [0u8; 8];
/// store_be(&mut buf, 0, original);
/// assert_eq!(load_be(&buf, 0), original);
/// ```
#[inline]
#[must_use]
pub fn load_be(base: &[u8], offset: usize) -> u64 {
    u64::from_be_bytes(base[offset..offset + 8].try_into().unwrap())
}

/// Stores a 64-bit unsigned integer into a byte slice at a given offset in big-endian order.
///
/// This is the inverse of [`load_be`]. It writes the 8 bytes of `x` in big-endian
/// format starting at `base[offset]`. The caller must ensure that `base` is at
/// least `offset + 8` bytes long.
///
/// # Design
///
/// The method uses [`u64::to_be_bytes`] and [`copy_from_slice`], which allows the
/// compiler to generate optimal store sequences (e.g., a `bswap` + `mov` on
/// little-endian).
///
/// # Examples
///
/// ```
/// # use libvctrl_sha512::utils::store_be;
/// let mut buf = [0u8; 8];
/// store_be(&mut buf, 0, 0xdeadbeef);
/// assert_eq!(buf, [0x00, 0x00, 0x00, 0x00, 0xde, 0xad, 0xbe, 0xef]);
/// ```
#[inline]
pub fn store_be(base: &mut [u8], offset: usize, x: u64) {
    base[offset..offset + 8].copy_from_slice(&x.to_be_bytes());
}

/// Compares two byte slices in a way that aims to resist timing side-channel attacks.
///
/// Standard `==` for slices short-circuits on the first differing byte, which
/// can leak information through timing. This function accumulates the XOR
/// differences of all bytes and returns `true` only if all bytes are equal.
/// Additionally, on WASM targets an extra hash-based accumulator is used to
/// further hinder timing analysis, because WASM linear memory does not guarantee
/// constant-time access patterns.
///
/// # How it works
///
/// 1. Length mismatch returns `false` immediately (length is not secret).
/// 2. A 32-bit accumulator `v` is initialised to 0.
/// 3. **WASM only:** two independent hashes (`h1` and `h2`) mix the bytes of
///    each slice using a simple 5-bit left-rotate + XOR scheme. The XOR of
///    `h1` and `h2` is OR-ed into `v`. This scrambles the intermediate
///    accumulator so that even small differences spread across many bits.
/// 4. For every byte pair `(a, b)`, `v |= (a ^ b)`.
/// 5. A [`core::ptr::read_volatile`] is used to read `v` at the end. This
///    prevents the compiler from optimising away the accumulator chain or
///    reducing it to a short-circuit comparison.
/// 6. Returns `true` if `v == 0`.
///
/// # Limitations
///
/// This is **not** a fully constant-time implementation on all targets. It
/// significantly raises the bar for timing attacks but does not provide the
/// guarantees of formally verified constant-time code. For high-security
/// applications, prefer a dedicated constant-time library.
///
/// # Examples
///
/// Comparing two equal slices:
///
/// ```
/// # use libvctrl_sha512::utils::verify;
/// let a = [1, 2, 3];
/// let b = [1, 2, 3];
/// assert!(verify(&a, &b));
/// ```
///
/// Different slices:
///
/// ```
/// # use libvctrl_sha512::utils::verify;
/// let a = [1, 2, 3];
/// let b = [1, 2, 4];
/// assert!(!verify(&a, &b));
/// ```
///
/// Length mismatch:
///
/// ```
/// # use libvctrl_sha512::utils::verify;
/// assert!(!verify(&[1, 2], &[1, 2, 3]));
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

    // SAFETY: read_volatile is used to force the compiler to actually read `v`
    // and prevent it from optimizing away the accumulation.
    #[allow(unsafe_code)]
    let v = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(v)) };
    v == 0
}
