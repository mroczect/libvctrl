//! # `libvctrl_handler` – The Unshakeable Contract
//!
//! This crate **only** defines the fundamental traits, types, errors, and constants
//! for building a version control system. **No implementations are allowed here.**
//!
//! It is the single source of truth for the entire `libvctrl` ecosystem.
//! Every other component must depend on this crate and must never redefine
//! these fundamental contracts.
//!
//! ## Philosophy
//! - **Mechanism, not policy** – no assumptions about branches, workflows, or defaults.
//! - **Unbounded flexibility, high discipline** – everything is generic and replaceable,
//!   but every input is strictly validated.
//! - **This crate is the constitution** – all fundamental traits, types, and errors
//!   live exclusively here.
//!
//! ## Usage
//! ```rust
//! use libvctrl_handler::*;
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod constants;
pub mod enums;
pub mod errors;
pub mod traits;
pub mod types;

pub use constants::*;
pub use enums::*;
pub use errors::*;
pub use traits::*;
pub use types::*;
