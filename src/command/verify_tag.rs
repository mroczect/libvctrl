use crate::codec::Encoder;
use crate::command::Command;
use crate::crypto::Verifier;
use crate::domain::hash::Hash;
use crate::domain::object::Object;
use crate::error::VctrlError;
use crate::hashing::Hasher;
use crate::storage::traits::{ObjectStore, RefStore};

pub struct VerifyTag {
    pub tag_hash: Hash,
    pub verifier: Box<dyn Verifier>,
    pub encoder: Box<dyn Encoder>,
    pub hasher: Box<dyn Hasher>,
}

impl Command for VerifyTag {
    type Output = bool;

    fn execute(
        &self,
        store: &mut dyn ObjectStore,
        _refs: &mut dyn RefStore,
    ) -> Result<bool, VctrlError> {
        let tag = match store.get(&self.tag_hash)? {
            Some(Object::Tag(t)) => *t,
            _ => return Err(VctrlError::NotFound("tag not found".into())),
        };

        let sig_bytes = match &tag.signature {
            Some(s) => s.clone(),
            None => return Ok(false),
        };

        let pre_sig_tag = crate::domain::tag::Tag {
            target: tag.target,
            tagger: tag.tagger.clone(),
            timestamp: tag.timestamp,
            message: tag.message.clone(),
            signature: None,
        };

        let mut buf = Vec::new();
        self.encoder.encode_tag(&pre_sig_tag, &mut buf)?;
        let pre_sig_hash = self.hasher.hash_tag_encoded(&buf);

        self.verifier.verify(pre_sig_hash.as_bytes(), &sig_bytes)
    }
}
