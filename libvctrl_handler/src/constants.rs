//! System-wide constants and limits for `libvctrl_handler`.
//!
//! # Purpose
//!
//! This module centralizes all magic numbers and structural limits used across
//! the version control system. Defining them here ensures that validation
//! logic in type constructors, encoders, and storage backends remains
//! consistent and easily tunable.
//!
//! # Design Rationale
//!
//! - **Resource Exhaustion Prevention**: Limits like [`MAX_BLOB_SIZE`] and
//!   [`MAX_TREE_ENTRIES`] exist to prevent malicious or accidental resource
//!   exhaustion (e.g., a 100GB blob crashing the indexer). They define safe
//!   upper bounds for memory and disk usage.
//! - **Compatibility**: [`HASH_LENGTH`] is fixed to 64 bytes (512 bits),
//!   aligning with SHA-512 or BLAKE3 (extended) outputs.
//! - **Wire Format Separation**: The [`entry_mode`] submodule isolates raw
//!   Unix-style mode bits used in the serialized tree format from the
//!   high-level [`EntryKind`] enum.
//!
//! # How Constants Are Used
//!
//! These constants are referenced by validators such as
//! `validate_name`,
//! `validate_tree_entry_name`, and
//! by constructors like [`Hash::from_bytes`].
//! Keeping them as plain `pub const` items makes them eligible for
//! compile-time evaluation and ensures zero runtime overhead.
//!
//! # Examples
//!
//! Importing frequently used constants from the crate root:
//!
//! ```
//! use libvctrl_handler::{
//!     HASH_LENGTH,
//!     MAX_BLOB_SIZE,
//!     MAX_MESSAGE_LENGTH,
//!     MAX_NAME_LENGTH,
//!     MAX_TREE_ENTRIES,
//! };
//!
//! assert_eq!(HASH_LENGTH, 64);
//! assert_eq!(MAX_NAME_LENGTH, 255);
//! assert_eq!(MAX_BLOB_SIZE, 100 * 1024 * 1024);
//! assert_eq!(MAX_TREE_ENTRIES, 100_000);
//! assert_eq!(MAX_MESSAGE_LENGTH, 1024 * 1024);
//! ```

/// The expected length of a `Hash`(crate::Hash) in bytes (64 bytes = 512 bits).
///
/// # Design Rationale
///
/// A 512-bit digest is chosen to provide a very low probability of collision
/// in a version control system that may store millions of objects. It aligns
/// with cryptographic hash functions such as SHA-512 or BLAKE3 configured for
/// 64-byte output.
///
/// # How It Is Used
///
/// This constant is used by [`Hash::from_bytes`] to
/// validate slice lengths, and by `Hash`(crate::Hash) itself as the size of
/// its internal byte array.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::constants::HASH_LENGTH;
///
/// assert_eq!(HASH_LENGTH, 64);
/// ```
pub const HASH_LENGTH: usize = 64;

/// The maximum allowed byte length for names (e.g., branches, tags, file entries).
///
/// # Design Rationale
///
/// 255 bytes is a common filesystem limit for filenames. Enforcing this
/// ensures compatibility with most mainstream filesystems when objects are
/// checked out to disk. It also prevents pathologically long identifiers
/// that could degrade sorting or hashing performance.
///
/// # How It Is Used
///
/// This constant is checked by `validate_name`
/// and `validate_tree_entry_name`.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::constants::MAX_NAME_LENGTH;
///
/// assert_eq!(MAX_NAME_LENGTH, 255_u64);
/// ```
pub const MAX_NAME_LENGTH: u64 = 255;

/// The maximum allowed size in bytes for a single [`Blob`].
///
/// # Design Rationale
///
/// Set to 100 MiB to prevent out-of-memory errors when loading objects into
/// memory for hashing or encoding, while still accommodating large binary
/// assets like media files. This value balances operational safety with
/// practical use cases.
///
/// # How It Is Used
///
/// Backends that accept raw blob data should enforce this limit before
/// storing the object. This constant is a contract-level bound; concrete
/// [`ObjectStore`] implementations may choose stricter
/// limits if necessary.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::constants::MAX_BLOB_SIZE;
///
/// assert_eq!(MAX_BLOB_SIZE, 100 * 1024 * 1024_u64);
/// ```
pub const MAX_BLOB_SIZE: u64 = 100 * 1024 * 1024;

/// The maximum number of entries allowed in a single [`Tree`].
///
/// # Design Rationale
///
/// Set to 100,000 to prevent pathologically large directory listings that
/// would degrade traversal and encoding performance. Even with 100,000
/// entries, a tree object remains manageable in memory and can be encoded
/// efficiently.
///
/// # How It Is Used
///
/// Implementations of [`Encoder`] and
/// [`Decoder`] may consult this constant when validating
/// input or output sizes.
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
/// # Design Rationale
///
/// Set to 1 MiB to allow detailed changelogs while preventing abuse via
/// gigabyte-sized text payloads. This ensures that commit and tag objects
/// remain lightweight enough for efficient storage and transport.
///
/// # How It Is Used
///
/// Constructors for [`Commit`] and [`Tag`]
/// may consult this limit to reject oversized messages before they enter
/// the object database.
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
///
/// In version control systems like Git, tree entries store raw 32-bit mode
/// integers to represent file types and permissions. This module provides
/// those exact constants.
///
/// # Design Rationale
///
/// These constants are separated into their own module to keep the global
/// namespace clean. They represent the *wire format* and storage format,
/// distinct from the logical [`EntryKind`] enum used in
/// Rust memory. An encoder or decoder implementation will use these when
/// translating between [`TreeEntry`] and raw bytes.
///
/// # Internal Mechanism
///
/// The values are standard Unix mode bits expressed in octal. They are
/// intentionally stored as `u32` because the serialized tree format uses a
/// fixed-width 32-bit integer field for modes.
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
    ///
    /// # Value
    ///
    /// Unix octal `100644`, which corresponds to a regular file with
    /// owner read/write, group read, and others read permissions.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::constants::entry_mode::BLOB;
    ///
    /// assert_eq!(BLOB, 0o100_644);
    /// ```
    pub const BLOB: u32 = 0o100_644;

    /// Mode for an executable file.
    ///
    /// # Value
    ///
    /// Unix octal `100755`, which corresponds to a regular file with
    /// owner read/write/execute, group read/execute, and others read/execute
    /// permissions.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::constants::entry_mode::EXECUTABLE;
    ///
    /// assert_eq!(EXECUTABLE, 0o100_755);
    /// ```
    pub const EXECUTABLE: u32 = 0o100_755;

    /// Mode for a symbolic link.
    ///
    /// # Value
    ///
    /// Unix octal `120000`, which identifies a symbolic link. The target
    /// path is stored in the blob content, not in the mode itself.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::constants::entry_mode::SYMLINK;
    ///
    /// assert_eq!(SYMLINK, 0o120_000);
    /// ```
    pub const SYMLINK: u32 = 0o120_000;

    /// Mode for a subdirectory (tree).
    ///
    /// # Value
    ///
    /// Unix octal `040000`, which identifies a directory object. In a
    /// version control tree, this indicates that the entry points to another
    /// [`Tree`] rather than a [`Blob`].
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::constants::entry_mode::TREE;
    ///
    /// assert_eq!(TREE, 0o040_000);
    /// ```
    pub const TREE: u32 = 0o040_000;

    /// Mode for a submodule (gitlink).
    ///
    /// # Value
    ///
    /// Unix octal `160000`, which identifies a submodule reference. The
    /// entry's hash points to a commit in the submodule repository, and no
    /// actual blob content is stored.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::constants::entry_mode::SUBMODULE;
    ///
    /// assert_eq!(SUBMODULE, 0o160_000);
    /// ```
    pub const SUBMODULE: u32 = 0o160_000;
}
