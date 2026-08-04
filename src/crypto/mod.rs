pub mod signer;
pub use signer::*;

use crate::error::VctrlError;

pub trait Signer {
    fn sign(&self, commit_hash: &[u8]) -> Result<Vec<u8>, VctrlError>;
}
