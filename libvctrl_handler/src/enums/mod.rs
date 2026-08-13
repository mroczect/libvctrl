//! Logical object type enumerations for `libvctrl_handler`.
//!
//! # Purpose
//!
//! This module defines high-level, discriminative types that categorize the
//! logical kind of an object in the version control system. Rather than
//! exposing raw filesystem mode bits, it provides a semantic enum
//! ([`EntryKind`]) that distinguishes between regular files, executable
//! files, symbolic links, subdirectories, and submodule references.
//!
//! # Design Rationale
//!
//! The enum is kept separate from the low-level mode constants (like those
//! in [`crate::constants::entry_mode`]) to decouple the abstract data model
//! ("what kind of object is this?") from the serialized Unix-style
//! representation ("what permission bits does this object have?"). This
//! allows different backends to map their own mode systems to a uniform set
//! of logical kinds, and makes the core data structures independent of
//! POSIX-specific details.
//!
//! The module itself is deliberately small; it contains only the enum and
//! its documentation. This avoids pulling in dependencies or bloating the
//! crate with logic that belongs to higher-level components (e.g., a decoder
//! implementation).
//!
//! # Internal Module Structure
//!
//! The module is organized into a `core` submodule that houses the actual
//! enum definition. The re-export at this level lifts [`EntryKind`] into the
//! `enums` namespace, so consumers can write:
//!
//! ```
//! use libvctrl_handler::enums::EntryKind;
//! ```
//!
//! instead of the longer:
//!
//! ```
//! use libvctrl_handler::enums::core::entry_kind::EntryKind;
//! ```
//!
//! This indirection keeps the public API stable even if the internal file
//! layout changes in the future.
//!
//! # How It Relates to Other Crate Items
//!
//! - [`EntryKind`] is used by [`TreeEntry`](crate::TreeEntry) to describe
//!   whether an entry points to a blob or a tree.
//! - The `entry_mode` constants in
//!   [`crate::constants::entry_mode`] define the raw Unix mode bits that
//!   correspond to each logical kind. Decoder and encoder implementations are
//!   responsible for translating between the two representations.
//!
//! # Examples
//!
//! Comparing logical kinds:
//!
//! ```
//! use libvctrl_handler::enums::EntryKind;
//!
//! assert_ne!(EntryKind::Blob, EntryKind::Tree);
//! assert_ne!(EntryKind::Executable, EntryKind::Blob);
//! assert_ne!(EntryKind::Symlink, EntryKind::Tree);
//! assert_ne!(EntryKind::Submodule, EntryKind::Tree);
//! ```

pub mod core;

/// Re-export of [`EntryKind`] from the internal `core` submodule.
///
/// This public re-export ensures that downstream code can access the enum
/// through the stable path `libvctrl_handler::enums::EntryKind` without
/// depending on the internal module layout.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::enums::EntryKind;
///
/// let kind = EntryKind::Blob;
/// assert_eq!(kind, EntryKind::Blob);
/// ```
pub use core::entry_kind::EntryKind;
