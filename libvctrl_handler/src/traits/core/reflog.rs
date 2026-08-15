//! Reflog store trait.

use crate::VctrlError;
use crate::types::{Hash, ReflogEntry};

/// Trait for managing reflogs.
pub trait ReflogStore {
    /// The reference name type.
    type RefName;

    /// Appends an entry to the reflog for a reference.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the append fails.
    fn append(
        &mut self,
        reference: &Self::RefName,
        old_hash: Option<Hash>,
        new_hash: Option<Hash>,
        reason: &str,
        timestamp: u64,
    ) -> Result<(), VctrlError>;

    /// Returns all reflog entries for a reference.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the entries cannot be read.
    fn entries(&self, reference: &Self::RefName) -> Result<Vec<ReflogEntry>, VctrlError>;
}
