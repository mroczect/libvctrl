//! # SHA‑512 (FIPS 180‑4)
//!
//! This module implements the **SHA‑512** cryptographic hash function as defined
//! in [FIPS 180‑4](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf).
//! It is the backbone of the entire `libvctrl_sha512` crate and also serves as
//! the underlying primitive for [`HMAC`](crate::HMAC) and [`HKDF`](crate::HKDF).
//!
//! ## Overview
//!
//! SHA‑512 takes an arbitrary‑length message and produces a **64‑byte**
//! (512‑bit) digest. It belongs to the SHA‑2 family and is designed for
//! environments where 256 bits of security are required. Common use‑cases
//! include:
//!
//! - File / data integrity verification
//! - Digital signatures (with RSASSA‑PSS or ECDSA)
//! - Password hashing (when combined with a salt, e.g., in PBKDF2)
//! - Building MACs (HMAC‑SHA‑512) and KDFs (HKDF‑SHA‑512)
//!
//! ## Implementation details
//!
//! This implementation is:
//!
//! - **FIPS 180‑4 compliant** – message padding uses a 128‑bit big‑endian
//!   length field, as required by the standard.
//! - **Zero‑dependency** – it relies only on `core` and is `#![no_std]`
//!   compatible.
//! - **Audited** – the code has undergone a security audit (v0.2.0) and
//!   all findings have been addressed.
//! - **Configurable code size** – enable the `opt_size` feature to trade
//!   a small amount of speed for a significantly smaller binary.
//!
//! ## Internal architecture
//!
//! SHA‑512 operates on 1024‑bit (128‑byte) message blocks. The hashing
//! state consists of:
//!
//! - Eight 64‑bit working variables (`a`–`h`) stored in [`State`].
//! - A message schedule of 16 × 64‑bit words stored in [`W`].
//! - An input buffer, a read pointer, and a total message length counter
//!   kept in the [`Hash`] struct.
//!
//! The compression function performs 80 rounds of bitwise operations
//! (Σ, Maj, Ch, etc.) using round‑specific constants.
//!
//! ## Core types
//!
//! The primary public type is [`Hash`]. It provides three usage patterns:
//!
//! 1. **One‑shot** – [`Hash::hash`] computes the digest in a single call.
//! 2. **Streaming** – [`Hash::new`] creates a state, [`update`](Hash::update)
//!    feeds data incrementally, and [`finalize`](Hash::finalize) produces
//!    the digest (consuming the state).
//! 3. **Constant‑time verification** – [`Hash::verify`] checks a digest
//!    against an expected value using a timing‑side‑channel‑resistant
//!    comparison.
//!
//! ## Examples
//!
//! ### One‑shot hashing
//!
//! ```rust
//! use libvctrl_sha512::Hash;
//!
//! let digest = Hash::hash(b"hello world");
//! assert_eq!(digest.len(), 64);
//! ```
//!
//! ### Streaming hashing
//!
//! ```rust
//! use libvctrl_sha512::Hash;
//!
//! let mut hasher = Hash::new();
//! hasher.update(b"hello ");
//! hasher.update(b"world");
//! let d = hasher.finalize();
//! assert_eq!(d, Hash::hash(b"hello world"));
//! ```
//!
//! ### Constant‑time verification
//!
//! ```rust
//! use libvctrl_sha512::Hash;
//!
//! let expected = Hash::hash(b"secret");
//! let mut verifier = Hash::new();
//! verifier.update(b"secret");
//! assert!(verifier.verify(&expected));
//! assert!(!verifier.verify(&[0u8; 64]));
//! ```
//!
//! ## Internals (for contributors)
//!
//! The module also exposes two `pub(crate)` helpers:
//!
//! - [`State`] – the eight 64‑bit working variables of SHA‑512.
//! - [`W`] – the message schedule array.
//!
//! These are used by the [`sha384`](crate::sha384) module to reuse the
//! compression function.

use crate::utils::{load_be, store_be, verify};

/// Message schedule – 16 × 64‑bit words used during compression.
///
/// This struct wraps the 16 working variables `W_0` … `W_15` from the
/// SHA‑512 specification.  It handles message expansion and the main
/// compression loop (round functions).
struct W([u64; 16]);

/// SHA‑512 working state (eight 64‑bit registers `a`–`h`).
///
/// This is `pub(crate)` because SHA‑384 reuses the same compression
/// function; it is not part of the public API.
#[derive(Copy, Clone)]
pub(crate) struct State(pub(crate) [u64; 8]);

impl W {
    /// Load the first 128 bytes of a message block as sixteen big‑endian `u64`s.
    fn new(input: &[u8]) -> Self {
        let mut w = [0u64; 16];
        for (i, e) in w.iter_mut().enumerate() {
            *e = load_be(input, i * 8);
        }
        W(w)
    }

    // ----- SHA‑512 logical functions -----
    /// The `Ch` function: `(x & y) ^ (!x & z)`.
    #[inline(always)]
    fn Ch(x: u64, y: u64, z: u64) -> u64 {
        (x & y) ^ (!x & z)
    }
    /// The `Maj` function: `(x & y) ^ (x & z) ^ (y & z)`.
    #[inline(always)]
    fn Maj(x: u64, y: u64, z: u64) -> u64 {
        (x & y) ^ (x & z) ^ (y & z)
    }
    /// The `Σ0` function for the hash computation.
    #[inline(always)]
    fn Sigma0(x: u64) -> u64 {
        x.rotate_right(28) ^ x.rotate_right(34) ^ x.rotate_right(39)
    }
    /// The `Σ1` function for the hash computation.
    #[inline(always)]
    fn Sigma1(x: u64) -> u64 {
        x.rotate_right(14) ^ x.rotate_right(18) ^ x.rotate_right(41)
    }
    /// The `σ0` function for message schedule expansion.
    #[inline(always)]
    fn sigma0(x: u64) -> u64 {
        x.rotate_right(1) ^ x.rotate_right(8) ^ (x >> 7)
    }
    /// The `σ1` function for message schedule expansion.
    #[inline(always)]
    fn sigma1(x: u64) -> u64 {
        x.rotate_right(19) ^ x.rotate_right(61) ^ (x >> 6)
    }

    /// Message expansion helper: `w[a] += σ1(w[b]) + w[c] + σ0(w[d])`.
    #[cfg_attr(feature = "opt_size", inline(never))]
    #[cfg_attr(not(feature = "opt_size"), inline(always))]
    fn M(&mut self, a: usize, b: usize, c: usize, d: usize) {
        let w = &mut self.0;
        w[a] = w[a]
            .wrapping_add(Self::sigma1(w[b]))
            .wrapping_add(w[c])
            .wrapping_add(Self::sigma0(w[d]));
    }

    /// Expand the 16‑word message schedule into 80 words.
    ///
    /// This method generates the full 80‑word message schedule required
    /// for the SHA‑512 compression function. It applies the message
    /// schedule recurrence to compute new words from previous ones.
    #[inline]
    fn expand(&mut self) {
        // Each call updates one word of the schedule using the circular
        // queue pattern (index modulo 16).
        self.M(0, (0 + 14) & 15, (0 + 9) & 15, (0 + 1) & 15);
        self.M(1, (1 + 14) & 15, (1 + 9) & 15, (1 + 1) & 15);
        self.M(2, (2 + 14) & 15, (2 + 9) & 15, (2 + 1) & 15);
        self.M(3, (3 + 14) & 15, (3 + 9) & 15, (3 + 1) & 15);
        self.M(4, (4 + 14) & 15, (4 + 9) & 15, (4 + 1) & 15);
        self.M(5, (5 + 14) & 15, (5 + 9) & 15, (5 + 1) & 15);
        self.M(6, (6 + 14) & 15, (6 + 9) & 15, (6 + 1) & 15);
        self.M(7, (7 + 14) & 15, (7 + 9) & 15, (7 + 1) & 15);
        self.M(8, (8 + 14) & 15, (8 + 9) & 15, (8 + 1) & 15);
        self.M(9, (9 + 14) & 15, (9 + 9) & 15, (9 + 1) & 15);
        self.M(10, (10 + 14) & 15, (10 + 9) & 15, (10 + 1) & 15);
        self.M(11, (11 + 14) & 15, (11 + 9) & 15, (11 + 1) & 15);
        self.M(12, (12 + 14) & 15, (12 + 9) & 15, (12 + 1) & 15);
        self.M(13, (13 + 14) & 15, (13 + 9) & 15, (13 + 1) & 15);
        self.M(14, (14 + 14) & 15, (14 + 9) & 15, (14 + 1) & 15);
        self.M(15, (15 + 14) & 15, (15 + 9) & 15, (15 + 1) & 15);
    }

    /// Perform one SHA‑512 round (`F` function) on the working state.
    ///
    /// `i` is the round index (0..=79), `k` is the round constant.
    /// This method updates the eight working variables according to
    /// the SHA‑512 round operation.
    #[cfg_attr(feature = "opt_size", inline(never))]
    #[cfg_attr(not(feature = "opt_size"), inline(always))]
    fn F(&mut self, state: &mut State, i: usize, k: u64) {
        let t = &mut state.0;
        // T1 = h + Σ1(e) + Ch(e, f, g) + K[i] + W[i]
        t[(16 - i + 7) & 7] = t[(16 - i + 7) & 7]
            .wrapping_add(Self::Sigma1(t[(16 - i + 4) & 7]))
            .wrapping_add(Self::Ch(
                t[(16 - i + 4) & 7],
                t[(16 - i + 5) & 7],
                t[(16 - i + 6) & 7],
            ))
            .wrapping_add(k)
            .wrapping_add(self.0[i]);
        // d = d + T1
        t[(16 - i + 3) & 7] = t[(16 - i + 3) & 7].wrapping_add(t[(16 - i + 7) & 7]);
        // T2 = Σ0(a) + Maj(a, b, c);  h = T1 + T2
        t[(16 - i + 7) & 7] = t[(16 - i + 7) & 7]
            .wrapping_add(Self::Sigma0(t[(16 - i + 0) & 7]))
            .wrapping_add(Self::Maj(
                t[(16 - i + 0) & 7],
                t[(16 - i + 1) & 7],
                t[(16 - i + 2) & 7],
            ));
    }

    /// Run 16 consecutive rounds starting at offset `s` (0‑4).
    ///
    /// Each call processes 16 rounds (out of the total 80 rounds).
    /// The four rounds are interleaved with message expansion operations.
    fn G(&mut self, state: &mut State, s: usize) {
        const ROUND_CONSTANTS: [u64; 80] = [
            0x428a2f98d728ae22,
            0x7137449123ef65cd,
            0xb5c0fbcfec4d3b2f,
            0xe9b5dba58189dbbc,
            0x3956c25bf348b538,
            0x59f111f1b605d019,
            0x923f82a4af194f9b,
            0xab1c5ed5da6d8118,
            0xd807aa98a3030242,
            0x12835b0145706fbe,
            0x243185be4ee4b28c,
            0x550c7dc3d5ffb4e2,
            0x72be5d74f27b896f,
            0x80deb1fe3b1696b1,
            0x9bdc06a725c71235,
            0xc19bf174cf692694,
            0xe49b69c19ef14ad2,
            0xefbe4786384f25e3,
            0x0fc19dc68b8cd5b5,
            0x240ca1cc77ac9c65,
            0x2de92c6f592b0275,
            0x4a7484aa6ea6e483,
            0x5cb0a9dcbd41fbd4,
            0x76f988da831153b5,
            0x983e5152ee66dfab,
            0xa831c66d2db43210,
            0xb00327c898fb213f,
            0xbf597fc7beef0ee4,
            0xc6e00bf33da88fc2,
            0xd5a79147930aa725,
            0x06ca6351e003826f,
            0x142929670a0e6e70,
            0x27b70a8546d22ffc,
            0x2e1b21385c26c926,
            0x4d2c6dfc5ac42aed,
            0x53380d139d95b3df,
            0x650a73548baf63de,
            0x766a0abb3c77b2a8,
            0x81c2c92e47edaee6,
            0x92722c851482353b,
            0xa2bfe8a14cf10364,
            0xa81a664bbc423001,
            0xc24b8b70d0f89791,
            0xc76c51a30654be30,
            0xd192e819d6ef5218,
            0xd69906245565a910,
            0xf40e35855771202a,
            0x106aa07032bbd1b8,
            0x19a4c116b8d2d0c8,
            0x1e376c085141ab53,
            0x2748774cdf8eeb99,
            0x34b0bcb5e19b48a8,
            0x391c0cb3c5c95a63,
            0x4ed8aa4ae3418acb,
            0x5b9cca4f7763e373,
            0x682e6ff3d6b2b8a3,
            0x748f82ee5defb2fc,
            0x78a5636f43172f60,
            0x84c87814a1f0ab72,
            0x8cc702081a6439ec,
            0x90befffa23631e28,
            0xa4506cebde82bde9,
            0xbef9a3f7b2c67915,
            0xc67178f2e372532b,
            0xca273eceea26619c,
            0xd186b8c721c0c207,
            0xeada7dd6cde0eb1e,
            0xf57d4f7fee6ed178,
            0x06f067aa72176fba,
            0x0a637dc5a2c898a6,
            0x113f9804bef90dae,
            0x1b710b35131c471b,
            0x28db77f523047d84,
            0x32caab7b40c72493,
            0x3c9ebe0a15c9bebc,
            0x431d67c49c100d4c,
            0x4cc5d4becb3e42b6,
            0x597f299cfc657e2a,
            0x5fcb6fab3ad6faec,
            0x6c44198c4a475817,
        ];
        let rc = &ROUND_CONSTANTS[s * 16..];
        self.F(state, 0, rc[0]);
        self.F(state, 1, rc[1]);
        self.F(state, 2, rc[2]);
        self.F(state, 3, rc[3]);
        self.F(state, 4, rc[4]);
        self.F(state, 5, rc[5]);
        self.F(state, 6, rc[6]);
        self.F(state, 7, rc[7]);
        self.F(state, 8, rc[8]);
        self.F(state, 9, rc[9]);
        self.F(state, 10, rc[10]);
        self.F(state, 11, rc[11]);
        self.F(state, 12, rc[12]);
        self.F(state, 13, rc[13]);
        self.F(state, 14, rc[14]);
        self.F(state, 15, rc[15]);
    }
}

impl State {
    /// Initialise the working state with the SHA‑512 initial vector (IV).
    ///
    /// Returns a new `State` with the eight 64‑bit words set to the
    /// standard initial hash values of SHA‑512.
    pub(crate) fn new() -> Self {
        const IV: [u8; 64] = [
            0x6a, 0x09, 0xe6, 0x67, 0xf3, 0xbc, 0xc9, 0x08, 0xbb, 0x67, 0xae, 0x85, 0x84, 0xca,
            0xa7, 0x3b, 0x3c, 0x6e, 0xf3, 0x72, 0xfe, 0x94, 0xf8, 0x2b, 0xa5, 0x4f, 0xf5, 0x3a,
            0x5f, 0x1d, 0x36, 0xf1, 0x51, 0x0e, 0x52, 0x7f, 0xad, 0xe6, 0x82, 0xd1, 0x9b, 0x05,
            0x68, 0x8c, 0x2b, 0x3e, 0x6c, 0x1f, 0x1f, 0x83, 0xd9, 0xab, 0xfb, 0x41, 0xbd, 0x6b,
            0x5b, 0xe0, 0xcd, 0x19, 0x13, 0x7e, 0x21, 0x79,
        ];
        let mut t = [0u64; 8];
        for (i, e) in t.iter_mut().enumerate() {
            *e = load_be(&IV, i * 8);
        }
        State(t)
    }

    /// Add another `State` to this one (word‑wise wrapping addition).
    ///
    /// This is used after processing a message block to add the result
    /// to the initial state of the block.
    #[inline(always)]
    pub(crate) fn add(&mut self, x: &State) {
        let sx = &mut self.0;
        let ex = &x.0;
        sx[0] = sx[0].wrapping_add(ex[0]);
        sx[1] = sx[1].wrapping_add(ex[1]);
        sx[2] = sx[2].wrapping_add(ex[2]);
        sx[3] = sx[3].wrapping_add(ex[3]);
        sx[4] = sx[4].wrapping_add(ex[4]);
        sx[5] = sx[5].wrapping_add(ex[5]);
        sx[6] = sx[6].wrapping_add(ex[6]);
        sx[7] = sx[7].wrapping_add(ex[7]);
    }

    /// Serialise the state into 64 bytes of big‑endian output.
    ///
    /// Each 64‑bit word is converted to 8 bytes in big‑endian order.
    pub(crate) fn store(&self, out: &mut [u8]) {
        for (i, &e) in self.0.iter().enumerate() {
            store_be(out, i * 8, e);
        }
    }

    /// Process as many complete 128‑byte blocks as possible from `input`.
    ///
    /// Returns the number of unprocessed bytes (0..127).
    ///
    /// # Algorithm
    /// For each complete 128‑byte block, it creates a message schedule,
    /// performs 80 rounds of compression (in groups of 16), and adds the
    /// result to the current state.
    pub(crate) fn blocks(&mut self, mut input: &[u8]) -> usize {
        let mut t = *self;
        let mut inlen = input.len();
        while inlen >= 128 {
            let mut w = W::new(input);
            w.G(&mut t, 0);
            w.expand();
            w.G(&mut t, 1);
            w.expand();
            w.G(&mut t, 2);
            w.expand();
            w.G(&mut t, 3);
            w.expand();
            w.G(&mut t, 4);
            t.add(self);
            self.0 = t.0;
            input = &input[128..];
            inlen -= 128;
        }
        inlen
    }
}

/// SHA‑512 hash state.
///
/// `Hash` allows incremental digestion of data through a builder‑like API.
/// It implements `Copy` and `Clone`, and its `Default` is equivalent to
/// `Hash::new()`.
///
/// # Examples
///
/// ```rust
/// use libvctrl_sha512::Hash;
///
/// let mut h = Hash::new();
/// h.update(b"some data");
/// let digest = h.finalize();
/// assert_eq!(digest.len(), 64);
/// ```
#[derive(Copy, Clone)]
pub struct Hash {
    pub(crate) state: State,
    pub(crate) w: [u8; 128],
    pub(crate) r: usize,
    pub(crate) len: usize,
}

impl Hash {
    /// Create a new SHA‑512 hasher.
    ///
    /// The initial state is set to the standard SHA‑512 IV. The internal
    /// buffer and length counter are zeroed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use libvctrl_sha512::Hash;
    ///
    /// let hasher = Hash::new();
    /// ```
    pub fn new() -> Self {
        Hash {
            state: State::new(),
            r: 0,
            w: [0u8; 128],
            len: 0,
        }
    }

    /// Feed additional data into the hasher (crate‑internal).
    ///
    /// This is equivalent to [`update`](Hash::update) but is `pub(crate)`
    /// so that the `sha384` module can reuse the same logic.
    pub(crate) fn _update<T: AsRef<[u8]>>(&mut self, input: T) {
        let input = input.as_ref();
        let mut n = input.len();
        self.len += n;
        let av = 128 - self.r;
        let tc = core::cmp::min(n, av);
        self.w[self.r..self.r + tc].copy_from_slice(&input[0..tc]);
        self.r += tc;
        n -= tc;
        let pos = tc;
        if self.r == 128 {
            self.state.blocks(&self.w);
            self.r = 0;
        }
        if self.r == 0 && n > 0 {
            let rb = self.state.blocks(&input[pos..]);
            if rb > 0 {
                self.w[..rb].copy_from_slice(&input[pos + n - rb..]);
                self.r = rb;
            }
        }
    }

    /// Feed additional data into the hasher.
    ///
    /// This method can be called any number of times before finalisation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use libvctrl_sha512::Hash;
    ///
    /// let mut h = Hash::new();
    /// h.update(b"hello ");
    /// h.update(b"world");
    /// ```
    pub fn update<T: AsRef<[u8]>>(&mut self, input: T) {
        self._update(input)
    }

    /// Finalize the hash and produce the 64‑byte SHA‑512 digest.
    ///
    /// This method consumes the `Hash` instance.  Message padding is applied
    /// according to FIPS 180‑4: a `1` bit, zero or more `0` bits, and a
    /// 128‑bit big‑endian length field (with the upper 64 bits set to zero).
    ///
    /// # Security
    ///
    /// After finalisation, the internal state is consumed.  No further
    /// updates are possible, and the digest is returned as a `[u8; 64]`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use libvctrl_sha512::Hash;
    ///
    /// let digest = Hash::new().finalize();
    /// assert_eq!(digest.len(), 64);
    /// ```
    pub fn finalize(mut self) -> [u8; 64] {
        let mut padded = [0u8; 256];
        padded[..self.r].copy_from_slice(&self.w[..self.r]);
        padded[self.r] = 0x80;
        let r = if self.r < 112 { 128 } else { 256 };
        // 128‑bit length representation (high 8 bytes are zero)
        let low_bits = (self.len as u64).wrapping_mul(8);
        padded[r - 16..r - 8].fill(0);
        store_be(&mut padded, r - 8, low_bits);

        self.state.blocks(&padded[..r]);
        let mut out = [0u8; 64];
        self.state.store(&mut out);
        out
    }

    /// One‑shot hashing: compute the SHA‑512 digest of `input`.
    ///
    /// This is equivalent to creating a fresh `Hash`, calling
    /// [`update`](Hash::update) once, and then [`finalize`](Hash::finalize).
    ///
    /// # Example
    ///
    /// ```rust
    /// use libvctrl_sha512::Hash;
    ///
    /// let digest = Hash::hash(b"hello world");
    /// ```
    pub fn hash<T: AsRef<[u8]>>(input: T) -> [u8; 64] {
        let mut h = Self::new();
        h.update(input);
        h.finalize()
    }

    /// Finalize and verify the digest against `expected` in constant time.
    ///
    /// This method consumes the hasher, computes the final digest, and
    /// compares it byte‑by‑byte with `expected` using a timing‑attack
    /// resistant comparison.  The comparison is performed via
    /// [`utils::verify`](crate::utils::verify).
    ///
    /// # Returns
    ///
    /// `true` if the computed digest matches `expected`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use libvctrl_sha512::Hash;
    ///
    /// let expected = Hash::hash(b"secret");
    /// let mut h = Hash::new();
    /// h.update(b"secret");
    /// assert!(h.verify(&expected));
    /// ```
    pub fn verify(self, expected: &[u8; 64]) -> bool {
        let out = self.finalize();
        verify(&out, expected)
    }
}

impl Default for Hash {
    /// Returns a new `Hash` instance with the standard SHA‑512 IV.
    fn default() -> Self {
        Self::new()
    }
}
