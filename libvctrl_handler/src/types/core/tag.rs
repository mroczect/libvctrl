//! Tag object representation.
//!
//! # Architecture
//! This module defines the [`Tag`] struct, which represents a Git annotated tag object.
//! Unlike lightweight tags (which are simply references), an annotated tag is a full
//! object in the object database. It stores metadata (tagger, timestamp, message)
//! and points to another object (usually a commit).
//!
//! # Design Rationale: Security by Construction
//! Tag names map directly to the filesystem (e.g., `refs/tags/v1.0`). Without strict
//! validation, a malicious tag name like `../../etc/passwd` could cause path traversal
//! vulnerabilities. The [`Tag::with_meta`] constructor enforces strict reference naming
//! rules via [`validate_ref_name`](crate::validation::validate_ref_name), ensuring that
//! a `Tag` instance cannot exist with an invalid or dangerous name.

use super::commit::CommitMeta;
use super::hash::Hash;
use super::user_id::UserID;
use crate::constants::MAX_MESSAGE_LENGTH;
use crate::errors::VctrlError;
use crate::validation::validate_ref_name;

/// A Git tag object.
///
/// # Why this exists
/// Provides a strongly-typed, immutable representation of an annotated tag. Tags are
/// used to mark specific points in history, such as release versions. By requiring
/// construction via [`new`](Self::new) or [`with_meta`](Self::with_meta), the crate
/// guarantees that every `Tag` in memory adheres to naming and size constraints,
/// preventing filesystem corruption and memory exhaustion.
///
/// # How it works
/// The struct stores the tag's `name`, the `target` hash it points to, an optional
/// `tagger` identity, a `message`, and temporal `meta`. It reuses [`CommitMeta`]
/// for timestamp data to avoid duplicating temporal logic between commits and tags.
///
/// # Examples
///
/// Creating a valid annotated tag:
///
/// ```
/// # use libvctrl_handler::types::core::tag::Tag;
/// # use libvctrl_handler::types::core::hash::Hash;
/// # use libvctrl_handler::types::core::user_id::UserID;
/// # use libvctrl_handler::VctrlError;
/// # let target = Hash::from_bytes(&[0_u8; 64])?;
/// # let tagger = UserID::new("Alice".to_string(), "alice@example.com".to_string())?;
/// let tag = Tag::new("v1.0.0".to_string(), target, Some(tagger), "Initial release".to_string())?;
/// assert_eq!(tag.name(), "v1.0.0");
/// # Ok::<(), VctrlError>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tag {
    name: String,
    target: Hash,
    tagger: Option<UserID>,
    message: String,
    meta: CommitMeta,
}

impl Tag {
    /// Creates a new tag with default metadata.
    ///
    /// # How it works
    /// Delegates to [`with_meta`](Self::with_meta), passing a default [`CommitMeta`]
    /// (timestamp 0, offset 0, no encoding). This is useful for testing or when
    /// temporal metadata is injected later.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the name or message fails validation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::types::core::tag::Tag;
    /// # use libvctrl_handler::types::core::hash::Hash;
    /// # use libvctrl_handler::VctrlError;
    /// # let target = Hash::from_bytes(&[0_u8; 64])?;
    /// let tag = Tag::new("v2.0".to_string(), target, None, "Release".to_string())?;
    /// assert_eq!(tag.message(), "Release");
    /// # Ok::<(), VctrlError>(())
    /// ```
    pub fn new(
        name: String,
        target: Hash,
        tagger: Option<UserID>,
        message: String,
    ) -> Result<Self, VctrlError> {
        Self::with_meta(name, target, tagger, message, CommitMeta::default())
    }

    /// Creates a new tag with timestamp metadata.
    ///
    /// # How it works
    /// Performs two critical validation steps:
    /// 1. Checks the `name` against Git's reference naming rules using
    ///    [`validate_ref_name`](crate::validation::validate_ref_name). This rejects
    ///    names containing `..`, leading/trailing slashes, or control characters.
    /// 2. Checks `message.len()` against [`MAX_MESSAGE_LENGTH`](crate::constants::MAX_MESSAGE_LENGTH).
    ///    Uses `usize::try_from` to safely handle 32-bit architectures.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::InvalidName`] if the name violates Git reference rules.
    /// Returns [`VctrlError::ExceededMaxSize`] if the message is too long.
    ///
    /// # Examples
    ///
    /// Detecting an invalid tag name:
    ///
    /// ```
    /// # use libvctrl_handler::types::core::tag::Tag;
    /// # use libvctrl_handler::types::core::hash::Hash;
    /// # use libvctrl_handler::types::core::commit::CommitMeta;
    /// # use libvctrl_handler::VctrlError;
    /// # let target = Hash::from_bytes(&[0_u8; 64])?;
    /// # let meta = CommitMeta::default();
    /// // Names containing ".." are forbidden to prevent path traversal.
    /// let result = Tag::with_meta("../evil".to_string(), target, None, "msg".to_string(), meta);
    /// assert!(matches!(result, Err(VctrlError::InvalidName(_))));
    /// # Ok::<(), VctrlError>(())
    /// ```
    pub fn with_meta(
        name: String,
        target: Hash,
        tagger: Option<UserID>,
        message: String,
        meta: CommitMeta,
    ) -> Result<Self, VctrlError> {
        validate_ref_name(&name)?;
        let max_len = usize::try_from(MAX_MESSAGE_LENGTH).unwrap_or(usize::MAX);
        if message.len() > max_len {
            return Err(VctrlError::ExceededMaxSize(format!(
                "message length exceeds maximum allowed length {MAX_MESSAGE_LENGTH}"
            )));
        }
        Ok(Self {
            name,
            target,
            tagger,
            message,
            meta,
        })
    }

    /// Returns the tag name.
    ///
    /// # How it works
    /// Returns a string slice (`&str`) borrowing from the internal `String`. This
    /// avoids allocation when the caller only needs to read the name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the target hash.
    ///
    /// # How it works
    /// Returns a reference to the [`Hash`] identifying the object this tag points to
    /// (usually a commit).
    #[must_use]
    pub const fn target(&self) -> &Hash {
        &self.target
    }

    /// Returns the tagger, if any.
    ///
    /// # How it works
    /// Returns `Option<&UserID>`. Lightweight tags might not have a tagger, but
    /// annotated tags usually do. Returns `None` if the tagger was not specified.
    #[must_use]
    pub const fn tagger(&self) -> Option<&UserID> {
        self.tagger.as_ref()
    }

    /// Returns the tag message.
    ///
    /// # How it works
    /// Returns a string slice (`&str`) borrowing from the internal `String`.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the tag metadata.
    ///
    /// # How it works
    /// Returns a reference to the [`CommitMeta`] struct containing timestamp and
    /// timezone data for the tag's creation.
    #[must_use]
    pub const fn meta(&self) -> &CommitMeta {
        &self.meta
    }
}
