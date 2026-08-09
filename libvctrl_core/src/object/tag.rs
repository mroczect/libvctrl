//! Builder pattern for constructing [`Tag`](libvctrl_handler::Tag) objects.
//!
//! # Purpose
//! This module provides the [`TagBuilder`], an ergonomic utility for
//! incrementally assembling version control tags. It provides a fluent API to
//! configure required and optional fields before finalizing the immutable
//! [`Tag`] struct.
//!
//! # Design rationale
//! - **API Consistency**: Similar to `CommitBuilder` and `BlobBuilder`, this
//!   builder provides a uniform construction experience across all version
//!   control objects.
//! - **Required vs. Optional Handling**: The builder enforces that mandatory
//!   fields (`name`, `target`) are provided before construction. It returns a
//!   `Result` during `build()` to gracefully handle missing data.
//! - **Ownership Management**: The builder takes ownership of the underlying
//!   data during the configuration phase. When `build` is called, the data is
//!   moved directly into the final `Tag` without cloning.

use libvctrl_handler::{CommitMeta, Hash, Tag, UserID, VctrlError};

/// A builder for creating [`Tag`](libvctrl_handler::Tag) objects.
///
/// # Purpose
/// Provides a fluent interface for assembling a tag's data before finalizing
/// it into an immutable object.
///
/// # Design rationale
/// Implements the standard builder pattern. It derives [`Default`] so it can
/// be easily instantiated, and [`Debug`] for logging purposes. The `build`
/// method consumes `self`, preventing the reuse of the builder after the data
/// has been moved into the final tag.
///
/// # Examples
///
/// Building a standard annotated tag:
///
/// ```
/// use libvctrl_core::object::TagBuilder;
/// use libvctrl_handler::{Hash, UserID};
///
/// let target = Hash::from_bytes(&[0u8; 64]).unwrap();
/// let tagger = UserID::new("Alice".to_string(), "a@b.com".to_string()).unwrap();
///
/// let tag = TagBuilder::new()
///     .name("v1.0.0")
///     .target(target)
///     .tagger(tagger)
///     .message("Initial release")
///     .build()
///     .unwrap();
///
/// assert_eq!(tag.name(), "v1.0.0");
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
    /// Creates a new, empty `TagBuilder`.
    ///
    /// # Design rationale
    /// This is a `const fn`, allowing the builder to be instantiated in
    /// compile-time contexts if needed. All fields are initialized to `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_core::object::TagBuilder;
    ///
    /// let builder = TagBuilder::new();
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

    /// Sets the name of the tag (e.g., "v1.0.0").
    ///
    /// # Design rationale
    /// This is a required field. If `build` is called without setting this,
    /// it will fail. It takes `impl Into<String>` for ergonomics, allowing
    /// string literals to be passed easily.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_core::object::TagBuilder;
    ///
    /// let builder = TagBuilder::new().name("v2.0");
    /// ```
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the target hash that this tag points to (usually a commit hash).
    ///
    /// # Design rationale
    /// This is a required field. The method is `const fn` to maximize
    /// flexibility.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_core::object::TagBuilder;
    /// use libvctrl_handler::Hash;
    ///
    /// let target = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let builder = TagBuilder::new().target(target);
    /// ```
    #[must_use]
    pub const fn target(mut self, target: Hash) -> Self {
        self.target = Some(target);
        self
    }

    /// Sets the optional tagger identity.
    ///
    /// # Design rationale
    /// Unlike commits, tags do not strictly require a tagger. This field
    /// remains optional. If set, it will be included in the final `Tag`.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_core::object::TagBuilder;
    /// use libvctrl_handler::UserID;
    ///
    /// let tagger = UserID::new("Bob".to_string(), "b@c.com".to_string()).unwrap();
    /// let builder = TagBuilder::new().tagger(tagger);
    /// ```
    #[must_use]
    pub fn tagger(mut self, tagger: UserID) -> Self {
        self.tagger = Some(tagger);
        self
    }

    /// Sets the annotation message for the tag.
    ///
    /// # Design rationale
    /// Takes `impl Into<String>` for ergonomics. If not provided, `build`
    /// will default to an empty string.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_core::object::TagBuilder;
    ///
    /// let builder = TagBuilder::new().message("Release candidate");
    /// ```
    #[must_use]
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    /// Sets optional metadata (timestamp, timezone, encoding).
    ///
    /// # Design rationale
    /// If this method is not called, `build` will use default metadata
    /// (timestamp 0, offset 0).
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_core::object::TagBuilder;
    /// use libvctrl_handler::CommitMeta;
    ///
    /// let meta = CommitMeta { timestamp: 1000, ..Default::default() };
    /// let builder = TagBuilder::new().meta(meta);
    /// ```
    #[must_use]
    pub fn meta(mut self, meta: CommitMeta) -> Self {
        self.meta = Some(meta);
        self
    }

    /// Consumes the builder and returns a finalized [`Tag`](libvctrl_handler::Tag).
    ///
    /// # Design rationale
    /// This method consumes `self` to enforce a linear flow. It validates that
    /// all required fields (`name`, `target`) are present, returning a
    /// `Result` to gracefully handle missing data.
    ///
    /// # Errors
    /// Returns [`VctrlError::Other`](libvctrl_handler::VctrlError::Other) if
    /// `name` or `target` have not been set.
    ///
    /// # Internal mechanism
    /// If `meta` was provided, it delegates to `Tag::with_meta`. Otherwise,
    /// it uses `Tag::new`. If `message` was not provided, it uses
    /// `String::default()` (an empty string).
    ///
    /// # Examples
    ///
    /// Successful build:
    ///
    /// ```
    /// use libvctrl_core::object::TagBuilder;
    /// use libvctrl_handler::Hash;
    ///
    /// let target = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let tag = TagBuilder::new()
    ///     .name("v1.0")
    ///     .target(target)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(tag.name(), "v1.0");
    /// ```
    ///
    /// Failed build (missing required fields):
    ///
    /// ```
    /// use libvctrl_core::object::TagBuilder;
    /// use libvctrl_handler::VctrlError;
    ///
    /// let result = TagBuilder::new().build();
    /// assert!(matches!(result, Err(VctrlError::Other(_))));
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
