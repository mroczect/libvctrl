#![forbid(unsafe_code)]
#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub,
    unused_qualifications
)]

//! Core facade crate for the version control SDK.

/// Re‑export of the contracts crate.
pub use libvctrl_handler as handler;

/// Re‑export of the reference implementations crate.
pub use libvctrl_core as reference;

/// Re‑export of the cryptographic primitives crate.
pub use libvctrl_sha512 as crypto;

/// Re‑export of system constants.
pub use handler::constants;

/// Re‑export of logical object type enumerations.
pub use handler::enums;

/// Re‑export of error types.
pub use handler::errors;

/// Re‑export of helper macros.
pub use handler::macros;

/// Re‑export of core traits.
pub use handler::traits;

/// Re‑export of core data types.
pub use handler::types;

/// Re‑export of fundamental constants.
pub use handler::{
    HASH_LENGTH, MAX_BLOB_SIZE, MAX_MESSAGE_LENGTH, MAX_NAME_LENGTH, MAX_TREE_ENTRIES,
};

/// Re‑export of the entry kind enum.
pub use handler::EntryKind;

/// Re‑export of the unified error type.
pub use handler::VctrlError;

/// Re‑export of all core traits.
pub use handler::{Decoder, Encoder, Hasher, ObjectStore, RefStore, Signer, Transport, Verifier};

/// Re‑export of all core data structures.
pub use handler::{Blob, Commit, CommitMeta, Hash, Tag, Tree, TreeEntry, UserID};

/// Re‑export of the binary codec module.
pub use reference::codec;

/// Re‑export of the object builders module.
pub use reference::object;

/// Re‑export of the storage implementations module.
pub use reference::store;

/// Re‑export of the validation utilities module.
pub use reference::validate;

/// Re‑export of the binary decoder.
pub use reference::codec::BinaryDecoder;

/// Re‑export of the binary encoder.
pub use reference::codec::BinaryEncoder;

/// Re‑export of the SHA‑512 hasher adapter.
pub use reference::hash::Sha512Hasher;

/// Re‑export of the blob builder.
pub use reference::object::BlobBuilder;

/// Re‑export of the commit builder.
pub use reference::object::CommitBuilder;

/// Re‑export of the tag builder.
pub use reference::object::TagBuilder;

/// Re‑export of the tree builder.
pub use reference::object::TreeBuilder;

/// Re‑export of the tree entry builder.
pub use reference::object::TreeEntryBuilder;

/// Re‑export of the in‑memory reference store.
pub use reference::store::MemoryRefStore;

/// Re‑export of the in‑memory object store.
pub use reference::store::MemoryStore;

/// Re‑export of the hash validation function.
pub use reference::validate::hash::validate_hash_bytes;

/// Re‑export of the name validation function.
pub use reference::validate::name::validate_name;
