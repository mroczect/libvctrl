//! # Internal Utilities – Endianness & Constant‑Time Comparison
//!
//! This module provides the low‑level helper functions used throughout the
//! `libvctrl_sha512` crate.  None of these items are part of the public API,
//! but they are documented here for maintainers and security reviewers.
//!
//! ## Contents
//!
//! - **Constants** – the SHA‑512 block size and output size, available as
//!   [`BLOCKBYTES`] and [`BYTES`].
//! - **Big‑endian conversion** – [`load_be`] and [`store_be`] convert between
//!   byte slices and `u64` values in a portable, efficient way.
//! - **Constant‑time comparison** – [`verify`] compares two byte slices in a
//!   way that does not depend on the data, preventing timing side‑channel
//!   leakage.
//!
//! ## Security Considerations
//!
//! ### `verify`
//!
//! The `verify` function is used for every MAC and hash comparison in this
//! crate.  It has the following properties:
//!
//! - **Length check first** – if the slices have different lengths, the
//!   function returns `false` immediately.  This is safe because the length
//!   is always publicly known (e.g., a 64‑byte HMAC).
//! - **Bitwise accumulation** – for each byte pair `(a, b)`, the XOR
//!   difference `a ^ b` is OR‑ed into an accumulator `v`.  The loop does not
//!   branch on the value of the bytes, so its runtime is independent of the
//!   number of matching bytes.
//! - **Volatile read barrier** – before the final comparison, `v` is read
//!   through `core::ptr::read_volatile`.  This prevents the compiler from
//!   optimizing away the accumulation loop or short‑circuiting the final
//!   check, both of which could leak timing information.
//! - **WebAssembly hardening** – on `wasm32` and `wasm64` targets, an
//!   additional mixing step is performed before the byte‑wise XOR.  This
//!   compensates for the lack of a constant‑time instruction set in some
//!   WASM runtimes.
//!
//! ### Endian‑handling
//!
//! `load_be` and `store_be` use `u64::from_be_bytes` and `u64::to_be_bytes`,
//! which are guaranteed to compile to efficient single‑instruction loads and
//! stores on both big‑ and little‑endian architectures.  This is both more
//! readable and more optimizer‑friendly than manual byte shuffling.
//!
//! ## Examples
//!
//! The utilities are `pub` within the crate.  Typical usage looks like this:
//!
//! ```rust
//! use libvctrl_sha512::utils::verify;
//!
//! let a = [0xAB; 64];
//! let b = [0xAB; 64];
//! assert!(verify(&a, &b));
//!
//! let c = [0xCD; 64];
//! assert!(!verify(&a, &c));
//! ```

/// SHA‑512 block size in bytes.
///
/// Every complete message block processed by the compression function is
/// exactly 128 bytes (1024 bits).
pub const BLOCKBYTES: usize = 128;

/// SHA‑512 output size in bytes.
///
/// The final digest is always 64 bytes (512 bits).
pub const BYTES: usize = 64;

/// Load a big‑endian `u64` from `base` starting at `offset`.
///
/// This is equivalent to reading 8 bytes from `base[offset..offset+8]` and
/// interpreting them as a big‑endian unsigned 64‑bit integer.
///
/// # Panics
///
/// Panics if `offset + 8` exceeds the length of `base`.  In practice, all
/// callers ensure sufficient buffer space, so this panic indicates a bug.
///
/// # Performance
///
/// Uses [`u64::from_be_bytes`], which maps to a single `bswap` instruction
/// on little‑endian targets and a simple load on big‑endian targets.
#[inline(always)]
pub fn load_be(base: &[u8], offset: usize) -> u64 {
    u64::from_be_bytes(base[offset..offset + 8].try_into().unwrap())
}

/// Store a `u64` as big‑endian bytes into `base` starting at `offset`.
///
/// Writes the 8‑byte big‑endian representation of `x` into
/// `base[offset..offset+8]`.
///
/// # Panics
///
/// Panics if `offset + 8` exceeds the length of `base`.
///
/// # Performance
///
/// Uses [`u64::to_be_bytes`], which generates optimal code on all
/// architectures.
#[inline(always)]
pub fn store_be(base: &mut [u8], offset: usize, x: u64) {
    base[offset..offset + 8].copy_from_slice(&x.to_be_bytes());
}

/// Constant‑time comparison of two byte slices.
///
/// This function is used to compare digests, HMAC tags, and other
/// cryptographic material without leaking information about the byte values
/// through timing.
///
/// # Algorithm
///
/// 1. If `x.len() != y.len()`, return `false` immediately (lengths are
///    public knowledge in our protocols, so this does not leak secrets).
/// 2. On WebAssembly targets (`wasm32`/`wasm64`), an additional mixing
///    phase is applied to each byte before XORing, because some WASM
///    runtimes do not guarantee constant‑time behaviour for all operations.
/// 3. The XOR of each corresponding byte pair is accumulated into a `u32`
///    accumulator `v`.  The loop processes all bytes unconditionally.
/// 4. A volatile read of `v` prevents compiler optimizations from
///    short‑circuiting the comparison.
/// 5. The result is `v == 0`, i.e., `true` only if all bytes matched.
///
/// # Security
///
/// This implementation follows the recommendations from several widely
/// deployed cryptographic libraries.  The volatile barrier is a common
/// idiom to stop the optimizer from collapsing the comparison into a
/// data‑dependent branch.  The additional WASM mixing compensates for
/// the fact that some WASM engines may not implement bitwise operations
/// in constant time.
///
/// # Examples
///
/// ```rust
/// use libvctrl_sha512::utils::verify;
///
/// let a = [0xAB; 64];
/// let b = [0xAB; 64];
/// assert!(verify(&a, &b));
/// assert!(!verify(&a, &[0; 64]));
/// ```
pub fn verify(x: &[u8], y: &[u8]) -> bool {
    if x.len() != y.len() {
        return false;
    }
    let mut v: u32 = 0;

    // WASM hardening: mix each byte into a hash state to avoid
    // potentially non‑constant‑time comparisons in the runtime.
    #[cfg(any(target_arch = "wasm32", target_arch = "wasm64"))]
    {
        let (mut h1, mut h2) = (0u32, 0u32);
        for (b1, b2) in x.iter().zip(y.iter()) {
            h1 ^= (h1 << 5).wrapping_add((h1 >> 2) ^ *b1 as u32);
            h2 ^= (h2 << 5).wrapping_add((h2 >> 2) ^ *b2 as u32);
        }
        v |= h1 ^ h2;
    }

    // Bitwise accumulation (executed on all platforms, including WASM).
    for (a, b) in x.iter().zip(y.iter()) {
        v |= (a ^ b) as u32;
    }

    // Volatile read barrier – prevents the compiler from optimising away
    // the loop or short‑circuiting the comparison.
    let v = unsafe { core::ptr::read_volatile(&v) };
    v == 0
}
