pub mod binary;
pub use binary::*;

use crate::domain::commit::Commit;
use crate::domain::tag::Tag;
use crate::domain::tree::Tree;
use crate::error::VctrlError;

pub trait Encoder {
    fn encode_tree(&self, tree: &Tree, buf: &mut Vec<u8>) -> Result<(), VctrlError>;
    fn encode_commit(&self, commit: &Commit, buf: &mut Vec<u8>) -> Result<(), VctrlError>;
    fn encode_tag(&self, tag: &Tag, buf: &mut Vec<u8>) -> Result<(), VctrlError>;
}

pub trait Decoder {
    fn decode_tree(&self, data: &[u8]) -> Result<Tree, VctrlError>;
    fn decode_commit(&self, data: &[u8]) -> Result<Commit, VctrlError>;
    fn decode_tag(&self, data: &[u8]) -> Result<Tag, VctrlError>;
}
