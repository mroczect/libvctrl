#![allow(clippy::indexing_slicing)]
use crate::sha512::Hash;

impl_hmac!(Hash, 64, 128);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prepare_key_short_pads_with_zeroes() {
        let key = [1_u8, 2, 3];
        let prepared = HMAC::prepare_key(&key);
        assert_eq!(&prepared[..3], &key[..]);
        assert!(prepared[3..].iter().all(|&b| b == 0));
    }

    #[test]
    fn test_prepare_key_exact_block_size() {
        let key = [0xAB_u8; 128];
        let prepared = HMAC::prepare_key(&key);
        assert_eq!(prepared, key);
    }

    #[test]
    fn test_prepare_key_longer_hashes_key() {
        let key = [0x61_u8; 200];
        let prepared = HMAC::prepare_key(&key);
        let hash = Hash::hash(key);
        assert_eq!(&prepared[..64], &hash[..]);
        assert!(prepared[64..].iter().all(|&b| b == 0));
    }

    #[test]
    fn test_mac_equals_update_finalize() {
        let key = b"secret";
        let input = b"message";
        let one_shot = HMAC::mac(input, key);

        let mut hmac = HMAC::new(key);
        hmac.update(input);
        assert_eq!(hmac.finalize(), one_shot);
    }

    #[test]
    fn test_finalize_verify_and_verify() {
        let key = b"secret";
        let input = b"message";
        let tag = HMAC::mac(input, key);

        let mut hmac = HMAC::new(key);
        hmac.update(input);
        assert!(hmac.finalize_verify(&tag));
        assert!(HMAC::verify(input, key, &tag));

        let bad = [0_u8; 64];
        let mut hmac = HMAC::new(key);
        hmac.update(input);
        assert!(!hmac.finalize_verify(&bad));
        assert!(!HMAC::verify(input, key, &bad));
    }
}
