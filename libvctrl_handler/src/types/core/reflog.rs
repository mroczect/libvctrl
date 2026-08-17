//! Reflog entry type.
//!
//! # Architecture
//! This module defines the [`ReflogEntry`] struct, which represents a single
//! timestamped record in a reference log (reflog). Reflogs act as an append-only
//! audit trail, tracking every mutation to a reference (e.g., commits, resets,
//! checkouts). This history is crucial for recovering from accidental operations
//! and for garbage collection pruning.
//!
//! # Design Rationale: Immutable State Transitions
//! A [`ReflogEntry`] captures a state transition: it records the `old_id` and the
//! `new_id` of a reference. By using `Option<Hash>`, the type elegantly handles
//! edge cases:
//! - `old_id` is `None`: The reference was just created (born).
//! - `new_id` is `None`: The reference was deleted (died).
//! Once constructed, the entry is immutable, ensuring that the audit history
//! cannot be tampered with.

use crate::Hash;
use crate::errors::VctrlError;

/// A single entry in a reflog.
///
/// # Why this exists
/// Provides a strongly-typed, validated record of a reference update. By requiring
/// construction via [`new`](Self::new), the crate guarantees that every `ReflogEntry`
/// in memory adheres to temporal constraints (e.g., valid timezone offsets). This
/// prevents malformed historical data from corrupting repository recovery tools.
///
/// # How it works
/// The struct stores the old and new hashes as `Option<Hash>`. Because [`Hash`] is
/// a `Copy` type (a 64-byte array wrapper), storing and copying these options is
/// a fast stack operation. The `reason` is stored as an owned `String` to ensure
/// the entry is self-contained and `'static` safe.
///
/// # Examples
///
/// Creating a reflog entry for a new commit:
///
/// ```
/// # use libvctrl_handler::types::core::reflog::ReflogEntry;
/// # use libvctrl_handler::Hash;
/// # use libvctrl_handler::VctrlError;
/// # let old_hash = Hash::from_bytes(&[0_u8; 64])?;
/// # let new_hash = Hash::from_bytes(&[1u8; 64])?;
/// let entry = ReflogEntry::new(Some(old_hash), Some(new_hash), "commit: Add feature".to_string(), 1600000000, 0)?;
/// assert_eq!(entry.reason(), "commit: Add feature");
/// # Ok::<(), VctrlError>(())
/// ```
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
    /// # How it works
    /// Validates that the `timezone_offset` falls within the valid range of
    /// -1440 to 1440 minutes (UTC-24:00 to UTC+24:00). This strict validation
    /// prevents arithmetic overflows or logic errors during date formatting and
    /// historical chronological sorting.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::InvalidTimezoneOffset`] if the offset is out of range (-1440..=1440).
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::types::core::reflog::ReflogEntry;
    /// # use libvctrl_handler::Hash;
    /// # use libvctrl_handler::VctrlError;
    /// # let hash = Hash::from_bytes(&[0_u8; 64])?;
    /// // Creating an entry for the birth of a reference (old_id is None)
    /// let entry = ReflogEntry::new(None, Some(hash), "branch: Created from HEAD".to_string(), 0, 0)?;
    /// assert!(entry.old_id().is_none());
    /// # Ok::<(), VctrlError>(())
    /// ```
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
    ///
    /// # How it works
    /// Returns `Option<Hash>`. Because `Hash` is `Copy`, this returns a copy of
    /// the hash rather than a reference, simplifying lifetime management. Returns
    /// `None` if this entry records the creation of a new reference.
    #[must_use]
    pub const fn old_id(&self) -> Option<Hash> {
        self.old_id
    }

    /// Returns the new hash.
    ///
    /// # How it works
    /// Returns `Option<Hash>`. Returns `None` if this entry records the deletion
    /// of a reference.
    #[must_use]
    pub const fn new_id(&self) -> Option<Hash> {
        self.new_id
    }

    /// Returns the reason for the change.
    ///
    /// # How it works
    /// Returns a string slice (`&str`) borrowing from the internal `String`. This
    /// avoids allocation when the caller only needs to read the reason.
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// Returns the timestamp of the change.
    ///
    /// # How it works
    /// Returns the Unix timestamp (seconds since epoch) as an `i64`. This is a
    /// `const fn`, allowing compile-time evaluation.
    #[must_use]
    pub const fn timestamp(&self) -> i64 {
        self.timestamp
    }

    /// Returns the timezone offset.
    ///
    /// # How it works
    /// Returns the timezone offset in minutes as an `i16`. This is a `const fn`,
    /// allowing compile-time evaluation.
    #[must_use]
    pub const fn timezone_offset(&self) -> i16 {
        self.timezone_offset
    }
}
