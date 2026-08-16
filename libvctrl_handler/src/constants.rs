//! Constants related to Git object formats and operational limits.
//!
//! # Architecture
//! This module centralizes all magic numbers and structural limits used across the crate.
//! By extracting these into named constants, we eliminate "magic numbers" from the business
//! logic, making the codebase easier to audit and maintain.
//!
//! # Design Rationale: Resource Exhaustion Prevention
//! Version control systems frequently handle untrusted or malformed data. Without strict
//! upper limits, a maliciously crafted repository could instruct the parser to allocate
//! gigabytes of memory (e.g., a blob claiming to be 10 Exabytes). The `MAX_*` constants
//! act as fail-fast circuit breakers during object construction, ensuring that memory
//! allocation remains bounded and predictable.
//!
//! # Git Protocol Compliance
//! Constants like [`HASH_LENGTH`] and the modes in [`entry_mode`] are dictated by the Git
//! core specification. Hardcoding them ensures strict compliance with standard Git clients
//! and servers, preventing protocol violations.

/// The length of a hash in bytes (SHA-512 = 64).
///
/// # Why this exists
/// This crate mandates SHA-512 for cryptographic integrity. By hardcoding the length
/// to 64 bytes, we enable the use of fixed-size arrays (e.g., `[u8; HASH_LENGTH]`)
/// instead of dynamically allocated `Vec<u8>`. This shifts memory management to the
/// compile-time stack, eliminating heap allocation overhead and fragmentation for
/// every hash operation.
///
/// # How it works
/// The constant is evaluated at compile time. Any array sized with this constant
/// benefits from fixed stack layout, and the compiler can aggressively optimize
/// loops iterating exactly `HASH_LENGTH` times.
///
/// # Examples
///
/// ```
/// # use my_crate::constants::HASH_LENGTH;
/// assert_eq!(HASH_LENGTH, 64);
/// let hash_array = [0u8; HASH_LENGTH];
/// assert_eq!(hash_array.len(), 64);
/// ```
pub const HASH_LENGTH: usize = 64;

/// The maximum allowed length for names (in bytes).
///
/// # Why this exists
/// Enforces a sane upper bound on file, directory, and reference names. This aligns
/// closely with typical filesystem limits (e.g., 255 bytes in most Unix filesystems).
/// It prevents malicious inputs from causing excessive memory consumption or
/// triggering filesystem errors during checkout operations.
///
/// # Examples
///
/// ```
/// # use my_crate::constants::MAX_NAME_LENGTH;
/// assert_eq!(MAX_NAME_LENGTH, 255);
/// ```
pub const MAX_NAME_LENGTH: u64 = 255;

/// The maximum allowed size for blob objects (in bytes).
///
/// # Why this exists
/// To prevent denial-of-service (DoS) via memory exhaustion. If unbounded, a parser
/// reading a malformed packfile could attempt to allocate gigabytes of memory for a
/// single blob. The 100 MiB limit provides ample room for legitimate source code and
/// small binary assets while acting as a circuit breaker against malicious payloads.
///
/// # Examples
///
/// ```
/// # use my_crate::constants::MAX_BLOB_SIZE;
/// assert_eq!(MAX_BLOB_SIZE, 100 * 1024 * 1024);
/// ```
pub const MAX_BLOB_SIZE: u64 = 100 * 1024 * 1024;

/// The maximum number of entries allowed in a tree.
///
/// # Why this exists
/// While Git allows a technically unlimited number of entries in a tree object,
/// performance degrades quadratically if entries are not handled correctly. Capping
/// this at 100,000 ensures that tree parsing, diffing, and serialization remain
/// performant and bounded in memory usage.
///
/// # Examples
///
/// ```
/// # use my_crate::constants::MAX_TREE_ENTRIES;
/// assert_eq!(MAX_TREE_ENTRIES, 100_000);
/// ```
pub const MAX_TREE_ENTRIES: u64 = 100_000;

/// The maximum allowed length for commit/tag messages (in bytes).
///
/// # Why this exists
/// Commit and tag messages are metadata. A 1 MiB limit is exceedingly generous for
/// textual descriptions but strictly prevents malicious actors from embedding massive
/// payloads (e.g., encoded binaries) into commit logs, which would bloat repository
/// history and memory usage during traversal.
///
/// # Examples
///
/// ```
/// # use my_crate::constants::MAX_MESSAGE_LENGTH;
/// assert_eq!(MAX_MESSAGE_LENGTH, 1024 * 1024);
/// ```
pub const MAX_MESSAGE_LENGTH: u64 = 1024 * 1024;

/// The maximum number of parent commits allowed (binary format uses u16).
///
/// # Why this exists
/// Restricts the complexity of octopus merges. While Git supports many parents,
/// allowing an unbounded number can lead to pathological graph structures that are
/// expensive to traverse. The limit of 65,535 corresponds to the maximum value of
/// an unsigned 16-bit integer, ensuring it can be packed efficiently if a binary
/// format is introduced.
///
/// # Examples
///
/// ```
/// # use my_crate::constants::MAX_PARENT_COUNT;
/// assert_eq!(MAX_PARENT_COUNT, u16::MAX as u64);
/// ```
pub const MAX_PARENT_COUNT: u64 = 65535;

/// Git object entry modes.
///
/// # Architecture
/// In Git, filesystem objects are identified by a 32-bit mode. This module exposes
/// the specific constants recognized by the Git protocol. Using named constants
/// instead of raw integers prevents invalid mode combinations and makes tree
/// manipulation code self-documenting.
///
/// # How it works
/// The modes combine Unix permission bits with Git-specific object types.
/// For example, `0o100_644` indicates a regular file (`0o100`) with read/write
/// permissions for the owner and read-only for others (`0o644`).
pub mod entry_mode {
    /// Regular file mode.
    ///
    /// # Examples
    ///
    /// ```
    /// # use my_crate::constants::entry_mode::BLOB;
    /// assert_eq!(BLOB, 0o100_644);
    /// ```
    pub const BLOB: u32 = 0o100_644;

    /// Executable file mode.
    ///
    /// # Examples
    ///
    /// ```
    /// # use my_crate::constants::entry_mode::EXECUTABLE;
    /// assert_eq!(EXECUTABLE, 0o100_755);
    /// ```
    pub const EXECUTABLE: u32 = 0o100_755;

    /// Symbolic link mode.
    ///
    /// # Examples
    ///
    /// ```
    /// # use my_crate::constants::entry_mode::SYMLINK;
    /// assert_eq!(SYMLINK, 0o120_000);
    /// ```
    pub const SYMLINK: u32 = 0o120_000;

    /// Directory (tree) mode.
    ///
    /// # Examples
    ///
    /// ```
    /// # use my_crate::constants::entry_mode::TREE;
    /// assert_eq!(TREE, 0o40_000);
    /// ```
    pub const TREE: u32 = 0o40_000;

    /// Submodule commit mode.
    ///
    /// # Examples
    ///
    /// ```
    /// # use my_crate::constants::entry_mode::SUBMODULE;
    /// assert_eq!(SUBMODULE, 0o160_000);
    /// ```
    pub const SUBMODULE: u32 = 0o160_000;
}
