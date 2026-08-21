use std::io::Read;

use crate::errors::VctrlError;
use crate::types::{Blob, Commit, Tag, Tree};

pub trait Decoder: Send + Sync {
    fn decode_blob<R: Read + Send>(&self, reader: R) -> Result<Blob, VctrlError>;
    fn decode_tree<R: Read + Send>(&self, reader: R) -> Result<Tree, VctrlError>;
    fn decode_commit<R: Read + Send>(&self, reader: R) -> Result<Commit, VctrlError>;
    fn decode_tag<R: Read + Send>(&self, reader: R) -> Result<Tag, VctrlError>;
}
