use crate::codec::Encoder;
use crate::command::Command;
use crate::error::VctrlError;
use crate::hashing::Hasher;
use crate::storage::traits::{ObjectStore, ObjectStoreExt, RefStore};

pub struct Fsck {
    pub encoder: Box<dyn Encoder>,
    pub hasher: Box<dyn Hasher>,
}

impl Command for Fsck {
    type Output = Vec<VctrlError>;

    fn execute(
        &self,
        store: &mut dyn ObjectStore,
        _refs: &mut dyn RefStore,
    ) -> Result<Vec<VctrlError>, VctrlError> {
        let hashes = match store.all_hashes() {
            Ok(h) => h,
            Err(e) => return Ok(vec![e]),
        };
        let mut errors = Vec::new();
        for hash in hashes {
            if let Err(e) = store.get_verified(&hash, self.encoder.as_ref(), self.hasher.as_ref()) {
                errors.push(e);
            }
        }
        Ok(errors)
    }
}
