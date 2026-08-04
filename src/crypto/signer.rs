use crate::crypto::Signer;
use crate::error::VctrlError;
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::RngCore;
use std::path::Path;

#[derive(Debug)]
pub struct LibrageSigner {
    signing_key: SigningKey,
}

impl LibrageSigner {
    pub fn from_seed_file(path: impl AsRef<Path>) -> Result<Self, VctrlError> {
        let bytes = std::fs::read(path.as_ref()).map_err(VctrlError::Io)?;
        if bytes.len() != 32 {
            return Err(VctrlError::Other(
                "seed file must be exactly 32 bytes".into(),
            ));
        }
        let seed: [u8; 32] = bytes.try_into().unwrap();
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
        use ed25519_dalek::Signer as _;
        let signature = self.signing_key.sign(commit_hash);
        Ok(signature.to_bytes().to_vec())
    }
}
