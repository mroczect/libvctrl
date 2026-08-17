//! # Tree Builders
//!
//! This module provides ergonomic builders for constructing [`Tree`] and
//! [`TreeEntry`] objects.
//!
//! A [`Tree`] is a sorted collection of entries. The invariant is enforced by
//! [`Tree::new`], which rejects unsorted or duplicate entry names. These
//! builders defer that validation to the final `build()` step, allowing
//! callers to assemble entries incrementally.
//!
//! The module exposes two builder types:
//!
//! - [`TreeBuilder`] for building a full tree from individual entries.
//! - [`TreeEntryBuilder`] for building a single entry.

use libvctrl_handler::{EntryKind, Hash, Tree, TreeEntry, VctrlError};

/// A builder for creating [`Tree`] objects.
///
/// `TreeBuilder` accumulates [`TreeEntry`] values and produces a validated
/// [`Tree`] when [`build`](Self::build) is called.
///
/// # Why this struct exists
///
/// A [`Tree`] requires its entries to be sorted and free of duplicates. If
/// callers constructed a [`Tree`] directly and supplied entries one by one,
/// they would need to sort and validate manually. This builder centralizes
/// that concern and provides a chainable API.
///
/// # How it works
///
/// The builder stores entries in an internal `Vec<TreeEntry>`. The `entry` and
/// `add_entry` methods push entries without performing any ordering checks.
/// Validation occurs only when [`build`](Self::build) consumes the builder and
/// calls [`Tree::new`], which enforces the ordering invariant.
///
/// # Examples
///
/// Building a tree with two sorted entries:
///
/// ```
/// # use libvctrl_core::object::TreeBuilder;
/// # use libvctrl_handler::{EntryKind, Hash};
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
///
/// let tree = TreeBuilder::new()
///     .add_entry("a.txt".to_owned(), EntryKind::Blob, hash)
///     .unwrap()
///     .add_entry("b.txt".to_owned(), EntryKind::Blob, hash)
///     .unwrap()
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
    /// Creates a new `TreeBuilder` with no entries.
    ///
    /// The builder is initially empty. Use [`entry`](Self::entry) or
    /// [`add_entry`](Self::add_entry) to add entries, then call
    /// [`build`](Self::build) to construct the [`Tree`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_core::object::TreeBuilder;
    /// let builder = TreeBuilder::new();
    /// let tree = builder.build().unwrap();
    /// assert!(tree.entries().is_empty());
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Adds an existing [`TreeEntry`].
    ///
    /// This method consumes the builder and returns a new builder with the
    /// given entry appended. No validation is performed at this point.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_core::object::{TreeBuilder, TreeEntryBuilder};
    /// # use libvctrl_handler::{EntryKind, Hash};
    /// let hash = Hash::from_bytes(&[1u8; 64]).unwrap();
    /// let entry = TreeEntryBuilder::new("file.txt".to_owned(), EntryKind::Blob, hash)
    ///     .build()
    ///     .unwrap();
    ///
    /// let tree = TreeBuilder::new()
    ///     .entry(entry)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(tree.entries().len(), 1);
    /// ```
    #[must_use]
    pub fn entry(mut self, entry: TreeEntry) -> Self {
        self.entries.push(entry);
        self
    }

    /// Creates and adds a new [`TreeEntry`].
    ///
    /// This method consumes the builder, constructs a [`TreeEntry`] using
    /// [`TreeEntry::new`], appends it, and returns the updated builder.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the entry name is invalid according to
    /// [`TreeEntry::new`]. No ordering validation is performed here; it is
    /// deferred to [`build`](Self::build).
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_core::object::TreeBuilder;
    /// # use libvctrl_handler::{EntryKind, Hash};
    /// let hash = Hash::from_bytes(&[2u8; 64]).unwrap();
    ///
    /// let builder = TreeBuilder::new()
    ///     .add_entry("a.txt".to_owned(), EntryKind::Blob, hash)
    ///     .unwrap();
    ///
    /// let tree = builder.build().unwrap();
    /// assert_eq!(tree.len(), 1);
    /// # Ok::<(), libvctrl_handler::VctrlError>(())
    /// ```
    ///
    /// This example uses `?` inside a function returning `Result`:
    ///
    /// ```
    /// # use libvctrl_core::object::TreeBuilder;
    /// # use libvctrl_handler::{EntryKind, Hash, VctrlError};
    /// # fn example() -> Result<(), VctrlError> {
    /// let hash = Hash::from_bytes(&[3u8; 64])?;
    /// let tree = TreeBuilder::new()
    ///     .add_entry("a.txt".to_owned(), EntryKind::Blob, hash)?
    ///     .build()?;
    /// assert_eq!(tree.entries().len(), 1);
    /// # Ok(())
    /// # }
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

    /// Builds the [`Tree`].
    ///
    /// Consumes the builder, moves all entries into the new [`Tree`], and
    /// validates the ordering invariant.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the entries are not sorted lexicographically
    /// by name or if duplicate names exist. The exact variant depends on the
    /// `libvctrl_handler` implementation.
    ///
    /// # Examples
    ///
    /// Successful build:
    ///
    /// ```
    /// # use libvctrl_core::object::TreeBuilder;
    /// # use libvctrl_handler::{EntryKind, Hash};
    /// let hash = Hash::from_bytes(&[4u8; 64]).unwrap();
    ///
    /// let tree = TreeBuilder::new()
    ///     .add_entry("a.txt".to_owned(), EntryKind::Blob, hash)
    ///     .unwrap()
    ///     .add_entry("b.txt".to_owned(), EntryKind::Blob, hash)
    ///     .unwrap()
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(tree.entries().len(), 2);
    /// ```
    pub fn build(self) -> Result<Tree, VctrlError> {
        Tree::new(self.entries)
    }
}

/// A builder for creating [`TreeEntry`] objects.
///
/// `TreeEntryBuilder` holds the fields required to construct a [`TreeEntry`]:
/// name, kind, and hash. It performs validation only when
/// [`build`](Self::build) is called.
///
/// # Why this struct exists
///
/// [`TreeEntry::new`] can fail if the name is invalid. This builder gives
/// callers an explicit place to defer that error while keeping construction
/// straightforward. It is particularly useful when entries are generated or
/// configured dynamically.
///
/// # How it works
///
/// The builder stores the three fields by value. `build` moves them into
/// [`TreeEntry::new`] and returns the result, consuming the builder.
///
/// # Examples
///
/// ```
/// # use libvctrl_core::object::TreeEntryBuilder;
/// # use libvctrl_handler::{EntryKind, Hash};
/// let hash = Hash::from_bytes(&[5u8; 64]).unwrap();
/// let entry = TreeEntryBuilder::new(
///     "file.txt".to_owned(),
///     EntryKind::Blob,
///     hash,
/// )
/// .build()
/// .unwrap();
///
/// assert_eq!(entry.name(), "file.txt");
/// assert_eq!(entry.kind(), EntryKind::Blob);
/// ```
#[derive(Debug)]
pub struct TreeEntryBuilder {
    name: String,
    kind: EntryKind,
    hash: Hash,
}

impl TreeEntryBuilder {
    /// Creates a new `TreeEntryBuilder`.
    ///
    /// The builder stores the supplied `name`, `kind`, and `hash`. No
    /// validation is performed until [`build`](Self::build) is called.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_core::object::TreeEntryBuilder;
    /// # use libvctrl_handler::{EntryKind, Hash};
    /// let hash = Hash::from_bytes(&[6u8; 64]).unwrap();
    /// let builder = TreeEntryBuilder::new(
    ///     "file.txt".to_owned(),
    ///     EntryKind::Blob,
    ///     hash,
    /// );
    ///
    /// let entry = builder.build().unwrap();
    /// assert_eq!(entry.name(), "file.txt");
    /// ```
    #[must_use]
    pub const fn new(name: String, kind: EntryKind, hash: Hash) -> Self {
        Self { name, kind, hash }
    }

    /// Builds the [`TreeEntry`].
    ///
    /// Consumes the builder and constructs the [`TreeEntry`] by moving all
    /// fields into [`TreeEntry::new`].
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the entry name is invalid according to
    /// [`TreeEntry::new`]. The exact variant is implementation-defined.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_core::object::TreeEntryBuilder;
    /// # use libvctrl_handler::{EntryKind, Hash};
    /// let hash = Hash::from_bytes(&[7u8; 64]).unwrap();
    /// let entry = TreeEntryBuilder::new(
    ///     "file.txt".to_owned(),
    ///     EntryKind::Blob,
    ///     hash,
    /// )
    /// .build()
    /// .unwrap();
    ///
    /// assert_eq!(entry.name(), "file.txt");
    /// ```
    pub fn build(self) -> Result<TreeEntry, VctrlError> {
        TreeEntry::new(self.name, self.kind, self.hash)
    }
}
