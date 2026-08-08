//! # libvctrl_sha512 – Self‑Contained SHA‑512, HMAC, and HKDF
//!
//! A **zero‑dependency**, `#![no_std]` implementation of the SHA‑512 cryptographic
//! hash function, HMAC‑SHA‑512 message authentication code, and HKDF‑SHA‑512 key
//! derivation function (RFC 5869). Optionally includes SHA‑384, HMAC‑SHA‑384, and
//! HKDF‑SHA‑384 via the `sha384` feature.
//!
//! This crate is a modular fork and adaptation of the excellent
//! [hmac-sha512](https://github.com/jedisct1/rust-hmac-sha512) crate by
//! [Frank Denis](https://github.com/jedisct1), released under the ISC license.
//! All modifications are also ISC‑licensed.
//!
//! ---
//!
//! ##  Features
//!
//! - **Pure Rust** – no external dependencies, only `core`.
//! - **`#![no_std]`** – works in embedded systems, kernels, and bootloaders.
//! - **Constant‑time verification** – all comparisons (`verify`, `finalize_verify`)
//!   resist timing side‑channel attacks.
//! - **Streaming and one‑shot APIs** – process data incrementally or in a single call.
//! - **HMAC‑SHA512** – one‑shot and incremental modes.
//! - **HKDF‑SHA512 (RFC 5869)** – extract‑then‑expand key derivation.
//! - **Optional SHA‑384, HMAC‑SHA384, HKDF‑SHA384** – enabled via the `sha384` feature
//!   (on by default).
//! - **Size optimisation** – the `opt_size` feature reduces binary size (~75% smaller)
//!   at a moderate performance cost (~16% slower).
//!
//! ---
//!
//! ##  Installation
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! libvctrl_sha512 = { path = "./libvctrl_sha512" }
//! ```
//!
//! To disable the default `sha384` feature:
//! ```toml
//! libvctrl_sha512 = { version = "0.1.0", default-features = false }
//! ```
//!
//! ---
//!
//! ##  Quick Start
//!
//! ```rust
//! use libvctrl_sha512::{Hash, HMAC, HKDF};
//!
//! // SHA‑512 hash (one‑shot)
//! let digest = Hash::hash(b"Hello, world!");
//! assert_eq!(digest.len(), 64);
//!
//! // HMAC‑SHA512 (one‑shot)
//! let mac = HMAC::mac(b"message", b"secret-key");
//! assert_eq!(mac.len(), 64);
//!
//! // HKDF‑SHA512: derive a 32‑byte key
//! let prk = HKDF::extract(b"salt", b"ikm");
//! let mut okm = [0u8; 32];
//! HKDF::expand(&mut okm, prk, b"info");
//! ```
//!
//! For streaming:
//! ```rust
//! # use libvctrl_sha512::Hash;
//! let mut hasher = Hash::new();
//! hasher.update(b"first part ");
//! hasher.update(b"second part");
//! let digest = hasher.finalize();
//! ```
//!
//! ---
//!
//! ##  Modules
//!
//! - [`utils`] – low‑level endianness and constant‑time helpers.
//! - [`sha512`] – SHA‑512 hash implementation.
//! - [`hmac`] – HMAC‑SHA512 streaming and one‑shot.
//! - [`hkdf`] – HKDF‑SHA512 extract/expand (RFC 5869).
//! - [`sha384`] – SHA‑384, HMAC‑SHA384, HKDF‑SHA384 (feature‑gated).
//!
//! ---
//!
//! ##  Security Considerations
//!
//! - **Constant‑time verification**: All `verify` functions use a bitwise XOR
//!   accumulator with a volatile read to prevent compiler optimisations that
//!   could leak timing information. This is the recommended way to compare
//!   secret values.
//! - **HMAC key handling**: Keys longer than the block size (128 bytes) are
//!   hashed using SHA‑512 before use, following the HMAC specification.
//! - **HKDF**: The `salt` should be random and non‑secret for maximum security;
//!   the `info` string provides domain separation – never reuse the same `info`
//!   for different contexts.
//! - **No `std`**: The crate does not rely on the standard library, reducing
//!   attack surface in trusted execution environments.
//!
//! ---
//!
//! ##  Performance
//!
//! Benchmarks are included in the `benches/` directory. On modern x86‑64 CPUs,
//! hashing 1 KB of data with SHA‑512 takes a few microseconds. The `opt_size`
//! feature reduces binary size by about 75% at the cost of roughly 16% lower
//! throughput.
//!
//! ---
//!
//! ##  License
//!
//! This crate is distributed under the terms of the **ISC License**.
//! See [LICENSE](https://github.com/mroczect/libvctrl/blob/main/libvctrl_sha512/LICENSE)
//! for details.
//!
//! ## Acknowledgements
//!
//! This project is a fork and modularisation of the excellent
//! [hmac-sha512](https://github.com/jedisct1/rust-hmac-sha512) crate by
//! [Frank Denis](https://github.com/jedisct1). All cryptographic logic and
//! implementation details originate from his work.
//!
//! ---
//!
//! ##  Minimum Supported Rust Version (MSRV)
//!
//! This crate requires Rust **1.70** or later.

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

// Expose public modules
pub mod hkdf;
pub mod hmac;
pub mod sha512;
pub mod utils;

#[cfg(feature = "sha384")]
pub mod sha384;

// Re‑export main types for convenience
pub use hkdf::HKDF;
pub use hmac::HMAC;
pub use sha512::Hash;

// Re‑export constants
pub use utils::{BLOCKBYTES, BYTES};
