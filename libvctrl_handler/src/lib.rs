extern crate alloc;

#[cfg(test)]
use criterion as _;

pub mod constants;
pub mod enums;
pub mod errors;
pub mod macros;
pub mod traits;
pub mod types;
pub mod validation;

pub use constants::{
    HASH_LENGTH, MAX_BLOB_SIZE, MAX_MESSAGE_LENGTH, MAX_NAME_LENGTH, MAX_PARENT_COUNT,
    MAX_TREE_ENTRIES,
};
pub use enums::EntryKind;
pub use errors::VctrlError;
pub use traits::core::{
    blame::{Blame, BlameEntry},
    config::ConfigStore,
    decoder::Decoder,
    diff::TreeDiffer,
    encoder::Encoder,
    hasher::Hasher,
    index::Index,
    object_store::ObjectStore,
    pack::{PackReader, PackWriter},
    ref_store::RefStore,
    reflog::ReflogStore,
    remote::Remote,
    revwalk::RevWalk,
    signer::Signer,
    transport::Transport,
    verifier::Verifier,
};
pub use types::{
    Blob, ChangeKind, Commit, CommitMeta, Conflict, FileDelta, Hash, MergeResult, ReflogEntry, Tag,
    Tree, TreeDelta, TreeEntry, UserID,
};
pub use validation::{
    validate_hash_bytes, validate_name, validate_ref_name, validate_tree_entry_name,
};
