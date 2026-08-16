//! Enums for Git object types.
//!
//! # Architecture
//! This module serves as the central registry for enumerations representing
//! discrete, finite states within the Git protocol. By grouping these types
//! together, the crate isolates protocol-level definitions from higher-level
//! domain logic and data structures.
//!
//! # Design Rationale: Strong Typing over Raw Integers
//! The Git protocol frequently relies on raw integers or specific byte sequences
//! to denote object types (such as mode bits in tree objects). Parsing these
//! directly into integers throughout the codebase invites logic errors and
//! security vulnerabilities. This module transforms those raw values into
//! strongly-typed enums, allowing the Rust compiler to enforce exhaustive
//! matching and guarantee that invalid states are unrepresentable at compile time.
//!
//! # Examples
//! *Note: The following example assumes this crate is named `libvctrl_handler`.*
//!
//! ```
//! # use libvctrl_handler::enums::EntryKind;
//! let kind = EntryKind::Tree;
//! assert_eq!(kind.mode(), 0o40_000);
//! ```

/// Core enum definitions representing fundamental Git protocol types.
///
/// # Why this exists
/// This submodule houses the primary enumerations used across the crate.
/// Separating them into a `core` module allows the top-level `enums` module
/// to remain organized, distinguishing between essential protocol types and
/// any auxiliary or implementation-specific enums that may be added in the future.
pub mod core;

/// Re-export of the [`EntryKind`](core::entry_kind::EntryKind) enum for ergonomic access.
///
/// # Why this exists
/// Provides a flattened import path. Consumers can directly use
/// `libvctrl_handler::enums::EntryKind` instead of navigating the full
/// `libvctrl_handler::enums::core::entry_kind::EntryKind` path. This reduces
/// boilerplate in consumer code while keeping the internal module
/// structure logically separated.
///
/// # Examples
///
/// ```
/// # use libvctrl_handler::enums::EntryKind;
/// let kind = EntryKind::Blob;
/// assert_eq!(kind.mode(), 0o100_644);
/// ```
pub use core::entry_kind::EntryKind;
