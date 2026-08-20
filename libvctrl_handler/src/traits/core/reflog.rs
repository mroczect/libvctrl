use crate::errors::VctrlError;
use crate::types::{Hash, ReflogEntry};

pub trait ReflogStore: Send + Sync {
    type RefName: Send + Sync;

    fn append(
        &mut self,
        reference: &Self::RefName,
        old_hash: Option<Hash>,
        new_hash: Option<Hash>,
        reason: &str,
        timestamp: i64,
        timezone_offset: i16,
    ) -> Result<(), VctrlError>;

    fn entries(&self, reference: &Self::RefName) -> Result<Vec<ReflogEntry>, VctrlError>;
}
