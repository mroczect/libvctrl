//! Commit object and metadata representation.
//!
//! # Architecture
//! This module defines the [`Commit`] struct, which acts as the node in the Git
//! Directed Acyclic Graph (DAG). A commit links a tree state (snapshot) to its
//! historical predecessors (parents), annotated with authorship and temporal metadata.
//!
//! # Design Rationale: DAG Integrity
//! Git's history relies on the assumption that the parent graph is acyclic and
//! structurally sound. To enforce this at the type level, the [`Commit::with_meta`]
//! constructor performs strict validation:
//! - **Duplicate Parents**: Uses a `HashSet` to ensure no parent hash appears twice.
//!   Because [`Hash`] is `Copy`, inserting into the set requires no allocation,
//!   providing O(1) duplicate detection.
//! - **Parent Count Limits**: Enforces [`MAX_PARENT_COUNT`](crate::constants::MAX_PARENT_COUNT)
//!   to prevent pathological merge structures.
//! - **Message Bounds**: Enforces [`MAX_MESSAGE_LENGTH`](crate::constants::MAX_MESSAGE_LENGTH)
//!   to prevent memory exhaustion via commit messages.

use super::hash::Hash;
use super::user_id::UserID;
use crate::constants::{MAX_MESSAGE_LENGTH, MAX_PARENT_COUNT};
use crate::errors::VctrlError;
use std::collections::HashSet;

/// Metadata associated with a commit or tag.
///
/// # Why this exists
/// Separates temporal and environmental data (timestamps, timezones, encoding)
/// from the core graph structure. This allows the metadata to be default-constructed
/// (e.g., for testing) and shared between commits and annotated tags.
///
/// # How it works
/// The timezone offset is stored as an `i16` representing minutes. The constructor
/// strictly validates this range (-1440 to 1440 minutes, i.e., -24 to +24 hours)
/// to prevent malformed historical data.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct CommitMeta {
    timestamp: i64,
    timezone_offset: i16,
    encoding: Option<String>,
}

impl CommitMeta {
    /// Creates new commit metadata.
    ///
    /// # How it works
    /// Validates that the `timezone_offset` falls within the valid range of
    /// -1440 to 1440 minutes. This range covers all valid global timezones
    /// (UTC-24:00 to UTC+24:00). Rejecting out-of-bounds offsets early prevents
    /// arithmetic overflows or logic errors during date formatting.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::InvalidTimezoneOffset`] if the offset is out of range (-1440..=1440).
    ///
    /// # Examples
    ///
    /// ```
    /// # use my_crate::types::core::commit::CommitMeta;
    /// # use my_crate::VctrlError;
    /// let meta = CommitMeta::new(1600000000, 120, None)?;
    /// assert_eq!(meta.timezone_offset(), 120);
    ///
    /// let invalid = CommitMeta::new(0, 1500, None);
    /// assert!(matches!(invalid, Err(VctrlError::InvalidTimezoneOffset(1500))));
    /// # Ok::<(), VctrlError>(())
    /// ```
    pub fn new(
        timestamp: i64,
        timezone_offset: i16,
        encoding: Option<String>,
    ) -> Result<Self, VctrlError> {
        if !(-1440..=1440).contains(&timezone_offset) {
            return Err(VctrlError::InvalidTimezoneOffset(timezone_offset));
        }
        Ok(Self {
            timestamp,
            timezone_offset,
            encoding,
        })
    }

    /// Returns the timestamp.
    ///
    /// # How it works
    /// Returns the Unix timestamp (seconds since epoch) as an `i64` to handle dates
    /// far in the past or future. This is a `const fn`, allowing compile-time evaluation.
    #[must_use]
    pub const fn timestamp(&self) -> i64 {
        self.timestamp
    }

    /// Returns the timezone offset in minutes.
    ///
    /// # Examples
    ///
    /// ```
    /// # use my_crate::types::core::commit::CommitMeta;
    /// # use my_crate::VctrlError;
    /// let meta = CommitMeta::new(0, -300, None)?;
    /// assert_eq!(meta.timezone_offset(), -300);
    /// # Ok::<(), VctrlError>(())
    /// ```
    #[must_use]
    pub const fn timezone_offset(&self) -> i16 {
        self.timezone_offset
    }

    /// Returns the encoding, if any.
    ///
    /// # How it works
    /// Uses `as_deref()` to return `Option<&str>`, borrowing from the internal
    /// `Option<String>` without allocating.
    #[must_use]
    pub fn encoding(&self) -> Option<&str> {
        self.encoding.as_deref()
    }
}

/// A Git commit object.
///
/// # Why this exists
/// Represents a snapshot of the repository at a specific point in time, authored
/// by a user. It links a [`Tree`] to its parent commits, forming the history graph.
///
/// # How it works
/// The struct stores the root tree hash, a vector of parent hashes (empty for the
/// initial commit), author/committer identities, the message, and metadata. All
/// fields are owned, ensuring the commit is self-contained and can be cloned or
/// sent across threads without lifetime constraints.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Commit {
    tree: Hash,
    parents: Vec<Hash>,
    author: UserID,
    committer: UserID,
    message: String,
    meta: CommitMeta,
}

impl Commit {
    /// Creates a new commit with default metadata.
    ///
    /// # How it works
    /// Delegates to [`with_meta`](Self::with_meta), passing a default [`CommitMeta`]
    /// (timestamp 0, offset 0, no encoding). This is useful for testing or when
    /// metadata is injected later.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::DuplicateParent`] if parents contain duplicates.
    /// Returns [`VctrlError::ExceededMaxSize`] if the message is too long or too many parents.
    ///
    /// # Examples
    ///
    /// ```
    /// # use my_crate::types::core::commit::{Commit, CommitMeta};
    /// # use my_crate::types::core::hash::Hash;
    /// # use my_crate::types::core::user_id::UserID;
    /// # use my_crate::VctrlError;
    /// # let tree = Hash::from_bytes(&[0u8; 64])?;
    /// # let author = UserID::new("Alice".to_string(), "alice@example.com".to_string())?;
    /// let commit = Commit::new(tree, vec![], author.clone(), author, "initial".to_string())?;
    /// assert_eq!(commit.message(), "initial");
    /// # Ok::<(), VctrlError>(())
    /// ```
    pub fn new(
        tree: Hash,
        parents: Vec<Hash>,
        author: UserID,
        committer: UserID,
        message: String,
    ) -> Result<Self, VctrlError> {
        Self::with_meta(
            tree,
            parents,
            author,
            committer,
            message,
            CommitMeta::default(),
        )
    }

    /// Creates a new commit with timestamp metadata.
    ///
    /// # How it works
    /// Performs three critical validation steps:
    /// 1. Checks `parents.len()` against [`MAX_PARENT_COUNT`](crate::constants::MAX_PARENT_COUNT).
    ///    Uses `usize::try_from` to safely handle 32-bit architectures.
    /// 2. Checks `message.len()` against [`MAX_MESSAGE_LENGTH`](crate::constants::MAX_MESSAGE_LENGTH).
    /// 3. Iterates through `parents` and inserts each [`Hash`] into a `HashSet`. Because
    ///    `Hash` implements `Copy` and `Hash`, the insertion is a fast stack operation.
    ///    If `insert` returns `false`, a duplicate was found, and an error is returned.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if validation fails (duplicate parents, size limits exceeded).
    ///
    /// # Examples
    ///
    /// ```
    /// # use my_crate::types::core::commit::{Commit, CommitMeta};
    /// # use my_crate::types::core::hash::Hash;
    /// # use my_crate::types::core::user_id::UserID;
    /// # use my_crate::VctrlError;
    /// # let tree = Hash::from_bytes(&[0u8; 64])?;
    /// # let parent = Hash::from_bytes(&[1u8; 64])?;
    /// # let author = UserID::new("Bob".to_string(), "bob@example.com".to_string())?;
    /// # let meta = CommitMeta::new(1000, 0, None)?;
    /// // Detecting a duplicate parent
    /// let result = Commit::with_meta(tree, vec![parent, parent], author.clone(), author, "msg".to_string(), meta);
    /// assert!(matches!(result, Err(VctrlError::DuplicateParent)));
    /// # Ok::<(), VctrlError>(())
    /// ```
    pub fn with_meta(
        tree: Hash,
        parents: Vec<Hash>,
        author: UserID,
        committer: UserID,
        message: String,
        meta: CommitMeta,
    ) -> Result<Self, VctrlError> {
        let max_parents = usize::try_from(MAX_PARENT_COUNT).unwrap_or(usize::MAX);
        if parents.len() > max_parents {
            return Err(VctrlError::ExceededMaxSize(format!(
                "commit has {} parents, exceeding maximum of {MAX_PARENT_COUNT}",
                parents.len()
            )));
        }

        let max_len = usize::try_from(MAX_MESSAGE_LENGTH).unwrap_or(usize::MAX);
        if message.len() > max_len {
            return Err(VctrlError::ExceededMaxSize(format!(
                "message length exceeds maximum allowed length {MAX_MESSAGE_LENGTH}"
            )));
        }

        let mut seen = HashSet::new();
        for p in &parents {
            if !seen.insert(*p) {
                return Err(VctrlError::DuplicateParent);
            }
        }

        Ok(Self {
            tree,
            parents,
            author,
            committer,
            message,
            meta,
        })
    }

    /// Returns the tree hash of this commit.
    ///
    /// # How it works
    /// Returns a reference to the root [`Hash`] identifying the tree object associated
    /// with this commit's snapshot.
    #[must_use]
    pub const fn tree(&self) -> &Hash {
        &self.tree
    }

    /// Returns the parent commit hashes.
    ///
    /// # How it works
    /// Returns a slice `&[Hash]` borrowing from the internal vector. This allows
    /// callers to iterate over parents without cloning the hashes.
    #[must_use]
    pub fn parents(&self) -> &[Hash] {
        &self.parents
    }

    /// Returns the author information.
    ///
    /// # How it works
    /// Returns a reference to the [`UserID`] representing the person who originally
    /// wrote the changes.
    #[must_use]
    pub const fn author(&self) -> &UserID {
        &self.author
    }

    /// Returns the committer information.
    ///
    /// # How it works
    /// Returns a reference to the [`UserID`] representing the person who applied
    /// the changes to the repository (e.g., rebasing or merging).
    #[must_use]
    pub const fn committer(&self) -> &UserID {
        &self.committer
    }

    /// Returns the commit message.
    ///
    /// # How it works
    /// Returns a string slice (`&str`) borrowing from the internal `String`.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the commit metadata.
    ///
    /// # How it works
    /// Returns a reference to the [`CommitMeta`] struct containing timestamp and
    /// timezone data.
    #[must_use]
    pub const fn meta(&self) -> &CommitMeta {
        &self.meta
    }
}
