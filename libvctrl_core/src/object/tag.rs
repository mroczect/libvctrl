//! Builder for [`Tag`] objects.
//!
//! A [`TagBuilder`] constructs a [`Tag`] step by step. The mandatory fields
//! are `name` and `target`. The `tagger` and `message` are optional.
//!
//! # Why use a builder?
//!
//! Tags have optional fields (`tagger`, `message`) and the builder pattern
//! makes it clear which fields are being set. It also provides a natural
//! place to validate that the required fields are present.
//!
//! ```rust
//! # use libvctrl_core::object::TagBuilder;
//! # use libvctrl_handler::*;
//! let hash = Hash::from_bytes(&[0xAB; 64]).unwrap();
//! let tagger = UserID::new("Releaser".into(), "rel@example.com".into()).unwrap();
//!
//! let tag = TagBuilder::new()
//!     .name("v2.0.0")
//!     .target(hash)
//!     .tagger(tagger)
//!     .message("Final release")
//!     .build()
//!     .unwrap();
//!
//! // Lightweight tag (no tagger, no message)
//! let lightweight = TagBuilder::new()
//!     .name("quick-fix")
//!     .target(hash)
//!     .build()
//!     .unwrap();
//! ```
//!
//! # Error handling
//!
//! The `build()` method returns `Result<Tag, VctrlError>`. It will fail with:
//! - `VctrlError::Other` if `name` or `target` is missing.
//! - `VctrlError::InvalidName` if the name is empty or too long (delegated
//!   to [`Tag::new`]).

use libvctrl_handler::{Hash, Tag, UserID, VctrlError};

/// Builder for [`Tag`] objects.
///
/// # Example
///
/// ```rust
/// # use libvctrl_core::object::TagBuilder;
/// # use libvctrl_handler::*;
/// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
/// let tag = TagBuilder::new()
///     .name("v1.0")
///     .target(hash)
///     .build()
///     .unwrap();
/// ```
#[derive(Debug, Default)]
pub struct TagBuilder {
    name: Option<String>,
    target: Option<Hash>,
    tagger: Option<UserID>,
    message: Option<String>,
}

impl TagBuilder {
    /// Creates a new empty builder.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            name: None,
            target: None,
            tagger: None,
            message: None,
        }
    }

    /// Sets the tag name.
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the target hash.
    #[must_use]
    pub const fn target(mut self, target: Hash) -> Self {
        self.target = Some(target);
        self
    }

    /// Sets the tagger (optional).
    #[must_use]
    pub fn tagger(mut self, tagger: UserID) -> Self {
        self.tagger = Some(tagger);
        self
    }

    /// Sets the tag message (optional).
    #[must_use]
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    /// Builds the tag, validating the name and required fields.
    ///
    /// # Errors
    /// Returns [`VctrlError::InvalidName`] if the name is invalid, or
    /// [`VctrlError::Other`] if `name` or `target` is missing.
    pub fn build(self) -> Result<Tag, VctrlError> {
        let name = self
            .name
            .ok_or_else(|| VctrlError::Other("tag name is required".into()))?;
        let target = self
            .target
            .ok_or_else(|| VctrlError::Other("target is required".into()))?;
        Tag::new(name, target, self.tagger, self.message.unwrap_or_default())
    }
}
