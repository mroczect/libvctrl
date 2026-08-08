//! Core enums used by the fundamental types.
//!
//! This module defines the enumeration types that are referenced throughout the
//! `libvctrl` ecosystem. They are intentionally kept small and stable to avoid
//! fragmentation. Currently the only enum is [`EntryKind`], which describes the
//! type of a tree entry.

/// The kind of an entry inside a [`Tree`](crate::Tree).
///
/// A tree object represents a directory listing. Each entry has a name,
/// a kind (this enum), and a hash pointing to the actual content. The kind
/// tells the version control system whether the hash refers to a file
/// (a [`Blob`](crate::Blob)) or to another directory (a [`Tree`](crate::Tree)
/// itself).
///
/// # Why `#[non_exhaustive]`?
///
/// This enum is marked `#[non_exhaustive]` so that new variants can be added
/// in the future without breaking existing code. For example, support for
/// symlinks, submodules, or custom object types may be added later. Users must
/// write their `match` expressions with a wildcard arm (`_`) to be forward‑compatible.
///
/// # Examples
///
/// ```rust
/// use libvctrl_handler::{EntryKind, Hash, TreeEntry};
///
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// let blob_entry = TreeEntry::new("file.txt".into(), EntryKind::Blob, hash).unwrap();
/// assert_eq!(blob_entry.kind(), EntryKind::Blob);
///
/// let tree_entry = TreeEntry::new("subdir".into(), EntryKind::Tree, hash).unwrap();
/// assert_eq!(tree_entry.kind(), EntryKind::Tree);
/// ```
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EntryKind {
    /// A regular file. The entry's hash points to a [`Blob`](crate::Blob).
    Blob,

    /// A sub‑directory. The entry's hash points to another [`Tree`](crate::Tree).
    Tree,
}
