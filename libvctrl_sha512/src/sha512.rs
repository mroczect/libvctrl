//! SHA-512 and SHA-384 hash function implementations.
//!
//! # Purpose
//!
//! This module implements the SHA-512 cryptographic hash function as defined in
//! FIPS 180-4, with optional support for SHA-384 when the `sha384` feature is
//! enabled. The implementation is designed for `no_std` environments, requires
//! zero heap allocations, and is written with minimal external dependencies.
//!
//! # Design Rationale
//!
//! The core design revolves around a 1024-bit (128-byte) message block buffer and
//! an 8-element 64-bit state vector. The [`State`] struct is `Copy` and `Clone`,
//! enabling efficient buffering and the Merkle-Damgard construction without
//! dynamic allocation. All arithmetic is done with wrapping operations to avoid
//! undefined behavior on overflow and to match the standard's modular arithmetic.
//!
//! The [`W`] (message schedule) struct encapsulates the 16-word message block
//! expansion and round logic. The inner function `f` uses circular index
//! arithmetic on the state array to avoid explicit variable shuffling and reduce
//! register pressure. This approach was chosen because it allows the compiler to
//! generate highly optimized code for both speed and size (via the `opt_size`
//! feature).
//!
//! # Message Schedule and Compression
//!
//! SHA-512 processes input in 1024-bit (128-byte) blocks. Each block is expanded
//! into an 80-round message schedule. The compression function operates on the
//! 512-bit state (eight 64-bit words) using bitwise majority, choice, and sigma
//! functions. The implementation follows the standard FIPS 180-4 definition.
//!
//! # Features
//!
//! - **`opt_size`**: When enabled, the largest functions (`m` and `f`) are
//!   marked `#[inline(never)]` to reduce code size at the cost of some speed.
//! - **`sha384`**: When enabled, the [`crate::sha384`] module provides a
//!   SHA-384 hasher that shares this module's core compression logic.
//!
//! # Security Considerations
//!
//! The [`Hash::zeroize`] method overwrites all internal state and uses a
//! compiler fence to prevent reordering, helping to clear sensitive data from
//! memory. However, this crate is not suitable for constant-time comparisons on
//! all targets; for `no_std` verification, use [`Hash::verify`] which employs a
//! basic XOR accumulation (and on WASM, an additional hash-based accumulator) to
//! reduce timing side-channel leakage, but it is not a full constant-time
//! implementation.
//!
//! # Performance
//!
//! The implementation avoids heap allocation entirely. The `Hash` struct stores
//! its state and buffer inline, so the total size is about 192 bytes. The
//! compression loop is written to allow the compiler to fully unroll the 80
//! rounds and generate efficient SIMD-like code on modern CPUs. The
//! `opt_size` feature trades speed for smaller code when binary size is more
//! important than throughput.
//!
//! # Examples
//!
//! Computing a SHA-512 hash in one shot:
//!
//! ```
//! # use libvctrl_sha512::Hash;
//! let digest = Hash::hash(b"hello world");
//! assert_eq!(digest.len(), 64);
//! ```
//!
//! Incremental hashing with verification:
//!
//! ```
//! # use libvctrl_sha512::Hash;
//! let expected = Hash::hash(b"verify this");
//! let mut h = Hash::new();
//! h.update(b"verify ");
//! h.update(b"this");
//! assert!(h.verify(&expected));
//! ```
#![allow(clippy::inline_always)]
use crate::utils::{load_be, store_be, verify};

/// Internal message schedule for SHA-512 block compression.
///
/// Holds the 16 64-bit words of the current message block and provides methods
/// for the SHA-512 round functions. This structure is not exposed publicly; it
/// exists solely to encapsulate the message expansion and compression logic
/// within [`State::blocks`].
///
/// # Design
///
/// The message schedule expansion is split into two phases per 16-round cycle:
///
/// 1. Perform 16 rounds using the initial 16 words (method `g` with `s=0`).
/// 2. Expand the schedule ([`expand`]) and perform the next 16 rounds (method `g`
///    with `s=1`).
///
/// This pattern is repeated four times for the full 80 rounds. The expansion
/// uses the standard SHA-512 `sigma0` and `sigma1` functions as defined in FIPS
/// 180-4.
///
/// # Performance Note
///
/// The functions are annotated with `#[inline(always)]` by default, but when
/// the `opt_size` feature is enabled, the larger functions `m` and `f` become
/// `#[inline(never)]` to reduce code size at the expense of some speed.
struct W([u64; 16]);

/// Internal state vector for SHA-512 (8 × 64 bits).
///
/// This struct is `Copy` and `Clone`, which simplifies the Merkle-Damgard
/// construction: the current working state is copied before processing a block,
/// allowing the previous state to be added back after compression without
/// extra bookkeeping.
///
/// # Module Visibility
///
/// `pub(crate)` because other modules in this crate (e.g., [`crate::sha384`])
/// reuse the same state structure with a different initialization vector.
#[derive(Copy, Clone)]
pub(crate) struct State(pub(crate) [u64; 8]);

impl W {
    /// Loads a 128-byte block from `input` and converts it to 16 big-endian 64-bit words.
    ///
    /// # Purpose
    ///
    /// This is the entry point for processing a message block. It reads the
    /// raw bytes, converts each 8-byte chunk from big-endian to a `u64`, and
    /// stores the result in the message schedule.
    ///
    /// # How It Works
    ///
    /// The function iterates over 16 chunks, each of 8 bytes, using
    /// [`load_be`]. The input slice must be at least 128 bytes long.
    fn new(input: &[u8]) -> Self {
        let mut words = [0u64; 16];
        for (i, e) in words.iter_mut().enumerate() {
            *e = load_be(input, i * 8);
        }
        Self(words)
    }

    /// SHA-512 choose function: `(x & y) ^ (!x & z)`.
    ///
    /// # Purpose
    ///
    /// Used in the compression round. It selects bits from `y` where `x` is
    /// set, and from `z` where `x` is clear. This is a standard bitwise
    /// operation in SHA-512.
    #[inline(always)]
    const fn ch(x: u64, y: u64, z: u64) -> u64 {
        (x & y) ^ (!x & z)
    }

    /// SHA-512 majority function: `(x & y) ^ (x & z) ^ (y & z)`.
    ///
    /// # Purpose
    ///
    /// Returns the bitwise majority of three input words. Used in the
    /// compression round to combine state variables.
    #[inline(always)]
    const fn maj(x: u64, y: u64, z: u64) -> u64 {
        (x & y) ^ (x & z) ^ (y & z)
    }

    /// SHA-512 upper-case sigma 0: `ROTR28(x) ^ ROTR34(x) ^ ROTR39(x)`.
    ///
    /// # Purpose
    ///
    /// This is one of the two large sigma functions used in the compression
    /// round. It provides diffusion by mixing bits across three different
    /// right rotations.
    #[inline(always)]
    const fn big_sigma0(x: u64) -> u64 {
        x.rotate_right(28) ^ x.rotate_right(34) ^ x.rotate_right(39)
    }

    /// SHA-512 upper-case sigma 1: `ROTR14(x) ^ ROTR18(x) ^ ROTR41(x)`.
    ///
    /// # Purpose
    ///
    /// This is the second large sigma function used in the compression round.
    /// It complements [`big_sigma0`](Self::big_sigma0) for stronger diffusion.
    #[inline(always)]
    const fn big_sigma1(x: u64) -> u64 {
        x.rotate_right(14) ^ x.rotate_right(18) ^ x.rotate_right(41)
    }

    /// SHA-512 lower-case sigma 0: `ROTR1(x) ^ ROTR8(x) ^ SHR7(x)`.
    ///
    /// # Purpose
    ///
    /// Used in the message schedule expansion to mix bits from previous words.
    #[inline(always)]
    const fn small_sigma0(x: u64) -> u64 {
        x.rotate_right(1) ^ x.rotate_right(8) ^ (x >> 7)
    }

    /// SHA-512 lower-case sigma 1: `ROTR19(x) ^ ROTR61(x) ^ SHR6(x)`.
    ///
    /// # Purpose
    ///
    /// Used in the message schedule expansion. It provides additional
    /// diffusion when generating schedule words 16 through 79.
    #[inline(always)]
    const fn small_sigma1(x: u64) -> u64 {
        x.rotate_right(19) ^ x.rotate_right(61) ^ (x >> 6)
    }

    /// Performs message schedule expansion step:
    /// `W[a] += σ1(W[b]) + W[c] + σ0(W[d])`.
    ///
    /// # Purpose
    ///
    /// This is one step of the SHA-512 message schedule expansion. It combines
    /// four previous words to compute a new schedule word. The method is
    /// called repeatedly by [`expand`] to generate the full 80-word schedule.
    ///
    /// # Inline Control
    ///
    /// When `opt_size` is enabled, this function is marked `#[inline(never)]`
    /// to reduce code bloat from the many call sites in [`expand`].
    #[cfg_attr(feature = "opt_size", inline(never))]
    #[cfg_attr(not(feature = "opt_size"), inline(always))]
    #[allow(clippy::many_single_char_names, clippy::missing_const_for_fn)]
    fn m(&mut self, dest: usize, src_b: usize, src_c: usize, src_d: usize) {
        let words = &mut self.0;
        words[dest] = words[dest]
            .wrapping_add(Self::small_sigma1(words[src_b]))
            .wrapping_add(words[src_c])
            .wrapping_add(Self::small_sigma0(words[src_d]));
    }

    /// Expands the initial 16-word message schedule into the full 80-word
    /// schedule by computing the remaining 64 words in-place.
    ///
    /// # Purpose
    ///
    /// After the first 16 words are loaded from the input block, this method
    /// generates the next 64 words using the SHA-512 expansion recurrence.
    /// The generated words are stored in-place in the same array.
    ///
    /// # How It Works
    ///
    /// The method calls [`m`](Self::m) 16 times with the appropriate indices
    /// to compute the next 16 words from the previous ones. Because the
    /// expansion is circular in the first 16 positions, the code carefully
    /// overwrites values that are no longer needed.
    #[inline]
    fn expand(&mut self) {
        self.m(0, 14, 9, 1);
        self.m(1, 15, 10, 2);
        self.m(2, 0, 11, 3);
        self.m(3, 1, 12, 4);
        self.m(4, 2, 13, 5);
        self.m(5, 3, 14, 6);
        self.m(6, 4, 15, 7);
        self.m(7, 5, 0, 8);
        self.m(8, 6, 1, 9);
        self.m(9, 7, 2, 10);
        self.m(10, 8, 3, 11);
        self.m(11, 9, 4, 12);
        self.m(12, 10, 5, 13);
        self.m(13, 11, 6, 14);
        self.m(14, 12, 7, 15);
        self.m(15, 13, 8, 0);
    }

    /// Applies a single SHA-512 round transformation using the i-th message
    /// word and round constant `k`.
    ///
    /// # Purpose
    ///
    /// This is the core of the SHA-512 compression function. It mixes one
    /// message word and one round constant into the state using the standard
    /// SHA-512 round operations. The state array is accessed with circular
    /// offsets `(16 - i + x) & 7` to simulate the standard working variable
    /// rotation without explicitly shifting all eight registers. This reduces
    /// register pressure and improves instruction-level parallelism.
    ///
    /// # Inline Control
    ///
    /// Same as [`m`](Self::m): conditionally `inline(never)` for `opt_size`.
    #[cfg_attr(feature = "opt_size", inline(never))]
    #[cfg_attr(not(feature = "opt_size"), inline(always))]
    #[allow(clippy::missing_const_for_fn)]
    fn f(&self, state: &mut State, i: usize, k: u64) {
        let t = &mut state.0;
        t[(16 - i + 7) & 7] = t[(16 - i + 7) & 7]
            .wrapping_add(Self::big_sigma1(t[(16 - i + 4) & 7]))
            .wrapping_add(Self::ch(
                t[(16 - i + 4) & 7],
                t[(16 - i + 5) & 7],
                t[(16 - i + 6) & 7],
            ))
            .wrapping_add(k)
            .wrapping_add(self.0[i]);
        t[(16 - i + 3) & 7] = t[(16 - i + 3) & 7].wrapping_add(t[(16 - i + 7) & 7]);
        t[(16 - i + 7) & 7] = t[(16 - i + 7) & 7]
            .wrapping_add(Self::big_sigma0(t[(16 - i) & 7]))
            .wrapping_add(Self::maj(
                t[(16 - i) & 7],
                t[(16 - i + 1) & 7],
                t[(16 - i + 2) & 7],
            ));
    }

    /// Executes 16 consecutive rounds of SHA-512 starting at round `s*16`.
    ///
    /// # Purpose
    ///
    /// The 80 round constants are stored as a single constant array and
    /// sliced by the starting round index. This avoids per-round constant
    /// loads and allows the compiler to fully unroll the 16 calls to
    /// [`f`](Self::f).
    ///
    /// # How It Works
    ///
    /// The method calls [`f`](Self::f) 16 times, once for each word in the
    /// current message schedule segment, with the appropriate round constant
    /// from the sliced array. The `s` parameter selects the starting segment:
    /// 0, 1, 2, 3, or 4.
    #[allow(clippy::unreadable_literal)]
    fn g(&self, state: &mut State, s: usize) {
        const ROUND_CONSTANTS: [u64; 80] = [
            0x428a_2f98_d728_ae22,
            0x7137_4491_23ef_65cd,
            0xb5c0_fbcf_ec4d_3b2f,
            0xe9b5_dba5_8189_dbbc,
            0x3956_c25b_f348_b538,
            0x59f1_11f1_b605_d019,
            0x923f_82a4_af19_4f9b,
            0xab1c_5ed5_da6d_8118,
            0xd807_aa98_a303_0242,
            0x1283_5b01_4570_6fbe,
            0x2431_85be_4ee4_b28c,
            0x550c_7dc3_d5ff_b4e2,
            0x72be_5d74_f27b_896f,
            0x80de_b1fe_3b16_96b1,
            0x9bdc_06a7_25c7_1235,
            0xc19b_f174_cf69_2694,
            0xe49b_69c1_9ef1_4ad2,
            0xefbe_4786_384f_25e3,
            0x0fc1_9dc6_8b8c_d5b5,
            0x240c_a1cc_77ac_9c65,
            0x2de9_2c6f_592b_0275,
            0x4a74_84aa_6ea6_e483,
            0x5cb0_a9dc_bd41_fbd4,
            0x76f9_88da_8311_53b5,
            0x983e_5152_ee66_dfab,
            0xa831_c66d_2db4_3210,
            0xb003_27c8_98fb_213f,
            0xbf59_7fc7_beef_0ee4,
            0xc6e0_0bf3_3da8_8fc2,
            0xd5a7_9147_930a_a725,
            0x06ca_6351_e003_826f,
            0x1429_2967_0a0e_6e70,
            0x27b7_0a85_46d2_2ffc,
            0x2e1b_2138_5c26_c926,
            0x4d2c_6dfc_5ac4_2aed,
            0x5338_0d13_9d95_b3df,
            0x650a_7354_8baf_63de,
            0x766a_0abb_3c77_b2a8,
            0x81c2_c92e_47ed_aee6,
            0x9272_2c85_1482_353b,
            0xa2bf_e8a1_4cf1_0364,
            0xa81a_664b_bc42_3001,
            0xc24b_8b70_d0f8_9791,
            0xc76c_51a3_0654_be30,
            0xd192_e819_d6ef_5218,
            0xd699_0624_5565_a910,
            0xf40e_3585_5771_202a,
            0x106a_a070_32bb_d1b8,
            0x19a4_c116_b8d2_d0c8,
            0x1e37_6c08_5141_ab53,
            0x2748_774c_df8e_eb99,
            0x34b0_bcb5_e19b_48a8,
            0x391c_0cb3_c5c9_5a63,
            0x4ed8_aa4a_e341_8acb,
            0x5b9c_ca4f_7763_e373,
            0x682e_6ff3_d6b2_b8a3,
            0x748f_82ee_5def_b2fc,
            0x78a5_636f_4317_2f60,
            0x84c8_7814_a1f0_ab72,
            0x8cc7_0208_1a64_39ec,
            0x90be_fffa_2363_1e28,
            0xa450_6ceb_de82_bde9,
            0xbef9_a3f7_b2c6_7915,
            0xc671_78f2_e372_532b,
            0xca27_3ece_ea26_619c,
            0xd186_b8c7_21c0_c207,
            0xeada_7dd6_cde0_eb1e,
            0xf57d_4f7f_ee6e_d178,
            0x06f0_67aa_7217_6fba,
            0x0a63_7dc5_a2c8_98a6,
            0x113f_9804_bef9_0dae,
            0x1b71_0b35_131c_471b,
            0x28db_77f5_2304_7d84,
            0x32ca_ab7b_40c7_2493,
            0x3c9e_be0a_15c9_bebc,
            0x431d_67c4_9c10_0d4c,
            0x4cc5_d4be_cb3e_42b6,
            0x597f_299c_fc65_7e2a,
            0x5fcb_6fab_3ad6_faec,
            0x6c44_198c_4a47_5817,
        ];
        let rc = &ROUND_CONSTANTS[s * 16..];
        self.f(state, 0, rc[0]);
        self.f(state, 1, rc[1]);
        self.f(state, 2, rc[2]);
        self.f(state, 3, rc[3]);
        self.f(state, 4, rc[4]);
        self.f(state, 5, rc[5]);
        self.f(state, 6, rc[6]);
        self.f(state, 7, rc[7]);
        self.f(state, 8, rc[8]);
        self.f(state, 9, rc[9]);
        self.f(state, 10, rc[10]);
        self.f(state, 11, rc[11]);
        self.f(state, 12, rc[12]);
        self.f(state, 13, rc[13]);
        self.f(state, 14, rc[14]);
        self.f(state, 15, rc[15]);
    }
}

impl State {
    /// Creates the SHA-512 initial state from the standard FIPS 180-4 IV.
    ///
    /// The IV is the first 64 bytes of the fractional parts of the square roots
    /// of the first eight primes, stored as eight big-endian 64-bit words.
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
        Self(t)
    }

    /// Adds another [`State`] to this one element-wise with wrapping addition.
    ///
    /// Used in the Merkle-Damgard construction to fold the working state back
    /// into the digest state after processing a block.
    #[inline(always)]
    #[allow(clippy::missing_const_for_fn)] // cannot be const because of &mut self
    pub(crate) fn add(&mut self, x: &Self) {
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

    /// Writes this state as 64 big-endian bytes into `out`.
    ///
    /// The output buffer must have at least 64 bytes starting at index 0.
    pub(crate) fn store(&self, out: &mut [u8]) {
        for (i, &e) in self.0.iter().enumerate() {
            store_be(out, i * 8, e);
        }
    }

    /// Processes complete 128-byte blocks from `input`, returning the number of
    /// remaining bytes (always less than 128).
    ///
    /// This is the main compression loop. It copies the current state into a
    /// working variable `t`, processes full blocks by constructing a [`W`],
    /// running the 80 rounds, and adding `t` back to `self`. The loop stops
    /// when fewer than 128 bytes remain.
    pub(crate) fn blocks(&mut self, mut input: &[u8]) -> usize {
        let mut t = *self;
        let mut inlen = input.len();
        while inlen >= 128 {
            let mut w = W::new(input);
            w.g(&mut t, 0);
            w.expand();
            w.g(&mut t, 1);
            w.expand();
            w.g(&mut t, 2);
            w.expand();
            w.g(&mut t, 3);
            w.expand();
            w.g(&mut t, 4);
            t.add(self);
            self.0 = t.0;
            input = &input[128..];
            inlen -= 128;
        }
        inlen
    }
}

/// SHA-512 hasher state.
///
/// This structure maintains the internal buffer, message length, and digest
/// state needed to compute a SHA-512 hash incrementally. It implements the
/// standard Merkle-Damgard construction: input data is accumulated in a
/// 128-byte block buffer; each full block is compressed via the SHA-512
/// compression function.
///
/// # Examples
///
/// Incremental hashing:
///
/// ```
/// # use libvctrl_sha512::Hash;
/// let mut hasher = Hash::new();
/// hasher.update(b"hello ");
/// hasher.update(b"world");
/// let result = hasher.finalize();
/// assert_eq!(result.len(), 64);
/// ```
///
/// One-shot hashing:
///
/// ```
/// # use libvctrl_sha512::Hash;
/// let result = Hash::hash(b"test");
/// assert_eq!(result.len(), 64);
/// ```
///
/// Verifying a known hash:
///
/// ```
/// # use libvctrl_sha512::Hash;
/// let expected = Hash::hash(b"data");
/// let mut h = Hash::new();
/// h.update(b"data");
/// assert!(h.verify(&expected));
/// ```
#[derive(Clone)]
pub struct Hash {
    pub(crate) state: State,
    pub(crate) w: [u8; 128],
    pub(crate) r: usize,
    pub(crate) len: u128,
}

impl Hash {
    /// Creates a new SHA-512 hasher with the default initial state.
    ///
    /// This initializes the internal state to the FIPS 180-4 initial vector,
    /// resets the buffer index and message length to zero.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_sha512::Hash;
    /// let hasher = Hash::new();
    /// // Ready to accept data via `update`.
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: State::new(),
            r: 0,
            w: [0u8; 128],
            len: 0,
        }
    }

    /// Internal incremental update without visibility restriction.
    ///
    /// This is the same as [`update`] but is `pub(crate)` to allow the
    /// [`crate::sha384`] wrapper to forward calls without an extra public method.
    pub(crate) fn update_inner<T: AsRef<[u8]>>(&mut self, input: T) {
        let input = input.as_ref();
        let mut n = input.len();
        self.len += n as u128;
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

    /// Processes input data, updating the hash state.
    ///
    /// Data is buffered in 128-byte blocks. When a full block is accumulated,
    /// the internal compression function is invoked. This method can be called
    /// multiple times to feed large inputs piecewise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_sha512::Hash;
    /// let mut h = Hash::new();
    /// h.update(b"abc");
    /// h.update(b"def");
    /// ```
    pub fn update<T: AsRef<[u8]>>(&mut self, input: T) {
        self.update_inner(input);
    }

    /// Finalizes the hash computation and returns the 64-byte digest.
    ///
    /// This method applies the SHA-512 padding (a `1` bit followed by zeros,
    /// then the 128-bit message length in big-endian), processes the final
    /// block(s), and outputs the digest. The internal state is consumed (`self`
    /// is moved).
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_sha512::Hash;
    /// let mut h = Hash::new();
    /// h.update(b"finalize example");
    /// let digest = h.finalize();
    /// assert_eq!(digest.len(), 64);
    /// ```
    #[must_use]
    pub fn finalize(mut self) -> [u8; 64] {
        let mut padded = [0u8; 256];
        padded[..self.r].copy_from_slice(&self.w[..self.r]);
        padded[self.r] = 0x80;
        let r = if self.r < 112 { 128 } else { 256 };
        let total_bits: u128 = self.len * 8;
        let high = (total_bits >> 64) as u64;
        #[allow(clippy::cast_possible_truncation)]
        let low = total_bits as u64;
        store_be(&mut padded, r - 16, high);
        store_be(&mut padded, r - 8, low);

        self.state.blocks(&padded[..r]);
        let mut out = [0u8; 64];
        self.state.store(&mut out);
        out
    }

    /// One-shot SHA-512 hash computation.
    ///
    /// This is a convenience method that creates a new hasher, feeds it the
    /// provided input, finalizes, and returns the 64-byte digest.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_sha512::Hash;
    /// let digest = Hash::hash(b"shortcut");
    /// assert_eq!(digest.len(), 64);
    /// ```
    pub fn hash<T: AsRef<[u8]>>(input: T) -> [u8; 64] {
        let mut h = Self::new();
        h.update(input);
        h.finalize()
    }

    /// Verifies the hash of the currently buffered data matches an expected value.
    ///
    /// This finalizes the hasher, computing the digest, and compares it with
    /// `expected` using a non-short-circuiting comparison that helps mitigate
    /// timing side-channels (see [`crate::utils::verify`] for details). Returns
    /// `true` if the digests match.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_sha512::Hash;
    /// let expected = Hash::hash(b"verify me");
    /// let mut h = Hash::new();
    /// h.update(b"verify me");
    /// assert!(h.verify(&expected));
    /// ```
    #[must_use]
    pub fn verify(self, expected: &[u8; 64]) -> bool {
        let out = self.finalize();
        verify(&out, expected)
    }

    /// Overwrites the entire internal state with zeros and inserts a compiler fence.
    ///
    /// This is a best-effort measure to clear sensitive data from memory. The
    /// compiler fence prevents the optimizer from reordering or removing the
    /// zeroing operations. Note: on some targets, this does not guarantee
    /// complete erasure due to possible register spilling; use proper OS-level
    /// memory clearing for high-security contexts.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_sha512::Hash;
    /// let mut h = Hash::new();
    /// h.update(b"secret");
    /// h.zeroize();
    /// // The internal state is now cleared.
    /// ```
    pub fn zeroize(&mut self) {
        self.state.0.fill(0);
        self.w.fill(0);
        self.r = 0;
        self.len = 0;
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    }
}

impl Default for Hash {
    /// Returns a new hasher with the default state, identical to [`Hash::new`].
    fn default() -> Self {
        Self::new()
    }
}
