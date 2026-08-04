use crate::codec::Encoder;
use crate::command::Command;
use crate::domain::hash::Hash;
use crate::domain::object::Object;
use crate::domain::tag::Tag;
use crate::domain::user::UserID;
use crate::error::VctrlError;
use crate::hashing::Hasher;
use crate::storage::traits::{ObjectStore, RefStore};

pub struct CreateLightweightTag {
    pub name: String,
    pub target: Hash,
}

impl Command for CreateLightweightTag {
    type Output = ();
    fn execute(
        &self,
        _store: &mut dyn ObjectStore,
        refs: &mut dyn RefStore,
    ) -> Result<(), VctrlError> {
        let ref_name = format!("refs/tags/{}", self.name);
        refs.set_ref(&ref_name, &self.target)
    }
}

pub struct CreateAnnotatedTag {
    pub name: String,
    pub target: Hash,
    pub tagger: UserID,
    pub message: String,
    pub encoder: Box<dyn Encoder>,
    pub hasher: Box<dyn Hasher>,
}

impl Command for CreateAnnotatedTag {
    type Output = Hash;
    fn execute(
        &self,
        store: &mut dyn ObjectStore,
        refs: &mut dyn RefStore,
    ) -> Result<Hash, VctrlError> {
        let tag = Tag::new(self.target, self.tagger.clone(), self.message.clone());
        let mut buf = Vec::new();
        self.encoder.encode_tag(&tag, &mut buf)?;
        let hash = self.hasher.hash_tag_encoded(&buf);
        store.put(&hash, &Object::Tag(Box::new(tag)))?;

        let ref_name = format!("refs/tags/{}", self.name);
        refs.set_ref(&ref_name, &hash)?;

        Ok(hash)
    }
}

pub struct DeleteTag {
    pub name: String,
}

impl Command for DeleteTag {
    type Output = ();
    fn execute(
        &self,
        _store: &mut dyn ObjectStore,
        refs: &mut dyn RefStore,
    ) -> Result<(), VctrlError> {
        let ref_name = format!("refs/tags/{}", self.name);
        refs.delete_ref(&ref_name)
    }
}

pub struct ListTags;

impl Command for ListTags {
    type Output = Vec<String>;
    fn execute(
        &self,
        _store: &mut dyn ObjectStore,
        refs: &mut dyn RefStore,
    ) -> Result<Vec<String>, VctrlError> {
        let refs = refs.list_refs("refs/tags/")?;
        Ok(refs
            .into_iter()
            .map(|r| r.trim_start_matches("refs/tags/").to_string())
            .collect())
    }
}
