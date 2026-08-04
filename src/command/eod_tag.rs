use crate::codec::Encoder;
use crate::command::Command;
use crate::domain::hash::Hash;
use crate::domain::object::Object;
use crate::domain::tag::Tag;
use crate::domain::user::UserID;
use crate::error::VctrlError;
use crate::hashing::Hasher;
use crate::storage::traits::{ObjectStore, RefStore};
use chrono::Local;

pub struct CreateEndOfDayTag {
    pub tagger: UserID,
    pub message: Option<String>,
    pub date: Option<chrono::NaiveDate>,
    pub encoder: Box<dyn Encoder>,
    pub hasher: Box<dyn Hasher>,
}

impl Command for CreateEndOfDayTag {
    type Output = Hash;

    fn execute(
        &self,
        store: &mut dyn ObjectStore,
        refs: &mut dyn RefStore,
    ) -> Result<Hash, VctrlError> {
        let head_hash = refs
            .head()?
            .ok_or_else(|| VctrlError::Other("no HEAD – cannot create EOD tag".into()))?;

        let date = self.date.unwrap_or_else(|| Local::now().date_naive());
        let date_str = date.format("%Y-%m-%d").to_string();
        let tag_name = format!("EOD-{}", date_str);

        let message = self
            .message
            .clone()
            .unwrap_or_else(|| format!("End of day {}", date_str));

        let tag = Tag::new(head_hash, self.tagger.clone(), message);
        let mut buf = Vec::new();
        self.encoder.encode_tag(&tag, &mut buf)?;
        let tag_hash = self.hasher.hash_tag_encoded(&buf);

        store.put(&tag_hash, &Object::Tag(Box::new(tag)))?;

        let ref_name = format!("refs/tags/{}", tag_name);
        refs.set_ref(&ref_name, &tag_hash)?;

        Ok(tag_hash)
    }
}
