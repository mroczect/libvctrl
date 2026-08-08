//! # libvctrl_sha512
//!
//! This crate is a fork/adaptation of the [hmac-sha512](https://github.com/jedisct1/rust-hmac-sha512)
//! crate by Frank Denis (ISC license). It has been refactored into modular
//! components while preserving the original implementation.
//!
//! ## Original Source
//!
//! - Repository: <https://github.com/jedisct1/rust-hmac-sha512>
//! - Copyright (c) 2019–2026 Frank Denis
//! - Licensed under the ISC License (see below).
//!
//! ## License
//!
//! This crate is distributed under the terms of the **ISC License**:
//!
//! ```text
//! Copyright (c) 2019–2026 Frank Denis
//!
//! Permission to use, copy, modify, and/or distribute this software for any
//! purpose with or without fee is hereby granted, provided that the above
//! copyright notice and this permission notice appear in all copies.
//!
//! THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
//! WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
//! MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
//! ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
//! WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
//! ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
//! OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
//! ```
//!
//! All modifications are also released under the same ISC license.

#![no_std]
#![allow(
    non_snake_case,
    clippy::cast_lossless,
    clippy::eq_op,
    clippy::identity_op,
    clippy::many_single_char_names,
    clippy::unreadable_literal,
    clippy::cargo_common_metadata
)]

pub mod hkdf;
pub mod hmac;
pub mod sha512;
pub mod utils;

#[cfg(feature = "sha384")]
pub mod sha384;

pub use hkdf::HKDF;
pub use hmac::HMAC;
pub use sha512::Hash;

pub use utils::{BLOCKBYTES, BYTES};
