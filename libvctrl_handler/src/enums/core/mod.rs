//! Core enum definitions for version control object kinds.
//!
//! # Purpose
//!
//! This submodule is the canonical home for the `EntryKind` enum, which
//! discriminates the logical kind of a tree entry (regular file, executable,
//! symlink, directory, or submodule). The enum is defined in the nested
//! `entry_kind` module and is re-exported by parent modules for convenient
//! access.
//!
//! # Design Rationale
//!
//! The crate intentionally separates low-level mode constants from logical
//! object kinds. This module holds the logical side of that separation. By
//! keeping the enum definition in a dedicated module, the public API remains
//! stable even if the internal file structure changes.
//!
//! # Internal Structure
//!
//! The module is arranged as follows:
//!
//! - `entry_kind`: contains the actual `EntryKind` enum definition.
//! - The parent module `enums` re-exports `EntryKind` so that callers can
//!   write `use libvctrl_handler::enums::EntryKind;`.
//! - The crate root re-exports the same enum as
//!   `libvctrl_handler::EntryKind`.
//!
//! # How to Use
//!
//! You can import the enum using any of the public re-export paths:
//!
//! ```
//! use libvctrl_handler::enums::core::entry_kind::EntryKind;
//!
//! let kind = EntryKind::Blob;
//! assert_ne!(kind, EntryKind::Tree);
//! ```
//!
//! Or, more conveniently:
//!
//! ```
//! use libvctrl_handler::EntryKind;
//!
//! assert_ne!(EntryKind::Blob, EntryKind::Executable);
//! ```
//!
//! # Relationship to Other Modules
//!
//! - `crate::constants::entry_mode` defines raw Unix mode bits for
//!   serialization. Decoders and encoders translate between those constants
//!   and `EntryKind`.
//! - `crate::TreeEntry` uses `EntryKind` as a field to describe the
//!   kind of object it points to.

/// The `EntryKind` enum,
/// which discriminates file, directory, symlink, and submodule entries.
///
/// # Purpose
///
/// This module contains the single public enum used throughout the crate to
/// represent the logical kind of a tree entry. Keeping it in its own file
/// makes the definition easy to locate and keeps the parent module
/// documentation focused on the broader organization.
///
/// # Examples
///
/// Importing directly from this module:
///
/// ```
/// use libvctrl_handler::enums::core::entry_kind::EntryKind;
///
/// let blob = EntryKind::Blob;
/// let tree = EntryKind::Tree;
///
/// assert_ne!(blob, tree);
/// ```
///
/// The same type is also available at the crate root:
///
/// ```
/// use libvctrl_handler::EntryKind;
///
/// assert_eq!(EntryKind::Symlink, EntryKind::Symlink);
/// ```
pub mod entry_kind;
