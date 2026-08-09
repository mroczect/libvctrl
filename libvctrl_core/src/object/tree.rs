//! Builder patterns for constructing [`Tree`](libvctrl_handler::Tree) and
//! [`TreeEntry`](libvctrl_handler::TreeEntry) objects.
//!
//! # Purpose
//! This module provides [`TreeBuilder`] and [`TreeEntryBuilder`], ergonomic
//! utilities for assembling version control trees and their entries. They
//! offer a fluent API to configure fields before finalizing the immutable
//! structs.
//!
//! # Design rationale
//! - **Validation Encapsulation**: The builders defer validation to the final
//!   `build()` step. [`TreeEntryBuilder`] checks name constraints, while
//!   [`TreeBuilder`] ensures the resulting tree entries are correctly sorted.
//! - **Flexible Accumulation**: [`TreeBuilder`] allows adding pre-built
//!   [`TreeEntry`] objects via `entry()` or raw components via `add_entry()`.
//! - **Ownership Management**: The builders take ownership of the underlying
//!   data during configuration. When `build` is called, the data is moved
//!   directly into the final structs without cloning.

use libvctrl_handler::{EntryKind, Hash, Tree, TreeEntry, VctrlError};

/// A builder for creating [`Tree`](libvctrl_handler::Tree) objects.
///
/// # Purpose
/// Provides a fluent interface for accumulating [`TreeEntry`] objects and
/// finalizing them into an immutable [`Tree`].
///
/// # Design rationale
/// Implements the standard builder pattern. It derives [`Default`] for easy
/// instantiation. The `build` method consumes `self` and delegates to
/// [`Tree::new`](libvctrl_handler::Tree::new), which enforces the structural
/// invariant that tree entries must be sorted by name and contain no duplicates.
///
/// # Examples
///
/// Building a tree with pre-built entries (note: entries must be sorted):
///
/// ```
/// use libvctrl_core::object::TreeBuilder;
/// use libvctrl_handler::{EntryKind, Hash, TreeEntry};
///
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// let e1 = TreeEntry::new("a.txt".to_string(), EntryKind::Blob, hash).unwrap();
/// let e2 = TreeEntry::new("b.txt".to_string(), EntryKind::Blob, hash).unwrap();
///
/// let tree = TreeBuilder::new()
///     .entry(e1)
///     .entry(e2)
///     .build()
///     .unwrap();
///
/// assert_eq!(tree.entries().len(), 2);
/// ```
#[derive(Debug, Default)]
pub struct TreeBuilder {
    entries: Vec<TreeEntry>,
}

impl TreeBuilder {
    /// Creates a new, empty `TreeBuilder`.
    ///
    /// # Design rationale
    /// This is a `const fn`, allowing instantiation in compile-time contexts.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_core::object::TreeBuilder;
    ///
    /// let builder = TreeBuilder::new();
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Adds a pre-built [`TreeEntry`] to the tree.
    ///
    /// # Design rationale
    /// This method provides a fast path for adding entries that have already
    /// been constructed and validated. It does not perform additional validation.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_core::object::TreeBuilder;
    /// use libvctrl_handler::{EntryKind, Hash, TreeEntry};
    ///
    /// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let entry = TreeEntry::new("file.txt".to_string(), EntryKind::Blob, hash).unwrap();
    ///
    /// let builder = TreeBuilder::new().entry(entry);
    /// ```
    #[must_use]
    pub fn entry(mut self, entry: TreeEntry) -> Self {
        self.entries.push(entry);
        self
    }

    /// Constructs and adds a [`TreeEntry`] from raw components.
    ///
    /// # Design rationale
    /// This is a convenience method that encapsulates [`TreeEntry::new`]. It
    /// returns a `Result` to gracefully handle name validation failures without
    /// breaking the builder chain.
    ///
    /// # Errors
    /// Returns [`VctrlError`] if the name is empty or exceeds the maximum length.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_core::object::TreeBuilder;
    /// use libvctrl_handler::{EntryKind, Hash};
    ///
    /// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
    ///
    /// let tree = TreeBuilder::new()
    ///     .add_entry("a.txt".to_string(), EntryKind::Blob, hash)?
    ///     .build()
    ///     .unwrap();
    ///
    /// # Ok::<(), libvctrl_handler::VctrlError>(())
    /// ```
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

    /// Consumes the builder and returns a finalized [`Tree`](libvctrl_handler::Tree).
    ///
    /// # Design rationale
    /// This method consumes `self` to enforce a linear flow. It delegates to
    /// [`Tree::new`](libvctrl_handler::Tree::new), which enforces the structural
    /// invariant that entries must be sorted by name.
    ///
    /// # Errors
    /// Returns [`VctrlError::InvalidName`] if the entries are not sorted or
    /// contain duplicates.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_core::object::TreeBuilder;
    /// use libvctrl_handler::{EntryKind, Hash};
    ///
    /// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
    ///
    /// let tree = TreeBuilder::new()
    ///     .add_entry("a.txt".to_string(), EntryKind::Blob, hash)?
    ///     .add_entry("b.txt".to_string(), EntryKind::Blob, hash)?
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(tree.entries().len(), 2);
    /// # Ok::<(), libvctrl_handler::VctrlError>(())
    /// ```
    pub fn build(self) -> Result<Tree, VctrlError> {
        Tree::new(self.entries)
    }
}

/// A builder for creating [`TreeEntry`](libvctrl_handler::TreeEntry) objects.
///
/// # Purpose
/// Provides a fluent interface for assembling a tree entry's data (name, kind,
/// hash) before finalizing it into an immutable object.
///
/// # Design rationale
/// This builder stores the raw components. The `build` method delegates to
/// [`TreeEntry::new`](libvctrl_handler::TreeEntry::new), which performs name
/// validation.
///
/// # Examples
///
/// Building a standard entry:
///
/// ```
/// use libvctrl_core::object::TreeEntryBuilder;
/// use libvctrl_handler::{EntryKind, Hash};
///
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// let entry = TreeEntryBuilder::new("file.txt".to_string(), EntryKind::Blob, hash)
///     .build()
///     .unwrap();
///
/// assert_eq!(entry.name(), "file.txt");
/// ```
#[derive(Debug)]
pub struct TreeEntryBuilder {
    name: String,
    kind: EntryKind,
    hash: Hash,
}

impl TreeEntryBuilder {
    /// Creates a new `TreeEntryBuilder` with the specified raw components.
    ///
    /// # Design rationale
    /// This is a `const fn` that simply stores the components. Validation is
    /// deferred to the `build` step.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_core::object::TreeEntryBuilder;
    /// use libvctrl_handler::{EntryKind, Hash};
    ///
    /// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let builder = TreeEntryBuilder::new("file.txt".to_string(), EntryKind::Blob, hash);
    /// ```
    #[must_use]
    pub const fn new(name: String, kind: EntryKind, hash: Hash) -> Self {
        Self { name, kind, hash }
    }

    /// Consumes the builder and returns a finalized
    /// [`TreeEntry`](libvctrl_handler::TreeEntry).
    ///
    /// # Design rationale
    /// This method consumes `self` and delegates to
    /// [`TreeEntry::new`](libvctrl_handler::TreeEntry::new), enforcing name
    /// length and emptiness constraints.
    ///
    /// # Errors
    /// Returns [`VctrlError::InvalidName`] if the name is empty or exceeds the
    /// maximum length.
    ///
    /// # Examples
    ///
    /// Successful build:
    ///
    /// ```
    /// use libvctrl_core::object::TreeEntryBuilder;
    /// use libvctrl_handler::{EntryKind, Hash};
    ///
    /// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let entry = TreeEntryBuilder::new("file.txt".to_string(), EntryKind::Blob, hash)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(entry.kind(), EntryKind::Blob);
    /// ```
    pub fn build(self) -> Result<TreeEntry, VctrlError> {
        TreeEntry::new(self.name, self.kind, self.hash)
    }
}
