use crate::errors::VctrlError;
use crate::types::blob::Blob;
use crate::types::commit::Commit;
use crate::types::tag::Tag;
use crate::types::tree::Tree;

pub trait Encoder {
    fn encode_blob(&self, blob: &Blob) -> Result<Vec<u8>, VctrlError>;

    fn encode_tree(&self, tree: &Tree) -> Result<Vec<u8>, VctrlError>;

    fn encode_commit(&self, commit: &Commit) -> Result<Vec<u8>, VctrlError>;

    fn encode_tag(&self, tag: &Tag) -> Result<Vec<u8>, VctrlError>;
}
