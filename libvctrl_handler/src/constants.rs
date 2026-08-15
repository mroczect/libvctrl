//! Global constants for the crate.

/// The length of a hash in bytes (e.g., SHA-512).
pub const HASH_LENGTH: usize = 64;

/// The maximum allowed length for names (in bytes).
pub const MAX_NAME_LENGTH: u64 = 255;

/// The maximum allowed size for blob objects (in bytes).
pub const MAX_BLOB_SIZE: u64 = 100 * 1024 * 1024;

/// The maximum number of entries allowed in a tree.
pub const MAX_TREE_ENTRIES: u64 = 100_000;

/// The maximum allowed length for commit/tag messages (in bytes).
pub const MAX_MESSAGE_LENGTH: u64 = 1024 * 1024;

/// Git object entry modes.
pub mod entry_mode {
    /// Regular file mode.
    pub const BLOB: u32 = 0o100_644;
    /// Executable file mode.
    pub const EXECUTABLE: u32 = 0o100_755;
    /// Symbolic link mode.
    pub const SYMLINK: u32 = 0o120_000;
    /// Directory (tree) mode.
    pub const TREE: u32 = 0o40_000;
    /// Submodule commit mode.
    pub const SUBMODULE: u32 = 0o160_000;
}
