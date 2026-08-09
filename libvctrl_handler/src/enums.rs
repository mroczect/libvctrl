//! Logical object type enumerations for `libvctrl_handler`.
//!
//! # Purpose
//! This module defines high-level, discriminative types that categorize the
//! logical kind of an object in the version control system (e.g., whether a
//! tree entry represents a file or a subdirectory).
//!
//! # Design rationale
//! The enums here are intentionally kept separate from the raw filesystem mode
//! constants (like those in [`crate::constants::entry_mode`]). This decouples
//! the abstract data model (what kind of object is this?) from the serialized
//! filesystem representation (what Unix permissions does this object have?).

/// Represents the logical kind of an entry in a version control tree.
///
/// # Purpose
/// A [`TreeEntry`](crate::TreeEntry) must distinguish between a file ([`Blob`](crate::Blob))
/// and a subdirectory ([`Tree`](crate::Tree)). This enum provides that
/// discrimination without tying the type to specific filesystem permission bits.
///
/// # Design rationale
/// - **`#[non_exhaustive]`**: This attribute ensures that adding new variants
///   in the future (for example, a hypothetical `Commit` variant for submodules
///   pointing directly to commit objects) will not break exhaustive `match`
///   statements in downstream code.
/// - **`Copy` and `Clone`**: The enum is a lightweight, 1-byte tag (or similar
///   primitive). Marking it `Copy` makes it trivially cheap to pass by value
///   without cloning overhead.
/// - **`Hash` and `Eq`**: Allows entries to be grouped, compared, or used as
///   keys in hash maps if a storage backend needs to index them.
///
/// # Internal mechanism
/// This is a standard C-like enum. Rust guarantees it occupies the minimum
/// required memory (typically a single byte).
///
/// # Examples
///
/// ```
/// use libvctrl_handler::EntryKind;
///
/// let kind = EntryKind::Blob;
/// assert_eq!(kind, EntryKind::Blob);
/// assert_ne!(kind, EntryKind::Tree);
/// ```
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EntryKind {
    /// The entry points to a [`Blob`](crate::Blob) (regular file content).
    Blob,

    /// The entry points to another [`Tree`](crate::Tree) (subdirectory).
    Tree,
}
