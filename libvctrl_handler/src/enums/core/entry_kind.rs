//! Core enum definitions for Git object types.
//!
//! # Architecture
//! This module replaces raw integer mode bits (e.g., `0o100644`) with strongly-typed
//! enumerations. By using [`EntryKind`], the compiler enforces exhaustive matching,
//! preventing invalid or unrecognized file modes from propagating through the system.
//!
//! # Design Rationale
//! Raw mode bits are error-prone; a typo like `0o100646` is a valid integer but an invalid Git
//! mode. Enum variants encode domain logic directly into the type system, making the API
//! self-documenting and eliminating entire classes of runtime errors associated with
//! bit manipulation.

use crate::constants::entry_mode;

/// The kind of an entry in a Git tree.
///
/// # Why this exists
/// Git stores filesystem objects (files, directories, symlinks) in tree objects.
/// Each entry is identified by a 32-bit mode. This enum abstracts those raw bits into
/// a strongly-typed domain model. It ensures that only valid Git object types can be
/// represented, preventing invalid states (e.g., a mode of `0o000000`) from being
/// constructed.
///
/// # How it works
/// The enum is marked as `#[non_exhaustive]` to allow for the addition of new Git
/// object types in the future without breaking downstream API compatibility. Consumers
/// must include a `_` catch-all arm when matching.
///
/// # Examples
///
/// ```
/// # use libvctrl_handler::enums::EntryKind;
/// let kind = EntryKind::Blob;
/// assert_eq!(kind.mode(), 0o100_644);
/// ```
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EntryKind {
    /// A regular file.
    Blob,
    /// An executable file.
    Executable,
    /// A symbolic link.
    Symlink,
    /// A directory (tree).
    Tree,
    /// A submodule commit.
    Submodule,
}

impl EntryKind {
    /// Returns the Git mode bits for this entry kind.
    ///
    /// # Why this exists
    /// Provides a seamless conversion from the strongly-typed [`EntryKind`] back to the
    /// raw `u32` mode bits required for serializing Git tree objects or interacting with
    /// lower-level filesystem APIs.
    ///
    /// # How it works
    /// Implemented as a `const fn`, this allows the conversion to be evaluated at compile
    /// time if the variant is known statically. This incurs zero runtime cost and enables
    /// its use in other `const` contexts.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::enums::EntryKind;
    /// assert_eq!(EntryKind::Executable.mode(), 0o100_755);
    /// ```
    #[must_use]
    pub const fn mode(self) -> u32 {
        match self {
            Self::Blob => entry_mode::BLOB,
            Self::Executable => entry_mode::EXECUTABLE,
            Self::Symlink => entry_mode::SYMLINK,
            Self::Tree => entry_mode::TREE,
            Self::Submodule => entry_mode::SUBMODULE,
        }
    }

    /// Converts raw Git mode bits into an [`EntryKind`].
    ///
    /// # Why this exists
    /// When parsing raw Git packfiles or loose objects, data is read as integers. This
    /// function safely translates those integers into the domain model. By returning an
    /// `Option`, it gracefully handles malformed or unrecognized mode bits without
    /// panicking, allowing the caller to decide whether to ignore the entry or error out.
    ///
    /// # How it works
    /// Matches the input against known Git mode constants defined in [`entry_mode`].
    /// If no match is found, `None` is returned. Like [`mode`](Self::mode), this is a
    /// `const fn` to enable compile-time evaluation.
    ///
    /// # Examples
    ///
    /// Parsing a valid mode:
    ///
    /// ```
    /// # use libvctrl_handler::enums::EntryKind;
    /// let mode = 0o120_000; // Symlink
    /// let kind = EntryKind::from_mode(mode);
    /// assert_eq!(kind, Some(EntryKind::Symlink));
    /// ```
    ///
    /// Handling an invalid mode:
    ///
    /// ```
    /// # use libvctrl_handler::enums::EntryKind;
    /// let invalid_mode = 0o000_000;
    /// assert_eq!(EntryKind::from_mode(invalid_mode), None);
    /// ```
    #[must_use]
    pub const fn from_mode(mode: u32) -> Option<Self> {
        match mode {
            entry_mode::BLOB => Some(Self::Blob),
            entry_mode::EXECUTABLE => Some(Self::Executable),
            entry_mode::SYMLINK => Some(Self::Symlink),
            entry_mode::TREE => Some(Self::Tree),
            entry_mode::SUBMODULE => Some(Self::Submodule),
            _ => None,
        }
    }
}
