use libvctrl_handler::VctrlError;
use libvctrl_handler::{Hash, Hasher};
use libvctrl_sha512::Hash as Sha512Hash;

#[derive(Debug, Default, Clone)]
pub struct Sha512Hasher;

impl Hasher for Sha512Hasher {
    fn hash(&self, data: &[u8]) -> Result<Hash, VctrlError> {
        let digest = Sha512Hash::hash(data);
        Ok(Hash::from_bytes(&digest).unwrap())
    }
}
