//! Builder for [`Tree`] and [`TreeEntry`] objects.
//!
//! This module provides two builders:
//! - [`TreeBuilder`] for constructing a full [`Tree`] object.
//! - [`TreeEntryBuilder`] for constructing a single [`TreeEntry`].
//!
//! # Building a tree
//!
//! A tree is built by adding entries one by one. Entries can be added as
//! pre‑built [`TreeEntry`] values, or from parts using the convenience
//! method [`add_entry`](TreeBuilder::add_entry).
//!
//! The [`build`](TreeBuilder::build) method delegates to [`Tree::new`],
//! which enforces that entries are sorted lexicographically by name and
//! have no duplicates. If entries are not in the correct order, the build
//! will fail with an error.
//!
//! # Example
//!
//! ```rust
//! use libvctrl_core::object::{TreeBuilder, TreeEntryBuilder};
//! use libvctrl_handler::*;
//!
//! let hash = Hash::from_bytes(&[0xAA; 64]).unwrap();
//!
//! // Build a tree with two entries
//! let tree = TreeBuilder::new()
//!     .add_entry("a.txt".into(), EntryKind::Blob, hash).unwrap()
//!     .add_entry("b.txt".into(), EntryKind::Blob, hash).unwrap()
//!     .build()
//!     .unwrap();
//!
//! // Build a single entry
//! let entry = TreeEntryBuilder::new("file".into(), EntryKind::Blob, hash)
//!     .build()
//!     .unwrap();
//! ```
//!
//! # Sorting requirement
//!
//! The entries **must** be added in sorted order. If you add entries out
//! of order and then call `build()`, you will get an error. This is
//! intentional – it prevents accidental creation of invalid trees.

use libvctrl_handler::{EntryKind, Hash, Tree, TreeEntry, VctrlError};

/// Builder for [`Tree`] objects.
///
/// Entries are collected and then validated upon [`build`](TreeBuilder::build).
///
/// # Example
///
/// ```rust
/// # use libvctrl_core::object::TreeBuilder;
/// # use libvctrl_handler::*;
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// let tree = TreeBuilder::new()
///     .add_entry("README.md".into(), EntryKind::Blob, hash)
///     .unwrap()
///     .build()
///     .unwrap();
/// ```
#[derive(Debug, Default)]
pub struct TreeBuilder {
    entries: Vec<TreeEntry>,
}

impl TreeBuilder {
    /// Creates an empty builder.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Adds a pre‑built entry.
    #[must_use]
    pub fn entry(mut self, entry: TreeEntry) -> Self {
        self.entries.push(entry);
        self
    }

    /// Convenience method to add an entry from parts.
    ///
    /// # Errors
    /// Returns [`VctrlError::InvalidName`] if the name is invalid.
    pub fn add_entry(
        mut self,
        name: String,
        kind: EntryKind,
        hash: Hash,
    ) -> Result<Self, VctrlError> {
        let entry = TreeEntry::new(name, kind, hash)?;
        self.entries.push(entry);
        Ok(self)
    }

    /// Builds the tree, ensuring entries are sorted and unique.
    ///
    /// # Errors
    /// Returns an error if entries are not sorted or contain duplicates.
    pub fn build(self) -> Result<Tree, VctrlError> {
        Tree::new(self.entries)
    }
}

/// Builder for a single [`TreeEntry`].
///
/// # Example
///
/// ```rust
/// # use libvctrl_core::object::TreeEntryBuilder;
/// # use libvctrl_handler::*;
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// let entry = TreeEntryBuilder::new("file.txt".into(), EntryKind::Blob, hash)
///     .build()
///     .unwrap();
/// ```
#[derive(Debug)]
pub struct TreeEntryBuilder {
    name: String,
    kind: EntryKind,
    hash: Hash,
}

impl TreeEntryBuilder {
    /// Starts a new entry builder.
    #[must_use]
    pub const fn new(name: String, kind: EntryKind, hash: Hash) -> Self {
        Self { name, kind, hash }
    }

    /// Builds the entry.
    ///
    /// # Errors
    /// Returns [`VctrlError::InvalidName`] if the name is invalid.
    pub fn build(self) -> Result<TreeEntry, VctrlError> {
        TreeEntry::new(self.name, self.kind, self.hash)
    }
}
