extern crate alloc;

#[cfg(test)]
use libvctrl_core as _;

pub mod cat_file;

pub use cat_file::{BatchOptions, CatFileMode, ObjectType, cat_file, cat_file_batch};
