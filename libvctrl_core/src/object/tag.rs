//! # Tag Builder
//!
//! This module provides a fluent, ownership-driven builder for constructing
//! [`Tag`] objects. The builder pattern is used because a [`Tag`] is an
//! immutable value object with several fields, some mandatory and some
//! optional. The builder allows setting each field separately and defers
//! validation and object creation to the final `build()` call.

use libvctrl_handler::{CommitMeta, Hash, Tag, UserID, VctrlError};

/// A builder for creating [`Tag`] objects.
///
/// `TagBuilder` provides a safe, ergonomic way to construct a [`Tag`] by
/// setting fields individually. The builder consumes itself with each method
/// and returns a new builder state, enabling method chaining. The final
/// `build()` call validates required fields and constructs the [`Tag`].
///
/// # Why this struct exists
///
/// The [`Tag`] constructor may fail if required fields are missing or
/// validation fails. A builder delays those operations, allowing callers to
/// supply fields in any order and to provide optional values only when
/// necessary. It also gives a uniform construction API across all object
/// types in this crate.
///
/// # How it works
///
/// The builder stores each field in an `Option`. Required fields (`name`,
/// `target`) must be set before `build()`; otherwise `build()` returns a
/// [`VctrlError::Other`] describing the missing field. Optional fields
/// (`tagger`, `message`, `meta`) default to `None` (or an empty string for
/// message). `build()` consumes the builder and moves the values into the new
/// [`Tag`].
///
/// # Examples
///
/// Basic construction with a tagger:
///
/// ```
/// # use libvctrl_core::object::TagBuilder;
/// # use libvctrl_handler::{Hash, UserID};
/// let target = Hash::from_bytes(&[0u8; 64]).unwrap();
/// let tagger = UserID::new("Alice".to_owned(), "alice@example.com".to_owned()).unwrap();
///
/// let tag = TagBuilder::new()
///     .name("v1.0.0")
///     .target(target)
///     .tagger(tagger)
///     .message("Release 1.0")
///     .build()
///     .unwrap();
///
/// assert_eq!(tag.name(), "v1.0.0");
/// assert!(tag.tagger().is_some());
/// assert_eq!(tag.message(), "Release 1.0");
/// ```
///
/// Building without a tagger:
///
/// ```
/// # use libvctrl_core::object::TagBuilder;
/// # use libvctrl_handler::Hash;
/// let target = Hash::from_bytes(&[1u8; 64]).unwrap();
///
/// let tag = TagBuilder::new()
///     .name("v2.0.0")
///     .target(target)
///     .build()
///     .unwrap();
///
/// assert_eq!(tag.name(), "v2.0.0");
/// assert!(tag.tagger().is_none());
/// ```
#[derive(Debug, Default)]
pub struct TagBuilder {
    name: Option<String>,
    target: Option<Hash>,
    tagger: Option<UserID>,
    message: Option<String>,
    meta: Option<CommitMeta>,
}

impl TagBuilder {
    /// Creates a new `TagBuilder` with all fields unset.
    ///
    /// The builder is initially empty. Use the setter methods to populate
    /// fields, then call [`build`](Self::build) to produce a [`Tag`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_core::object::TagBuilder;
    /// let builder = TagBuilder::new();
    /// // The builder can be consumed by chaining setters:
    /// let _ = builder.name("v0.0.0"); // Example only; typically followed by target()
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self {
            name: None,
            target: None,
            tagger: None,
            message: None,
            meta: None,
        }
    }

    /// Sets the tag name.
    ///
    /// This method consumes the builder and returns a new builder with `name`
    /// set. The name must be a non-empty string and is validated during
    /// [`build`](Self::build).
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_core::object::TagBuilder;
    /// # use libvctrl_handler::Hash;
    /// let target = Hash::from_bytes(&[2u8; 64]).unwrap();
    ///
    /// let tag = TagBuilder::new()
    ///     .name("v1.2.3")
    ///     .target(target)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(tag.name(), "v1.2.3");
    /// ```
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the target hash.
    ///
    /// This method consumes the builder and returns a new builder with
    /// `target` set. The target must point to another object (usually a commit
    /// or tree) and is validated during [`build`](Self::build).
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_core::object::TagBuilder;
    /// # use libvctrl_handler::Hash;
    /// let target = Hash::from_bytes(&[3u8; 64]).unwrap();
    ///
    /// let tag = TagBuilder::new()
    ///     .name("v1.0.0")
    ///     .target(target)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(tag.target(), target);
    /// ```
    #[must_use]
    pub const fn target(mut self, target: Hash) -> Self {
        self.target = Some(target);
        self
    }

    /// Sets the tagger.
    ///
    /// This method consumes the builder and returns a new builder with
    /// `tagger` set. The tagger is optional; omit this method to create an
    /// unsigned tag.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_core::object::TagBuilder;
    /// # use libvctrl_handler::{Hash, UserID};
    /// let target = Hash::from_bytes(&[4u8; 64]).unwrap();
    /// let tagger = UserID::new("Bob".to_owned(), "bob@example.com".to_owned()).unwrap();
    ///
    /// let tag = TagBuilder::new()
    ///     .name("v1.0.0")
    ///     .target(target)
    ///     .tagger(tagger)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert!(tag.tagger().is_some());
    /// ```
    #[must_use]
    pub fn tagger(mut self, tagger: UserID) -> Self {
        self.tagger = Some(tagger);
        self
    }

    /// Sets the tag message.
    ///
    /// This method consumes the builder and returns a new builder with
    /// `message` set. The message is optional and defaults to an empty string
    /// if not set.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_core::object::TagBuilder;
    /// # use libvctrl_handler::Hash;
    /// let target = Hash::from_bytes(&[5u8; 64]).unwrap();
    ///
    /// let tag = TagBuilder::new()
    ///     .name("v1.0.0")
    ///     .target(target)
    ///     .message("Annotated tag")
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(tag.message(), "Annotated tag");
    /// ```
    #[must_use]
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    /// Sets the tag metadata.
    ///
    /// This method consumes the builder and returns a new builder with `meta`
    /// set. Metadata includes timestamp, timezone offset, and optional
    /// encoding. If omitted, the [`Tag`] is created without metadata.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_core::object::TagBuilder;
    /// # use libvctrl_handler::{CommitMeta, Hash};
    /// let target = Hash::from_bytes(&[6u8; 64]).unwrap();
    /// let meta = CommitMeta::new(1_700_000_000, 0, None).unwrap();
    ///
    /// let tag = TagBuilder::new()
    ///     .name("v1.0.0")
    ///     .target(target)
    ///     .meta(meta)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(tag.meta().timestamp(), 1_700_000_000);
    /// ```
    #[must_use]
    pub fn meta(mut self, meta: CommitMeta) -> Self {
        self.meta = Some(meta);
        self
    }

    /// Builds the [`Tag`].
    ///
    /// This consumes the builder, moves all fields into the new [`Tag`], and
    /// performs validation. Required fields (`name` and `target`) must be set;
    /// otherwise an error is returned.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::Other`] if `name` or `target` is missing.
    /// If metadata is present, validation errors from
    /// [`Tag::with_meta`](libvctrl_handler::Tag::with_meta) may also be
    /// returned. Similarly, if metadata is absent, errors from
    /// [`Tag::new`](libvctrl_handler::Tag::new) are propagated.
    ///
    /// # Examples
    ///
    /// Successful build:
    ///
    /// ```
    /// # use libvctrl_core::object::TagBuilder;
    /// # use libvctrl_handler::Hash;
    /// let target = Hash::from_bytes(&[7u8; 64]).unwrap();
    ///
    /// let tag = TagBuilder::new()
    ///     .name("v1.0.0")
    ///     .target(target)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(tag.name(), "v1.0.0");
    /// ```
    ///
    /// Missing required field:
    ///
    /// ```
    /// # use libvctrl_core::object::TagBuilder;
    /// let result = TagBuilder::new().name("v1.0.0").build();
    /// assert!(result.is_err());
    /// ```
    pub fn build(self) -> Result<Tag, VctrlError> {
        let name = self
            .name
            .ok_or_else(|| VctrlError::Other("tag name is required".into()))?;
        let target = self
            .target
            .ok_or_else(|| VctrlError::Other("target is required".into()))?;

        if let Some(meta) = self.meta {
            Tag::with_meta(
                name,
                target,
                self.tagger,
                self.message.unwrap_or_default(),
                meta,
            )
        } else {
            Tag::new(name, target, self.tagger, self.message.unwrap_or_default())
        }
    }
}
