//! Tag object representation.

use super::commit::CommitMeta;
use super::hash::Hash;
use super::user_id::UserID;
use crate::errors::VctrlError;
use crate::types::validate_name;

/// A Git tag object.
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
    /// Creates a new tag without timestamp metadata.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::InvalidName`] if the tag name is invalid.
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

    /// Creates a new tag with timestamp metadata.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::InvalidName`] if the tag name is invalid.
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

    /// Returns the tag timestamp.
    #[must_use]
    pub const fn timestamp(&self) -> i64 {
        self.timestamp
    }

    /// Returns the timezone offset.
    #[must_use]
    pub const fn timezone_offset(&self) -> i16 {
        self.timezone_offset
    }

    /// Returns the encoding, if any.
    #[must_use]
    pub fn encoding(&self) -> Option<&str> {
        self.encoding.as_deref()
    }
}
