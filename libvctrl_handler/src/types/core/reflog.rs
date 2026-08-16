use crate::Hash;
use crate::errors::VctrlError;

/// A single entry in a reflog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReflogEntry {
    old_id: Option<Hash>,
    new_id: Option<Hash>,
    reason: String,
    timestamp: i64,
    timezone_offset: i16,
}

impl ReflogEntry {
    /// Creates a new reflog entry.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::InvalidTimezoneOffset`] if the offset is out of range (-1440..=1440).
    pub fn new(
        old_id: Option<Hash>,
        new_id: Option<Hash>,
        reason: String,
        timestamp: i64,
        timezone_offset: i16,
    ) -> Result<Self, VctrlError> {
        if !(-1440..=1440).contains(&timezone_offset) {
            return Err(VctrlError::InvalidTimezoneOffset(timezone_offset));
        }
        Ok(Self {
            old_id,
            new_id,
            reason,
            timestamp,
            timezone_offset,
        })
    }

    /// Returns the old hash.
    #[must_use]
    pub const fn old_id(&self) -> Option<Hash> {
        self.old_id
    }

    /// Returns the new hash.
    #[must_use]
    pub const fn new_id(&self) -> Option<Hash> {
        self.new_id
    }

    /// Returns the reason for the change.
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// Returns the timestamp of the change.
    #[must_use]
    pub const fn timestamp(&self) -> i64 {
        self.timestamp
    }

    /// Returns the timezone offset.
    #[must_use]
    pub const fn timezone_offset(&self) -> i16 {
        self.timezone_offset
    }
}
