use crate::constants::entry_mode;

/// The kind of an entry in a Git tree.
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
