use crate::sha512::Hash;
use crate::utils::verify;

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
    #[inline]
    pub fn mac<T: AsRef<[u8]>, U: AsRef<[u8]>>(input: T, k: U) -> [u8; 64] {
        let input = input.as_ref();
        let k = k.as_ref();
        let mut hk = [0u8; 64];
        let k2 = if k.len() > 128 {
            hk.copy_from_slice(&Hash::hash(k));
            &hk
        } else {
            k
        };
        let mut ih = Hash::new();
        let mut padded = [0x36; 128];
        for (p, &k) in padded.iter_mut().zip(k2.iter()) {
            *p ^= k;
        }
        ih.update(&padded[..]);
        ih.update(input);

        let mut oh = Hash::new();
        padded = [0x5c; 128];
        for (p, &k) in padded.iter_mut().zip(k2.iter()) {
            *p ^= k;
        }
        oh.update(&padded[..]);
        oh.update(&ih.finalize()[..]);
        let mac = oh.finalize();

        hk.fill(0);
        padded.fill(0);
        mac
    }

    pub fn new(k: impl AsRef<[u8]>) -> Self {
        let k = k.as_ref();
        let mut hk = [0u8; 64];
        let k2 = if k.len() > 128 {
            hk.copy_from_slice(&Hash::hash(k));
            &hk
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

    pub fn finalize(mut self) -> [u8; 64] {
        for p in self.padded.iter_mut() {
            *p ^= 0x6a;
        }
        let mut oh = Hash::new();
        oh.update(&self.padded[..]);
        oh.update(self.ih.finalize());
        oh.finalize()
    }

    #[inline]
    pub fn finalize_verify(self, expected: &[u8; 64]) -> bool {
        let out = self.finalize();
        verify(&out, expected)
    }

    #[inline]
    pub fn verify<T: AsRef<[u8]>, U: AsRef<[u8]>>(input: T, k: U, expected: &[u8; 64]) -> bool {
        let mac = Self::mac(input, k);
        verify(&mac, expected)
    }
}
