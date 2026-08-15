use crate::errors::VctrlError;
use crate::types::blob::Blob;
use crate::types::commit::Commit;
use crate::types::tag::Tag;
use crate::types::tree::Tree;

pub trait Decoder {
    fn decode_blob(&self, data: &[u8]) -> Result<Blob, VctrlError>;

    fn decode_tree(&self, data: &[u8]) -> Result<Tree, VctrlError>;

    fn decode_commit(&self, data: &[u8]) -> Result<Commit, VctrlError>;

    fn decode_tag(&self, data: &[u8]) -> Result<Tag, VctrlError>;
}
