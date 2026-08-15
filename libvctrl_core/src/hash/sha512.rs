use libvctrl_handler::{Hash, Hasher, VctrlError};
use libvctrl_sha512::Hash as Sha512Hash;

/// A hasher that uses the SHA-512 algorithm.
#[derive(Debug, Default, Clone)]
pub struct Sha512Hasher;

impl Hasher for Sha512Hasher {
    fn hash<R: std::io::Read + Send>(&self, mut reader: R) -> Result<Hash, VctrlError> {
        let mut hasher = Sha512Hash::new();
        let mut buffer = [0u8; 4096];
        loop {
            let n = reader
                .read(&mut buffer)
                .map_err(|e| VctrlError::IoError(std::sync::Arc::new(e)))?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }
        let digest = hasher.finalize();
        Hash::from_bytes(&digest)
    }
}
