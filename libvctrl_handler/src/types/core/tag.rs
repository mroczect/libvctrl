//! Tag object representation.

use super::commit::CommitMeta;
use super::hash::Hash;
use super::user_id::UserID;
use crate::constants::MAX_MESSAGE_LENGTH;
use crate::errors::VctrlError;
use crate::types::validate_ref_name;

/// A Git tag object.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tag {
    name: String,
    target: Hash,
    tagger: Option<UserID>,
    message: String,
    meta: CommitMeta,
}

impl Tag {
    /// Creates a new tag without timestamp metadata.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if validation fails.
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
    /// # Errors
    ///
    /// Returns [`VctrlError`] if validation fails.
    pub fn with_meta(
        name: String,
        target: Hash,
        tagger: Option<UserID>,
        message: String,
        meta: CommitMeta,
    ) -> Result<Self, VctrlError> {
        validate_ref_name(&name)?;
        if message.len() > MAX_MESSAGE_LENGTH as usize {
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
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the target hash.
    #[must_use]
    pub const fn target(&self) -> &Hash {
        &self.target
    }

    /// Returns the tagger, if any.
    #[must_use]
    pub const fn tagger(&self) -> Option<&UserID> {
        self.tagger.as_ref()
    }

    /// Returns the tag message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the tag metadata.
    #[must_use]
    pub const fn meta(&self) -> &CommitMeta {
        &self.meta
    }
}
