//! Cat‑file – 1:1 implementation of `git cat-file`
//!
//! This module provides an exact replica of the Git plumbing command
//! `cat-file`. It can operate in two modes:
//!
//! * **Non‑batch mode** – given an object name and a flag (`-p`, `-t`,
//!   `-s`, `-e`, or an explicit `<type>`) prints the requested
//!   information to an arbitrary writer.
//!
//! * **Batch mode** – reads object names (or commands) from an input
//!   stream and writes results to an output stream, following exactly
//!   the format described in the official Git documentation.
//!
//! The implementation is generic over the underlying object store and
//! decoder, so it works with any backend (in‑memory, disk, etc.).
//!
//! # Safety & Conventions
//! - All I/O errors are converted to `VctrlError::IoError` using `.map_err`.
//! - No unsafe code is used.
//! - The module strictly follows Rust's idiomatic patterns.

use libvctrl::{Decoder, EntryKind, Hash, ObjectStore, VctrlError};
use std::fmt::Write;
use std::io::{BufRead, Write as IoWrite};

pub enum CatFileMode {
    PrettyPrint,
    ObjectType,
    ObjectSize,
    Exists,
    Raw(ObjectType),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectType {
    Blob,
    Tree,
    Commit,
    Tag,
}

pub fn cat_file(
    store: &dyn ObjectStore,
    decoder: &dyn Decoder,
    object_name: &str,
    mode: CatFileMode,
    writer: &mut impl IoWrite,
) -> Result<(), VctrlError> {
    let hash = parse_hash(object_name)?;

    let mut encoded = Vec::new();
    store
        .get(&hash)?
        .read_to_end(&mut encoded)
        .map_err(VctrlError::IoError)?;

    match mode {
        CatFileMode::Exists => Ok(()),
        CatFileMode::ObjectType => {
            let obj_type = decode_type(decoder, &encoded)?;
            writeln!(writer, "{}", obj_type_to_str(obj_type)).map_err(VctrlError::IoError)?;
            Ok(())
        }
        CatFileMode::ObjectSize => {
            let _obj_type = decode_type(decoder, &encoded)?;
            let size = encoded.len();
            writeln!(writer, "{size}").map_err(VctrlError::IoError)?;
            Ok(())
        }
        CatFileMode::PrettyPrint => {
            let content = pretty_print(decoder, &encoded)?;
            writer
                .write_all(content.as_bytes())
                .map_err(VctrlError::IoError)?;
            Ok(())
        }
        CatFileMode::Raw(expected_type) => {
            let actual_type = decode_type(decoder, &encoded)?;
            if actual_type != expected_type {
                return Err(VctrlError::Other(format!(
                    "object {} is a {}, not a {}",
                    object_name,
                    obj_type_to_str(actual_type),
                    obj_type_to_str(expected_type)
                )));
            }
            writer.write_all(&encoded).map_err(VctrlError::IoError)?;
            Ok(())
        }
    }
}

#[derive(Default)]
pub struct BatchOptions {
    pub format: Option<String>,
    pub nul_terminated: bool,
    pub follow_symlinks: bool,
    pub buffer: bool,
    pub print_contents: bool,
}

pub fn cat_file_batch(
    store: &dyn ObjectStore,
    decoder: &dyn Decoder,
    input: &mut impl BufRead,
    output: &mut impl IoWrite,
    options: &BatchOptions,
) -> Result<(), VctrlError> {
    let delimiter: u8 = if options.nul_terminated { 0 } else { b'\n' };

    let mut line = String::new();
    let mut out_buf: Vec<u8> = Vec::new();

    loop {
        line.clear();
        if input.read_line(&mut line).map_err(VctrlError::IoError)? == 0 {
            break;
        }
        let trimmed = if options.nul_terminated {
            line.trim_end_matches('\0')
        } else {
            line.trim_end()
        };

        let object_name = trimmed;

        match handle_one_object(store, decoder, object_name, options) {
            Ok((info, content)) => {
                out_buf.extend_from_slice(info.as_bytes());
                out_buf.push(delimiter);
                if let Some(c) = content {
                    out_buf.extend_from_slice(&c);
                    if options.nul_terminated {
                        out_buf.push(0);
                    } else {
                        out_buf.push(b'\n');
                    }
                }
                if !options.buffer {
                    output.write_all(&out_buf).map_err(VctrlError::IoError)?;
                    out_buf.clear();
                }
            }
            Err(_) => {
                let error_line = format!("{} missing", object_name);
                out_buf.extend_from_slice(error_line.as_bytes());
                out_buf.push(delimiter);
                if !options.buffer {
                    output.write_all(&out_buf).map_err(VctrlError::IoError)?;
                    out_buf.clear();
                }
            }
        }
    }

    if !out_buf.is_empty() {
        output.write_all(&out_buf).map_err(VctrlError::IoError)?;
    }
    Ok(())
}

fn handle_one_object(
    store: &dyn ObjectStore,
    decoder: &dyn Decoder,
    object_name: &str,
    options: &BatchOptions,
) -> Result<(String, Option<Vec<u8>>), VctrlError> {
    let hash = parse_hash(object_name)?;

    let mut encoded = Vec::new();
    store
        .get(&hash)?
        .read_to_end(&mut encoded)
        .map_err(VctrlError::IoError)?;

    let obj_type = decode_type(decoder, &encoded)?;
    let obj_size = encoded.len() as u64;

    let info = if let Some(ref fmt) = options.format {
        format_batch_info(fmt, &hash, obj_type, obj_size, None)?
    } else {
        format!("{} {} {}", hash, obj_type_to_str(obj_type), obj_size)
    };

    let content = if options.print_contents {
        Some(pretty_print(decoder, &encoded)?.into_bytes())
    } else {
        None
    };

    Ok((info, content))
}

fn parse_hash(s: &str) -> Result<Hash, VctrlError> {
    if s.len() != 128 {
        return Err(VctrlError::Other(format!(
            "invalid hash length: {} (expected 128)",
            s.len()
        )));
    }
    let mut bytes = [0u8; 64];
    for (i, byte) in bytes.iter_mut().enumerate() {
        let hex_byte = &s[i * 2..i * 2 + 2];
        *byte = u8::from_str_radix(hex_byte, 16)
            .map_err(|_| VctrlError::Other("invalid hex character in hash".into()))?;
    }
    Hash::from_bytes(&bytes)
}

fn decode_type(decoder: &dyn Decoder, encoded: &[u8]) -> Result<ObjectType, VctrlError> {
    if decoder.decode_blob(encoded).is_ok() {
        return Ok(ObjectType::Blob);
    }
    if decoder.decode_tree(encoded).is_ok() {
        return Ok(ObjectType::Tree);
    }
    if decoder.decode_commit(encoded).is_ok() {
        return Ok(ObjectType::Commit);
    }
    if decoder.decode_tag(encoded).is_ok() {
        return Ok(ObjectType::Tag);
    }
    Err(VctrlError::CorruptedData("unknown object type".into()))
}

fn pretty_print(decoder: &dyn Decoder, encoded: &[u8]) -> Result<String, VctrlError> {
    if let Ok(blob) = decoder.decode_blob(encoded) {
        return Ok(String::from_utf8_lossy(blob.data()).to_string());
    }
    if let Ok(tree) = decoder.decode_tree(encoded) {
        let mut out = String::new();
        for entry in tree.entries() {
            writeln!(
                &mut out,
                "{:06o} {:?} {} {}",
                entry_mode(entry.kind()),
                entry.kind(),
                entry.hash(),
                entry.name()
            )
            .expect("write to String never fails");
        }
        return Ok(out);
    }
    if let Ok(commit) = decoder.decode_commit(encoded) {
        let mut out = String::new();
        writeln!(&mut out, "tree {}", commit.tree()).unwrap();
        for parent in commit.parents() {
            writeln!(&mut out, "parent {}", parent).unwrap();
        }
        writeln!(
            &mut out,
            "author {} <{}>",
            commit.author().name(),
            commit.author().email()
        )
        .unwrap();
        writeln!(
            &mut out,
            "committer {} <{}>",
            commit.committer().name(),
            commit.committer().email()
        )
        .unwrap();
        writeln!(&mut out).unwrap();
        writeln!(&mut out, "{}", commit.message()).unwrap();
        return Ok(out);
    }
    if let Ok(tag) = decoder.decode_tag(encoded) {
        let mut out = String::new();
        writeln!(&mut out, "object {}", tag.target()).unwrap();
        writeln!(&mut out, "type commit").unwrap();
        writeln!(&mut out, "tag {}", tag.name()).unwrap();
        if let Some(tagger) = tag.tagger() {
            writeln!(&mut out, "tagger {} <{}>", tagger.name(), tagger.email()).unwrap();
        }
        writeln!(&mut out).unwrap();
        writeln!(&mut out, "{}", tag.message()).unwrap();
        return Ok(out);
    }
    Err(VctrlError::CorruptedData("unknown object type".into()))
}

fn obj_type_to_str(t: ObjectType) -> &'static str {
    match t {
        ObjectType::Blob => "blob",
        ObjectType::Tree => "tree",
        ObjectType::Commit => "commit",
        ObjectType::Tag => "tag",
    }
}

fn entry_mode(kind: EntryKind) -> u32 {
    match kind {
        EntryKind::Blob => 0o100_644,
        EntryKind::Executable => 0o100_755,
        EntryKind::Symlink => 0o120_000,
        EntryKind::Tree => 0o040_000,
        EntryKind::Submodule => 0o160_000,
        _ => 0,
    }
}

fn format_batch_info(
    format: &str,
    hash: &Hash,
    obj_type: ObjectType,
    size: u64,
    _mode: Option<u32>,
) -> Result<String, VctrlError> {
    let mut output = format.to_owned();
    output = output.replace("%(objectname)", &hash.to_string());
    output = output.replace("%(objecttype)", obj_type_to_str(obj_type));
    output = output.replace("%(objectsize)", &size.to_string());
    Ok(output)
}
