//! Unified facade for the libvctrl ecosystem.
//!
//! Re-exports all types, traits, constants, and reference implementations
//! from [`libvctrl_handler`] and [`libvctrl_core`].

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
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::missing_safety_doc,
    clippy::must_use_candidate
)]

pub use libvctrl_core as reference;
pub use libvctrl_handler as handler;
pub use libvctrl_sha512 as crypto;

// --- Handler re-exports ---

pub use handler::constants;
pub use handler::enums;
pub use handler::errors;
pub use handler::macros;
pub use handler::traits;
pub use handler::types;
pub use handler::validation;

pub use handler::{
    HASH_LENGTH, MAX_BLOB_SIZE, MAX_MESSAGE_LENGTH, MAX_NAME_LENGTH, MAX_PARENT_COUNT,
    MAX_TREE_ENTRIES,
};

pub use handler::EntryKind;
pub use handler::VctrlError;

pub use handler::{Decoder, Encoder, Hasher, ObjectStore, RefStore, Signer, Transport, Verifier};

pub use handler::{Blob, Commit, CommitMeta, Hash, Tag, Tree, TreeEntry, UserID};

pub use handler::{
    validate_hash_bytes, validate_name, validate_ref_name, validate_tree_entry_name,
};

// --- Core (reference) re-exports ---

pub use reference::codec;
pub use reference::object;
pub use reference::store;

pub use reference::codec::BinaryDecoder;
pub use reference::codec::BinaryEncoder;

pub use reference::hash::Sha512Hasher;

pub use reference::object::BlobBuilder;
pub use reference::object::CommitBuilder;
pub use reference::object::TagBuilder;
pub use reference::object::TreeBuilder;
pub use reference::object::TreeEntryBuilder;

pub use reference::store::MemoryRefStore;
pub use reference::store::MemoryStore;
