#![allow(clippy::inline_always)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::arithmetic_side_effects)]

use crate::utils::{load_be, store_be, verify};

struct W([u64; 16]);

#[derive(Copy, Clone)]
pub(crate) struct State(pub(crate) [u64; 8]);

impl W {
    fn new(input: &[u8]) -> Self {
        let mut words = [0_u64; 16];
        for (index, word) in words.iter_mut().enumerate() {
            *word = load_be(input, index * 8);
        }
        Self(words)
    }

    #[inline(always)]
    const fn ch(x: u64, y: u64, z: u64) -> u64 {
        (x & y) ^ (!x & z)
    }

    #[inline(always)]
    const fn maj(x: u64, y: u64, z: u64) -> u64 {
        (x & y) ^ (x & z) ^ (y & z)
    }

    #[inline(always)]
    const fn big_sigma0(x: u64) -> u64 {
        x.rotate_right(28) ^ x.rotate_right(34) ^ x.rotate_right(39)
    }

    #[inline(always)]
    const fn big_sigma1(x: u64) -> u64 {
        x.rotate_right(14) ^ x.rotate_right(18) ^ x.rotate_right(41)
    }

    #[inline(always)]
    const fn small_sigma0(x: u64) -> u64 {
        x.rotate_right(1) ^ x.rotate_right(8) ^ (x >> 7)
    }

    #[inline(always)]
    const fn small_sigma1(x: u64) -> u64 {
        x.rotate_right(19) ^ x.rotate_right(61) ^ (x >> 6)
    }

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
    pub(crate) fn new() -> Self {
        const IV: [u8; 64] = [
            0x6a, 0x09, 0xe6, 0x67, 0xf3, 0xbc, 0xc9, 0x08, 0xbb, 0x67, 0xae, 0x85, 0x84, 0xca,
            0xa7, 0x3b, 0x3c, 0x6e, 0xf3, 0x72, 0xfe, 0x94, 0xf8, 0x2b, 0xa5, 0x4f, 0xf5, 0x3a,
            0x5f, 0x1d, 0x36, 0xf1, 0x51, 0x0e, 0x52, 0x7f, 0xad, 0xe6, 0x82, 0xd1, 0x9b, 0x05,
            0x68, 0x8c, 0x2b, 0x3e, 0x6c, 0x1f, 0x1f, 0x83, 0xd9, 0xab, 0xfb, 0x41, 0xbd, 0x6b,
            0x5b, 0xe0, 0xcd, 0x19, 0x13, 0x7e, 0x21, 0x79,
        ];
        let mut state = [0_u64; 8];
        for (index, word) in state.iter_mut().enumerate() {
            *word = load_be(&IV, index * 8);
        }
        Self(state)
    }

    #[inline(always)]
    #[allow(clippy::missing_const_for_fn)]
    pub(crate) fn add(&mut self, other: &Self) {
        let self_state = &mut self.0;
        let other_state = &other.0;
        self_state[0] = self_state[0].wrapping_add(other_state[0]);
        self_state[1] = self_state[1].wrapping_add(other_state[1]);
        self_state[2] = self_state[2].wrapping_add(other_state[2]);
        self_state[3] = self_state[3].wrapping_add(other_state[3]);
        self_state[4] = self_state[4].wrapping_add(other_state[4]);
        self_state[5] = self_state[5].wrapping_add(other_state[5]);
        self_state[6] = self_state[6].wrapping_add(other_state[6]);
        self_state[7] = self_state[7].wrapping_add(other_state[7]);
    }

    pub(crate) fn store(&self, out: &mut [u8]) {
        for (index, &word) in self.0.iter().enumerate() {
            store_be(out, index * 8, word);
        }
    }

    pub(crate) fn blocks(&mut self, mut input: &[u8]) -> usize {
        let mut temp = *self;
        let mut inlen = input.len();
        while inlen >= 128 {
            let mut w = W::new(input);
            w.g(&mut temp, 0);
            w.expand();
            w.g(&mut temp, 1);
            w.expand();
            w.g(&mut temp, 2);
            w.expand();
            w.g(&mut temp, 3);
            w.expand();
            w.g(&mut temp, 4);
            temp.add(self);
            self.0 = temp.0;
            input = &input[128..];
            inlen -= 128;
        }
        inlen
    }
}

#[derive(Clone)]
pub struct Hash {
    pub(crate) state: State,
    pub(crate) w: [u8; 128],
    pub(crate) r: usize,
    pub(crate) len: u128,
}

impl core::fmt::Debug for Hash {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Hash")
    }
}

impl zeroize::Zeroize for Hash {
    fn zeroize(&mut self) {
        zeroize::Zeroize::zeroize(&mut self.state.0);
        zeroize::Zeroize::zeroize(&mut self.w);
        zeroize::Zeroize::zeroize(&mut self.r);
        zeroize::Zeroize::zeroize(&mut self.len);
    }
}

impl Drop for Hash {
    fn drop(&mut self) {
        zeroize::Zeroize::zeroize(self);
    }
}

impl Hash {
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: State::new(),
            r: 0,
            w: [0_u8; 128],
            len: 0,
        }
    }

    pub(crate) fn update_inner<T: AsRef<[u8]>>(&mut self, input: T) {
        let input = input.as_ref();
        let mut remaining = input.len();
        self.len += remaining as u128;
        let available = 128 - self.r;
        let take = core::cmp::min(remaining, available);
        self.w[self.r..self.r + take].copy_from_slice(&input[0..take]);
        self.r += take;
        remaining -= take;
        let pos = take;
        if self.r == 128 {
            let _ = self.state.blocks(&self.w);
            self.r = 0;
        }
        if self.r == 0 && remaining > 0 {
            let leftover = self.state.blocks(&input[pos..]);
            if leftover > 0 {
                self.w[..leftover].copy_from_slice(&input[pos + remaining - leftover..]);
                self.r = leftover;
            }
        }
    }

    pub fn update<T: AsRef<[u8]>>(&mut self, input: T) {
        self.update_inner(input);
    }

    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn finalize(mut self) -> [u8; 64] {
        let mut padded = zeroize::Zeroizing::new([0_u8; 256]);
        padded[..self.r].copy_from_slice(&self.w[..self.r]);
        padded[self.r] = 0x80;
        let r = if self.r < 112 { 128 } else { 256 };
        let total_bits: u128 = self.len * 8;
        let high = (total_bits >> 64) as u64;
        let low = total_bits as u64;
        store_be(&mut *padded, r - 16, high);
        store_be(&mut *padded, r - 8, low);

        let _ = self.state.blocks(&padded[..r]);
        let mut out = [0_u8; 64];
        self.state.store(&mut out);
        out
    }

    pub fn hash<T: AsRef<[u8]>>(input: T) -> [u8; 64] {
        let mut hasher = Self::new();
        hasher.update(input);
        hasher.finalize()
    }

    #[must_use]
    pub fn verify(self, expected: &[u8; 64]) -> bool {
        let out = self.finalize();
        verify(&out, expected)
    }

    pub fn zeroize(&mut self) {
        zeroize::Zeroize::zeroize(self);
    }
}

impl Default for Hash {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_empty_vector() {
        let expected: [u8; 64] = [
            0xcf, 0x83, 0xe1, 0x35, 0x7e, 0xef, 0xb8, 0xbd, 0xf1, 0x54, 0x28, 0x50, 0xd6, 0x6d,
            0x80, 0x07, 0xd6, 0x20, 0xe4, 0x05, 0x0b, 0x57, 0x15, 0xdc, 0x83, 0xf4, 0xa9, 0x21,
            0xd3, 0x6c, 0xe9, 0xce, 0x47, 0xd0, 0xd1, 0x3c, 0x5d, 0x85, 0xf2, 0xb0, 0xff, 0x83,
            0x18, 0xd2, 0x87, 0x7e, 0xec, 0x2f, 0x63, 0xb9, 0x31, 0xbd, 0x47, 0x41, 0x7a, 0x81,
            0xa5, 0x38, 0x32, 0x7a, 0xf9, 0x27, 0xda, 0x3e,
        ];
        assert_eq!(Hash::hash(b""), expected);
    }

    #[test]
    fn test_hash_abc_vector() {
        let expected: [u8; 64] = [
            0xdd, 0xaf, 0x35, 0xa1, 0x93, 0x61, 0x7a, 0xba, 0xcc, 0x41, 0x73, 0x49, 0xae, 0x20,
            0x41, 0x31, 0x12, 0xe6, 0xfa, 0x4e, 0x89, 0xa9, 0x7e, 0xa2, 0x0a, 0x9e, 0xee, 0xe6,
            0x4b, 0x55, 0xd3, 0x9a, 0x21, 0x92, 0x99, 0x2a, 0x27, 0x4f, 0xc1, 0xa8, 0x36, 0xba,
            0x3c, 0x23, 0xa3, 0xfe, 0xeb, 0xbd, 0x45, 0x4d, 0x44, 0x23, 0x64, 0x3c, 0xe8, 0x0e,
            0x2a, 0x9a, 0xc9, 0x4f, 0xa5, 0x4c, 0xa4, 0x9f,
        ];
        assert_eq!(Hash::hash(b"abc"), expected);
    }

    #[test]
    fn test_update_multiple_calls_equals_one_shot() {
        let mut hasher = Hash::new();
        hasher.update(b"abc");
        hasher.update(b"def");
        let multi = hasher.finalize();
        let single = Hash::hash(b"abcdef");
        assert_eq!(multi, single);
    }

    #[test]
    fn test_verify_correct_and_incorrect() {
        let expected = Hash::hash(b"abc");

        let mut hasher = Hash::new();
        hasher.update(b"abc");
        assert!(hasher.verify(&expected));

        let mut hasher = Hash::new();
        hasher.update(b"abd");
        assert!(!hasher.verify(&expected));
    }

    #[test]
    fn test_w_new_loads_big_endian_words() {
        let input = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16,
            0x17, 0x18,
        ];
        let w = W::new(&input);
        assert_eq!(w.0[0], 0x0102_0304_0506_0708);
        assert_eq!(w.0[1], 0x1112_1314_1516_1718);
        assert_eq!(w.0[2], 0);
    }

    #[test]
    fn test_w_ch_maj_bitwise_helpers() {
        assert_eq!(W::ch(0b1100, 0b1010, 0b0110), 0b1010);
        assert_eq!(W::maj(0b1100, 0b1010, 0b0110), 0b1110);
    }

    #[test]
    fn test_state_add_merges_state_words() {
        let mut state = State([1, 2, 3, 4, 5, 6, 7, 8]);
        let other = State([10, 20, 30, 40, 50, 60, 70, 80]);
        state.add(&other);
        assert_eq!(state.0, [11, 22, 33, 44, 55, 66, 77, 88]);
    }
}
