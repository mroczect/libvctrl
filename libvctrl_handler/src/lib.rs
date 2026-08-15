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

/// Re-export of common constants.
pub use constants::{
    HASH_LENGTH, MAX_BLOB_SIZE, MAX_MESSAGE_LENGTH, MAX_NAME_LENGTH, MAX_TREE_ENTRIES,
};

/// Re-export of the entry kind enum.
pub use enums::EntryKind;

/// Re-export of the error type.
pub use errors::VctrlError;

/// Re-export of core traits.
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

/// Re-export of core data types.
pub use types::{
    Blob, ChangeKind, Commit, CommitMeta, Conflict, FileDelta, Hash, MergeResult, ReflogEntry, Tag,
    Tree, TreeDelta, TreeEntry, UserID,
};
