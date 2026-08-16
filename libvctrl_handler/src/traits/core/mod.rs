//! Core traits for repository operations.
//!
//! # Architecture
//! This module defines the fundamental contracts required to build a functional
//! version control backend. By segregating these traits into a dedicated `core`
//! module, we establish a strict boundary between abstract domain logic and
//! concrete I/O implementations.
//!
//! # Design Rationale: Dependency Inversion
//! The entire crate operates against these traits, never against concrete types.
//! This allows consumers to inject custom backends (in-memory, disk-based, or
//! network-attached) seamlessly. It also simplifies unit testing, as mock
//! implementations can be substituted without altering the core algorithms.
//!
//! # Bounded Contexts
//! Each submodule represents a distinct bounded context within the Git architecture:
//! - **Storage**: [`object_store`], [`pack`]
//! - **State**: [`ref_store`], [`reflog`], [`index`]
//! - **Serialization**: [`encoder`], [`decoder`], [`hasher`]
//! - **Analysis**: [`diff`], [`blame`], [`revwalk`]
//! - **Security**: [`signer`], [`verifier`]
//! - **Networking**: [`remote`], [`transport`]
//! - **Configuration**: [`config`]
//!
//! # Examples
//! *Note: The following example assumes this crate is named `libvctrl_handler`.*
//!
//! ```
//! # use libvctrl_handler::traits::core::{
//! #     blame, config, decoder, diff, encoder, hasher, index, object_store,
//! #     pack, ref_store, reflog, remote, revwalk, signer, transport, verifier,
//! # };
//! // All core trait modules are publicly accessible.
//! ```

/// Blame computation trait.
///
/// # Why this exists
/// Provides the contract for attributing lines in a file to specific commits.
/// This is separated from standard diffing because blame requires traversing
/// history and tracking line movements across revisions, which is computationally
/// distinct from simple tree-to-tree comparisons.
///
/// # How it works
/// Implementors will analyze the history of a given path and return a sequence
/// of [`BlameEntry`](blame::BlameEntry) items, mapping line ranges to commits.
///
/// # Examples
///
/// ```
/// # use libvctrl_handler::traits::core::blame;
/// // The blame submodule is accessible.
/// ```
pub mod blame;

/// Configuration store trait.
///
/// # Why this exists
/// Abstracts the reading and writing of repository configuration (e.g., `.git/config`).
/// Decoupling this allows the core engine to query settings (like user name or
/// signing keys) without being tied to a specific file format or key-value backend.
///
/// # How it works
/// Defines a key-value interface segmented by sections, enabling persistent
/// configuration management across different storage mediums.
///
/// # Examples
///
/// ```
/// # use libvctrl_handler::traits::core::config;
/// // The config submodule is accessible.
/// ```
pub mod config;

/// Object decoder trait.
///
/// # Why this exists
/// Defines the contract for deserializing raw bytes into strongly-typed Git objects
/// (e.g., [`Blob`](crate::Blob), [`Tree`](crate::Tree)). This abstraction allows
/// the engine to support multiple wire formats or compression algorithms.
///
/// # How it works
/// Implementors read from a generic `std::io::Read` source, parse the headers
/// and payloads, and construct the corresponding domain types, enforcing structural
/// validity during the process.
///
/// # Examples
///
/// ```
/// # use libvctrl_handler::traits::core::decoder;
/// // The decoder submodule is accessible.
/// ```
pub mod decoder;

/// Tree differencing trait.
///
/// # Why this exists
/// Provides the contract for computing the delta between two tree objects.
/// Separating this logic allows for different diffing algorithms (e.g., Myers,
/// patience) to be plugged in without modifying the core comparison logic.
///
/// # How it works
/// Accepts two tree identifiers and returns a [`TreeDelta`](crate::TreeDelta),
/// enumerating all added, deleted, or modified entries between the two states.
///
/// # Examples
///
/// ```
/// # use libvctrl_handler::traits::core::diff;
/// // The diff submodule is accessible.
/// ```
pub mod diff;

/// Object encoder trait.
///
/// # Why this exists
/// Defines the contract for serializing strongly-typed Git objects into raw bytes.
/// This is the inverse of the [`decoder`] module, ensuring that objects can be
/// written to disk or transmitted over the network in a standardized format.
///
/// # How it works
/// Implementors write the canonical Git representation of the object to a generic
/// `std::io::Write` destination, handling headers and payload formatting.
///
/// # Examples
///
/// ```
/// # use libvctrl_handler::traits::core::encoder;
/// // The encoder submodule is accessible.
/// ```
pub mod encoder;

/// Hashing trait.
///
/// # Why this exists
/// Abstracts the cryptographic hashing mechanism. While Git traditionally uses
/// SHA-1 or SHA-256, this trait allows the engine to support arbitrary hash
/// functions or custom hashing contexts.
///
/// # How it works
/// Reads data from a generic `std::io::Read` source and computes the final
/// [`Hash`](crate::Hash) digest, ensuring that the object's content matches its
/// identifier.
///
/// # Examples
///
/// ```
/// # use libvctrl_handler::traits::core::hasher;
/// // The hasher submodule is accessible.
/// ```
pub mod hasher;

/// Index (staging area) trait.
///
/// # Why this exists
/// Defines the contract for managing the staging area between the working directory
/// and the object database. This abstraction is crucial for orchestrating commits
/// and tracking file states.
///
/// # How it works
/// Provides methods to add, remove, and query entries by path, and to serialize
/// the staged state into a tree object ready for committing.
///
/// # Examples
///
/// ```
/// # use libvctrl_handler::traits::core::index;
/// // The index submodule is accessible.
/// ```
pub mod index;

/// Object storage trait.
///
/// # Why this exists
/// Provides the fundamental contract for storing and retrieving content-addressed
/// objects. This is the backbone of the version control system, allowing backends
/// to use plain directories, packed files, or databases.
///
/// # How it works
/// Defines `put`, `get`, `delete`, and `exists` operations keyed by [`Hash`](crate::Hash),
/// ensuring that object retrieval is opaque to the caller.
///
/// # Examples
///
/// ```
/// # use libvctrl_handler::traits::core::object_store;
/// // The object_store submodule is accessible.
/// ```
pub mod object_store;

/// Pack file reader/writer traits.
///
/// # Why this exists
/// Packfiles are Git's compressed archive format for objects. This module defines
/// contracts for both writing and reading packfiles, isolating the complex
/// delta-compression and indexing logic from the standard object store.
///
/// # How it works
/// The writer trait handles object insertion and finalization, while the reader
/// trait provides random access to objects within the pack via their identifiers.
///
/// # Examples
///
/// ```
/// # use libvctrl_handler::traits::core::pack;
/// // The pack submodule is accessible.
/// ```
pub mod pack;

/// Reference store trait.
///
/// # Why this exists
/// Abstracts the management of symbolic references (branches, tags, HEAD).
/// Decoupling this allows the engine to manage mutable state independently of
/// the immutable object database.
///
/// # How it works
/// Defines operations to set, get, delete, and list references, mapping human-readable
/// names to [`Hash`](crate::Hash) values.
///
/// # Examples
///
/// ```
/// # use libvctrl_handler::traits::core::ref_store;
/// // The ref_store submodule is accessible.
/// ```
pub mod ref_store;

/// Reflog store trait.
///
/// # Why this exists
/// Provides the contract for recording the history of reference updates.
/// Reflogs are essential for recovering from mistakes and tracking branch movement.
///
/// # How it works
/// Appends timestamped entries to a reference's log and retrieves them, ensuring
/// that the chronological history of repository mutations is preserved.
///
/// # Examples
///
/// ```
/// # use libvctrl_handler::traits::core::reflog;
/// // The reflog submodule is accessible.
/// ```
pub mod reflog;

/// Remote repository trait.
///
/// # Why this exists
/// Defines the contract for interacting with remote repositories.
/// This abstraction normalizes operations like fetching and pushing across
/// different protocols (e.g., HTTP, SSH, Git).
///
/// # How it works
/// Manages refspecs and remote references, coordinating the transfer of objects
/// and updates between local and remote states.
///
/// # Examples
///
/// ```
/// # use libvctrl_handler::traits::core::remote;
/// // The remote submodule is accessible.
/// ```
pub mod remote;

/// Revision walking trait.
///
/// # Why this exists
/// Provides the contract for traversing the commit graph.
/// Walking history is a fundamental operation for log generation, bisecting,
/// and ancestry queries.
///
/// # How it works
/// Returns a lazy iterator over commit identifiers starting from a given point,
/// allowing efficient traversal without loading the entire graph into memory.
///
/// # Examples
///
/// ```
/// # use libvctrl_handler::traits::core::revwalk;
/// // The revwalk submodule is accessible.
/// ```
pub mod revwalk;

/// Signing trait.
///
/// # Why this exists
/// Abstracts the cryptographic signing of data (e.g., commits or tags).
/// This allows the engine to support various signing backends (GPG, SSH, X.509)
/// without hardcoding the cryptographic primitives.
///
/// # How it works
/// Accepts a key identifier and raw data, returning a cryptographic signature
/// that can be appended to the object.
///
/// # Examples
///
/// ```
/// # use libvctrl_handler::traits::core::signer;
/// // The signer submodule is accessible.
/// ```
pub mod signer;

/// Transport trait.
///
/// # Why this exists
/// Defines the low-level contract for sending and receiving raw Git objects
/// over a network. This is distinct from the [`remote`] module, which handles
/// higher-level repository semantics.
///
/// # How it works
/// Provides simple fetch and push primitives based on object hashes, acting as
/// the pipe between local and remote object stores.
///
/// # Examples
///
/// ```
/// # use libvctrl_handler::traits::core::transport;
/// // The transport submodule is accessible.
/// ```
pub mod transport;

/// Verification trait.
///
/// # Why this exists
/// Abstracts the verification of cryptographic signatures. It is the counterpart
/// to the [`signer`] module, ensuring that objects can be authenticated against
/// trusted keys.
///
/// # How it works
/// Accepts a key identifier, raw data, and a signature, returning a boolean
/// indicating the validity of the signature.
///
/// # Examples
///
/// ```
/// # use libvctrl_handler::traits::core::verifier;
/// // The verifier submodule is accessible.
/// ```
pub mod verifier;
