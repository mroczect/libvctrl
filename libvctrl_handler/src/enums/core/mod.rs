//! Core enum definitions for Git object types.
//!
//! # Architecture
//! This module acts as the central registry for enumerations that represent
//! discrete, finite states in the Git protocol. By isolating these enums into
//! a dedicated `core` submodule, the crate separates raw protocol definitions
//! from higher-level domain logic and data structures.
//!
//! # Design Rationale: Strong Typing over Raw Integers
//! The Git protocol frequently relies on raw integers or specific byte sequences
//! to denote object types (e.g., mode bits in tree objects). Parsing these directly
//! into integers throughout the codebase invites logic errors and security vulnerabilities.
//! This module transforms those raw values into strongly-typed enums, allowing the
//! Rust compiler to enforce exhaustive matching and guarantee that invalid states
//! are unrepresentable at compile time.

/// Provides the [`EntryKind`](crate::enums::EntryKind) enum, which classifies
/// the type of filesystem objects stored within a Git tree.
///
/// # Why this exists
/// Git tree objects map directory structures. Each entry in a tree requires a
/// mode to distinguish between regular files, executable files, symbolic links,
/// subdirectories (trees), and submodule commits. This submodule exposes the
/// canonical enum for those classifications, ensuring that mode handling across
/// the crate is type-safe and self-documenting.
pub mod entry_kind;
