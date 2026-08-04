pub mod binary;
pub use binary::*;

use crate::domain::commit::Commit;
use crate::domain::tag::Tag;
use crate::domain::tree::Tree;

pub trait Encoder {
    fn encode_tree(&self, tree: &Tree, buf: &mut Vec<u8>);
    fn encode_commit(&self, commit: &Commit, buf: &mut Vec<u8>);
    fn encode_tag(&self, tag: &Tag, buf: &mut Vec<u8>);
}
