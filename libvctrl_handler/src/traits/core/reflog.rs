use crate::{Hash, ReflogEntry, VctrlError};

pub trait ReflogStore {
    type RefName;

    fn append(
        &mut self,
        reference: &Self::RefName,
        old_hash: Option<Hash>,
        new_hash: Option<Hash>,
        reason: &str,
        timestamp: u64,
    ) -> Result<(), VctrlError>;

    fn entries(&self, reference: &Self::RefName) -> Result<Vec<ReflogEntry>, VctrlError>;
}
