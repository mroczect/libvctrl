#![forbid(unsafe_code)]
#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub,
    unused_crate_dependencies,
    unused_qualifications
)]

#[cfg(test)]
use proptest as _;

pub mod codec;
pub mod hash;
pub mod object;
pub mod store;
pub mod validate;
