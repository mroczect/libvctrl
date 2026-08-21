use libvctrl_handler::{CommitMeta, Hash, Tag, UserID, VctrlError};

#[derive(Debug, Default)]
pub struct TagBuilder {
    name: Option<String>,
    target: Option<Hash>,
    tagger: Option<UserID>,
    message: Option<String>,
    meta: Option<CommitMeta>,
}

impl TagBuilder {
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

    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    #[must_use]
    pub const fn target(mut self, target: Hash) -> Self {
        self.target = Some(target);
        self
    }

    #[must_use]
    pub fn tagger(mut self, tagger: UserID) -> Self {
        self.tagger = Some(tagger);
        self
    }

    #[must_use]
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    #[must_use]
    pub fn meta(mut self, meta: CommitMeta) -> Self {
        self.meta = Some(meta);
        self
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    fn hash_byte(byte: u8) -> Result<Hash, VctrlError> {
        Hash::from_bytes(&[byte; 64])
    }

    fn user(name: &str, email: &str) -> Result<UserID, VctrlError> {
        UserID::new(name.to_string(), email.to_string())
    }

    #[test]
    fn build_missing_name_errors() -> Result<(), VctrlError> {
        let result = TagBuilder::new()
            .target(hash_byte(0x01)?)
            .message("msg")
            .build();
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn build_missing_target_errors() {
        let result = TagBuilder::new().name("v1.0").message("msg").build();
        assert!(result.is_err());
    }

    #[test]
    fn build_valid_tag_without_tagger_or_meta() -> Result<(), VctrlError> {
        let name = "v1.0".to_string();
        let target = hash_byte(0x22)?;
        let message = "release".to_string();

        let tag = TagBuilder::new()
            .name(name.clone())
            .target(target)
            .message(message.clone())
            .build()?;

        assert_eq!(tag.name(), name);
        assert_eq!(tag.target(), &target);
        assert!(tag.tagger().is_none());
        assert_eq!(tag.message(), message);
        Ok(())
    }

    #[test]
    fn build_valid_tag_with_tagger_and_meta() -> Result<(), VctrlError> {
        let name = "v2.0".to_string();
        let target = hash_byte(0x23)?;
        let tagger = user("Tagger", "tagger@example.com")?;
        let message = "release".to_string();
        let meta = CommitMeta::new(42, 0, Some("utf-8".to_string()))?;

        let tag = TagBuilder::new()
            .name(name)
            .target(target)
            .tagger(tagger)
            .message(message)
            .meta(meta)
            .build()?;

        assert_eq!(
            tag.tagger()
                .ok_or_else(|| VctrlError::Other("expected tagger".into()))?
                .name(),
            "Tagger"
        );
        assert_eq!(tag.meta().timestamp(), 42);
        assert_eq!(tag.meta().encoding(), Some("utf-8"));
        Ok(())
    }
}
