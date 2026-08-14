//! Tag objects for naming specific points in the repository history.
//!
//! # Purpose
//!
//! A [`Tag`] associates a human-readable name with a target object
//! (typically a commit), along with optional metadata such as tagger,
//! message, timestamp, and encoding. It is the mechanism used to mark
//! releases, milestones, or other significant states in a version control
//! repository.
//!
//! # Design Notes
//!
//! - **Name validation**: The tag name is validated through the internal
//!   `validate_name` function, which enforces
//!   length and non-emptiness constraints. This prevents invalid names from
//!   ever being stored.
//! - **Optional tagger**: Unlike commits, a tag does not require an author
//!   or committer. Lightweight tags omit the tagger entirely.
//! - **Reuse of [`CommitMeta`]**: To avoid duplication, the optional
//!   timestamp and encoding information is passed via `CommitMeta`.
//! - **Immutable by design**: All fields are private; once created, a tag
//!   cannot be altered. This preserves the integrity of the tag's hash.
//!
//! # Relationship to Other Types
//!
//! A [`Tag`] points to any object (commonly a [`Commit`])
//! identified by its `Hash`. It uses [`UserID`] for the
//! optional tagger identity and [`CommitMeta`] for
//! timestamp and encoding information, ensuring consistency with commits.
//!
//! # Memory Layout
//!
//! The struct owns all its fields: a [`String`] for the name, a `Hash`
//! (64 bytes) for the target, an [`Option`] containing a [`UserID`] for the
//! tagger, a [`String`] for the message, and scalar metadata fields. The
//! struct is not `Copy` because it owns heap-allocated data; cloning
//! performs a deep copy.
//!
//! # Examples
//!
//! Creating a simple lightweight tag (no tagger, no message):
//!
//! ```
//! use libvctrl_handler::types::core::{Tag, Hash};
//! use libvctrl_handler::constants::HASH_LENGTH;
//!
//! let target = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
//! let tag = Tag::new("v1.0.0".into(), target, None, "".into()).unwrap();
//! assert_eq!(tag.name(), "v1.0.0");
//! ```

use super::commit::CommitMeta;
use super::hash::Hash;
use super::user_id::UserID;
use crate::errors::VctrlError;
use crate::types::validate_name;

/// A named reference to a specific object in the repository.
///
/// # Purpose
///
/// A `Tag` points to any object (commonly a [`Commit`])
/// identified by its `Hash`. It records who created the tag, an optional
/// message, and the usual timestamp/offset metadata. Tags are often used to
/// mark releases or important milestones, providing a stable, human-readable
/// alias for a specific commit.
///
/// # Design Rationale
///
/// - The `name` field is validated at construction to ensure it is non-empty
///   and within length limits.
/// - The `tagger` field is an [`Option`] to support lightweight tags (no
///   tagger) and annotated tags (with tagger).
/// - The `target` field is a `Hash` rather than a concrete object reference
///   to keep the tag independent of the object's type and storage location.
/// - The `message` field allows a tag to carry an annotation.
/// - The `timestamp`, `timezone_offset`, and `encoding` fields are optional
///   metadata, provided via [`CommitMeta`] for the full constructor.
///
/// # Immutability
///
/// All fields are private and there are no mutable accessors. Once created,
/// a tag cannot be altered. This is essential because a tag's hash is
/// derived from its content; mutating any field would change the hash and
/// break the content-addressable model.
///
/// # Constructors
///
/// Two constructors are provided:
///
/// - [`new`](Self::new) - creates a tag with zeroed timestamp/offset and no
///   encoding.
/// - [`with_meta`](Self::with_meta) - accepts a [`CommitMeta`] for full
///   control over the metadata.
///
/// # Examples
///
/// Creating a simple lightweight tag (no tagger, no message):
///
/// ```
/// use libvctrl_handler::types::core::{Tag, Hash};
/// use libvctrl_handler::constants::HASH_LENGTH;
///
/// let target = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
/// let tag = Tag::new("v1.0.0".into(), target, None, "".into()).unwrap();
/// assert_eq!(tag.name(), "v1.0.0");
/// ```
///
/// Creating an annotated tag with tagger and message:
///
/// ```
/// use libvctrl_handler::types::core::{Tag, Hash, UserID};
/// use libvctrl_handler::constants::HASH_LENGTH;
///
/// let target = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
/// let tagger = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
/// let tag = Tag::new("v2.0".into(), target, Some(tagger), "Stable release".into()).unwrap();
/// assert_eq!(tag.tagger().unwrap().name(), "Alice");
/// assert_eq!(tag.message(), "Stable release");
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
    /// Creates a new tag with default metadata (zeroed timestamps, no
    /// encoding).
    ///
    /// The `name` is validated via `validate_name` and must be non-empty
    /// and not exceed the maximum length. If validation fails, an error is
    /// returned.
    ///
    /// # Arguments
    ///
    /// * `name` - The tag name (e.g., `"v1.0.0"`). It is moved into the tag.
    /// * `target` - The `Hash` of the object being tagged.
    /// * `tagger` - Optional identity of the person creating the tag.
    /// * `message` - An optional annotation message (can be empty).
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::InvalidName`] if `name` is empty or exceeds
    /// the maximum allowed length.
    ///
    /// # Why Fallible?
    ///
    /// The constructor returns a [`Result`] to force callers to handle
    /// invalid names immediately, preventing malformed tags from entering
    /// the system. This is consistent with other constructors in the crate.
    ///
    /// # How It Works Internally
    ///
    /// 1. Calls `validate_name` on the provided name. If invalid, an
    ///    error is returned early.
    /// 2. Constructs the `Tag` with default metadata: `timestamp = 0`,
    ///    `timezone_offset = 0`, `encoding = None`.
    /// 3. Wraps the result in `Ok`.
    ///
    /// # Examples
    ///
    /// Successful creation:
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Tag, Hash, UserID};
    /// use libvctrl_handler::constants::HASH_LENGTH;
    ///
    /// let target = Hash::from_bytes(&[0x42; HASH_LENGTH]).unwrap();
    /// let tagger = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
    /// let tag = Tag::new(
    ///     "release-1.0".into(),
    ///     target,
    ///     Some(tagger),
    ///     "Stable release".into(),
    /// ).unwrap();
    /// assert_eq!(tag.message(), "Stable release");
    /// ```
    ///
    /// Invalid name triggers an error:
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Tag, Hash};
    /// use libvctrl_handler::constants::HASH_LENGTH;
    ///
    /// let target = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
    /// let err = Tag::new("".into(), target, None, "".into()).unwrap_err();
    /// assert!(matches!(err, libvctrl_handler::VctrlError::InvalidName(_)));
    /// ```
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

    /// Creates a new tag with the given metadata.
    ///
    /// This constructor is identical to [`new`] except that it also accepts a
    /// [`CommitMeta`], which supplies the timestamp, timezone offset, and
    /// encoding. The `name` is validated in the same way.
    ///
    /// # Arguments
    ///
    /// * `name` - The tag name (e.g., `"v2.0"`).
    /// * `target` - The `Hash` of the object being tagged.
    /// * `tagger` - Optional identity of the person creating the tag.
    /// * `message` - An optional annotation message (can be empty).
    /// * `meta` - A [`CommitMeta`] containing timestamp, timezone offset,
    ///   and encoding.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::InvalidName`] if `name` is invalid.
    ///
    /// # Why Use `with_meta`?
    ///
    /// This constructor is preferred when timestamp and timezone information
    /// is available, e.g., from the environment during tag creation. It
    /// avoids the need to set these fields later (which is impossible due to
    /// immutability).
    ///
    /// # How It Works Internally
    ///
    /// 1. Calls `validate_name` on the provided name.
    /// 2. Constructs the `Tag` using the provided [`CommitMeta`] fields.
    /// 3. Wraps the result in `Ok`.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Tag, Hash, UserID, CommitMeta};
    /// use libvctrl_handler::constants::HASH_LENGTH;
    ///
    /// let target = Hash::from_bytes(&[0x99; HASH_LENGTH]).unwrap();
    /// let tagger = UserID::new("Bob".into(), "bob@example.com".into()).unwrap();
    /// let meta = CommitMeta {
    ///     timestamp: 1_700_000_000,
    ///     timezone_offset: -300,
    ///     encoding: Some("utf-8".into()),
    /// };
    /// let tag = Tag::with_meta(
    ///     "v2.0".into(),
    ///     target,
    ///     Some(tagger),
    ///     "Major update".into(),
    ///     meta,
    /// ).unwrap();
    /// assert_eq!(tag.timestamp(), 1_700_000_000);
    /// assert_eq!(tag.encoding(), Some("utf-8"));
    /// ```
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
    ///
    /// # Returns
    ///
    /// A string slice containing the validated tag name.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Tag, Hash};
    /// use libvctrl_handler::constants::HASH_LENGTH;
    ///
    /// let target = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
    /// let tag = Tag::new("my-tag".into(), target, None, "".into()).unwrap();
    /// assert_eq!(tag.name(), "my-tag");
    /// ```
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the target object's hash.
    ///
    /// # Returns
    ///
    /// A reference to the `Hash` of the object the tag points to.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Tag, Hash};
    /// use libvctrl_handler::constants::HASH_LENGTH;
    ///
    /// let target = Hash::from_bytes(&[0x11; HASH_LENGTH]).unwrap();
    /// let tag = Tag::new("v1".into(), target, None, "".into()).unwrap();
    /// assert_eq!(tag.target(), &target);
    /// ```
    #[must_use]
    pub const fn target(&self) -> &Hash {
        &self.target
    }

    /// Returns the tagger identity, if present.
    ///
    /// # Returns
    ///
    /// An [`Option`] containing a reference to the [`UserID`] of the tagger
    /// if this is an annotated tag, or `None` for lightweight tags.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Tag, Hash, UserID};
    /// use libvctrl_handler::constants::HASH_LENGTH;
    ///
    /// let target = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
    /// let tagger = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
    /// let tag = Tag::new("v1".into(), target, Some(tagger), "".into()).unwrap();
    /// assert!(tag.tagger().is_some());
    /// assert_eq!(tag.tagger().unwrap().email(), "alice@example.com");
    /// ```
    #[must_use]
    pub const fn tagger(&self) -> Option<&UserID> {
        self.tagger.as_ref()
    }

    /// Returns the tag message (annotation).
    ///
    /// # Returns
    ///
    /// A string slice containing the annotation message, which may be empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Tag, Hash};
    /// use libvctrl_handler::constants::HASH_LENGTH;
    ///
    /// let target = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
    /// let tag = Tag::new("v2".into(), target, None, "Release notes".into()).unwrap();
    /// assert_eq!(tag.message(), "Release notes");
    /// ```
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the Unix timestamp (seconds since epoch).
    ///
    /// # Returns
    ///
    /// The timestamp as an `i64`.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Tag, Hash, CommitMeta};
    /// use libvctrl_handler::constants::HASH_LENGTH;
    ///
    /// let target = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
    /// let meta = CommitMeta { timestamp: 1_600_000_000, timezone_offset: 0, encoding: None };
    /// let tag = Tag::with_meta("t".into(), target, None, "".into(), meta).unwrap();
    /// assert_eq!(tag.timestamp(), 1_600_000_000);
    /// ```
    #[must_use]
    pub const fn timestamp(&self) -> i64 {
        self.timestamp
    }

    /// Returns the timezone offset from UTC in minutes.
    ///
    /// # Returns
    ///
    /// The offset as an `i16`. Positive values indicate east of UTC,
    /// negative values indicate west.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Tag, Hash, CommitMeta};
    /// use libvctrl_handler::constants::HASH_LENGTH;
    ///
    /// let target = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
    /// let meta = CommitMeta { timestamp: 0, timezone_offset: 330, encoding: None };
    /// let tag = Tag::with_meta("t".into(), target, None, "".into(), meta).unwrap();
    /// assert_eq!(tag.timezone_offset(), 330);
    /// ```
    #[must_use]
    pub const fn timezone_offset(&self) -> i16 {
        self.timezone_offset
    }

    /// Returns the character encoding of the tag message, if specified.
    ///
    /// # Returns
    ///
    /// An [`Option<&str>`] containing the encoding name if present.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Tag, Hash, CommitMeta};
    /// use libvctrl_handler::constants::HASH_LENGTH;
    ///
    /// let target = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
    /// let meta = CommitMeta { timestamp: 0, timezone_offset: 0, encoding: Some("utf-16".into()) };
    /// let tag = Tag::with_meta("t".into(), target, None, "msg".into(), meta).unwrap();
    /// assert_eq!(tag.encoding(), Some("utf-16"));
    /// ```
    #[must_use]
    pub fn encoding(&self) -> Option<&str> {
        self.encoding.as_deref()
    }
}
