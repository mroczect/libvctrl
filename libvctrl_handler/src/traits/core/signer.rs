use crate::errors::VctrlError;

/// Trait for signing data.
pub trait Signer: Send + Sync {
    /// Signs the given data with the specified key ID and returns the signature.
    fn sign(&mut self, key_id: &str, data: &[u8]) -> Result<Vec<u8>, VctrlError>;
}
