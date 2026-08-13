//! Builder patterns for constructing [`Tree`](libvctrl_handler::Tree) and
//! [`TreeEntry`](libvctrl_handler::TreeEntry) objects.
//!
//! # Purpose
//!
//! This module provides [`TreeBuilder`] and [`TreeEntryBuilder`], ergonomic
//! utilities for assembling version control trees and their entries. They
//! offer a fluent API to configure fields before finalizing the immutable
//! structs.
//!
//! # Design Rationale
//!
//! - **Validation encapsulation**: The builders defer validation to the final
//!   `build()` step. [`TreeEntryBuilder`] checks name constraints, while
//!   [`TreeBuilder`] ensures the resulting tree entries are correctly sorted.
//! - **Flexible accumulation**: [`TreeBuilder`] allows adding pre-built
//!   [`TreeEntry`] objects via [`entry`](TreeBuilder::entry) or raw components
//!   via [`add_entry`](TreeBuilder::add_entry).
//! - **Ownership management**: The builders take ownership of the underlying
//!   data during configuration. When `build` is called, the data is moved
//!   directly into the final structs without cloning.
//! - **Deferred structural validation**: Tree entries must be strictly sorted
//!   by name. The builder does not enforce sorting until `build()`, allowing
//!   callers to add entries in any order and still receive a descriptive
//!   error if the final order is invalid.
//!
//! # Internal Mechanism
//!
//! [`TreeBuilder`] stores a [`Vec<TreeEntry>`] and appends entries as they are
//! added. The `build` method delegates to
//! [`Tree::new`](libvctrl_handler::Tree::new), which iterates over adjacent
//! entries and rejects any pair that is not strictly ascending.
//!
//! [`TreeEntryBuilder`] stores raw components (`name`, `kind`, `hash`) and
//! delegates to [`TreeEntry::new`](libvctrl_handler::TreeEntry::new) during
//! `build`, which validates the name against length and path traversal rules.
//!
//! # Examples
//!
//! Building a tree with pre-built sorted entries:
//!
//! ```
//! use libvctrl_core::object::TreeBuilder;
//! use libvctrl_handler::{EntryKind, Hash, TreeEntry};
//!
//! let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
//! let e1 = TreeEntry::new("a.txt".to_string(), EntryKind::Blob, hash).unwrap();
//! let e2 = TreeEntry::new("b.txt".to_string(), EntryKind::Blob, hash).unwrap();
//!
//! let tree = TreeBuilder::new()
//!     .entry(e1)
//!     .entry(e2)
//!     .build()
//!     .unwrap();
//!
//! assert_eq!(tree.entries().len(), 2);
//! ```

use libvctrl_handler::{EntryKind, Hash, Tree, TreeEntry, VctrlError};

/// A builder for creating [`Tree`](libvctrl_handler::Tree) objects.
///
/// # Purpose
///
/// Provides a fluent interface for accumulating [`TreeEntry`] objects and
/// finalizing them into an immutable [`Tree`].
///
/// # Design Rationale
///
/// Implements the standard builder pattern. It derives [`Default`] for easy
/// instantiation. The `build` method consumes `self` and delegates to
/// [`Tree::new`](libvctrl_handler::Tree::new), which enforces the structural
/// invariant that tree entries must be sorted by name and contain no
/// duplicates.
///
/// # Field Privacy
///
/// The internal `entries` vector is private. This ensures that entries are
/// only added through the builder methods, preserving the linear construction
/// flow and preventing accidental modification of the final sorted structure.
///
/// # Memory Layout
///
/// The builder owns a [`Vec<TreeEntry>`], which grows as entries are added.
/// Each [`TreeEntry`] owns a [`String`] and a [`Hash`]. When `build` is
/// called, the vector is moved into the resulting [`Tree`] without cloning.
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
    /// # Design Rationale
    ///
    /// This is a `const fn`, allowing instantiation in compile-time contexts.
    /// It initializes the internal entry vector as empty, so no heap
    /// allocation occurs until the first entry is added.
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
    /// # Design Rationale
    ///
    /// This method provides a fast path for adding entries that have already
    /// been constructed and validated. It does not perform additional
    /// validation beyond what [`TreeEntry::new`] already did. This is useful
    /// when entries are created elsewhere and simply need to be collected.
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
    /// # Design Rationale
    ///
    /// This is a convenience method that encapsulates [`TreeEntry::new`]. It
    /// returns a `Result` to gracefully handle name validation failures without
    /// breaking the builder chain. This allows callers to build entries
    /// directly from primitive fields.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the name is empty, exceeds the maximum
    /// length, or contains forbidden path characters (`/`, `.`, `..`).
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

    /// Consumes the builder and returns a finalized
    /// [`Tree`](libvctrl_handler::Tree).
    ///
    /// # Design Rationale
    ///
    /// This method consumes `self` to enforce a linear flow. It delegates to
    /// [`Tree::new`](libvctrl_handler::Tree::new), which enforces the
    /// structural invariant that entries must be sorted by name and contain
    /// no duplicates.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::InvalidName`] if the entries are not sorted or
    /// contain duplicates. The error message identifies the conflicting
    /// pair.
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
///
/// Provides a fluent interface for assembling a tree entry's data (name, kind,
/// hash) before finalizing it into an immutable object.
///
/// # Design Rationale
///
/// This builder stores the raw components. The `build` method delegates to
/// [`TreeEntry::new`](libvctrl_handler::TreeEntry::new), which performs name
/// validation. This separation allows callers to prepare components
/// incrementally and defer validation until the entry is finalized.
///
/// # Field Privacy
///
/// All fields are private. The builder owns the name, kind, and hash, and
/// moves them into the final [`TreeEntry`] on build. No additional allocation
/// or cloning occurs.
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
    /// # Design Rationale
    ///
    /// This is a `const fn` that simply stores the components. Validation is
    /// deferred to the [`build`](Self::build) step, allowing callers to
    /// construct a builder without immediately handling invalid input.
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
    /// # Design Rationale
    ///
    /// This method consumes `self` and delegates to
    /// [`TreeEntry::new`](libvctrl_handler::TreeEntry::new), enforcing name
    /// length, emptiness, and path traversal constraints. It provides the
    /// single transition point from raw components to a validated, immutable
    /// entry.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::InvalidName`] if the name is empty, exceeds the
    /// maximum length, or contains forbidden path characters.
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
