//! # Traits Module
//!
//! This module defines the abstract interfaces that concrete version control
//! backends must implement. By defining these as traits, the core data types
//! remain completely decoupled from storage, networking, and serialization logic.
//!
//! ## Design Rationale
//!
//! The traits are split by responsibility:
//!
//! - **`ObjectStore`** and **`RefStore`** handle persistence.
//! - **`Encoder`** and **`Decoder`** handle serialization.
//! - **`Hasher`** handles content addressing.
//! - **`Signer`** and **`Verifier`** handle cryptographic integrity.
//! - **`Transport`** handles remote synchronization.
//!
//! This separation of concerns allows mixing and matching implementations
//! (e.g., an in-memory store with a binary encoder) and vastly simplifies
//! unit testing of individual components.
//!
//! ## Module Structure
//!
//! Each trait now resides in its own file under the [`core`] submodule to
//! improve maintainability and reduce merge conflicts.
//!
//! ## Streaming and Memory Efficiency
//!
//! Starting from version 3.2, [`ObjectStore::get`] returns a
//! [`Box<dyn std::io::Read>`] instead of a [`Vec<u8>`]. This enables
//! streaming of object data directly from the backing store without
//! forcing large contiguous allocations.

pub mod core;
