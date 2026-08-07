//! Fundamental constants that apply across the entire `libvctrl` ecosystem.

/// The length of a hash in bytes.
///
/// We use SHA-512, which produces a 64‑byte digest.
/// Every [`Hasher`](crate::Hasher) implementation **must** return
/// exactly this many bytes.
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
