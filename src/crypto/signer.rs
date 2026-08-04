use crate::crypto::Signer;
use crate::error::VctrlError;
use ed25519_dalek::{Signer as EdSigner, SigningKey, VerifyingKey};
use rand::RngCore;
use std::path::Path;

pub struct LibrageSigner {
    signing_key: SigningKey,
}

impl std::fmt::Debug for LibrageSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LibrageSigner")
            .field("signing_key", &"[REDACTED]")
            .finish()
    }
}

impl LibrageSigner {
    #[cfg(unix)]
    pub fn from_seed_file(path: impl AsRef<Path>) -> Result<Self, VctrlError> {
        use std::os::unix::fs::PermissionsExt;
        let path = path.as_ref();
        let metadata = std::fs::metadata(path).map_err(VctrlError::Io)?;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err(VctrlError::Other(
                "seed file permissions too permissive; must be 0600".into(),
            ));
        }
        let bytes = std::fs::read(path).map_err(VctrlError::Io)?;
        let seed: [u8; 32] = bytes
            .try_into()
            .map_err(|_| VctrlError::Other("seed file must be exactly 32 bytes".into()))?;
        let signing_key = SigningKey::from_bytes(&seed);
        Ok(Self { signing_key })
    }

    #[cfg(not(unix))]
    pub fn from_seed_file(path: impl AsRef<Path>) -> Result<Self, VctrlError> {
        let bytes = std::fs::read(path.as_ref()).map_err(VctrlError::Io)?;
        let seed: [u8; 32] = bytes
            .try_into()
            .map_err(|_| VctrlError::Other("seed file must be exactly 32 bytes".into()))?;
        let signing_key = SigningKey::from_bytes(&seed);
        Ok(Self { signing_key })
    }

    pub fn generate() -> Self {
        let mut csprng = rand::rngs::OsRng;
        let mut seed = [0u8; 32];
        csprng.fill_bytes(&mut seed);
        let signing_key = SigningKey::from_bytes(&seed);
        Self { signing_key }
    }

    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }
}

impl Signer for LibrageSigner {
    fn sign(&self, commit_hash: &[u8]) -> Result<Vec<u8>, VctrlError> {
        let signature = self.signing_key.sign(commit_hash);
        Ok(signature.to_bytes().to_vec())
    }
}
