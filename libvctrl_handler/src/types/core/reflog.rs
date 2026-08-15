use crate::Hash;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReflogEntry {
    pub old_id: Option<Hash>,

    pub new_id: Option<Hash>,

    pub reason: String,

    pub timestamp: u64,
}

impl ReflogEntry {
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
