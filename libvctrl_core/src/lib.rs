#![allow(clippy::arithmetic_side_effects)]

extern crate alloc;

#[cfg(test)]
use proptest as _;

pub mod codec;
pub mod hash;
pub mod object;
pub mod store;
