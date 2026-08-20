#![allow(clippy::indexing_slicing)]
#![allow(clippy::arithmetic_side_effects)]

use crate::sha512::{Hash as Sha512Hash, State};
use crate::utils::load_be;

#[inline]
fn new_state() -> State {
    const IV: [u8; 64] = [
        0xcb, 0xbb, 0x9d, 0x5d, 0xc1, 0x05, 0x9e, 0xd8, 0x62, 0x9a, 0x29, 0x2a, 0x36, 0x7c, 0xd5,
        0x07, 0x91, 0x59, 0x01, 0x5a, 0x30, 0x70, 0xdd, 0x17, 0x15, 0x2f, 0xec, 0xd8, 0xf7, 0x0e,
        0x59, 0x39, 0x67, 0x33, 0x26, 0x67, 0xff, 0xc0, 0x0b, 0x31, 0x8e, 0xb4, 0x4a, 0x87, 0x68,
        0x58, 0x15, 0x11, 0xdb, 0x0c, 0x2e, 0x0d, 0x64, 0xf9, 0x8f, 0xa7, 0x47, 0xb5, 0x48, 0x1d,
        0xbe, 0xfa, 0x4f, 0xa4,
    ];
    let mut state = [0_u64; 8];
    for (index, word) in state.iter_mut().enumerate() {
        *word = load_be(&IV, index * 8);
    }
    State(state)
}

#[derive(Clone)]
pub struct Hash(Sha512Hash);

impl core::fmt::Debug for Hash {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Hash")
    }
}

impl Hash {
    #[must_use]
    pub fn new() -> Self {
        Self(Sha512Hash {
            state: new_state(),
            r: 0,
            w: [0_u8; 128],
            len: 0,
        })
    }

    pub(crate) fn update_inner<T: AsRef<[u8]>>(&mut self, input: T) {
        self.0.update_inner(input);
    }

    pub fn update<T: AsRef<[u8]>>(&mut self, input: T) {
        self.update_inner(input);
    }

    #[must_use]
    pub fn finalize(self) -> [u8; 48] {
        let mut out = [0_u8; 48];
        let full = zeroize::Zeroizing::new(self.0.finalize());
        out.copy_from_slice(&full[..48]);
        out
    }

    #[must_use]
    pub fn hash<T: AsRef<[u8]>>(input: T) -> [u8; 48] {
        let mut hasher = Self::new();
        hasher.update(input);
        hasher.finalize()
    }

    pub fn zeroize(&mut self) {
        zeroize::Zeroize::zeroize(self);
    }
}

impl zeroize::Zeroize for Hash {
    fn zeroize(&mut self) {
        zeroize::Zeroize::zeroize(&mut self.0);
    }
}

impl Default for Hash {
    fn default() -> Self {
        Self::new()
    }
}

impl_hmac!(Hash, 48, 128);
impl_hkdf!(Hash, 48, 128);
