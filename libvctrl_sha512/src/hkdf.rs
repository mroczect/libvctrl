use crate::hmac::HMAC;

pub struct HKDF;

impl HKDF {
    #[inline]
    pub fn extract(salt: impl AsRef<[u8]>, ikm: impl AsRef<[u8]>) -> [u8; 64] {
        HMAC::mac(ikm, salt)
    }

    #[inline]
    pub fn expand(out: &mut [u8], prk: impl AsRef<[u8]>, info: impl AsRef<[u8]>) {
        assert_eq!(prk.as_ref().len(), 64, "HKDF-SHA512 expects a 64‑byte PRK");
        let info = info.as_ref();
        let mut counter: u8 = 1;
        assert!(
            out.len() < 0xff * 64,
            "Requested output exceeds RFC 5869 limit"
        );
        let mut i = 0;
        while i < out.len() {
            let mut hmac = HMAC::new(&prk);
            if i != 0 {
                hmac.update(&out[i - 64..][..64]);
            }
            hmac.update(info);
            hmac.update([counter]);
            let left = core::cmp::min(64, out.len() - i);
            out[i..][..left].copy_from_slice(&hmac.finalize()[..left]);
            counter += 1;
            i += 64;
        }
    }
}
