use crate::errors::VctrlError;

pub trait Verifier {
    fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, VctrlError>;
}
