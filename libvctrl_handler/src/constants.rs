//! Fundamental constants that apply across the entire `libvctrl` ecosystem.
//!
//! This module defines the global invariants that every component in the
//! `libvctrl` workspace must obey.  These constants serve as the **single
//! source of truth** for:
//!
//! - The cryptographic hash length (`HASH_LENGTH`),
//! - Name length limits (`MAX_NAME_LENGTH`),
//! - Denial‑of‑Service prevention bounds (`MAX_BLOB_SIZE`, `MAX_TREE_ENTRIES`,
//!   `MAX_MESSAGE_LENGTH`),
//! - Entry mode bits for tree entries ([`entry_mode`]).
//!
//! # Why constants instead of associated types?
//!
//! Associated types on traits (e.g., `type HashLength` inside `Hasher`) would
//! force **every** generic structure to become parameterised, leading to
//! “generics hell”.  By locking the hash length and other limits at the
//! ecosystem level, we ensure that **all** components speak the same language
//! and that fundamental types like [`Hash`](crate::Hash) can be simple newtypes
//! rather than generic blobs.
//!
//! # Stability guarantees
//!
//! These constants are considered **semver‑stable**.  Changes to their values
//! (especially `HASH_LENGTH`) require a major version bump.  The DoS‑prevention
//! limits may be raised in a minor release but will never be lowered.

/// The length of a hash in bytes.
///
/// We use SHA‑512, which produces a 64‑byte (512‑bit) digest.  Every
/// [`Hasher`](crate::Hasher) implementation **must** return exactly this many
/// bytes.  No exceptions.
///
/// # Why 64 bytes?
///
/// - **Security margin** – SHA‑512 provides 256‑bit collision resistance and
///   512‑bit preimage resistance.  Even against quantum adversaries, the
///   effective security is still well above 128 bits.
/// - **Hardware efficiency** – SHA‑512 is designed for 64‑bit processors.
///   On modern x86‑64 and ARM64, it is often faster than SHA‑256.
/// - **Ecosystem simplicity** – a single, fixed size means `Hash` can be a
///   `[u8; 64]` under the hood.  No dynamic allocations, no generics.
///
/// # What if I need a different hash?
///
/// You can build a parallel ecosystem using the same trait patterns, but this
/// crate guarantees that **all** components inside the `libvctrl` workspace
/// speak the same “language” of 64‑byte hashes.  That trade‑off is intentional
/// and documented.
///
/// # See also
///
/// - [`Hash`](crate::Hash) – the newtype that enforces this length.
/// - [`HASH_LENGTH`] is used in [`Hash::from_bytes`](crate::Hash::from_bytes)
///   to reject slices that are not exactly 64 bytes.
///
/// ```rust
/// use libvctrl_handler::{HASH_LENGTH, Hash};
///
/// let valid   = Hash::from_bytes(&[0u8; HASH_LENGTH]);
/// let invalid = Hash::from_bytes(&[0u8; 10]);
/// assert!(valid.is_ok());
/// assert!(invalid.is_err());
/// ```
pub const HASH_LENGTH: usize = 64;

/// Maximum length of a name (tree entry, reference, tag, etc.) in bytes.
///
/// This limit applies to **all** human‑readable identifiers in the system:
/// file names inside a tree, reference names (`refs/heads/…`), tag names,
/// user names, etc.
///
/// # Rationale
///
/// - **Memory exhaustion** – without a limit, an attacker could supply a
///   multi‑megabyte name and exhaust heap space.
/// - **Interoperability** – virtually every version‑control system imposes a
///   limit on name length.  `MAX_NAME_LENGTH` = 255 bytes is consistent with
///   common file‑system restrictions (e.g., Linux `NAME_MAX`).
/// - **Deterministic encoding** – fixed‑width length prefixes (common in binary
///   formats) can use a single byte up to 255, which simplifies encoding.
///
/// Any name exceeding this length **must** be rejected with
/// [`VctrlError::InvalidName`](crate::VctrlError::InvalidName).  This is
/// enforced by the private `validate_name` helper inside
/// [`types`](crate::types).
///
/// ```rust
/// use libvctrl_handler::{MAX_NAME_LENGTH, VctrlError};
///
/// let long_name = "a".repeat(MAX_NAME_LENGTH + 1);
/// // Any constructor that takes a name will return an error:
/// // TreeEntry::new(long_name, ...) → Err(VctrlError::InvalidName(…))
/// ```
pub const MAX_NAME_LENGTH: usize = 255;

// ---------------------------------------------------------------------------
// Denial‑of‑Service prevention limits
// ---------------------------------------------------------------------------

/// Maximum size of a blob in bytes (100 MiB).
///
/// Decoders **should** reject blobs larger than this limit to prevent
/// memory‑exhaustion attacks.  This is **not** enforced at the type level
/// (i.e., `Blob::new(data)` will accept any `Vec<u8>`), because legitimate
/// use‑cases (e.g., scientific datasets) may require larger blobs.  However,
/// **any decoder that processes untrusted input must respect this bound** and
/// return [`VctrlError::CorruptedData`](crate::VctrlError::CorruptedData) if
/// the encoded size exceeds `MAX_BLOB_SIZE`.
///
/// ```rust
/// use libvctrl_handler::MAX_BLOB_SIZE;
///
/// // The reference decoder in libvctrl_core uses this limit:
/// // if data_len > MAX_BLOB_SIZE { return Err(…); }
/// ```
pub const MAX_BLOB_SIZE: usize = 100 * 1024 * 1024; // 100 MiB

/// Maximum number of entries in a single tree.
///
/// Decoders must reject trees with more than this many entries.  A typical
/// source‑code repository rarely exceeds a few thousand entries per directory;
/// `MAX_TREE_ENTRIES` = 100 000 provides ample headroom for monorepos and
/// generated trees while still protecting against malicious payloads.
///
/// # Why 100 000?
/// The largest monorepos (e.g., Google’s internal repository) contain millions
/// of files, but a single directory rarely exceeds tens of thousands of
/// entries.  100 000 is a conservative upper bound that covers real‑world
/// extremes without enabling memory‑exhaustion attacks.
pub const MAX_TREE_ENTRIES: usize = 100_000;

/// Maximum length of a commit or tag message in bytes (1 MiB).
///
/// Prevents an attacker from exhausting memory by supplying an extremely long
/// message.  Like [`MAX_BLOB_SIZE`], this is **not** enforced on construction
/// (`Commit::new` accepts any `String`), but decoders that handle untrusted
/// input **must** reject messages longer than this limit.
///
/// ```rust
/// use libvctrl_handler::MAX_MESSAGE_LENGTH;
///
/// // The reference decoder in libvctrl_core uses this limit:
/// // if msg_len > MAX_MESSAGE_LENGTH { return Err(…); }
/// ```
pub const MAX_MESSAGE_LENGTH: usize = 1024 * 1024; // 1 MiB

/// Mode bits for tree entries.
///
/// These constants define the **file type** and **permission** bits that
/// describe an entry inside a [`Tree`](crate::Tree).  The values follow
/// POSIX conventions and are compatible with Git's internal representation.
///
/// # Why a `u32` bit‑field?
///
/// A tree entry is not just “file” or “directory”.  Real systems need to
/// distinguish:
/// - regular files,
/// - executable files,
/// - symbolic links,
/// - sub‑directories,
/// - submodules (references to other repositories).
///
/// Using a `u32` mode word (instead of a simple enum) allows the system to
/// carry full POSIX file metadata without sacrificing extensibility.  New
/// mode bits can be added in the future without breaking existing parsers.
///
/// # How the modes are used
///
/// The [`EntryKind`](crate::EntryKind) struct stores the mode internally as a
/// `u32`.  It provides convenience constructors (`EntryKind::blob()`,
/// `EntryKind::tree()`, …) that set the correct mode bits.  This design keeps
/// the common cases ergonomic while still allowing arbitrary mode values when
/// necessary.
///
/// # Bit layout
///
/// The lower 16 bits follow POSIX file‑type conventions:
///
/// | Bit range | Meaning |
/// |---|---|
/// | 12‑15 | File type (0o100000 = regular, 0o040000 = directory, …) |
/// | 9‑11  | Owner permission (rwx) |
/// | 6‑8   | Group permission (rwx) |
/// | 0‑5   | Others permission (rwx) |
///
/// # Examples
///
/// ```rust
/// use libvctrl_handler::constants::entry_mode;
///
/// assert_eq!(entry_mode::BLOB, 0o100_644);    // regular file, rw-r--r--
/// assert_eq!(entry_mode::TREE, 0o040_000);    // directory
/// assert_eq!(entry_mode::EXECUTABLE, 0o100_755); // regular file, rwxr-xr-x
/// ```
///
/// # Compatibility with Git
///
/// Git uses the same mode values for its tree entries.  This means that a
/// `libvctrl` repository can be written in a Git‑compatible way, should that
/// be desired.
pub mod entry_mode {
    /// Regular file, non‑executable (`rw‑r--r--`).
    ///
    /// This is the most common mode for source files, documents, and
    /// other non‑executable content.
    pub const BLOB: u32 = 0o100_644;

    /// Regular file, executable (`rwxr‑xr‑x`).
    ///
    /// Used for scripts, compiled binaries, or any file that should be
    /// runnable.
    pub const EXECUTABLE: u32 = 0o100_755;

    /// Symbolic link.
    ///
    /// The entry's hash points to a blob whose content is the link target.
    pub const SYMLINK: u32 = 0o120_000;

    /// Sub‑directory (tree).
    ///
    /// The entry's hash points to another [`Tree`](crate::Tree) object.
    pub const TREE: u32 = 0o040_000;

    /// Git‑compatible submodule (commit link).
    ///
    /// The entry's hash points to a commit in another repository.
    /// This is used for tracking external dependencies.
    pub const SUBMODULE: u32 = 0o160_000;
}
