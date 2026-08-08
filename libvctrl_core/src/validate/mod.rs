//! Common validation utilities shared across modules.
//!
//! This module provides lightweight, reusable functions that validate
//! fundamental constraints before constructing the domain types defined
//! in `libvctrl_handler`. By centralising validation here, we ensure
//! that every component in `libvctrl_core` speaks the same "language"
//! of what constitutes a valid hash or a valid name.
//!
//! # Why separate validation?
//!
//! - **Single source of truth** – if the rules for a valid name ever
//!   change (e.g., adding forbidden characters), they only need to be
//!   updated in one place.
//! - **Deferred construction** – you can pre‑validate raw data before
//!   constructing a [`Hash`] or a name‑carrying type, which is useful
//!   when you need to collect validation errors without immediately
//!   building the final object.
//! - **Clear error messages** – each validation function returns a
//!   descriptive [`VctrlError`] that pinpoints exactly what went wrong.
//!
//! # Functions
//!
//! | Function | What it validates |
//! |---|---|
//! | [`hash::validate_hash_bytes`] | A byte slice has the exact length of a [`Hash`] |
//! | [`name::validate_name`] | A string is a valid name (non‑empty, length limit, no traversal) |
//!
//! # When to use
//!
//! Use these functions when you have raw input (e.g., from a file, from
//! the network, from a user) and you want to check its validity before
//! proceeding. If you already have a [`Hash`] or a type constructed by
//! `libvctrl_handler`, it is already valid – no need to call these.
//!
//! # Examples
//!
//! ```rust
//! use libvctrl_core::validate::hash::validate_hash_bytes;
//! use libvctrl_core::validate::name::validate_name;
//!
//! // Hash validation
//! assert!(validate_hash_bytes(&[0u8; 64]).is_ok());
//! assert!(validate_hash_bytes(&[0u8; 10]).is_err());
//!
//! // Name validation
//! assert!(validate_name("hello").is_ok());
//! assert!(validate_name("").is_err());
//! assert!(validate_name("src/main.rs").is_err());  // contains '/'
//! ```

pub mod hash;
pub mod name;
