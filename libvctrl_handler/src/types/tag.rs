//! Annotated tag type for version control systems.
//!
//! A [`Tag`] is a human‑readable label that points to a specific object
//! (usually a commit) in the repository history. Tags are immutable once
//! created and can carry an optional tagger identity, a message, and
//! timestamp metadata.

use crate::errors::VctrlError;
use crate::types::commit::CommitMeta;
use crate::types::hash::Hash;
use crate::types::user_id::UserID;

use super::validate_name;

/// A named reference to a specific repository object, typically a commit.
///
/// Tags are used to mark important points in history (e.g., releases). Each
/// tag has a unique `name` within the repository, points to a `target`
/// object (usually a commit hash), and can optionally store the identity of
/// the person who created the tag (`tagger`), a descriptive `message`, and
/// timing metadata (`timestamp`, `timezone_offset`, `encoding`).
///
/// # Design
///
/// All fields are private. Construction is only possible through the
/// fallible [`Tag::new`] and [`Tag::with_meta`] constructors, which validate
/// the tag name via the internal `validate_name` function. This ensures that
/// every `Tag` instance respects the naming constraints defined by the
/// system (non‑empty, maximum length).
///
/// Once built, a tag is immutable; only read‑only accessors are provided.
/// This guarantees that the tag's identity and hash remain stable throughout
/// its lifetime.
///
/// # Examples
///
/// Creating a simple tag without a tagger:
///
/// ```
/// # use libvctrl_handler::{Hash, Tag, UserID};
/// # // Helper to create a dummy hash (64 bytes of 0xAB)
/// # fn make_hash() -> Hash {
/// #     let bytes = [0xABu8; 64];
/// #     Hash::from_bytes(&bytes).unwrap()
/// # }
/// let target = make_hash();
/// let tag = Tag::new(
///     "v1.0.0".into(),
///     target,
///     None,                     // no tagger
///     "First release".into(),
/// ).unwrap();
///
/// assert_eq!(tag.name(), "v1.0.0");
/// assert!(tag.tagger().is_none());
/// ```
///
/// A tag with a tagger and metadata:
///
/// ```
/// # use libvctrl_handler::{CommitMeta, Hash, Tag, UserID};
/// # fn make_hash() -> Hash {
/// #     let bytes = [0xCDu8; 64];
/// #     Hash::from_bytes(&bytes).unwrap()
/// # }
/// let target = make_hash();
/// let tagger = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
/// let meta = CommitMeta {
///     timestamp: 1_700_000_000,
///     timezone_offset: 120,
///     encoding: Some("UTF-8".into()),
/// };
/// let tag = Tag::with_meta(
///     "v2.0.0".into(),
///     target,
///     Some(tagger),
///     "Second release".into(),
///     meta,
/// ).unwrap();
///
/// assert_eq!(tag.tagger().unwrap().name(), "Alice");
/// assert_eq!(tag.timestamp(), 1_700_000_000);
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
    /// Creates a new tag with minimal metadata.
    ///
    /// The tag name is validated to ensure it is non‑empty and does not
    /// exceed the maximum allowed length. The resulting tag has its
    /// timestamp, timezone offset, and encoding set to zero/`None`.
    ///
    /// # Parameters
    ///
    /// - `name` – A unique label for this tag (e.g., `"v1.0.0"`).
    /// - `target` – The hash of the object being tagged.
    /// - `tagger` – Optional identity of the person who created the tag.
    /// - `message` – A descriptive text.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::InvalidName`] if the name is empty or too long.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{Hash, Tag, UserID};
    /// # fn make_hash() -> Hash {
    /// #     let bytes = [0x11u8; 64];
    /// #     Hash::from_bytes(&bytes).unwrap()
    /// # }
    /// let target = make_hash();
    /// let tag = Tag::new("release".into(), target, None, "".into()).unwrap();
    /// assert_eq!(tag.message(), "");
    /// assert_eq!(tag.timestamp(), 0);
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

    /// Creates a new tag with full metadata.
    ///
    /// This constructor accepts the same fields as [`Tag::new`] plus a
    /// [`CommitMeta`] that supplies the timestamp, timezone offset, and
    /// optional encoding. The name is validated identically.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::InvalidName`] if the name fails validation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::{CommitMeta, Hash, Tag, UserID};
    /// # fn make_hash() -> Hash {
    /// #     let bytes = [0x22u8; 64];
    /// #     Hash::from_bytes(&bytes).unwrap()
    /// # }
    /// let target = make_hash();
    /// let meta = CommitMeta {
    ///     timestamp: 1_600_000_000,
    ///     timezone_offset: -300,
    ///     encoding: None,
    /// };
    /// let tag = Tag::with_meta(
    ///     "beta".into(),
    ///     target,
    ///     None,
    ///     "Beta release".into(),
    ///     meta,
    /// ).unwrap();
    /// assert_eq!(tag.timezone_offset(), -300);
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
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the hash of the object this tag points to.
    #[must_use]
    pub const fn target(&self) -> &Hash {
        &self.target
    }

    /// Returns a reference to the tagger’s identity, if present.
    #[must_use]
    pub const fn tagger(&self) -> Option<&UserID> {
        self.tagger.as_ref()
    }

    /// Returns the tag message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the Unix‑epoch timestamp (seconds since 1970‑01‑01 00:00:00 UTC).
    #[must_use]
    pub const fn timestamp(&self) -> i64 {
        self.timestamp
    }

    /// Returns the timezone offset in minutes from UTC.
    #[must_use]
    pub const fn timezone_offset(&self) -> i16 {
        self.timezone_offset
    }

    /// Returns the optional character‑encoding hint (e.g. `"UTF-8"`).
    #[must_use]
    pub fn encoding(&self) -> Option<&str> {
        self.encoding.as_deref()
    }
}
