//! Logical object type enumerations for `libvctrl_handler`.
//!
//! # Purpose
//!
//! This module defines high-level, discriminative types that categorize the
//! logical kind of an object in the version control system. Rather than
//! exposing raw filesystem mode bits, it provides a semantic enum
//! ([`EntryKind`]) that distinguishes between regular files, executable
//! files, symbolic links, subdirectories, and submodule references.
//!
//! # Design Rationale
//!
//! The enum is kept separate from the low-level mode constants (like those
//! in [`crate::constants::entry_mode`]) to decouple the abstract data model
//! ("what kind of object is this?") from the serialized Unix‑style
//! representation ("what permission bits does this object have?"). This
//! allows different backends to map their own mode systems to a uniform set
//! of logical kinds, and makes the core data structures independent of
//! POSIX‑specific details.
//!
//! The module itself is deliberately small; it contains only the enum and
//! its documentation. This avoids pulling in dependencies or bloating the
//! crate with logic that belongs to higher‑level components (e.g., a decoder
//! implementation).

pub mod core;

// Re-export EntryKind agar path `enums::EntryKind` tetap valid
pub use core::entry_kind::EntryKind;
