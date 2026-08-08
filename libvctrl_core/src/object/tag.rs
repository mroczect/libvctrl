//! Builder for [`Tag`] objects.

use libvctrl_handler::{Hash, Tag, UserID, VctrlError};

/// Builder for [`Tag`] objects.
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

    /// Sets the tagger.
    #[must_use]
    pub fn tagger(mut self, tagger: UserID) -> Self {
        self.tagger = Some(tagger);
        self
    }

    /// Sets the tag message.
    #[must_use]
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    /// Builds the tag, validating the name.
    ///
    /// # Errors
    /// Returns [`VctrlError::InvalidName`] if the name is invalid.
    ///
    /// # Panics
    /// Panics if `name` or `target` have not been set.
    pub fn build(self) -> Result<Tag, VctrlError> {
        Tag::new(
            self.name.expect("name not set"),
            self.target.expect("target not set"),
            self.tagger,
            self.message.unwrap_or_default(),
        )
    }
}
