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
