use crate::errors::VctrlError;
use crate::types::{Hash, ReflogEntry};

/// Trait for managing reflogs.
pub trait ReflogStore: Send + Sync {
    /// The reference name type.
    type RefName: Send + Sync;

    /// Appends an entry to the reflog for a reference.
    fn append(
        &mut self,
        reference: &Self::RefName,
        old_hash: Option<Hash>,
        new_hash: Option<Hash>,
        reason: &str,
        timestamp: i64,
        timezone_offset: i16,
    ) -> Result<(), VctrlError>;

    /// Returns all reflog entries for a reference.
    fn entries(&self, reference: &Self::RefName) -> Result<Vec<ReflogEntry>, VctrlError>;
}
