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
    use libvctrl_handler::HASH_LENGTH;

    fn make_hash(fill: u8) -> Hash {
        Hash::from_bytes(&vec![fill; HASH_LENGTH]).unwrap()
    }

    fn make_user_id(name: &str, email: &str) -> UserID {
        UserID::new(name.into(), email.into()).unwrap()
    }

    #[test]
    fn test_build_missing_name() {
        let result = TagBuilder::new().target(make_hash(0)).build();
        assert!(result.is_err(), "should fail without name");
    }

    #[test]
    fn test_build_missing_target() {
        let result = TagBuilder::new().name("v1.0").build();
        assert!(result.is_err(), "should fail without target");
    }

    #[test]
    fn test_build_missing_both() {
        let result = TagBuilder::new().build();
        assert!(result.is_err(), "should fail without name and target");
    }

    #[test]
    fn test_build_success_without_meta() {
        let result = TagBuilder::new()
            .name("v1.0")
            .target(make_hash(0xAA))
            .build();
        assert!(result.is_ok(), "should succeed with name and target");
    }

    #[test]
    fn test_build_success_with_tagger_and_meta() {
        let meta = CommitMeta::new(1_700_000_000, 0, None).unwrap();
        let result = TagBuilder::new()
            .name("release")
            .target(make_hash(0xBB))
            .tagger(make_user_id("Alice", "alice@example.com"))
            .message("v1.0 release")
            .meta(meta)
            .build();
        assert!(result.is_ok(), "should succeed with all fields");
        let tag = result.unwrap();
        assert_eq!(tag.name(), "release");
        assert!(tag.tagger().is_some());
        assert_eq!(tag.tagger().unwrap().name(), "Alice");
        assert_eq!(tag.message(), "v1.0 release");
    }

    #[test]
    fn test_build_default_message_when_none() {
        let result = TagBuilder::new()
            .name("v2.0")
            .target(make_hash(0xCC))
            .build();
        assert!(result.is_ok());
        let tag = result.unwrap();
        assert_eq!(tag.message(), "", "message should default to empty string");
    }

    #[test]
    fn test_build_without_tagger() {
        let meta = CommitMeta::new(1_700_000_000, 0, Some("UTF-8".into())).unwrap();
        let result = TagBuilder::new()
            .name("lightweight")
            .target(make_hash(0xDD))
            .meta(meta)
            .build();
        assert!(result.is_ok());
        let tag = result.unwrap();
        assert!(tag.tagger().is_none(), "tagger should be None when not set");
    }
}
