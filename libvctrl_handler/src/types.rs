//! Fundamental data types that serve as the building blocks of a version control system.
//!
//! These types are pure data containers. They do **not** contain business logic
//! beyond the minimal validation required to construct valid instances
//! (e.g., [`Hash::from_bytes`] checks the length).

use crate::constants::HASH_LENGTH;
use crate::enums::EntryKind;
use crate::errors::VctrlError;
use std::fmt;

/// A content hash – a fixed‑size array of 64 bytes (SHA‑512).
///
/// # Construction
/// Use [`Hash::from_bytes`] to create a hash from a byte slice.
/// It will return [`VctrlError::InvalidHashLength`] if the slice length
/// is not exactly [`HASH_LENGTH`].
///
/// # Display and Debug
/// [`Display`] prints the full hexadecimal representation.
/// [`Debug`] prints only the first 8 bytes for readability.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Hash([u8; HASH_LENGTH]);

impl Hash {
    /// Creates a `Hash` from a byte slice.
    ///
    /// # Errors
    /// Returns [`VctrlError::InvalidHashLength`] if `bytes.len() != HASH_LENGTH`.
    pub const fn from_bytes(bytes: &[u8]) -> Result<Self, VctrlError> {
        if bytes.len() != HASH_LENGTH {
            return Err(VctrlError::InvalidHashLength(bytes.len()));
        }
        let mut arr = [0u8; HASH_LENGTH];
        // `copy_from_slice` is const in Rust 1.46+
        let mut i = 0;
        while i < HASH_LENGTH {
            arr[i] = bytes[i];
            i += 1;
        }
        Ok(Self(arr))
    }

    /// Returns a reference to the underlying 64‑byte array.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; HASH_LENGTH] {
        &self.0
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash(")?;
        for &byte in &self.0[..8] {
            write!(f, "{byte:02x}")?;
        }
        write!(f, "…)")
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &byte in &self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

/// A single entry inside a [`Tree`].
///
/// Each entry associates a name, a kind ([`EntryKind`]), and a hash
/// pointing to the actual content ([`Blob`] or another [`Tree`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeEntry {
    /// The name of the file or sub‑directory.
    pub name: String,
    /// Whether this entry is a file or a directory.
    pub kind: EntryKind,
    /// The hash of the object this entry points to.
    pub hash: Hash,
}

/// A blob object – raw, uninterpreted data.
///
/// It represents the contents of a file.
/// No encoding or metadata is stored here; that is the responsibility
/// of higher‑level components (e.g., encoders).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Blob {
    /// The raw bytes of the file.
    pub data: Vec<u8>,
}

/// A tree object – a virtual directory listing.
///
/// It contains a sorted (by name) list of [`TreeEntry`] items,
/// each pointing to a blob or another tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tree {
    /// The list of entries in this directory.
    pub entries: Vec<TreeEntry>,
}

/// Identity of a user (author or committer).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserID {
    /// The user's name.
    pub name: String,
    /// The user's email address.
    pub email: String,
}

/// A commit object – a snapshot of the repository at a point in time.
///
/// It records the root tree, parent commit(s), author, committer, and
/// a human‑readable message. Timestamps are deliberately omitted;
/// they can be added later by an implementor if needed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Commit {
    /// The hash of the root tree representing the snapshot.
    pub tree: Hash,
    /// Hashes of the parent commit(s). Empty for the initial commit.
    pub parents: Vec<Hash>,
    /// The person who originally wrote the changes.
    pub author: UserID,
    /// The person who committed the changes (may differ from author).
    pub committer: UserID,
    /// The commit message describing the change.
    pub message: String,
}

/// A tag object – a named pointer to another object, usually a commit.
///
/// Tags are often used to mark releases or important points in history.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tag {
    /// The name of the tag.
    pub name: String,
    /// The hash of the object being tagged.
    pub target: Hash,
    /// The person who created the tag (optional).
    pub tagger: Option<UserID>,
    /// An optional message describing the tag.
    pub message: String,
}
