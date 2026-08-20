use crate::errors::VctrlError;

pub trait Verifier: Send + Sync {
    fn verify(&self, key_id: &str, data: &[u8], signature: &[u8]) -> Result<bool, VctrlError>;
}
