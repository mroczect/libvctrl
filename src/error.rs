pub use crate::domain::hash::HashError;
pub use crate::domain::tree::TreeError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VctrlError {
    #[error("hash error: {0}")]
    Hash(#[from] HashError),
    #[error("tree error: {0}")]
    Tree(#[from] TreeError),
    #[error("object not found: {0}")]
    NotFound(String),
    #[error("invalid reference: {0}")]
    InvalidRef(String),
    #[error("merge conflict at '{entry}': {reason}")]
    MergeConflict { entry: String, reason: String },
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("backend error: {0}")]
    Backend(String),
    #[error("{0}")]
    Other(String),
}
impl From<serde_json::Error> for VctrlError {
    fn from(e: serde_json::Error) -> Self {
        VctrlError::Serialization(e.to_string())
    }
}
