//! landmarks in the commit history, such as release versions (`v1.0.0`).
//! # let tagger = UserID::new("Release Bot".into(), "release@example.com".into()).unwrap();
//! # let tagger = UserID::new("Bob".into(), "bob@example.com".into()).unwrap();
use crate::errors::VctrlError;
use crate::types::commit::CommitMeta;
use crate::types::hash::Hash;
use crate::types::user_id::UserID;

use super::validate_name;

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

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn target(&self) -> &Hash {
        &self.target
    }

    #[must_use]
    pub const fn tagger(&self) -> Option<&UserID> {
        self.tagger.as_ref()
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    #[must_use]
    pub const fn timestamp(&self) -> i64 {
        self.timestamp
    }

    #[must_use]
    pub const fn timezone_offset(&self) -> i16 {
        self.timezone_offset
    }

    #[must_use]
    pub fn encoding(&self) -> Option<&str> {
        self.encoding.as_deref()
    }
}
