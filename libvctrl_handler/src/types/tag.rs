//! # Tags – Named Pointers to Objects
//!
//! A `Tag` is a human‑readable name that points to a specific object (usually a
//! commit). Tags are **immutable** references – once created, they always refer
//! to the same object (unlike branches, which move). They serve as stable
//! landmarks in the commit history, such as release versions (`v1.0.0`).
//!
//! ## Annotated vs Lightweight Tags
//!
//! - **Annotated tag** – stores extra metadata: tagger identity, a message,
//!   timestamp, and encoding. This is the recommended type for releases.
//! - **Lightweight tag** – stores only a name and a target hash. It is
//!   essentially a reference that never changes. It is created by passing
//!   `None` for the tagger and an empty message.
//!
//! ## Validation
//!
//! Tag **names** are validated by the constructor:
//! - Must be non‑empty.
//! - Must not exceed [`MAX_NAME_LENGTH`](crate::MAX_NAME_LENGTH) (255 bytes).
//!
//! If validation fails, [`VctrlError::InvalidName`] is returned.
//!
//! ## Relationship to References
//!
//! Tags are typically stored in a `RefStore` under names like `refs/tags/v1.0`.
//! They can be looked up and listed via the [`RefStore`](crate::RefStore) API.
//!
//! ## Examples
//!
//! ### Creating an Annotated Tag
//!
//! ```rust
//! use libvctrl_handler::{Hash, Tag, UserID, CommitMeta};
//!
//! # let commit_hash = Hash::from_bytes(&[0x66; 64]).unwrap();
//! # let tagger = UserID::new("Release Bot".into(), "release@example.com".into()).unwrap();
//! let tag = Tag::new(
//!     "v1.0.0".into(),
//!     commit_hash,
//!     Some(tagger.clone()),
//!     "Stable release".into(),
//! )?;
//! # Ok::<_, libvctrl_handler::VctrlError>(())
//! ```
//!
//! ### Creating a Lightweight Tag
//!
//! ```rust
//! # use libvctrl_handler::*;
//! # let commit_hash = Hash::from_bytes(&[0x77; 64]).unwrap();
//! let tag = Tag::new("temp".into(), commit_hash, None, "".into())?;
//! assert!(tag.tagger().is_none());
//! # Ok::<_, libvctrl_handler::VctrlError>(())
//! ```
//!
//! ### Using Custom Metadata
//!
//! ```rust
//! # use libvctrl_handler::*;
//! # let commit_hash = Hash::from_bytes(&[0x88; 64]).unwrap();
//! # let tagger = UserID::new("Bob".into(), "bob@example.com".into()).unwrap();
//! let meta = CommitMeta {
//!     timestamp: 1672531200,
//!     timezone_offset: -300, // UTC-5
//!     encoding: Some("UTF-8".into()),
//! };
//! let tag = Tag::with_meta(
//!     "v2.0".into(),
//!     commit_hash,
//!     Some(tagger),
//!     "Major release".into(),
//!     meta,
//! )?;
//! assert_eq!(tag.timestamp(), 1672531200);
//! # Ok::<_, libvctrl_handler::VctrlError>(())
//! ```
//!
//! ## Serialization
//!
//! Tags are encoded with an [`Encoder`](crate::Encoder) and decoded with a
//! [`Decoder`](crate::Decoder). The reference binary format includes the tag
//! name, target hash, optional tagger, message, and metadata.

use crate::errors::VctrlError;
use crate::types::commit::CommitMeta;
use crate::types::hash::Hash;
use crate::types::user_id::UserID;

use super::validate_name;

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
    /// This is the simplest way to create a tag. For explicit timestamp or
    /// encoding, use [`with_meta`](Self::with_meta).
    ///
    /// # Arguments
    ///
    /// * `name` – The tag name (must be valid).
    /// * `target` – The hash of the object (typically a commit) this tag points to.
    /// * `tagger` – Optional identity of the person who created the tag.
    ///   `None` indicates a lightweight tag (no annotation).
    /// * `message` – The tag message. For lightweight tags, this is often empty.
    ///
    /// # Errors
    /// Returns [`VctrlError::InvalidName`] if the name is empty or too long.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use libvctrl_handler::*;
    /// # let commit_hash = Hash::from_bytes(&[0x99; 64]).unwrap();
    /// # let tagger = UserID::new("Alice".into(), "alice@example.com".into()).unwrap();
    /// let tag = Tag::new("release".into(), commit_hash, Some(tagger), "Minor fix".into())?;
    /// # Ok::<_, libvctrl_handler::VctrlError>(())
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

    /// Creates a new `Tag` with explicit metadata.
    ///
    /// This allows full control over the tag’s timestamp, timezone, and encoding.
    ///
    /// # Arguments
    ///
    /// * `name` – The tag name.
    /// * `target` – The target hash.
    /// * `tagger` – Optional tagger identity.
    /// * `message` – The tag message.
    /// * `meta` – A [`CommitMeta`] struct containing timestamp, timezone, and encoding.
    ///
    /// # Errors
    /// Returns [`VctrlError::InvalidName`] if the name is invalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use libvctrl_handler::*;
    /// # let commit_hash = Hash::from_bytes(&[0xAA; 64]).unwrap();
    /// # let tagger = UserID::new("Bob".into(), "bob@example.com".into()).unwrap();
    /// let meta = CommitMeta {
    ///     timestamp: 1672531200,
    ///     timezone_offset: 0,
    ///     encoding: Some("UTF-8".into()),
    /// };
    /// let tag = Tag::with_meta(
    ///     "v3.0".into(),
    ///     commit_hash,
    ///     Some(tagger),
    ///     "Major release".into(),
    ///     meta,
    /// )?;
    /// # Ok::<_, libvctrl_handler::VctrlError>(())
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
    /// This is the human‑readable identifier, e.g., `"v1.0.0"`.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the target hash (usually a commit).
    ///
    /// This is the object that the tag points to.
    #[must_use]
    pub const fn target(&self) -> &Hash {
        &self.target
    }

    /// Returns the tagger, if any.
    ///
    /// If `None`, this is a lightweight tag (no annotation).
    /// If `Some`, this is an annotated tag with a creator identity.
    #[must_use]
    pub const fn tagger(&self) -> Option<&UserID> {
        self.tagger.as_ref()
    }

    /// Returns the tag message.
    ///
    /// For lightweight tags, this is typically an empty string.
    /// For annotated tags, it contains a description of the tag.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Unix timestamp (seconds since epoch). 0 if not set.
    ///
    /// A value of 0 indicates that the timestamp was not explicitly set
    /// (default metadata). This is common for lightweight tags or when
    /// the time is unimportant.
    #[must_use]
    pub const fn timestamp(&self) -> i64 {
        self.timestamp
    }

    /// Timezone offset in minutes east of UTC. 0 if not set.
    ///
    /// Positive values are east of UTC, negative values west.
    /// A value of 0 means either UTC or "not set".
    #[must_use]
    pub const fn timezone_offset(&self) -> i16 {
        self.timezone_offset
    }

    /// Encoding (e.g., "UTF-8") if set.
    ///
    /// This indicates the character encoding of the tag message.
    /// `None` means the encoding is not specified (interpret as UTF-8).
    #[must_use]
    pub fn encoding(&self) -> Option<&str> {
        self.encoding.as_deref()
    }
}
