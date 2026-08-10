//! System-wide constants and limits for `libvctrl_handler`.
//!
//! # Purpose
//! This module centralizes all magic numbers and structural limits used across
//! the version control system. Defining them here ensures that validation
//! logic in type constructors, encoders, and storage backends remains
//! consistent and easily tunable.
//!
//! # Design rationale
//! - **Resource Exhaustion Prevention**: Limits like [`MAX_BLOB_SIZE`] and
//!   [`MAX_TREE_ENTRIES`] exist to prevent malicious or accidental resource
//!   exhaustion (e.g., a 100GB blob crashing the indexer). They define safe
//!   upper bounds for memory and disk usage.
//! - **Compatibility**: [`HASH_LENGTH`] is fixed to 64 bytes (512 bits),
//!   aligning with SHA-512 or BLAKE3 (extended) outputs.
//! - **Wire Format Separation**: The [`entry_mode`] submodule isolates raw
//!   Unix-style mode bits used in the serialized tree format from the
//!   high-level [`EntryKind`](crate::EntryKind) enum.

/// The expected length of a [`Hash`](crate::Hash) in bytes (64 bytes = 512 bits).
///
/// # Examples
///
/// ```
/// use libvctrl_handler::constants::HASH_LENGTH;
///
/// assert_eq!(HASH_LENGTH, 64_u64);
/// ```
pub const HASH_LENGTH: usize = 64;

/// The maximum allowed byte length for names (e.g., branches, tags, file entries).
///
/// # Design rationale
/// 255 bytes is a common filesystem limit for filenames. Enforcing this
/// ensures compatibility with most mainstream filesystems when objects are
/// checked out to disk.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::constants::MAX_NAME_LENGTH;
///
/// assert_eq!(MAX_NAME_LENGTH, 255_u64);
/// ```
pub const MAX_NAME_LENGTH: u64 = 255;

/// The maximum allowed size in bytes for a single [`Blob`](crate::Blob).
///
/// # Design rationale
/// Set to 100 MiB to prevent out-of-memory errors when loading objects into
/// memory for hashing or encoding, while still accommodating large binary
/// assets like media files.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::constants::MAX_BLOB_SIZE;
///
/// assert_eq!(MAX_BLOB_SIZE, 100 * 1024 * 1024_u64);
/// ```
pub const MAX_BLOB_SIZE: u64 = 100 * 1024 * 1024;

/// The maximum number of entries allowed in a single [`Tree`](crate::Tree).
///
/// # Design rationale
/// Set to 100,000 to prevent pathologically large directory listings that
/// would degrade traversal and encoding performance.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::constants::MAX_TREE_ENTRIES;
///
/// assert_eq!(MAX_TREE_ENTRIES, 100_000_u64);
/// ```
pub const MAX_TREE_ENTRIES: u64 = 100_000;

/// The maximum allowed byte length for commit or tag messages.
///
/// # Design rationale
/// Set to 1 MiB to allow detailed changelogs while preventing abuse via
/// gigabyte-sized text payloads.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::constants::MAX_MESSAGE_LENGTH;
///
/// assert_eq!(MAX_MESSAGE_LENGTH, 1024 * 1024_u64);
/// ```
pub const MAX_MESSAGE_LENGTH: u64 = 1024 * 1024;

/// Standard Unix filesystem mode bits used in the serialized tree format.
///
/// # Purpose
/// In version control systems like Git, tree entries store raw 32-bit mode
/// integers to represent file types and permissions. This module provides
/// those exact constants.
///
/// # Design rationale
/// These constants are separated into their own module to keep the global
/// namespace clean. They represent the *wire format* and storage format,
/// distinct from the logical [`EntryKind`](crate::EntryKind) enum used in
/// Rust memory. An encoder or decoder implementation will use these when
/// translating between [`TreeEntry`](crate::TreeEntry) and raw bytes.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::constants::entry_mode;
///
/// assert_eq!(entry_mode::BLOB, 0o100_644);
/// assert_eq!(entry_mode::TREE, 0o040_000);
/// ```
pub mod entry_mode {
    /// Mode for a regular, non-executable file.
    pub const BLOB: u32 = 0o100_644;

    /// Mode for an executable file.
    pub const EXECUTABLE: u32 = 0o100_755;

    /// Mode for a symbolic link.
    pub const SYMLINK: u32 = 0o120_000;

    /// Mode for a subdirectory (tree).
    pub const TREE: u32 = 0o040_000;

    /// Mode for a submodule (gitlink).
    pub const SUBMODULE: u32 = 0o160_000;
}
