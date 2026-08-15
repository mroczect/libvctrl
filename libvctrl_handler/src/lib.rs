//! A library for working with Git objects and repository data.

#![forbid(unsafe_code)]
#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::cargo,
    clippy::nursery,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub,
    unused_crate_dependencies,
    unused_qualifications
)]

pub mod constants;
pub mod enums;
pub mod errors;
pub mod macros;
pub mod traits;
pub mod types;

pub use constants::{
    HASH_LENGTH, MAX_BLOB_SIZE, MAX_MESSAGE_LENGTH, MAX_NAME_LENGTH, MAX_TREE_ENTRIES,
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
