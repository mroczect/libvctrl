//! Traits for repository operations.
//!
//! # Architecture
//! This module defines the abstract contracts (interfaces) for interacting with
//! repository components. By leveraging Rust's trait system, the crate decouples
//! the *what* (domain logic and validation) from the *how* (I/O and storage implementations).
//!
//! # Design Rationale: Backend Agnosticism
//! Defining operations like object storage or reference management as traits
//! allows the core logic to remain agnostic of the underlying backend. Consumers
//! can implement these traits for in-memory storage, disk-based filesystems, or
//! remote network protocols without altering the core VCS algorithms. This also
//! drastically simplifies unit testing, as mock implementations can be injected
//! seamlessly via dependency injection.
//!
//! # Examples
//! *Note: The following example assumes this crate is named `libvctrl_handler`.*
//!
//! ```
//! // Importing the module ensures it is publicly accessible and compiled.
//! use libvctrl_handler::traits::core;
//! ```

/// Core operational traits required to implement a functional version control backend.
///
/// # Why this exists
/// Houses the fundamental, low-level traits (such as `ObjectStore`, `RefStore`, and
/// `Encoder`) that define the minimum viable surface area for a Git implementation.
/// Grouping these into a `core` submodule allows the parent `traits` module to
/// logically separate essential protocol traits from any auxiliary or high-level
/// behavioral traits that may be introduced in the future.
///
/// # Examples
///
/// ```
/// // The core submodule is accessible for custom backend implementations.
/// use libvctrl_handler::traits::core;
/// ```
pub mod core;
