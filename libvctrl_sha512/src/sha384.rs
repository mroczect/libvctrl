use crate::sha512::{Hash as Sha512Hash, State};
use crate::utils::load_be;
use crate::utils::verify;

#[inline]
fn new_state() -> State {
    const IV: [u8; 64] = [
        0xcb, 0xbb, 0x9d, 0x5d, 0xc1, 0x05, 0x9e, 0xd8, 0x62, 0x9a, 0x29, 0x2a, 0x36, 0x7c, 0xd5,
        0x07, 0x91, 0x59, 0x01, 0x5a, 0x30, 0x70, 0xdd, 0x17, 0x15, 0x2f, 0xec, 0xd8, 0xf7, 0x0e,
        0x59, 0x39, 0x67, 0x33, 0x26, 0x67, 0xff, 0xc0, 0x0b, 0x31, 0x8e, 0xb4, 0x4a, 0x87, 0x68,
        0x58, 0x15, 0x11, 0xdb, 0x0c, 0x2e, 0x0d, 0x64, 0xf9, 0x8f, 0xa7, 0x47, 0xb5, 0x48, 0x1d,
        0xbe, 0xfa, 0x4f, 0xa4,
    ];
    let mut t = [0u64; 8];
    for (i, e) in t.iter_mut().enumerate() {
        *e = load_be(&IV, i * 8);
    }
    State(t)
}

#[derive(Copy, Clone)]
pub struct Hash(Sha512Hash);

impl Hash {
    pub fn new() -> Self {
        Hash(Sha512Hash {
            state: new_state(),
            r: 0,
            w: [0u8; 128],
            len: 0,
        })
    }

    pub(crate) fn _update<T: AsRef<[u8]>>(&mut self, input: T) {
        self.0.update(input)
    }

    pub fn update<T: AsRef<[u8]>>(&mut self, input: T) {
        self._update(input)
    }

    pub fn finalize(self) -> [u8; 48] {
        let mut out = [0u8; 48];
        out.copy_from_slice(&self.0.finalize()[..48]);
        out
    }

    pub fn hash<T: AsRef<[u8]>>(input: T) -> [u8; 48] {
        let mut h = Self::new();
        h.update(input);
        h.finalize()
    }
}

impl Default for Hash {
    fn default() -> Self {
        Self::new()
    }
}

pub struct HMAC {
    ih: Hash,
    padded: [u8; 128],
}

impl Drop for HMAC {
    fn drop(&mut self) {
        self.padded.fill(0);
    }
}

impl HMAC {
    pub fn mac<T: AsRef<[u8]>, U: AsRef<[u8]>>(input: T, k: U) -> [u8; 48] {
        let mut hmac = Self::new(k);
        hmac.update(input);
        hmac.finalize()
    }

    pub fn new(k: impl AsRef<[u8]>) -> Self {
        let k = k.as_ref();
        let mut hk = [0u8; 48];
        let k2 = if k.len() > 128 {
            hk.copy_from_slice(&Hash::hash(k));
            &hk[..]
        } else {
            k
        };
        let mut padded = [0x36; 128];
        for (p, &k) in padded.iter_mut().zip(k2.iter()) {
            *p ^= k;
        }
        let mut ih = Hash::new();
        ih.update(&padded[..]);
        HMAC { ih, padded }
    }

    pub fn update(&mut self, input: impl AsRef<[u8]>) {
        self.ih.update(input);
    }

    pub fn finalize(mut self) -> [u8; 48] {
        for p in self.padded.iter_mut() {
            *p ^= 0x6a;
        }
        let mut oh = Hash::new();
        oh.update(&self.padded[..]);
        oh.update(&self.ih.finalize()[..]);
        oh.finalize()
    }

    #[inline]
    pub fn finalize_verify(self, expected: &[u8; 48]) -> bool {
        let out = self.finalize();
        verify(&out, expected)
    }

    #[inline]
    pub fn verify<T: AsRef<[u8]>, U: AsRef<[u8]>>(input: T, k: U, expected: &[u8; 48]) -> bool {
        let mac = Self::mac(input, k);
        verify(&mac, expected)
    }
}

pub struct HKDF;

impl HKDF {
    #[inline]
    pub fn extract(salt: impl AsRef<[u8]>, ikm: impl AsRef<[u8]>) -> [u8; 48] {
        HMAC::mac(ikm, salt)
    }

    #[inline]
    pub fn expand(out: &mut [u8], prk: impl AsRef<[u8]>, info: impl AsRef<[u8]>) {
        assert_eq!(prk.as_ref().len(), 48, "HKDF-SHA384 expects a 48‑byte PRK");
        let info = info.as_ref();
        let mut counter: u8 = 1;
        assert!(
            out.len() < 0xff * 48,
            "Requested output exceeds RFC 5869 limit"
        );
        let mut i = 0;
        while i < out.len() {
            let mut hmac = HMAC::new(&prk);
            if i != 0 {
                hmac.update(&out[i - 48..][..48]);
            }
            hmac.update(info);
            hmac.update([counter]);
            let left = core::cmp::min(48, out.len() - i);
            out[i..][..left].copy_from_slice(&hmac.finalize()[..left]);
            counter += 1;
            i += 48;
        }
    }
}
