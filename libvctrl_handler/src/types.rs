//! Fundamental data types that serve as the building blocks of a version control system.
//!
//! These types enforce their invariants at construction time.
//! Fields are private and can only be set through validated constructors.
//! Once created, an instance is **guaranteed** to be valid.
//!
//! # Design
//!
//! Every type is immutable after construction. If you need to change a value,
//! you must create a new instance (e.g., a new commit with different parents).
//! This immutability is a cornerstone of content‑addressable storage and
//! cryptographic integrity.
//!
//! All types implement `Debug`, `Clone`, and `PartialEq + Eq` for easy
//! comparison and display. Hashes also implement `Ord`, `Copy`, and `Hash`
//! so they can be used as keys in collections.
//!
//! # Validation at construction
//!
//! Constructors that accept strings (names, emails) validate their inputs:
//! - Names must not be empty.
//! - Names must not exceed [`MAX_NAME_LENGTH`](crate::constants::MAX_NAME_LENGTH).
//! - Emails must not be empty (for `UserID`).
//!
//! If validation fails, an [`VctrlError`](crate::VctrlError) is returned.
//! This ensures that invalid data never exists in your system.
//!
//! # Metadata
//!
//! [`CommitMeta`] bundles optional timestamp, timezone offset, and text encoding
//! for objects that carry these attributes ([`Commit`], [`Tag`]).
//!
//! # Examples
//!
//! ```rust
//! use libvctrl_handler::*;
//!
//! // Build a valid hash from known bytes.
//! let hash = Hash::from_bytes(&[0xAB; HASH_LENGTH]).unwrap();
//!
//! // Create a tree entry (a file named "README.md").
//! let entry = TreeEntry::new("README.md".into(), EntryKind::Blob, hash)
//!     .expect("valid entry");
//!
//! // Build a tree containing that entry.
//! let tree = Tree::new(vec![entry]).expect("entries are sorted");
//!
//! // Create an author identity.
//! let alice = UserID::new("Alice".into(), "alice@example.com".into())
//!     .expect("valid user");
//!
//! // Make a commit (no parents – initial commit) with default metadata.
//! let commit = Commit::new(
//!     hash,         // tree
//!     vec![],       // no parents
//!     alice.clone(),// author
//!     alice.clone(),// committer  ← tambahkan .clone()
//!     "Initial import".into(),
//! );
//!
//! // Make a commit with explicit metadata.
//! let meta = CommitMeta {
//!     timestamp: 1672531200,   // 2023-01-01T00:00:00 UTC
//!     timezone_offset: 0,
//!     encoding: Some("UTF-8".into()),
//! };
//! let commit2 = Commit::with_meta(
//!     hash,
//!     vec![],
//!     alice.clone(),
//!     alice.clone(),
//!     "Another commit".into(),
//!     meta,
//! );
//!
//! // Create an annotated tag.
//! let tag = Tag::new("v0.1.0".into(), hash, None, "First release".into())
//!     .expect("valid tag name");
//!
//! assert_eq!(commit.message(), "Initial import");
//! assert_eq!(tag.name(), "v0.1.0");
//! ```

use crate::constants::{HASH_LENGTH, MAX_NAME_LENGTH};
use crate::enums::EntryKind;
use crate::errors::VctrlError;
use std::fmt;

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
// Hash
// ---------------------------------------------------------------------------
/// A content hash – a fixed‑size array of 64 bytes (SHA‑512).
///
/// This is the fundamental identifier for all objects in the system.
/// A `Hash` is **always** 64 bytes; any attempt to create one with
/// a different length will fail with [`VctrlError::InvalidHashLength`].
///
/// # Construction
///
/// Use [`Hash::from_bytes`] to convert a byte slice. This function validates
/// the length and returns `Err` if it does not match [`HASH_LENGTH`].
///
/// ```rust
/// use libvctrl_handler::{Hash, HASH_LENGTH};
///
/// // Correct length → succeeds.
/// let h = Hash::from_bytes(&[0x00; HASH_LENGTH]).unwrap();
///
/// // Wrong length → fails.
/// assert!(Hash::from_bytes(&[0; 10]).is_err());
/// ```
///
/// # Display and Debug
///
/// - [`Display`] prints the full 64‑byte hex string (128 characters).
/// - [`Debug`] prints only the first 8 bytes followed by `…` for brevity.
///
/// ```rust
/// use libvctrl_handler::{Hash, HASH_LENGTH};
///
/// let h = Hash::from_bytes(&[0xAB; HASH_LENGTH]).unwrap();
///
/// // Display: "abababababababababab..."
/// // Debug:  "Hash(abababababababab…)"
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Hash([u8; HASH_LENGTH]);

impl Hash {
    /// Creates a `Hash` from a byte slice.
    ///
    /// # Errors
    /// Returns [`VctrlError::InvalidHashLength`] if `bytes.len()` ≠ [`HASH_LENGTH`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # use libvctrl_handler::*;
    /// let data = [0xAA; HASH_LENGTH];
    /// let hash = Hash::from_bytes(&data).unwrap();
    /// assert_eq!(hash.as_bytes(), &data);
    /// ```
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
    ///
    /// ```rust
    /// # use libvctrl_handler::*;
    /// let hash = Hash::from_bytes(&[0xCC; HASH_LENGTH]).unwrap();
    /// let bytes = hash.as_bytes();
    /// assert_eq!(bytes.len(), HASH_LENGTH);
    /// ```
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
// TreeEntry
// ---------------------------------------------------------------------------
/// A single entry inside a [`Tree`].
///
/// An entry is the basic building block of a directory listing. It pairs
/// a **name** with a **kind** (blob or subtree) and a **hash** that points to
/// the actual content.
///
/// # Validation
/// The name must be non‑empty and ≤ [`MAX_NAME_LENGTH`](crate::constants::MAX_NAME_LENGTH).
///
/// # Example
///
/// ```rust
/// use libvctrl_handler::{Hash, TreeEntry, EntryKind};
///
/// let hash = Hash::from_bytes(&[0x11; 64]).unwrap();
///
/// // A file entry.
/// let file = TreeEntry::new("src/main.rs".into(), EntryKind::Blob, hash)
///     .expect("valid entry");
/// assert_eq!(file.name(), "src/main.rs");
/// assert_eq!(file.kind(), EntryKind::Blob);
/// assert_eq!(file.hash().as_bytes().len(), 64);
///
/// // An empty name is rejected.
/// assert!(TreeEntry::new("".into(), EntryKind::Blob, hash).is_err());
/// ```
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

    /// Returns the kind of the entry (blob or tree).
    #[must_use]
    pub const fn kind(&self) -> EntryKind {
        self.kind
    }

    /// Returns the hash of the entry (points to a [`Blob`] or another [`Tree`]).
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
/// Represents the contents of a file. No encoding, compression, or metadata
/// is stored – just the raw bytes.
///
/// # Empty blobs
/// An empty blob (`Blob::new(vec![])`) is perfectly valid and represents
/// an empty file.
///
/// # Example
///
/// ```rust
/// use libvctrl_handler::Blob;
///
/// let data = b"Hello, world!".to_vec();
/// let blob = Blob::new(data.clone());
/// assert_eq!(blob.data(), b"Hello, world!");
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Blob {
    data: Vec<u8>,
}

impl Blob {
    /// Creates a new `Blob` with the given data.
    #[must_use]
    pub const fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Returns a reference to the blob's raw data.
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
/// A tree contains a **sorted** list of [`TreeEntry`] items. Entries are
/// ordered lexicographically by name, and duplicate names are forbidden.
/// These invariants are enforced at construction time.
///
/// # Errors
/// [`Tree::new`] will return an error if:
/// - Entries are not in sorted order.
/// - Two entries share the same name.
///
/// # Example
///
/// ```rust
/// use libvctrl_handler::{Hash, Tree, TreeEntry, EntryKind};
///
/// let hash = Hash::from_bytes(&[0x22; 64]).unwrap();
///
/// // Create sorted entries.
/// let file = TreeEntry::new("a.txt".into(), EntryKind::Blob, hash).unwrap();
/// let dir  = TreeEntry::new("sub".into(), EntryKind::Tree, hash).unwrap();
///
/// // Build the tree – entries must be in order.
/// let tree = Tree::new(vec![file, dir]).expect("sorted entries");
/// assert_eq!(tree.entries().len(), 2);
///
/// // Duplicate names are rejected.
/// let dup1 = TreeEntry::new("x".into(), EntryKind::Blob, hash).unwrap();
/// let dup2 = TreeEntry::new("x".into(), EntryKind::Blob, hash).unwrap();
/// assert!(Tree::new(vec![dup1, dup2]).is_err());
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tree {
    entries: Vec<TreeEntry>,
}

impl Tree {
    /// Creates a new `Tree` from a vector of entries.
    ///
    /// # Errors
    /// Returns an error if entries are not sorted by name or if duplicate names exist.
    pub fn new(entries: Vec<TreeEntry>) -> Result<Self, VctrlError> {
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
///
/// Contains a **name** and an **email**. Both are required to be non‑empty.
/// The name is also validated against [`MAX_NAME_LENGTH`].
///
/// # Example
///
/// ```rust
/// use libvctrl_handler::UserID;
///
/// let user = UserID::new("Alice".into(), "alice@example.com".into())
///     .expect("valid user");
/// assert_eq!(user.name(), "Alice");
/// assert_eq!(user.email(), "alice@example.com");
///
/// // Empty fields are rejected.
/// assert!(UserID::new("".into(), "x@y".into()).is_err());
/// assert!(UserID::new("Alice".into(), "".into()).is_err());
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserID {
    name: String,
    email: String,
}

impl UserID {
    /// Creates a new `UserID` after validating name and email.
    ///
    /// # Errors
    /// Returns [`VctrlError::InvalidName`] if:
    /// - `name` is empty or too long.
    /// - `email` is empty.
    pub fn new(name: String, email: String) -> Result<Self, VctrlError> {
        validate_name(&name)?;
        if email.is_empty() {
            return Err(VctrlError::InvalidName("email is empty".into()));
        }
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
// CommitMeta
// ---------------------------------------------------------------------------
/// Optional metadata for [`Commit`] and [`Tag`] objects.
///
/// Bundles timestamp, timezone offset, and text encoding so that constructors
/// can accept a single metadata argument instead of many individual parameters.
///
/// # Default
///
/// `CommitMeta::default()` returns `timestamp: 0`, `timezone_offset: 0`,
/// `encoding: None`. This is what `Commit::new` and `Tag::new` use internally.
///
/// # Example
///
/// ```rust
/// use libvctrl_handler::CommitMeta;
///
/// let meta = CommitMeta {
///     timestamp: 1672531200,   // 2023-01-01T00:00:00 UTC
///     timezone_offset: 0,
///     encoding: Some("UTF-8".into()),
/// };
///
/// let default_meta = CommitMeta::default();
/// assert_eq!(default_meta.timestamp, 0);
/// assert!(default_meta.encoding.is_none());
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct CommitMeta {
    /// Unix timestamp (seconds since the Unix epoch).  `0` means "not set".
    pub timestamp: i64,
    /// Timezone offset in minutes east of UTC.  `0` means "not set".
    pub timezone_offset: i16,
    /// Text encoding of the message (e.g., `"UTF-8"`).  `None` means "not specified".
    pub encoding: Option<String>,
}

// ---------------------------------------------------------------------------
// Commit
// ---------------------------------------------------------------------------
/// A commit object – a snapshot of the repository at a point in time.
///
/// Records the root tree, parent commit(s), author, committer, a
/// human‑readable message, and optional metadata ([`CommitMeta`]).
///
/// # Construction
///
/// - [`Commit::new`] creates a commit with default metadata.
/// - [`Commit::with_meta`] accepts explicit [`CommitMeta`].
///
/// # Example (single‑parent commit)
///
/// ```rust
/// use libvctrl_handler::{Commit, Hash, UserID, HASH_LENGTH, CommitMeta};
///
/// let tree_hash = Hash::from_bytes(&[0x33; HASH_LENGTH]).unwrap();
/// let parent_hash = Hash::from_bytes(&[0x44; HASH_LENGTH]).unwrap();
/// let author = UserID::new("Bob".into(), "bob@example.com".into()).unwrap();
///
/// let commit = Commit::new(
///     tree_hash,
///     vec![parent_hash],
///     author.clone(),
///     author.clone(),
///     "Fix bug #42".into(),
/// );
///
/// // With metadata
/// let meta = CommitMeta {
///     timestamp: 1672531200,
///     timezone_offset: 0,
///     encoding: Some("UTF-8".into()),
/// };
/// let commit2 = Commit::with_meta(
///     tree_hash,
///     vec![parent_hash],
///     author.clone(),
///     author.clone(),
///     "Fix bug #42".into(),
///     meta,
/// );
/// ```
///
/// # Example (initial commit)
///
/// ```rust
/// # use libvctrl_handler::*;
/// let tree = Hash::from_bytes(&[0x55; 64]).unwrap();
/// let user = UserID::new("Alice".into(), "alice@e.com".into()).unwrap();
///
/// let initial = Commit::new(tree, vec![], user.clone(), user, "init".into());
/// assert!(initial.parents().is_empty());
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Commit {
    tree: Hash,
    parents: Vec<Hash>,
    author: UserID,
    committer: UserID,
    message: String,
    timestamp: i64,
    timezone_offset: i16,
    encoding: Option<String>,
}

impl Commit {
    /// Creates a new `Commit` with default metadata (timestamp 0, no encoding).
    #[must_use]
    pub const fn new(
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
            timestamp: 0,
            timezone_offset: 0,
            encoding: None,
        }
    }

    /// Creates a new `Commit` with explicit metadata.
    #[must_use]
    pub fn with_meta(
        tree: Hash,
        parents: Vec<Hash>,
        author: UserID,
        committer: UserID,
        message: String,
        meta: CommitMeta,
    ) -> Self {
        Self {
            tree,
            parents,
            author,
            committer,
            message,
            timestamp: meta.timestamp,
            timezone_offset: meta.timezone_offset,
            encoding: meta.encoding,
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

    /// Unix timestamp (seconds since epoch). 0 if not set.
    #[must_use]
    pub const fn timestamp(&self) -> i64 {
        self.timestamp
    }

    /// Timezone offset in minutes east of UTC. 0 if not set.
    #[must_use]
    pub const fn timezone_offset(&self) -> i16 {
        self.timezone_offset
    }

    /// Encoding (e.g., "UTF-8") if set.
    #[must_use]
    pub fn encoding(&self) -> Option<&str> {
        self.encoding.as_deref()
    }
}

// ---------------------------------------------------------------------------
// Tag
// ---------------------------------------------------------------------------
/// A tag object – a named pointer to another object, usually a commit.
///
/// Tags can optionally include a **tagger** identity, a message,
/// and metadata ([`CommitMeta`]).
///
/// # Construction
///
/// - [`Tag::new`] creates a tag with default metadata.
/// - [`Tag::with_meta`] accepts explicit [`CommitMeta`].
///
/// # Example (annotated tag)
///
/// ```rust
/// use libvctrl_handler::{Hash, Tag, UserID, CommitMeta};
///
/// let commit_hash = Hash::from_bytes(&[0x66; 64]).unwrap();
/// let tagger = UserID::new("Release Bot".into(), "release@example.com".into()).unwrap();
///
/// let tag = Tag::new(
///     "v1.0.0".into(),
///     commit_hash,
///     Some(tagger.clone()),
///     "Stable release".into(),
/// ).expect("valid tag name");
///
/// // With metadata
/// let meta = CommitMeta {
///     timestamp: 1672531200,
///     timezone_offset: 0,
///     encoding: Some("UTF-8".into()),
/// };
/// let tag2 = Tag::with_meta(
///     "v1.0.1".into(),
///     commit_hash,
///     Some(tagger.clone()),
///     "Patch release".into(),
///     meta,
/// ).unwrap();
/// ```
///
/// # Example (lightweight tag)
///
/// ```rust
/// # use libvctrl_handler::*;
/// let hash = Hash::from_bytes(&[0x77; 64]).unwrap();
/// let tag = Tag::new("temp".into(), hash, None, "".into()).unwrap();
/// assert!(tag.tagger().is_none());
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tag {
    name: String,
    target: Hash,
    tagger: Option<UserID>,
    message: String,
    timestamp: i64,
    timezone_offset: i16,
    encoding: Option<String>,
}

impl Tag {
    /// Creates a new `Tag` with default metadata (timestamp 0, no encoding).
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
            timestamp: 0,
            timezone_offset: 0,
            encoding: None,
        })
    }

    /// Creates a new `Tag` with explicit metadata.
    ///
    /// # Errors
    /// Returns [`VctrlError::InvalidName`] if the name is invalid.
    pub fn with_meta(
        name: String,
        target: Hash,
        tagger: Option<UserID>,
        message: String,
        meta: CommitMeta,
    ) -> Result<Self, VctrlError> {
        validate_name(&name)?;
        Ok(Self {
            name,
            target,
            tagger,
            message,
            timestamp: meta.timestamp,
            timezone_offset: meta.timezone_offset,
            encoding: meta.encoding,
        })
    }

    /// Returns the tag name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the target hash (usually a commit).
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

    /// Unix timestamp (seconds since epoch). 0 if not set.
    #[must_use]
    pub const fn timestamp(&self) -> i64 {
        self.timestamp
    }

    /// Timezone offset in minutes east of UTC. 0 if not set.
    #[must_use]
    pub const fn timezone_offset(&self) -> i16 {
        self.timezone_offset
    }

    /// Encoding (e.g., "UTF-8") if set.
    #[must_use]
    pub fn encoding(&self) -> Option<&str> {
        self.encoding.as_deref()
    }
}
