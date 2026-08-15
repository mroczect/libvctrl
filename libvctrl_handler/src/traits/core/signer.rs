use crate::errors::VctrlError;

pub trait Signer {
    fn sign(&mut self, data: &[u8]) -> Result<Vec<u8>, VctrlError>;
}
