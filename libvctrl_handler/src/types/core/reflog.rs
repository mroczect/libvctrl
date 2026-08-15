//! Reflog entry type.

use crate::Hash;

/// A single entry in a reflog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReflogEntry {
    /// The old hash (may be `None` for creation).
    pub old_id: Option<Hash>,
    /// The new hash (may be `None` for deletion).
    pub new_id: Option<Hash>,
    /// The reason for the change.
    pub reason: String,
    /// The timestamp of the change.
    pub timestamp: u64,
}

impl ReflogEntry {
    /// Creates a new reflog entry.
    #[must_use]
    pub const fn new(
        old_id: Option<Hash>,
        new_id: Option<Hash>,
        reason: String,
        timestamp: u64,
    ) -> Self {
        Self {
            old_id,
            new_id,
            reason,
            timestamp,
        }
    }
}
