//! Reflog store trait.
//!
//! # Architecture
//! This module defines the abstract contract for managing reference logs (reflogs).
//! Reflogs act as an append-only audit trail, recording every mutation to a reference
//! (e.g., commits, resets, checkouts). This history is crucial for recovering from
//! accidental operations and for garbage collection pruning.
//!
//! # Design Rationale: Strict Append-Only Semantics
//! The trait exposes only `append` and `entries` methods. There is no `delete` or
//! `update` operation for individual entries. This enforces the append-only nature
//! of reflogs at the type level, preventing consumers from accidentally rewriting
//! audit history.

use crate::errors::VctrlError;
use crate::types::{Hash, ReflogEntry};

/// Trait for managing reflogs.
///
/// # Why this exists
/// Provides a unified interface for recording and retrieving the history of
/// reference updates. By abstracting this into a trait, the crate allows the core
/// engine to track state changes without being tied to the standard `.git/logs`
/// filesystem layout. Consumers can inject in-memory reflogs for testing or
/// database-backed reflogs for enterprise persistence.
///
/// # How it works
/// The store maintains a mapping between reference names and a chronological list
/// of [`ReflogEntry`] items. The `append` method requires `&mut self` to enforce
/// exclusive access, ensuring that concurrent updates to the same reference's
/// reflog do not interleave and corrupt the history file. The `entries` method
/// takes `&self`, allowing safe, concurrent reads of the audit trail.
///
/// # Design Rationale: `Vec` over Iterators
/// Unlike [`RefStore::list_refs`](crate::traits::core::ref_store::RefStore::list_refs),
/// which returns an iterator to handle millions of refs, `entries` returns a `Vec`.
/// Reflogs are bounded in size (e.g., Git defaults to 90 days or 250 entries). The
/// memory footprint of loading a single reference's reflog is strictly bounded,
/// making a `Vec` more ergonomic and efficient than a streaming iterator.
///
/// # Examples
///
/// Implementing the trait for a mock in-memory store:
///
/// ```
/// # use libvctrl_handler::traits::core::reflog::ReflogStore;
/// # use libvctrl_handler::{Hash, ReflogEntry, VctrlError};
/// # use std::collections::HashMap;
/// #
/// #[derive(Default)]
/// struct MockReflogStore {
///     logs: HashMap<String, Vec<ReflogEntry>>,
/// }
///
/// impl ReflogStore for MockReflogStore {
///     type RefName = String;
///
///     fn append(
///         &mut self,
///         reference: &Self::RefName,
///         old_hash: Option<Hash>,
///         new_hash: Option<Hash>,
///         reason: &str,
///         timestamp: i64,
///         timezone_offset: i16,
///     ) -> Result<(), VctrlError> {
///         let entry = ReflogEntry::new(
///             old_hash,
///             new_hash,
///             reason.to_string(),
///             timestamp,
///             timezone_offset,
///         )?;
///         self.logs.entry(reference.clone()).or_default().push(entry);
///         Ok(())
///     }
///
///     fn entries(&self, reference: &Self::RefName) -> Result<Vec<ReflogEntry>, VctrlError> {
///         Ok(self.logs.get(reference).cloned().unwrap_or_default())
///     }
/// }
///
/// let mut store = MockReflogStore::default();
/// let hash = Hash::from_bytes(&[0_u8; 64])?;
/// store.append(&"refs/heads/main".to_string(), None, Some(hash), "initial commit", 0, 0)?;
/// assert_eq!(store.entries(&"refs/heads/main".to_string())?.len(), 1);
/// # Ok::<(), VctrlError>(())
/// ```
pub trait ReflogStore: Send + Sync {
    /// The reference name type.
    ///
    /// # Why this exists
    /// Decouples the reference name representation from the trait. While typically
    /// a `String`, this allows backends to use interned strings or specialized
    /// path types, ensuring interoperability with the associated [`RefStore`](crate::traits::core::ref_store::RefStore).
    type RefName: Send + Sync;

    /// Appends an entry to the reflog for a reference.
    ///
    /// # How it works
    /// Creates a new [`ReflogEntry`] with the provided transition (`old_hash` to
    /// `new_hash`), reason, and timestamp metadata. The entry is appended to the
    /// end of the reference's log. Requires `&mut self` to enforce exclusive access,
    /// mimicking the behavior of acquiring a `.lock` file on the reflog.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::InvalidTimezoneOffset`] if the `timezone_offset` is
    /// out of the valid range (-1440 to 1440). Returns [`VctrlError`] if the
    /// underlying storage fails to persist the new entry.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::reflog::ReflogStore;
    /// # use libvctrl_handler::{Hash, ReflogEntry, VctrlError};
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct MockReflogStore { logs: HashMap<String, Vec<ReflogEntry>> }
    /// # impl ReflogStore for MockReflogStore {
    /// #     type RefName = String;
    /// #     fn append(&mut self, r: &Self::RefName, o: Option<Hash>, n: Option<Hash>, re: &str, t: i64, tz: i16) -> Result<(), VctrlError> {
    /// #         let e = ReflogEntry::new(o, n, re.to_string(), t, tz)?; self.logs.entry(r.clone()).or_default().push(e); Ok(())
    /// #     }
    /// #     fn entries(&self, r: &Self::RefName) -> Result<Vec<ReflogEntry>, VctrlError> { Ok(self.logs.get(r).cloned().unwrap_or_default()) }
    /// # }
    /// let mut store = MockReflogStore::default();
    /// let hash = Hash::from_bytes(&[1u8; 64])?;
    /// store.append(&"HEAD".to_string(), None, Some(hash), "checkout", 100, 0)?;
    /// # Ok::<(), VctrlError>(())
    /// ```
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
    ///
    /// # How it works
    /// Retrieves the complete chronological history of updates for the specified
    /// reference. The entries are returned in a `Vec` ordered from oldest to newest.
    /// If the reference has no reflog (e.g., a newly created branch without commits),
    /// an empty `Vec` is returned. Takes `&self` to allow concurrent reads of the
    /// audit trail.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the underlying storage cannot be read.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::reflog::ReflogStore;
    /// # use libvctrl_handler::{Hash, ReflogEntry, VctrlError};
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct MockReflogStore { logs: HashMap<String, Vec<ReflogEntry>> }
    /// # impl ReflogStore for MockReflogStore {
    /// #     type RefName = String;
    /// #     fn append(&mut self, r: &Self::RefName, o: Option<Hash>, n: Option<Hash>, re: &str, t: i64, tz: i16) -> Result<(), VctrlError> {
    /// #         let e = ReflogEntry::new(o, n, re.to_string(), t, tz)?; self.logs.entry(r.clone()).or_default().push(e); Ok(())
    /// #     }
    /// #     fn entries(&self, r: &Self::RefName) -> Result<Vec<ReflogEntry>, VctrlError> { Ok(self.logs.get(r).cloned().unwrap_or_default()) }
    /// # }
    /// let store = MockReflogStore::default();
    /// let entries = store.entries(&"refs/heads/nonexistent".to_string())?;
    /// assert!(entries.is_empty());
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn entries(&self, reference: &Self::RefName) -> Result<Vec<ReflogEntry>, VctrlError>;
}
