//! Tag objects for naming specific points in the repository history.
//!
//! A [`Tag`] associates a human‑readable name with a target object (typically
//! a commit), along with optional metadata such as tagger, message, and
//! timestamp. It is the mechanism used to mark releases, milestones, or
//! other significant states.
//!
//! ## Design Notes
//!
//! - **Name validation**: The tag name is validated through the internal
//!   [`validate_name`](crate::types::validate_name) function, which enforces
//!   length and non‑emptiness constraints. This prevents invalid names from
//!   ever being stored.
//! - **Optional tagger**: Unlike commits, a tag does not require an author
//!   or committer. Lightweight tags omit the tagger entirely.
//! - **Reuse of [`CommitMeta`]**: To avoid duplication, the optional
//!   timestamp and encoding information is passed via `CommitMeta`.
//! - **Immutable by design**: All fields are private; once created, a tag
//!   cannot be altered. This preserves the integrity of the tag's hash.

use super::commit::CommitMeta;
use super::hash::Hash;
use super::user_id::UserID;
use crate::errors::VctrlError;
use crate::types::validate_name;

/// A named reference to a specific object in the repository.
///
/// A `Tag` points to any object (commonly a [`Commit`](super::Commit))
/// identified by its [`Hash`]. It records who created the tag, an optional
/// message, and the usual timestamp/offset metadata.
///
/// Two constructors are provided:
/// - [`new`](Self::new) – creates a tag with zeroed timestamp/offset and
///   no encoding.
/// - [`with_meta`](Self::with_meta) – accepts a [`CommitMeta`] for full
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
/// # let target = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
/// let tag = Tag::new("v1.0.0".into(), target, None, "".into()).unwrap();
/// assert_eq!(tag.name(), "v1.0.0");
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
    /// Creates a new tag with default metadata (zeroed timestamps, no encoding).
    ///
    /// The `name` is validated via [`validate_name`] and must be non‑empty
    /// and not exceed the maximum length. If validation fails, an error is
    /// returned.
    ///
    /// # Arguments
    ///
    /// * `name` - The tag name (e.g., `"v1.0.0"`).
    /// * `target` - The [`Hash`] of the object being tagged.
    /// * `tagger` - Optional identity of the person creating the tag.
    /// * `message` - An optional annotation message (can be empty).
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::InvalidName`] if `name` is empty or exceeds
    /// the maximum allowed length.
    ///
    /// # Examples
    ///
    /// Successful creation:
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Tag, Hash, UserID};
    /// use libvctrl_handler::constants::HASH_LENGTH;
    ///
    /// # let target = Hash::from_bytes(&[0x42; HASH_LENGTH]).unwrap();
    /// # let tagger = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
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
    /// # let target = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
    /// let err = Tag::new("".into(), target, None, "".into()).unwrap_err();
    /// assert!(matches!(err, libvctrl_handler::errors::VctrlError::InvalidName(_)));
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
    /// # Errors
    ///
    /// Returns [`VctrlError::InvalidName`] if `name` is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Tag, Hash, UserID, CommitMeta};
    /// use libvctrl_handler::constants::HASH_LENGTH;
    ///
    /// # let target = Hash::from_bytes(&[0x99; HASH_LENGTH]).unwrap();
    /// # let tagger = UserID::new("Bob".into(), "bob@example.com".into()).unwrap();
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
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Tag, Hash};
    /// use libvctrl_handler::constants::HASH_LENGTH;
    ///
    /// # let target = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
    /// let tag = Tag::new("my-tag".into(), target, None, "".into()).unwrap();
    /// assert_eq!(tag.name(), "my-tag");
    /// ```
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the target object's hash.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Tag, Hash};
    /// use libvctrl_handler::constants::HASH_LENGTH;
    ///
    /// # let target = Hash::from_bytes(&[0x11; HASH_LENGTH]).unwrap();
    /// let tag = Tag::new("v1".into(), target, None, "".into()).unwrap();
    /// assert_eq!(tag.target(), &target);
    /// ```
    #[must_use]
    pub const fn target(&self) -> &Hash {
        &self.target
    }

    /// Returns the tagger identity, if present.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Tag, Hash, UserID};
    /// use libvctrl_handler::constants::HASH_LENGTH;
    ///
    /// # let target = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
    /// # let tagger = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
    /// let tag = Tag::new("v1".into(), target, Some(tagger), "".into()).unwrap();
    /// assert!(tag.tagger().is_some());
    /// ```
    #[must_use]
    pub const fn tagger(&self) -> Option<&UserID> {
        self.tagger.as_ref()
    }

    /// Returns the tag message (annotation).
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Tag, Hash};
    /// use libvctrl_handler::constants::HASH_LENGTH;
    ///
    /// # let target = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
    /// let tag = Tag::new("v2".into(), target, None, "Release notes".into()).unwrap();
    /// assert_eq!(tag.message(), "Release notes");
    /// ```
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the Unix timestamp (seconds since epoch).
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Tag, Hash, CommitMeta};
    /// use libvctrl_handler::constants::HASH_LENGTH;
    ///
    /// # let target = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
    /// # let meta = CommitMeta { timestamp: 1_600_000_000, timezone_offset: 0, encoding: None };
    /// let tag = Tag::with_meta("t".into(), target, None, "".into(), meta).unwrap();
    /// assert_eq!(tag.timestamp(), 1_600_000_000);
    /// ```
    #[must_use]
    pub const fn timestamp(&self) -> i64 {
        self.timestamp
    }

    /// Returns the timezone offset from UTC in minutes.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Tag, Hash, CommitMeta};
    /// use libvctrl_handler::constants::HASH_LENGTH;
    ///
    /// # let target = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
    /// # let meta = CommitMeta { timestamp: 0, timezone_offset: 330, encoding: None };
    /// let tag = Tag::with_meta("t".into(), target, None, "".into(), meta).unwrap();
    /// assert_eq!(tag.timezone_offset(), 330);
    /// ```
    #[must_use]
    pub const fn timezone_offset(&self) -> i16 {
        self.timezone_offset
    }

    /// Returns the character encoding of the tag message, if specified.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::types::core::{Tag, Hash, CommitMeta};
    /// use libvctrl_handler::constants::HASH_LENGTH;
    ///
    /// # let target = Hash::from_bytes(&[0u8; HASH_LENGTH]).unwrap();
    /// # let meta = CommitMeta { timestamp: 0, timezone_offset: 0, encoding: Some("utf-16".into()) };
    /// let tag = Tag::with_meta("t".into(), target, None, "msg".into(), meta).unwrap();
    /// assert_eq!(tag.encoding(), Some("utf-16"));
    /// ```
    #[must_use]
    pub fn encoding(&self) -> Option<&str> {
        self.encoding.as_deref()
    }
}
