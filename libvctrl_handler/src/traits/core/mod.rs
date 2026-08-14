//! Core traits for the version control handler.
//!
//! # Purpose
//!
//! This module is the internal home for all behavior contracts in the crate.
//! Each trait is placed in its own submodule to maintain a clean separation
//! of concerns and to keep individual files focused. The traits defined here
//! are re-exported at `libvctrl_handler` (crate root) so that downstream
//! code can import them without delving into the module hierarchy.
//!
//! # Trait Overview
//!
//! The following traits are defined in this module:
//!
//! - `Decoder` – deserializes version control objects from byte slices.
//! - `Encoder` – serializes version control objects into byte vectors.
//! - `Hasher` – computes cryptographic hashes for content addressing.
//! - `ObjectStore` – manages content-addressable storage of raw objects.
//! - `RefStore` – stores and retrieves named references such as branches
//!   and tags.
//! - `RevWalk` – traverses the commit graph by retrieving parent commits.
//! - `Index` – represents a staging area for index operations.
//! - `ReflogStore` – records changes to references for historical safety.
//! - `Signer` – produces cryptographic signatures over data.
//! - `Transport` – abstracts remote object synchronization.
//! - `Verifier` – verifies cryptographic signatures.
//! - `TreeDiffer` - Contract for diffing two tree objects and producing a list of changes
//! - `ConfigStore` – provides access to configuration values.
//!
//! # Design Rationale
//!
//! The decision to place each trait in its own file under `core` provides
//! several benefits:
//!
//! - **Maintainability**: Each file contains only one trait and its
//!   associated documentation, making it easier to navigate and update.
//! - **Reduced merge conflicts**: In a collaborative project, developers
//!   working on different traits are less likely to modify the same file.
//! - **Clear responsibility boundaries**: The module structure mirrors the
//!   separation of concerns in the design.
//! - **Stable public API**: The crate root re-exports keep the public surface
//!   unchanged even if internal module paths evolve.
//!
//! # How to Use
//!
//! You can import individual traits directly from the crate root:
//!
//! ```
//! use libvctrl_handler::{Hasher, ObjectStore, Encoder};
//! ```
//!
//! Or, if you prefer the full path:
//!
//! ```
//! use libvctrl_handler::traits::core::hasher::Hasher;
//! ```
//!
//! Both styles refer to the same trait. The crate root re-export is the
//! recommended approach for ergonomic code.
//!
//! # Example: Checking Trait Existence
//!
//! The following example demonstrates that the traits are publicly accessible
//! and can be used as bounds:
//!
//! ```
//! use libvctrl_handler::traits::core::hasher::Hasher;
//!
//! fn assert_hasher<T: Hasher>() {}
//! ```
//!
//! # Internal Note
//!
//! The `core` module itself is not intended for direct external use beyond
//! advanced scenarios; the crate root re-exports provide the primary API.

/// Defines the `Decoder` trait for deserializing objects.
///
/// # Purpose
///
/// The `Decoder` trait provides the contract for converting
/// byte slices back into in-memory version control objects like
/// `Blob`, `Tree`, `Commit`, and `Tag`.
///
/// # Why a separate module
///
/// Keeping the trait in its own file isolates serialization-related concerns
/// and makes the trait easy to locate. It also allows future extensions to
/// the decoding interface without affecting other modules.
///
/// # Examples
///
/// Importing the trait from this module:
///
/// ```
/// use libvctrl_handler::traits::core::decoder::Decoder;
///
/// fn assert_decoder<T: Decoder>() {}
/// ```
///
/// The same trait is available at the crate root:
///
/// ```
/// use libvctrl_handler::Decoder;
/// ```
pub mod decoder;

/// Defines the `Encoder` trait for serializing objects.
///
/// # Purpose
///
/// The `Encoder` trait defines how version control objects
/// are transformed into byte vectors suitable for storage or transport.
///
/// # Why a separate module
///
/// Serialization logic is kept distinct from deserialization and other
/// concerns, promoting a clean separation of responsibilities. This also
/// allows encoder implementations to be swapped independently.
///
/// # Examples
///
/// Importing the trait from this module:
///
/// ```
/// use libvctrl_handler::traits::core::encoder::Encoder;
///
/// fn assert_encoder<T: Encoder>() {}
/// ```
///
/// The crate root also re-exports it:
///
/// ```
/// use libvctrl_handler::Encoder;
/// ```
pub mod encoder;

/// Defines the `Hasher` trait for content addressing.
///
/// # Purpose
///
/// The `Hasher` trait provides the contract for computing
/// cryptographic hashes from raw data, which is fundamental to
/// content-addressable storage.
///
/// # Why a separate module
///
/// Hashing algorithms can vary (SHA-512, BLAKE3, etc.). Keeping the trait
/// isolated allows the rest of the system to remain agnostic to the specific
/// hash function used.
///
/// # Examples
///
/// Importing the trait from this module:
///
/// ```
/// use libvctrl_handler::traits::core::hasher::Hasher;
///
/// fn assert_hasher<T: Hasher>() {}
/// ```
///
/// The crate root provides the same trait:
///
/// ```
/// use libvctrl_handler::Hasher;
/// ```
pub mod hasher;

/// Defines the `ObjectStore` trait for content-addressable storage.
///
/// # Purpose
///
/// The `ObjectStore` trait defines how raw objects are
/// stored and retrieved using their `Hash` as the key. It
/// supports streaming reads to avoid large allocations.
///
/// # Why a separate module
///
/// Storage backends can be in-memory, on-disk, or remote. Isolating the trait
/// keeps the storage abstraction clean and testable.
///
/// # Examples
///
/// Importing the trait from this module:
///
/// ```
/// use libvctrl_handler::traits::core::object_store::ObjectStore;
///
/// fn assert_object_store<T: ObjectStore>() {}
/// ```
///
/// The crate root re-exports it:
///
/// ```
/// use libvctrl_handler::ObjectStore;
/// ```
pub mod object_store;

/// Defines the `RefStore` trait for named references.
///
/// # Purpose
///
/// The `RefStore` trait manages human-readable names
/// (like branch and tag names) that point to specific object hashes.
///
/// # Why a separate module
///
/// Reference storage is conceptually distinct from object storage, even
/// though both are persistence concerns. Keeping the trait separate allows
/// independent evolution.
///
/// # Examples
///
/// Importing the trait from this module:
///
/// ```
/// use libvctrl_handler::traits::core::ref_store::RefStore;
///
/// fn assert_ref_store<T: RefStore>() {}
/// ```
///
/// The crate root also re-exports it:
///
/// ```
/// use libvctrl_handler::RefStore;
/// ```
pub mod ref_store;

/// Defines the `RevWalk` trait for traversing commit graphs.
///
/// # Purpose
///
/// The `RevWalk` trait provides the contract for retrieving the parents of a
/// commit, enabling reverse traversal of repository history. This is
/// essential for operations like `rev-list`, `merge-base`, and `rev-parse`.
///
/// # Why a separate module
///
/// Graph traversal is a distinct responsibility from storage or encoding.
/// Keeping the trait in its own file allows backends to provide different
/// commit graph implementations without affecting other components.
///
/// # Examples
///
/// Importing the trait from this module:
///
/// ```
/// use libvctrl_handler::traits::core::revwalk::RevWalk;
///
/// fn assert_revwalk<T: RevWalk>() {}
/// ```
///
/// The crate root re-exports it:
///
/// ```
/// use libvctrl_handler::RevWalk;
/// ```
pub mod revwalk;

/// Defines the `Signer` trait for cryptographic signatures.
///
/// # Purpose
///
/// The `Signer` trait provides the ability to produce
/// cryptographic signatures over arbitrary data, typically used for commit
/// or tag signing.
///
/// # Why a separate module
///
/// Signature algorithms (Ed25519, RSA, etc.) vary by backend. Keeping the
/// signing contract isolated allows the core system to remain agnostic to
/// the specific cryptographic primitives.
///
/// # Examples
///
/// Importing the trait from this module:
///
/// ```
/// use libvctrl_handler::traits::core::signer::Signer;
///
/// fn assert_signer<T: Signer>() {}
/// ```
///
/// The crate root re-exports it:
///
/// ```
/// use libvctrl_handler::Signer;
/// ```
pub mod signer;

/// Defines the `Transport` trait for remote synchronization.
///
/// # Purpose
///
/// The `Transport` trait abstracts the network layer used
/// to fetch and push objects between repositories.
///
/// # Why a separate module
///
/// Transport mechanisms (HTTP, SSH, custom protocols) are independent of
/// storage and serialization. Isolating the trait allows flexible
/// implementations.
///
/// # Examples
///
/// Importing the trait from this module:
///
/// ```
/// use libvctrl_handler::traits::core::transport::Transport;
///
/// fn assert_transport<T: Transport>() {}
/// ```
///
/// The crate root provides the same trait:
///
/// ```
/// use libvctrl_handler::Transport;
/// ```
pub mod transport;

/// Defines the `Verifier` trait for signature verification.
///
/// # Purpose
///
/// The `Verifier` trait provides the capability to verify
/// cryptographic signatures against data, ensuring authenticity and
/// integrity.
///
/// # Why a separate module
///
/// Verification is often paired with signing but can be implemented
/// separately. Keeping it in its own module allows independent testing and
/// alternative verification algorithms.
///
/// # Examples
///
/// Importing the trait from this module:
///
/// ```
/// use libvctrl_handler::traits::core::verifier::Verifier;
///
/// fn assert_verifier<T: Verifier>() {}
/// ```
///
/// The crate root re-exports it:
///
/// ```
/// use libvctrl_handler::Verifier;
/// ```
pub mod verifier;

pub mod config;
pub mod diff;
pub mod index;
pub mod pack;
pub mod reflog;
