use crate::constants::MAX_NAME_LENGTH;
use crate::errors::VctrlError;

pub mod blob;
pub mod commit;
pub mod hash;
pub mod tag;
pub mod tree;
pub mod user_id;

pub use blob::Blob;
pub use commit::{Commit, CommitMeta};
pub use hash::Hash;
pub use tag::Tag;
pub use tree::{Tree, TreeEntry};
pub use user_id::UserID;

fn validate_name(name: &str) -> Result<(), VctrlError> {
    if name.is_empty() {
        return Err(VctrlError::InvalidName("name is empty".into()));
    }
    if name.len() > MAX_NAME_LENGTH {
        return Err(VctrlError::InvalidName(format!(
            "name exceeds maximum length {MAX_NAME_LENGTH}: '{name}'"
        )));
    }
    Ok(())
}
