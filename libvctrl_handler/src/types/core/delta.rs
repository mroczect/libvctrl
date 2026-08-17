//! Delta and change types.
//!
//! # Architecture
//! This module provides structures for representing structural differences
//! (deltas) between two Git trees. Instead of loading full file contents into
//! memory to compute diffs, the engine operates on hashes and paths. This
//! "zero-knowledge" approach allows for extremely fast diffing of massive
//! repositories with a minimal memory footprint.
//!
//! # Design Rationale: Type-State via Factory Methods
//! The [`FileDelta`] struct uses private fields and `const fn` factory methods
//! (e.g., [`FileDelta::added`], [`FileDelta::deleted`]). This is a deliberate
//! architectural choice to enforce invariants at compile time. By restricting
//! construction to these factory methods, the crate guarantees that an `Added`
//! delta never has an `old_hash`, and a `Deleted` delta never has a `new_hash`.
//! Consumers cannot accidentally construct an invalid delta state.

use std::path::{Path, PathBuf};

use crate::Hash;

/// The kind of change between two objects.
///
/// # Why this exists
/// Classifies the nature of a modification between two tree states. By using a
/// strongly-typed enum instead of bitflags or strings, the compiler enforces
/// exhaustive matching, ensuring that diff consumers handle all possible change
/// types (or explicitly ignore them via a catch-all).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChangeKind {
    /// The object was added.
    Added,
    /// The object was deleted.
    Deleted,
    /// The object was modified.
    Modified,
    /// The object type changed (e.g., blob to tree).
    TypeChange,
    /// The object was renamed.
    Renamed,
    /// The object was copied.
    Copied,
}

/// A single file delta between two trees.
///
/// # Why this exists
/// Represents the atomic unit of a tree diff. It maps a file path transition
/// (if any) to the change in its content hash. This allows UI renderers or merge
/// drivers to understand exactly what happened to a specific file without needing
/// to inspect the underlying blob data.
///
/// # How it works
/// The struct holds the current `path`, an optional `old_path` (for renames/copies),
/// and optional `old_hash` and `new_hash` values. The presence of these hashes is
/// directly correlated to the [`ChangeKind`], an invariant strictly maintained by
/// the constructor methods.
///
/// # Examples
///
/// Creating a delta for an added file:
///
/// ```
/// # use libvctrl_handler::types::core::delta::FileDelta;
/// # use libvctrl_handler::Hash;
/// # let hash = Hash::from_bytes(&[0_u8; 64]).unwrap();
/// let delta = FileDelta::added("src/main.rs".into(), hash);
/// assert!(delta.is_added());
/// assert!(delta.old_hash().is_none());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileDelta {
    path: PathBuf,
    old_path: Option<PathBuf>,
    old_hash: Option<Hash>,
    new_hash: Option<Hash>,
    kind: ChangeKind,
}

impl FileDelta {
    /// Creates a new `FileDelta` representing an addition.
    ///
    /// # How it works
    /// Initializes the delta with the new path and hash, leaving `old_path` and
    /// `old_hash` as `None` to reflect that the file did not exist in the old tree.
    #[must_use]
    pub const fn added(path: PathBuf, new_hash: Hash) -> Self {
        Self {
            path,
            old_path: None,
            old_hash: None,
            new_hash: Some(new_hash),
            kind: ChangeKind::Added,
        }
    }

    /// Creates a new `FileDelta` representing a deletion.
    ///
    /// # How it works
    /// Initializes the delta with the old path and hash, leaving `new_hash` as
    /// `None` to reflect that the file no longer exists in the new tree.
    #[must_use]
    pub const fn deleted(path: PathBuf, old_hash: Hash) -> Self {
        Self {
            path,
            old_path: None,
            old_hash: Some(old_hash),
            new_hash: None,
            kind: ChangeKind::Deleted,
        }
    }

    /// Creates a new `FileDelta` representing a modification.
    ///
    /// # How it works
    /// The path remains the same, but both `old_hash` and `new_hash` are populated
    /// to indicate that the file content changed while its location did not.
    #[must_use]
    pub const fn modified(path: PathBuf, old_hash: Hash, new_hash: Hash) -> Self {
        Self {
            path,
            old_path: None,
            old_hash: Some(old_hash),
            new_hash: Some(new_hash),
            kind: ChangeKind::Modified,
        }
    }

    /// Creates a new `FileDelta` representing a type change.
    ///
    /// # How it works
    /// Similar to a modification, but signifies that the Git object type changed
    /// (e.g., a regular file became a symbolic link). Both hashes are populated.
    #[must_use]
    pub const fn type_change(path: PathBuf, old_hash: Hash, new_hash: Hash) -> Self {
        Self {
            path,
            old_path: None,
            old_hash: Some(old_hash),
            new_hash: Some(new_hash),
            kind: ChangeKind::TypeChange,
        }
    }

    /// Creates a new `FileDelta` representing a rename.
    ///
    /// # How it works
    /// Populates both `path` (the new path) and `old_path` (the original path).
    /// Depending on the diff algorithm, the hash might remain the same or change
    /// if the file was also modified during the rename.
    #[must_use]
    pub const fn renamed(
        old_path: PathBuf,
        new_path: PathBuf,
        old_hash: Hash,
        new_hash: Hash,
    ) -> Self {
        Self {
            path: new_path,
            old_path: Some(old_path),
            old_hash: Some(old_hash),
            new_hash: Some(new_hash),
            kind: ChangeKind::Renamed,
        }
    }

    /// Creates a new `FileDelta` representing a copy.
    ///
    /// # How it works
    /// Similar to a rename, but indicates the original file still exists at
    /// `old_path`. The `path` field holds the destination of the copy.
    #[must_use]
    pub const fn copied(
        old_path: PathBuf,
        new_path: PathBuf,
        old_hash: Hash,
        new_hash: Hash,
    ) -> Self {
        Self {
            path: new_path,
            old_path: Some(old_path),
            old_hash: Some(old_hash),
            new_hash: Some(new_hash),
            kind: ChangeKind::Copied,
        }
    }

    /// Returns the path of the changed file.
    ///
    /// # How it works
    /// Returns a reference to the current (new) path of the file. If the file was
    /// deleted, this returns the path it used to have.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the old path if the file was renamed or copied.
    ///
    /// # How it works
    /// Returns `Some(&Path)` only if the [`ChangeKind`] is `Renamed` or `Copied`.
    /// Otherwise, it returns `None`.
    #[must_use]
    pub fn old_path(&self) -> Option<&Path> {
        self.old_path.as_deref()
    }

    /// Returns the old hash, if the file previously existed.
    ///
    /// # How it works
    /// Returns `None` for additions, as there is no previous state.
    #[must_use]
    pub const fn old_hash(&self) -> Option<Hash> {
        self.old_hash
    }

    /// Returns the new hash, if the file exists now.
    ///
    /// # How it works
    /// Returns `None` for deletions, as the file no longer exists in the new state.
    #[must_use]
    pub const fn new_hash(&self) -> Option<Hash> {
        self.new_hash
    }

    /// Returns the kind of change.
    ///
    /// # How it works
    /// Provides the [`ChangeKind`] enum variant associated with this delta.
    #[must_use]
    pub const fn kind(&self) -> ChangeKind {
        self.kind
    }

    /// Returns `true` if this is an addition.
    #[must_use]
    pub fn is_added(&self) -> bool {
        self.kind == ChangeKind::Added
    }

    /// Returns `true` if this is a deletion.
    #[must_use]
    pub fn is_deleted(&self) -> bool {
        self.kind == ChangeKind::Deleted
    }

    /// Returns `true` if this is a modification.
    #[must_use]
    pub fn is_modified(&self) -> bool {
        self.kind == ChangeKind::Modified
    }

    /// Returns `true` if this is a type change.
    #[must_use]
    pub fn is_type_change(&self) -> bool {
        self.kind == ChangeKind::TypeChange
    }

    /// Returns `true` if this is a rename.
    #[must_use]
    pub fn is_renamed(&self) -> bool {
        self.kind == ChangeKind::Renamed
    }

    /// Returns `true` if this is a copy.
    #[must_use]
    pub fn is_copied(&self) -> bool {
        self.kind == ChangeKind::Copied
    }
}

/// A collection of file deltas between two trees.
///
/// # Why this exists
/// Aggregates all individual [`FileDelta`]s into a single, cohesive structure.
/// This provides a clean interface for consumers to query the total number of
/// changes, iterate over them, or pass the entire diff result between functions.
///
/// # How it works
/// Internally, it is a thin wrapper around a `Vec<FileDelta>`. It implements
/// `IntoIterator` for both owned and borrowed values, allowing consumers to
/// easily loop over the changes using `for` loops without needing to call
/// `.iter()` explicitly.
///
/// # Examples
///
/// Creating a `TreeDelta` and iterating over its changes:
///
/// ```
/// # use libvctrl_handler::types::core::delta::{FileDelta, TreeDelta};
/// # use libvctrl_handler::Hash;
/// # let hash = Hash::from_bytes(&[0_u8; 64]).unwrap();
/// let delta1 = FileDelta::added("file1.txt".into(), hash);
/// let delta2 = FileDelta::deleted("file2.txt".into(), hash);
/// let tree_delta = TreeDelta::from_changes(vec![delta1, delta2]);
///
/// assert_eq!(tree_delta.len(), 2);
/// for delta in &tree_delta {
///     assert!(delta.is_added() || delta.is_deleted());
/// }
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TreeDelta {
    changes: Vec<FileDelta>,
}

impl TreeDelta {
    /// Creates an empty `TreeDelta`.
    ///
    /// # How it works
    /// Initializes the internal vector without allocating capacity until elements
    /// are added. This is a `const fn`, allowing static initialization.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            changes: Vec::new(),
        }
    }

    /// Creates a `TreeDelta` from a vector of `FileDelta`.
    ///
    /// # How it works
    /// Takes ownership of the provided vector, wrapping it directly. This avoids
    /// unnecessary copying of the deltas.
    #[must_use]
    pub const fn from_changes(changes: Vec<FileDelta>) -> Self {
        Self { changes }
    }

    /// Returns the number of changes.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.changes.len()
    }

    /// Returns `true` if there are no changes.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// Iterates over the changes.
    ///
    /// # How it works
    /// Returns a standard slice iterator (`std::slice::Iter`), borrowing from the
    /// internal vector. This is highly efficient as it involves no allocations.
    pub fn iter(&self) -> std::slice::Iter<'_, FileDelta> {
        self.changes.iter()
    }

    /// Returns the changes.
    ///
    /// # How it works
    /// Returns a slice `&[FileDelta]` borrowing from the internal vector. This allows
    /// callers to index or iterate over the changes without taking ownership.
    #[must_use]
    pub fn changes(&self) -> &[FileDelta] {
        &self.changes
    }
}

impl IntoIterator for TreeDelta {
    type Item = FileDelta;
    type IntoIter = std::vec::IntoIter<FileDelta>;

    /// Consumes the `TreeDelta` and returns an owned iterator.
    ///
    /// # How it works
    /// Converts the internal `Vec<FileDelta>` into `std::vec::IntoIter`, yielding
    /// owned `FileDelta` items. This is useful when the consumer needs to take
    /// ownership of the deltas, e.g., to send them to another thread.
    fn into_iter(self) -> Self::IntoIter {
        self.changes.into_iter()
    }
}

impl<'a> IntoIterator for &'a TreeDelta {
    type Item = &'a FileDelta;
    type IntoIter = std::slice::Iter<'a, FileDelta>;

    /// Borrows the `TreeDelta` and returns a borrowing iterator.
    ///
    /// # How it works
    /// Delegates to [`TreeDelta::iter`], yielding `&FileDelta` items. This allows
    /// ergonomic `for delta in &tree_delta` loops without consuming the struct.
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
