//! Fundamental data types that serve as the building blocks of a version control system.
//!
//! These types now enforce their invariants at construction time.
//! Fields are private and can only be set through validated constructors.
//! Once created, an instance is guaranteed to be valid.

use crate::constants::{HASH_LENGTH, MAX_NAME_LENGTH};
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
/// The [`Display`](std::fmt::Display) implementation prints the full hexadecimal representation.
/// The [`Debug`](std::fmt::Debug) implementation prints only the first 8 bytes for readability.
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
        for &byte in self.0.iter().take(8) {
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

// ---------------------------------------------------------------------------
// Helper for name validation
// ---------------------------------------------------------------------------
fn validate_name(name: &str) -> Result<(), VctrlError> {
    if name.is_empty() {
        return Err(VctrlError::InvalidName("name is empty".into()));
    }
    if name.len() > MAX_NAME_LENGTH {
        return Err(VctrlError::InvalidName(format!(
            "name exceeds maximum length {MAX_NAME_LENGTH}: '{name}'"
        )));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// TreeEntry
// ---------------------------------------------------------------------------
/// A single entry inside a [`Tree`].
///
/// Each entry associates a name, a kind ([`EntryKind`]), and a hash
/// pointing to the actual content ([`Blob`] or another [`Tree`]).
///
/// The name is guaranteed to be non‑empty and ≤ [`MAX_NAME_LENGTH`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeEntry {
    name: String,
    kind: EntryKind,
    hash: Hash,
}

impl TreeEntry {
    /// Creates a new `TreeEntry` after validating the name.
    ///
    /// # Errors
    /// Returns [`VctrlError::InvalidName`] if `name` is empty or too long.
    pub fn new(name: String, kind: EntryKind, hash: Hash) -> Result<Self, VctrlError> {
        validate_name(&name)?;
        Ok(Self { name, kind, hash })
    }

    /// Returns the name of the entry.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the kind of the entry.
    #[must_use]
    pub const fn kind(&self) -> EntryKind {
        self.kind
    }

    /// Returns the hash of the entry.
    #[must_use]
    pub const fn hash(&self) -> &Hash {
        &self.hash
    }
}

// ---------------------------------------------------------------------------
// Blob
// ---------------------------------------------------------------------------
/// A blob object – raw, uninterpreted data.
///
/// It represents the contents of a file.
/// No encoding or metadata is stored here; that is the responsibility
/// of higher‑level components (e.g., encoders).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Blob {
    data: Vec<u8>,
}

impl Blob {
    /// Creates a new `Blob` with the given data.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Vec::new is not const
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Returns a reference to the blob's data.
    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

// ---------------------------------------------------------------------------
// Tree
// ---------------------------------------------------------------------------
/// A tree object – a virtual directory listing.
///
/// It contains a sorted (by name) list of [`TreeEntry`] items,
/// each pointing to a blob or another tree. The entries are guaranteed
/// to be sorted lexicographically by name and contain no duplicates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tree {
    entries: Vec<TreeEntry>,
}

impl Tree {
    /// Creates a new `Tree` from a vector of entries.
    ///
    /// # Errors
    /// Returns an error if entries are not sorted by name or if duplicate names exist.
    /// Each entry must also be valid (already guaranteed by `TreeEntry` construction).
    pub fn new(entries: Vec<TreeEntry>) -> Result<Self, VctrlError> {
        // Check duplicates and ordering
        for i in 1..entries.len() {
            if entries[i - 1].name() >= entries[i].name() {
                return Err(VctrlError::InvalidName(format!(
                    "Tree entries are not sorted or contain duplicates: '{}' vs '{}'",
                    entries[i - 1].name(),
                    entries[i].name()
                )));
            }
        }
        Ok(Self { entries })
    }

    /// Returns a reference to the list of entries.
    #[must_use]
    pub fn entries(&self) -> &[TreeEntry] {
        &self.entries
    }
}

// ---------------------------------------------------------------------------
// UserID
// ---------------------------------------------------------------------------
/// Identity of a user (author or committer).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserID {
    name: String,
    email: String,
}

impl UserID {
    /// Creates a new `UserID` after validating name and email.
    ///
    /// Both name and email must be non‑empty. Name must not exceed `MAX_NAME_LENGTH`.
    ///
    /// # Errors
    /// Returns [`VctrlError::InvalidName`] if validation fails.
    pub fn new(name: String, email: String) -> Result<Self, VctrlError> {
        validate_name(&name)?;
        if email.is_empty() {
            return Err(VctrlError::InvalidName("email is empty".into()));
        }
        // We do not enforce MAX_NAME_LENGTH on email deliberately.
        Ok(Self { name, email })
    }

    /// Returns the user's name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the user's email.
    #[must_use]
    pub fn email(&self) -> &str {
        &self.email
    }
}

// ---------------------------------------------------------------------------
// Commit
// ---------------------------------------------------------------------------
/// A commit object – a snapshot of the repository at a point in time.
///
/// It records the root tree, parent commit(s), author, committer, and
/// a human‑readable message. Timestamps are deliberately omitted;
/// they can be added later by an implementor if needed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Commit {
    tree: Hash,
    parents: Vec<Hash>,
    author: UserID,
    committer: UserID,
    message: String,
}

impl Commit {
    /// Creates a new `Commit`.
    ///
    /// All fields are assumed to be valid (hashes are valid by construction,
    /// `UserID`s are valid).
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // not const because of String/Vec moving
    pub fn new(
        tree: Hash,
        parents: Vec<Hash>,
        author: UserID,
        committer: UserID,
        message: String,
    ) -> Self {
        Self {
            tree,
            parents,
            author,
            committer,
            message,
        }
    }

    /// Returns the root tree hash.
    #[must_use]
    pub const fn tree(&self) -> &Hash {
        &self.tree
    }

    /// Returns a reference to the parent hashes.
    #[must_use]
    pub fn parents(&self) -> &[Hash] {
        &self.parents
    }

    /// Returns the author.
    #[must_use]
    pub const fn author(&self) -> &UserID {
        &self.author
    }

    /// Returns the committer.
    #[must_use]
    pub const fn committer(&self) -> &UserID {
        &self.committer
    }

    /// Returns the commit message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

// ---------------------------------------------------------------------------
// Tag
// ---------------------------------------------------------------------------
/// A tag object – a named pointer to another object, usually a commit.
///
/// Tags are often used to mark releases or important points in history.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tag {
    name: String,
    target: Hash,
    tagger: Option<UserID>,
    message: String,
}

impl Tag {
    /// Creates a new `Tag` after validating the tag name.
    ///
    /// # Errors
    /// Returns [`VctrlError::InvalidName`] if the name is invalid.
    pub fn new(
        name: String,
        target: Hash,
        tagger: Option<UserID>,
        message: String,
    ) -> Result<Self, VctrlError> {
        validate_name(&name)?;
        Ok(Self {
            name,
            target,
            tagger,
            message,
        })
    }

    /// Returns the tag name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the target hash.
    #[must_use]
    pub const fn target(&self) -> &Hash {
        &self.target
    }

    /// Returns the tagger, if any.
    #[must_use]
    pub const fn tagger(&self) -> Option<&UserID> {
        self.tagger.as_ref()
    }

    /// Returns the tag message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}
