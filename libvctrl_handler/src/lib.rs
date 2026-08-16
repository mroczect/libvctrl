//! handler for lnvctrl

#![forbid(unsafe_code)]
#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub,
    unused_crate_dependencies,
    unused_qualifications
)]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::missing_safety_doc,
    clippy::must_use_candidate
)]

/// Constants related to Git object formats.
pub mod constants;

/// Enums for Git object types.
pub mod enums;

/// Error types used throughout the crate.
pub mod errors;

/// Helper macros for the crate.
pub mod macros;

/// Traits defining repository operations.
pub mod traits;

/// Core data types for Git objects.
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
