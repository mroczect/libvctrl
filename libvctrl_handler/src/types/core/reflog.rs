//! Reflog entry type for recording reference changes.
//!
//! # Purpose
//!
//! A reflog entry records a single change to a reference (branch, tag, etc.).
//! It stores the old and new commit hashes, a human-readable reason for the
//! change, and a timestamp. These entries are used by the
//! `ReflogStore` trait and by plumbing commands such as `reflog`.
//!
//! # Examples
//!
//! Constructing a reflog entry:
//!
//! ```
//! use libvctrl_handler::{Hash, ReflogEntry};
//!
//! let old = Hash::from_bytes(&[0u8; 64]).unwrap();
//! let new = Hash::from_bytes(&[1u8; 64]).unwrap();
//!
//! let entry = ReflogEntry::new(Some(old), Some(new), "commit".to_string(), 1_700_000_000);
//!
//! assert_eq!(entry.old_id, Some(old));
//! assert_eq!(entry.new_id, Some(new));
//! assert_eq!(entry.reason, "commit");
//! assert_eq!(entry.timestamp, 1_700_000_000);
//! ```

use crate::Hash;

/// A single entry in a reference log (reflog).
///
/// The `old_id` and `new_id` fields are optional because a reference may be
/// created from nothing (`old_id` is `None`) or deleted (`new_id` is `None`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReflogEntry {
    /// The hash before the change, or `None` if the reference was created.
    pub old_id: Option<Hash>,
    /// The hash after the change, or `None` if the reference was deleted.
    pub new_id: Option<Hash>,
    /// A human-readable reason for the change (e.g., "commit").
    pub reason: String,
    /// Unix timestamp (seconds since epoch).
    pub timestamp: u64,
}

impl ReflogEntry {
    /// Creates a new reflog entry with the given hashes, reason, and timestamp.
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
