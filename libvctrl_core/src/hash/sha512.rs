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
