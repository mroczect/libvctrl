use super::commit::CommitMeta;
use super::hash::Hash;
use super::user_id::UserID;
use crate::constants::MAX_MESSAGE_LENGTH;
use crate::errors::VctrlError;
use crate::types::validate_ref_name;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tag {
    name: String,
    target: Hash,
    tagger: Option<UserID>,
    message: String,
    meta: CommitMeta,
}

impl Tag {
    pub fn new(
        name: String,
        target: Hash,
        tagger: Option<UserID>,
        message: String,
    ) -> Result<Self, VctrlError> {
        Self::with_meta(name, target, tagger, message, CommitMeta::default())
    }

    pub fn with_meta(
        name: String,
        target: Hash,
        tagger: Option<UserID>,
        message: String,
        meta: CommitMeta,
    ) -> Result<Self, VctrlError> {
        validate_ref_name(&name)?;
        let max_len = usize::try_from(MAX_MESSAGE_LENGTH).unwrap_or(usize::MAX);
        if message.len() > max_len {
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
    pub const fn meta(&self) -> &CommitMeta {
        &self.meta
    }
}
