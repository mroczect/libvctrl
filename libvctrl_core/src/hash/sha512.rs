use alloc::sync::Arc;
use std::io;

use libvctrl_handler::{Hash, Hasher, VctrlError};
use libvctrl_sha512::Hash as Sha512Hash;

#[derive(Debug, Default, Clone, Copy)]
pub struct Sha512Hasher;

impl Hasher for Sha512Hasher {
    fn hash<R: io::Read + Send>(&self, mut reader: R) -> Result<Hash, VctrlError> {
        let mut hasher = Sha512Hash::new();
        let mut buffer = [0u8; 4096];
        loop {
            let n = reader.read(&mut buffer).map_err(VctrlError::from_io)?;
            if n == 0 {
                break;
            }
            let chunk = buffer.get(..n).ok_or_else(|| {
                VctrlError::IoError(Arc::new(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "read returned invalid length",
                )))
            })?;
            hasher.update(chunk);
        }
        let digest = hasher.finalize();
        Hash::from_bytes(&digest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libvctrl_handler::HASH_LENGTH;
    use std::io::Cursor;

    #[test]
    fn test_hash_empty_input() {
        let cursor = Cursor::new(Vec::<u8>::new());
        let result = Sha512Hasher.hash(cursor);
        assert!(result.is_ok(), "hashing empty input should succeed");
        let hash = result.unwrap();
        assert_eq!(
            hash.as_bytes().len(),
            HASH_LENGTH,
            "hash should be HASH_LENGTH bytes"
        );
    }

    #[test]
    fn test_hash_non_empty_input() {
        let cursor = Cursor::new(b"hello world");
        let result = Sha512Hasher.hash(cursor);
        assert!(result.is_ok());
        let hash = result.unwrap();
        assert_eq!(hash.as_bytes().len(), HASH_LENGTH);
    }

    #[test]
    fn test_hash_deterministic() {
        let data = b"test data for determinism check";
        let h1 = Sha512Hasher.hash(Cursor::new(data.as_slice())).unwrap();
        let h2 = Sha512Hasher.hash(Cursor::new(data.as_slice())).unwrap();
        assert_eq!(
            h1.as_bytes(),
            h2.as_bytes(),
            "same input must produce identical hash"
        );
    }

    #[test]
    fn test_hash_different_inputs_produce_different_hashes() {
        let h1 = Sha512Hasher.hash(Cursor::new(b"input one")).unwrap();
        let h2 = Sha512Hasher.hash(Cursor::new(b"input two")).unwrap();
        assert_ne!(
            h1.as_bytes(),
            h2.as_bytes(),
            "different inputs should produce different hashes"
        );
    }

    #[test]
    fn test_hash_large_input() {
        let data = vec![0xABu8; 100_000];
        let result = Sha512Hasher.hash(Cursor::new(data));
        assert!(result.is_ok(), "hashing large input should succeed");
        let hash = result.unwrap();
        assert_eq!(hash.as_bytes().len(), HASH_LENGTH);
    }

    #[test]
    fn test_hash_single_byte() {
        let result = Sha512Hasher.hash(Cursor::new(b"\x00"));
        assert!(result.is_ok());
        let result2 = Sha512Hasher.hash(Cursor::new(b"\xFF"));
        assert!(result2.is_ok());
        assert_ne!(
            result.unwrap().as_bytes(),
            result2.unwrap().as_bytes(),
            "different single bytes should produce different hashes"
        );
    }
}
