//! let tagger = UserID::new("Releaser".into(), "rel@example.com".into()).unwrap();
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
