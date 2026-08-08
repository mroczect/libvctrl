//! # Internal Utilities
//!
//! This module provides low‑level helper functions used throughout the SHA‑512,
//! HMAC, and HKDF implementations. It is not intended for public consumption;
//! however, it is marked `pub` because other modules in the crate need to access
//! these utilities.
//!
//! ## Contents
//!
//! - **Constants**: `BLOCKBYTES` and `BYTES` – the SHA‑512 block size (128 bytes)
//!   and digest size (64 bytes).
//! - **Endianness helpers**: `load_be` and `store_be` – convert between `u64` and
//!   big‑endian byte arrays.
//! - **Constant‑time comparison**: `verify` – compares two byte slices in
//!   constant time to prevent timing side‑channel attacks.
//!
//! ## Endianness in SHA‑512
//!
//! SHA‑512 operates on 64‑bit words in **big‑endian** (network byte order).
//! All messages are padded and parsed as big‑endian, and the final digest is
//! also output in big‑endian. This module provides the necessary conversion
//! functions to handle this correctly on both little‑endian and big‑endian
//! architectures.
//!
//! ## Security Notes
//!
//! - The `verify` function is designed to be resistant to timing attacks.
//!   It uses a data‑independent loop and a volatile read to prevent compiler
//!   optimisations that could leak information.
//! - All functions are marked `#[inline(always)]` where appropriate to ensure
//!   optimal performance without sacrificing clarity.
//! - The `unsafe` block in `verify` is justified and safe because it reads
//!   a stack‑allocated integer that is always valid.

/// SHA‑512 block size in bytes.
///
/// This is the size of a single message block processed by the compression
/// function. It is `128` bytes (1024 bits).
///
/// # Definition
///
/// In SHA‑512, the message is padded to a multiple of 1024 bits (128 bytes)
/// before processing. Each block is then expanded and compressed individually.
///
/// # Example
/// ```
/// use libvctrl_sha512::BLOCKBYTES;
/// assert_eq!(BLOCKBYTES, 128);
/// ```
pub const BLOCKBYTES: usize = 128;

/// SHA‑512 output size in bytes.
///
/// This is the length of the final digest: `64` bytes (512 bits).
///
/// # Definition
///
/// The SHA‑512 hash output consists of 8 state words, each 64 bits wide,
/// concatenated in big‑endian order.
///
/// # Example
/// ```
/// use libvctrl_sha512::BYTES;
/// assert_eq!(BYTES, 64);
/// ```
pub const BYTES: usize = 64;

/// Loads a 64‑bit integer from a big‑endian byte slice at a given offset.
///
/// This function reads 8 bytes starting at `offset` from `base` and interprets
/// them as a big‑endian `u64`. It is used to parse message words, initial
/// vectors (IVs), and round constants.
///
/// # Arguments
///
/// * `base`   – The byte slice to read from. Must have at least `offset + 8`
///   bytes.
/// * `offset` – The starting index (in bytes) to read from.
///
/// # Returns
///
/// The `u64` value represented by the 8 bytes in big‑endian order.
///
/// # Panics
///
/// This function will **panic** if the slice is too short (less than
/// `offset + 8` bytes). It is the caller's responsibility to ensure bounds are
/// valid, and in this crate, all callers guarantee this.
///
/// # Example
/// ```
/// use libvctrl_sha512::utils::load_be;
///
/// let bytes = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
/// let value = load_be(&bytes, 0);
/// assert_eq!(value, 0x0102030405060708);
///
/// // Reading from an offset
/// let bytes = [0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
/// let value = load_be(&bytes, 2);
/// assert_eq!(value, 0x0102030405060708);
/// ```
///
/// # Performance
///
/// This function is marked `#[inline(always)]` and compiles to efficient
/// byte‑swapping instructions on little‑endian architectures (e.g., `bswap`
/// on x86_64). On big‑endian architectures, it compiles to a simple load.
#[inline(always)]
pub fn load_be(base: &[u8], offset: usize) -> u64 {
    let addr = &base[offset..];
    (addr[7] as u64)
        | (addr[6] as u64) << 8
        | (addr[5] as u64) << 16
        | (addr[4] as u64) << 24
        | (addr[3] as u64) << 32
        | (addr[2] as u64) << 40
        | (addr[1] as u64) << 48
        | (addr[0] as u64) << 56
}

/// Stores a 64‑bit integer as big‑endian bytes at a given offset.
///
/// This function writes the 8 bytes of `x` (in big‑endian order) into `base`
/// starting at `offset`. It is used to produce final digests and intermediate
/// values during hash computation.
///
/// # Arguments
///
/// * `base`   – The mutable byte slice to write to. Must have at least
///   `offset + 8` bytes.
/// * `offset` – The starting index (in bytes) to write to.
/// * `x`      – The 64‑bit value to store.
///
/// # Panics
///
/// This function will **panic** if the slice is too short (less than
/// `offset + 8` bytes). Callers must ensure bounds are valid.
///
/// # Example
/// ```
/// use libvctrl_sha512::utils::store_be;
///
/// let mut bytes = [0u8; 8];
/// store_be(&mut bytes, 0, 0x0102030405060708);
/// assert_eq!(bytes, [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
///
/// // Writing at an offset
/// let mut bytes = [0u8; 10];
/// store_be(&mut bytes, 2, 0x0102030405060708);
/// assert_eq!(bytes, [0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
/// ```
///
/// # Performance
///
/// Like `load_be`, this function is `#[inline(always)]` and compiles to
/// efficient byte‑swapping instructions on little‑endian architectures.
#[inline(always)]
pub fn store_be(base: &mut [u8], offset: usize, x: u64) {
    let addr = &mut base[offset..];
    addr[7] = x as u8;
    addr[6] = (x >> 8) as u8;
    addr[5] = (x >> 16) as u8;
    addr[4] = (x >> 24) as u8;
    addr[3] = (x >> 32) as u8;
    addr[2] = (x >> 40) as u8;
    addr[1] = (x >> 48) as u8;
    addr[0] = (x >> 56) as u8;
}

/// Compares two byte slices in **constant time**.
///
/// This function is designed to prevent timing side‑channel attacks that could
/// leak information about the compared values. It uses a bitwise XOR reduction
/// and a volatile read to ensure the comparison runs in O(n) with data‑
/// independent timing.
///
/// # Why Constant Time?
///
/// In cryptographic applications, comparing secret values (like MACs or hash
/// digests) must not reveal where the first difference occurs. A naive `==`
/// comparison can short‑circuit early, allowing an attacker to deduce the
/// correct value by measuring execution time. This function avoids that by
/// always iterating over the entire length and accumulating the result.
///
/// # Security
///
/// - The loop iterates over all bytes of the shorter slice (if lengths differ,
///   it returns `false` early, which is a length leak but unavoidable and
///   typically acceptable in practice because lengths are often known or fixed).
/// - For equal lengths, every byte pair is XORed and ORed into a single
///   accumulator; the loop always runs for the full length.
/// - A volatile read (`core::ptr::read_volatile`) is used to prevent the
///   compiler from optimising away the comparison when the result is unused.
/// - On WebAssembly (`wasm32`/`wasm64`), an additional mixing step is performed
///   to provide extra protection against certain timing variations specific to
///   that architecture.
///
/// # Arguments
///
/// * `x` – First byte slice.
/// * `y` – Second byte slice.
///
/// # Returns
///
/// `true` if both slices have the same length and contain identical bytes,
/// `false` otherwise.
///
/// # Example
/// ```
/// use libvctrl_sha512::utils::verify;
///
/// let a = [1, 2, 3];
/// let b = [1, 2, 3];
/// let c = [1, 2, 4];
/// let d = [1, 2];
///
/// assert!(verify(&a, &b));
/// assert!(!verify(&a, &c));
/// assert!(!verify(&a, &d)); // different lengths
/// ```
///
/// # Safety
///
/// The `unsafe` block in this function is **sound** because:
/// - It reads from a stack‑allocated `v: u32` that is always valid.
/// - The read is aligned and within bounds.
/// - The volatile read is used only to prevent compiler optimisations, not
///   to access memory that could be invalid.
///
/// # References
///
/// - See [Timing Attacks on Cryptographic Implementations](https://en.wikipedia.org/wiki/Timing_attack)
/// - See the [Rustonomicon on `read_volatile`](https://doc.rust-lang.org/nomicon/volatility.html)
pub fn verify(x: &[u8], y: &[u8]) -> bool {
    if x.len() != y.len() {
        return false;
    }
    let mut v: u32 = 0;

    // Extra protection for WebAssembly targets
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
        v |= (a ^ b) as u32;
    }
    // Use volatile read to prevent the compiler from removing the seemingly
    // unused `v`.
    let v = unsafe { core::ptr::read_volatile(&v) };
    v == 0
}
