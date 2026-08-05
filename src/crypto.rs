use crate::error::VctrlError;

pub trait Signer: Send + Sync {
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>, VctrlError>;
}

pub trait Verifier: Send + Sync {
    fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, VctrlError>;
}
