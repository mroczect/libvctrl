//! # SHA‑512 Hash Implementation
//!
//! This module implements the SHA‑512 cryptographic hash function as specified in
//! [FIPS 180‑4](https://csrc.nist.gov/publications/detail/fips/180/4/final).
//! SHA‑512 produces a 64‑byte (512‑bit) digest and is widely used in security
//! protocols, digital signatures, and data integrity checks.
//!
//! ## Overview
//!
//! SHA‑512 is a member of the SHA‑2 family. It operates on 1024‑bit (128‑byte)
//! message blocks and uses a 64‑bit word size. The algorithm consists of:
//!
//! 1. **Padding** – append a single '1' bit, then zeros, then the message length
//!    (as a 128‑bit integer) to make the total length a multiple of 1024 bits.
//! 2. **Parsing** – break the padded message into 1024‑bit blocks.
//! 3. **Processing** – for each block, expand it into 80 64‑bit words and run
//!    80 rounds of compression using the SHA‑512 functions (Ch, Maj, Σ₀, Σ₁,
//!    σ₀, σ₁) and constant round keys.
//! 4. **Output** – concatenate the final 8 state words to produce the 64‑byte
//!    digest.
//!
//! ## Implementation Notes
//!
//! - This implementation is **pure Rust** and does not use assembly or
//!   hardware intrinsics.
//! - It is fully `#![no_std]`‑compatible and has no external dependencies.
//! - The `opt_size` feature can be enabled to reduce binary size at the cost of
//!   some performance (about 75% size reduction, ~16% performance loss).
//! - All functions are `#[inline]` where appropriate to allow the compiler to
//!   optimize.
//!
//! ## Security Properties
//!
//! - SHA‑512 is considered **collision‑resistant**, **preimage‑resistant**, and
//!   **second‑preimage‑resistant** as of 2026.
//! - The algorithm is deterministic; the same input always produces the same
//!   output.
//! - This implementation includes a constant‑time `verify` function to compare
//!   digests without leaking timing information.
//!
//! ## Examples
//!
//! ### One‑shot hashing
//! ```
//! use libvctrl_sha512::Hash;
//! let digest = Hash::hash(b"Hello, world!");
//! assert_eq!(digest.len(), 64);
//! ```
//!
//! ### Streaming (incremental)
//! ```
//! use libvctrl_sha512::Hash;
//! let mut hasher = Hash::new();
//! hasher.update(b"Hello, ");
//! hasher.update(b"world!");
//! let digest = hasher.finalize();
//! ```
//!
//! ### Verification
//! ```
//! use libvctrl_sha512::Hash;
//! let digest = Hash::hash(b"data");
//! // Recompute later...
//! let computed = Hash::hash(b"data");
//! assert!(computed.verify(&digest));
//! ```
//!
//! ## Performance
//!
//! On modern 64‑bit CPUs, SHA‑512 is reasonably fast (often faster than SHA‑256
//! on 64‑bit platforms because it processes more data per round). For
//! higher throughput, consider using hardware‑accelerated implementations
//! (e.g., via the `sha2` crate with SHA‑NI support) if available, but this
//! pure‑Rust version is portable and reliable.
//!
//! ## References
//!
//! - [FIPS 180‑4: Secure Hash Standard](https://csrc.nist.gov/publications/detail/fips/180/4/final)
//! - [SHA‑512 on Wikipedia](https://en.wikipedia.org/wiki/SHA-2)

use crate::utils::{load_be, store_be, verify};

/// Message schedule array (16 64‑bit words).
///
/// This holds the current block's words during the compression function.
/// The schedule is expanded from the initial 16 words to 80 words on the fly.
struct W([u64; 16]);

/// Internal state of the SHA‑512 hash (8 64‑bit words).
///
/// This represents the current hash value (a, b, c, d, e, f, g, h) as defined
/// in the specification. The fields are made `pub(crate)` so that the SHA‑384
/// module can initialize the state with its own IV.
#[derive(Copy, Clone)]
pub(crate) struct State(pub(crate) [u64; 8]);

impl W {
    /// Creates a new message schedule from a 128‑byte input block.
    #[inline]
    fn new(input: &[u8]) -> Self {
        let mut w = [0u64; 16];
        for (i, e) in w.iter_mut().enumerate() {
            *e = load_be(input, i * 8);
        }
        W(w)
    }

    /// SHA‑512 `Ch` function: choose bits from y or z based on x.
    #[inline(always)]
    fn Ch(x: u64, y: u64, z: u64) -> u64 {
        (x & y) ^ (!x & z)
    }

    /// SHA‑512 `Maj` function: majority of three inputs.
    #[inline(always)]
    fn Maj(x: u64, y: u64, z: u64) -> u64 {
        (x & y) ^ (x & z) ^ (y & z)
    }

    /// SHA‑512 Σ₀ (uppercase sigma zero): rotate and xor.
    #[inline(always)]
    fn Sigma0(x: u64) -> u64 {
        x.rotate_right(28) ^ x.rotate_right(34) ^ x.rotate_right(39)
    }

    /// SHA‑512 Σ₁ (uppercase sigma one): rotate and xor.
    #[inline(always)]
    fn Sigma1(x: u64) -> u64 {
        x.rotate_right(14) ^ x.rotate_right(18) ^ x.rotate_right(41)
    }

    /// SHA‑512 σ₀ (lowercase sigma zero): rotate and shift.
    #[inline(always)]
    fn sigma0(x: u64) -> u64 {
        x.rotate_right(1) ^ x.rotate_right(8) ^ (x >> 7)
    }

    /// SHA‑512 σ₁ (lowercase sigma one): rotate and shift.
    #[inline(always)]
    fn sigma1(x: u64) -> u64 {
        x.rotate_right(19) ^ x.rotate_right(61) ^ (x >> 6)
    }

    /// Message expansion: combine four existing words to produce a new one.
    ///
    /// `a` = index of the word to update.
    /// `b, c, d` = indices of words used in the formula.
    #[cfg_attr(feature = "opt_size", inline(never))]
    #[cfg_attr(not(feature = "opt_size"), inline(always))]
    fn M(&mut self, a: usize, b: usize, c: usize, d: usize) {
        let w = &mut self.0;
        w[a] = w[a]
            .wrapping_add(Self::sigma1(w[b]))
            .wrapping_add(w[c])
            .wrapping_add(Self::sigma0(w[d]));
    }

    /// Expands the message schedule from 16 words to 80 words.
    ///
    /// This is done in‑place; after this call, `self.0` has 80 logical words,
    /// though we only store 16 at a time (the newer ones overwrite older ones).
    #[inline]
    fn expand(&mut self) {
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

    /// One round of the SHA‑512 compression function.
    ///
    /// `state` – the current hash state (a..h).
    /// `i`     – round index (0..15) used to pick which state word to update.
    /// `k`     – round constant.
    #[cfg_attr(feature = "opt_size", inline(never))]
    #[cfg_attr(not(feature = "opt_size"), inline(always))]
    fn F(&mut self, state: &mut State, i: usize, k: u64) {
        let t = &mut state.0;
        t[(16 - i + 7) & 7] = t[(16 - i + 7) & 7]
            .wrapping_add(Self::Sigma1(t[(16 - i + 4) & 7]))
            .wrapping_add(Self::Ch(
                t[(16 - i + 4) & 7],
                t[(16 - i + 5) & 7],
                t[(16 - i + 6) & 7],
            ))
            .wrapping_add(k)
            .wrapping_add(self.0[i]);
        t[(16 - i + 3) & 7] = t[(16 - i + 3) & 7].wrapping_add(t[(16 - i + 7) & 7]);
        t[(16 - i + 7) & 7] = t[(16 - i + 7) & 7]
            .wrapping_add(Self::Sigma0(t[(16 - i + 0) & 7]))
            .wrapping_add(Self::Maj(
                t[(16 - i + 0) & 7],
                t[(16 - i + 1) & 7],
                t[(16 - i + 2) & 7],
            ));
    }

    /// Applies 16 rounds of the compression function for a given group.
    ///
    /// Each group corresponds to 16 rounds; there are 5 groups total (80 rounds).
    fn G(&mut self, state: &mut State, s: usize) {
        const ROUND_CONSTANTS: [u64; 80] = [
            0x428a2f98d728ae22, 0x7137449123ef65cd, 0xb5c0fbcfec4d3b2f,
            0xe9b5dba58189dbbc, 0x3956c25bf348b538, 0x59f111f1b605d019,
            0x923f82a4af194f9b, 0xab1c5ed5da6d8118, 0xd807aa98a3030242,
            0x12835b0145706fbe, 0x243185be4ee4b28c, 0x550c7dc3d5ffb4e2,
            0x72be5d74f27b896f, 0x80deb1fe3b1696b1, 0x9bdc06a725c71235,
            0xc19bf174cf692694, 0xe49b69c19ef14ad2, 0xefbe4786384f25e3,
            0x0fc19dc68b8cd5b5, 0x240ca1cc77ac9c65, 0x2de92c6f592b0275,
            0x4a7484aa6ea6e483, 0x5cb0a9dcbd41fbd4, 0x76f988da831153b5,
            0x983e5152ee66dfab, 0xa831c66d2db43210, 0xb00327c898fb213f,
            0xbf597fc7beef0ee4, 0xc6e00bf33da88fc2, 0xd5a79147930aa725,
            0x06ca6351e003826f, 0x142929670a0e6e70, 0x27b70a8546d22ffc,
            0x2e1b21385c26c926, 0x4d2c6dfc5ac42aed, 0x53380d139d95b3df,
            0x650a73548baf63de, 0x766a0abb3c77b2a8, 0x81c2c92e47edaee6,
            0x92722c851482353b, 0xa2bfe8a14cf10364, 0xa81a664bbc423001,
            0xc24b8b70d0f89791, 0xc76c51a30654be30, 0xd192e819d6ef5218,
            0xd69906245565a910, 0xf40e35855771202a, 0x106aa07032bbd1b8,
            0x19a4c116b8d2d0c8, 0x1e376c085141ab53, 0x2748774cdf8eeb99,
            0x34b0bcb5e19b48a8, 0x391c0cb3c5c95a63, 0x4ed8aa4ae3418acb,
            0x5b9cca4f7763e373, 0x682e6ff3d6b2b8a3, 0x748f82ee5defb2fc,
            0x78a5636f43172f60, 0x84c87814a1f0ab72, 0x8cc702081a6439ec,
            0x90befffa23631e28, 0xa4506cebde82bde9, 0xbef9a3f7b2c67915,
            0xc67178f2e372532b, 0xca273eceea26619c, 0xd186b8c721c0c207,
            0xeada7dd6cde0eb1e, 0xf57d4f7fee6ed178, 0x06f067aa72176fba,
            0x0a637dc5a2c898a6, 0x113f9804bef90dae, 0x1b710b35131c471b,
            0x28db77f523047d84, 0x32caab7b40c72493, 0x3c9ebe0a15c9bebc,
            0x431d67c49c100d4c, 0x4cc5d4becb3e42b6, 0x597f299cfc657e2a,
            0x5fcb6fab3ad6faec, 0x6c44198c4a475817,
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
    /// Creates a new SHA‑512 state with the standard initial vector (IV).
    pub(crate) fn new() -> Self {
        const IV: [u8; 64] = [
            0x6a, 0x09, 0xe6, 0x67, 0xf3, 0xbc, 0xc9, 0x08,
            0xbb, 0x67, 0xae, 0x85, 0x84, 0xca, 0xa7, 0x3b,
            0x3c, 0x6e, 0xf3, 0x72, 0xfe, 0x94, 0xf8, 0x2b,
            0xa5, 0x4f, 0xf5, 0x3a, 0x5f, 0x1d, 0x36, 0xf1,
            0x51, 0x0e, 0x52, 0x7f, 0xad, 0xe6, 0x82, 0xd1,
            0x9b, 0x05, 0x68, 0x8c, 0x2b, 0x3e, 0x6c, 0x1f,
            0x1f, 0x83, 0xd9, 0xab, 0xfb, 0x41, 0xbd, 0x6b,
            0x5b, 0xe0, 0xcd, 0x19, 0x13, 0x7e, 0x21, 0x79,
        ];
        let mut t = [0u64; 8];
        for (i, e) in t.iter_mut().enumerate() {
            *e = load_be(&IV, i * 8);
        }
        State(t)
    }

    /// Adds two states together (mod 2^64) – used after processing a block.
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

    /// Stores the state as big‑endian bytes in the output slice.
    pub(crate) fn store(&self, out: &mut [u8]) {
        for (i, &e) in self.0.iter().enumerate() {
            store_be(out, i * 8, e);
        }
    }

    /// Processes one or more full 128‑byte blocks.
    ///
    /// Returns the number of bytes that were **not** processed (i.e., the
    /// remainder) because the input wasn't a multiple of the block size.
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

/// SHA‑512 hasher with streaming support.
///
/// This is the main public type for computing SHA‑512 digests. It maintains
/// an internal state, a buffer for partial blocks, and the total length of
/// processed data.
///
/// # Example
/// ```
/// use libvctrl_sha512::Hash;
/// let mut hasher = Hash::new();
/// hasher.update(b"Hello, world!");
/// let digest = hasher.finalize();
/// ```
#[derive(Copy, Clone)]
pub struct Hash {
    /// Internal hash state (8 64‑bit words).
    pub(crate) state: State,
    /// Buffer for incomplete data (up to 128 bytes).
    pub(crate) w: [u8; 128],
    /// Number of valid bytes in the buffer (0..128).
    pub(crate) r: usize,
    /// Total number of bytes processed so far.
    pub(crate) len: usize,
}

impl Hash {
    /// Creates a new SHA‑512 hasher with the initial state (IV).
    ///
    /// # Example
    /// ```
    /// use libvctrl_sha512::Hash;
    /// let hasher = Hash::new();
    /// ```
    #[inline]
    pub fn new() -> Self {
        Hash {
            state: State::new(),
            r: 0,
            w: [0u8; 128],
            len: 0,
        }
    }

    /// Internal update function used by the digest trait.
    #[doc(hidden)]
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

    /// Absorbs more data into the hash state.
    ///
    /// This method can be called multiple times to process a message in chunks.
    ///
    /// # Arguments
    /// * `input` – The chunk of data to hash (any `AsRef<[u8]>`).
    ///
    /// # Example
    /// ```
    /// use libvctrl_sha512::Hash;
    /// let mut hasher = Hash::new();
    /// hasher.update(b"first part ");
    /// hasher.update(b"second part");
    /// let digest = hasher.finalize();
    /// ```
    #[inline]
    pub fn update<T: AsRef<[u8]>>(&mut self, input: T) {
        self._update(input)
    }

    /// Finalizes the hash computation and returns the 64‑byte digest.
    ///
    /// This consumes the hasher; after calling `finalize`, the instance cannot
    /// be reused. The padding and length bits are appended automatically.
    ///
    /// # Returns
    /// A `[u8; 64]` array containing the SHA‑512 hash.
    ///
    /// # Example
    /// ```
    /// use libvctrl_sha512::Hash;
    /// let digest = Hash::new().finalize(); // hash of empty input
    /// assert_eq!(digest.len(), 64);
    /// ```
    #[inline]
    pub fn finalize(mut self) -> [u8; 64] {
        let mut padded = [0u8; 256];
        padded[..self.r].copy_from_slice(&self.w[..self.r]);
        padded[self.r] = 0x80;
        let r = if self.r < 112 { 128 } else { 256 };
        let bits = self.len * 8;
        for i in 0..8 {
            padded[r - 8 + i] = (bits as u64 >> (56 - i * 8)) as u8;
        }
        self.state.blocks(&padded[..r]);
        let mut out = [0u8; 64];
        self.state.store(&mut out);
        out
    }

    /// One‑shot SHA‑512 hash of the given input.
    ///
    /// This is a convenience function that creates a new hasher, updates it with
    /// the input, and finalizes it in one step.
    ///
    /// # Arguments
    /// * `input` – The data to hash.
    ///
    /// # Returns
    /// The 64‑byte hash digest.
    ///
    /// # Example
    /// ```
    /// use libvctrl_sha512::Hash;
    /// let digest = Hash::hash(b"hello");
    /// assert_eq!(digest.len(), 64);
    /// ```
    #[inline]
    pub fn hash<T: AsRef<[u8]>>(input: T) -> [u8; 64] {
        let mut h = Self::new();
        h.update(input);
        h.finalize()
    }

    /// Verifies that the computed hash matches an expected digest.
    ///
    /// This is a convenience method that finalizes the hash and compares the
    /// result against the expected value in **constant time**, mitigating
    /// timing side‑channel attacks.
    ///
    /// # Arguments
    /// * `expected` – The expected 64‑byte digest.
    ///
    /// # Returns
    /// `true` if the digests match, `false` otherwise.
    ///
    /// # Example
    /// ```
    /// use libvctrl_sha512::Hash;
    /// let digest = Hash::hash(b"data");
    /// // Later...
    /// let recomputed = Hash::hash(b"data");
    /// assert!(recomputed.verify(&digest));
    /// // Tampering:
    /// let wrong = [0u8; 64];
    /// assert!(!recomputed.verify(&wrong));
    /// ```
    #[inline]
    pub fn verify(self, expected: &[u8; 64]) -> bool {
        let out = self.finalize();
        verify(&out, expected)
    }
}

impl Default for Hash {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
