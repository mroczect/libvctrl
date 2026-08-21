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
    use std::io::Cursor;

    #[test]
    fn hash_empty_input() -> Result<(), VctrlError> {
        let hash = Sha512Hasher.hash(Cursor::new(Vec::<u8>::new()))?;
        assert_eq!(
            hash.as_bytes(),
            &[
                0xcf, 0x83, 0xe1, 0x35, 0x7e, 0xef, 0xb8, 0xbd, 0xf1, 0x54, 0x28, 0x50, 0xd6, 0x6d,
                0x80, 0x07, 0xd6, 0x20, 0xe4, 0x05, 0x0b, 0x57, 0x15, 0xdc, 0x83, 0xf4, 0xa9, 0x21,
                0xd3, 0x6c, 0xe9, 0xce, 0x47, 0xd0, 0xd1, 0x3c, 0x5d, 0x85, 0xf2, 0xb0, 0xff, 0x83,
                0x18, 0xd2, 0x87, 0x7e, 0xec, 0x2f, 0x63, 0xb9, 0x31, 0xbd, 0x47, 0x41, 0x7a, 0x81,
                0xa5, 0x38, 0x32, 0x7a, 0xf9, 0x27, 0xda, 0x3e
            ]
        );
        Ok(())
    }

    #[test]
    fn hash_abc() -> Result<(), VctrlError> {
        let hash = Sha512Hasher.hash(Cursor::new(b"abc"))?;
        assert_eq!(
            hash.as_bytes(),
            &[
                0xdd, 0xaf, 0x35, 0xa1, 0x93, 0x61, 0x7a, 0xba, 0xcc, 0x41, 0x73, 0x49, 0xae, 0x20,
                0x41, 0x31, 0x12, 0xe6, 0xfa, 0x4e, 0x89, 0xa9, 0x7e, 0xa2, 0x0a, 0x9e, 0xee, 0xe6,
                0x4b, 0x55, 0xd3, 0x9a, 0x21, 0x92, 0x99, 0x2a, 0x27, 0x4f, 0xc1, 0xa8, 0x36, 0xba,
                0x3c, 0x23, 0xa3, 0xfe, 0xeb, 0xbd, 0x45, 0x4d, 0x44, 0x23, 0x64, 0x3c, 0xe8, 0x0e,
                0x2a, 0x9a, 0xc9, 0x4f, 0xa5, 0x4c, 0xa4, 0x9f
            ]
        );
        Ok(())
    }

    #[test]
    fn hash_multiple_chunks() -> Result<(), VctrlError> {
        let data = vec![0xAB; 8192];
        let hash = Sha512Hasher.hash(Cursor::new(data))?;
        assert_eq!(hash.as_bytes().len(), 64);
        Ok(())
    }
}
