//! Core enums used by the fundamental types.

/// The kind of an entry inside a [`Tree`](crate::Tree).
///
/// In a version control system, a tree represents a directory.
/// Each entry can be either a file ([`Blob`](crate::Blob)) or a
/// sub‑directory ([`Tree`](crate::Tree) itself).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EntryKind {
    /// A regular file. The entry's hash points to a [`Blob`](crate::Blob).
    Blob,
    /// A sub‑directory. The entry's hash points to another [`Tree`](crate::Tree).
    Tree,
}
