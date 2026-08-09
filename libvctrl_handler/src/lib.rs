// #![forbid(unsafe_code)]
// #![deny(
//     clippy::all,
//     clippy::pedantic,
//     clippy::nursery,
//     clippy::cargo,
//     missing_docs,
//     rust_2018_idioms,
//     unreachable_pub,
//     unused_crate_dependencies,
//     unused_qualifications
// )]

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
pub use traits::{Decoder, Encoder, Hasher, ObjectStore, RefStore, Signer, Transport, Verifier};
pub use types::{Blob, Commit, CommitMeta, Hash, Tag, Tree, TreeEntry, UserID};
