//! Fundamental constants that apply across the entire `libvctrl` ecosystem.

/// The length of a hash in bytes.
///
/// We use SHA-512, which produces a 64‑byte digest.
/// Every [`Hasher`](crate::Hasher) implementation **must** return
/// exactly this many bytes.
///
/// # Why a fixed length?
/// Locking the hash length to 64 bytes ensures **ecosystem stability**.
/// All types (especially [`Hash`](crate::Hash)) are built around this constant.
/// Making it dynamic (e.g., an associated constant on `Hasher`) would force
/// `Hash` to become generic, adding complexity and fragmentation.
/// If you need a different hash length, you can still build your own
/// parallel ecosystem using the same trait patterns, but this crate
/// guarantees that all components in the `libvctrl` ecosystem speak
/// the same "language" of 64‑byte hashes.
///
/// # See also
/// - [`Hash`](crate::Hash) – the newtype that enforces this length.
pub const HASH_LENGTH: usize = 64;

/// Maximum length of a name (tree entry, reference, tag, etc.) in bytes.
///
/// This limit prevents memory exhaustion and ensures interoperability.
/// Any name exceeding this length must be rejected with
/// [`InvalidName`](crate::VctrlError::InvalidName).
pub const MAX_NAME_LENGTH: usize = 255;

// ---------------------------------------------------------------------------
// Denial‑of‑Service prevention limits
// ---------------------------------------------------------------------------

/// Maximum size of a blob in bytes (100 MiB).
///
/// [`Decoder`](crate::Decoder) implementations **should** reject blobs
/// larger than this limit to prevent memory‑exhaustion attacks.
/// This is not enforced at the type level because legitimate use‑cases
/// may require larger blobs, but decoders that process untrusted input
/// must respect this bound.
pub const MAX_BLOB_SIZE: usize = 100 * 1024 * 1024; // 100 MiB

/// Maximum number of entries in a single tree.
///
/// Decoders must reject trees with more than this many entries.
/// A typical Git repository rarely exceeds a few thousand entries
/// per directory; 100 000 provides ample headroom.
pub const MAX_TREE_ENTRIES: usize = 100_000;

/// Maximum length of a commit or tag message in bytes (1 MiB).
///
/// Prevents an attacker from exhausting memory by supplying an
/// extremely long message.
pub const MAX_MESSAGE_LENGTH: usize = 1024 * 1024; // 1 MiB
