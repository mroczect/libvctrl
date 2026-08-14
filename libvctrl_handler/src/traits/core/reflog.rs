//! Defines the `ReflogStore` trait for recording reference changes.
//!
//! # Purpose
//!
//! The `ReflogStore` trait abstracts the storage of reference logs
//! (reflogs). Reflogs record the history of changes to a reference, such as
//! branch tips or tags, providing safety and auditability. Operations like
//! `update-ref` and `commit` append entries to the reflog whenever a
//! reference is moved.
//!
//! # Why a separate module
//!
//! Reflog storage is a separate concern from object storage or reference
//! storage. Keeping the trait in its own file follows the same pattern as
//! other core traits, allowing different backends (in-memory, file-based,
//! database) to implement the same contract.
//!
//! # Examples
//!
//! A simple in-memory implementation:
//!
//! ```
//! use std::collections::HashMap;
//! use libvctrl_handler::{Hash, ReflogEntry, ReflogStore, VctrlError};
//!
//! struct MemoryReflog {
//!     logs: HashMap<String, Vec<ReflogEntry>>,
//! }
//!
//! impl ReflogStore for MemoryReflog {
//!     type RefName = String;
//!
//!     fn append(
//!         &mut self,
//!         reference: &String,
//!         old_hash: Option<Hash>,
//!         new_hash: Option<Hash>,
//!         reason: &str,
//!         timestamp: u64,
//!     ) -> Result<(), VctrlError> {
//!         self.logs
//!             .entry(reference.clone())
//!             .or_default()
//!             .push(ReflogEntry {
//!                 old_id: old_hash,
//!                 new_id: new_hash,
//!                 reason: reason.to_string(),
//!                 timestamp,
//!             });
//!         Ok(())
//!     }
//!
//!     fn entries(&self, reference: &String) -> Result<Vec<ReflogEntry>, VctrlError> {
//!         Ok(self.logs.get(reference).cloned().unwrap_or_default())
//!     }
//! }
//!
//! let mut reflog = MemoryReflog {
//!     logs: HashMap::new(),
//! };
//!
//! let old = Hash::from_bytes(&[0u8; 64]).unwrap();
//! let new = Hash::from_bytes(&[1u8; 64]).unwrap();
//!
//! reflog
//!     .append(&"refs/heads/main".to_string(), Some(old), Some(new), "commit", 123)
//!     .unwrap();
//!
//! let entries = reflog.entries(&"refs/heads/main".to_string()).unwrap();
//! assert_eq!(entries.len(), 1);
//! assert_eq!(entries[0].old_id, Some(old));
//! ```

use crate::{Hash, ReflogEntry, VctrlError};

/// Trait for recording changes to references.
///
/// # Purpose
///
/// `ReflogStore` abstracts the ability to append and retrieve reference
/// log entries. Reflogs provide a safety net by preserving the history of
/// reference updates, allowing users to recover from accidental changes.
///
/// # Associated Types
///
/// - `RefName`: the type used to identify a reference (e.g., `String` or
///   `PathBuf`).
///
/// # Examples
///
/// A trivial implementation that never stores any entries:
///
/// ```
/// use libvctrl_handler::{Hash, ReflogEntry, ReflogStore, VctrlError};
///
/// struct EmptyReflog;
///
/// impl ReflogStore for EmptyReflog {
///     type RefName = String;
///
///     fn append(
///         &mut self,
///         _reference: &String,
///         _old_hash: Option<Hash>,
///         _new_hash: Option<Hash>,
///         _reason: &str,
///         _timestamp: u64,
///     ) -> Result<(), VctrlError> {
///         Ok(())
///     }
///
///     fn entries(&self, _reference: &String) -> Result<Vec<ReflogEntry>, VctrlError> {
///         Ok(vec![])
///     }
/// }
///
/// let mut reflog = EmptyReflog;
/// reflog
///     .append(&"refs/heads/main".to_string(), None, None, "test", 0)
///     .unwrap();
/// assert!(reflog.entries(&"refs/heads/main".to_string()).unwrap().is_empty());
/// ```
///
/// # Errors
///
/// - [`VctrlError::Other`] if the underlying reflog backend fails.
pub trait ReflogStore {
    /// The type used to identify a reference.
    type RefName;

    /// Appends a new entry to the reflog for the given reference.
    ///
    /// # Parameters
    ///
    /// - `reference`: the reference whose log should be updated.
    /// - `old_hash`: the previous hash, or `None` for a new reference.
    /// - `new_hash`: the new hash, or `None` for a deleted reference.
    /// - `reason`: a human-readable message describing the change.
    /// - `timestamp`: a Unix timestamp (seconds since epoch) for the entry.
    ///
    /// # Errors
    ///
    /// Returns an error if the entry cannot be recorded.
    fn append(
        &mut self,
        reference: &Self::RefName,
        old_hash: Option<Hash>,
        new_hash: Option<Hash>,
        reason: &str,
        timestamp: u64,
    ) -> Result<(), VctrlError>;

    /// Retrieves all reflog entries for the given reference.
    ///
    /// If the reference has no reflog entries, an empty vector is returned.
    ///
    /// # Errors
    ///
    /// Returns an error if the reflog cannot be read.
    fn entries(&self, reference: &Self::RefName) -> Result<Vec<ReflogEntry>, VctrlError>;
}
