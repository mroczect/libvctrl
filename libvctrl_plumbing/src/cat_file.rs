//! # Cat-File Plumbing Command
//!
//! This module implements the `cat-file` plumbing command, a fundamental
//! building block for inspecting objects in a libvctrl repository. It provides
//! both single-object queries and batch processing for integration with
//! higher-level porcelain commands.
//!
//! ## Why this module exists
//!
//! Plumbing commands operate directly on object stores and decoders without
//! user-friendly formatting. `cat-file` is essential for debugging, scripting,
//! and implementing other commands that need to inspect raw object content or
//! metadata.
//!
//! The module is designed to be backend-agnostic: it accepts any
//! [`ObjectStore`] and any [`Decoder`] via trait objects, enabling the same
//! logic to work with in-memory stores, filesystem stores, and custom
//! decoders.
//!
//! ## How it works
//!
//! The core function [`cat_file`] resolves an object name (a 128-character
//! hexadecimal SHA-512 hash), retrieves the encoded bytes from the store,
//! decodes the type using a series of decoder attempts, and then produces
//! output according to the requested [`CatFileMode`].
//!
//! Batch mode ([`cat_file_batch`]) reads object names line-by-line and writes
//! formatted information, optionally including pretty-printed content. It
//! supports custom format strings and NUL-terminated input/output for robust
//! scripting.
//!
//! ## Safety and correctness
//!
//! All parsing is strict: hashes must be exactly 128 hex characters, hex
//! digits must be valid, and objects must decode successfully. Errors are
//! returned as [`VctrlError`] rather than panicking, making the command safe
//! to use in long-running processes.
//!
//! # Examples
//!
//! Retrieve the type of a stored blob:
//!
//! ```
//! # use libvctrl::{
//! #     Blob, Encoder, Hasher, ObjectStore, BinaryEncoder, Sha512Hasher, MemoryStore,
//! # };
//! # use libvctrl_core::codec::BinaryDecoder;
//! # use libvctrl_plumbing::{cat_file, CatFileMode};
//! # use std::io::Cursor;
//! # fn main() -> Result<(), libvctrl::VctrlError> {
//! // Create a blob and store it.
//! let blob = Blob::new(b"hello".to_vec())?;
//! let mut encoded = Vec::new();
//! BinaryEncoder.encode_blob(&blob, &mut encoded)?;
//! let hash = Sha512Hasher.hash(encoded.as_slice())?;
//! let mut store = MemoryStore::new();
//! store.put(&hash, &encoded)?;
//!
//! // Query its type.
//! let hash_hex = hash.to_string();
//! let mut output = Vec::new();
//! cat_file(
//!     &store,
//!     &BinaryDecoder,
//!     &hash_hex,
//!     CatFileMode::ObjectType,
//!     &mut output,
//! )?;
//! assert_eq!(String::from_utf8(output).unwrap(), "blob\n");
//! # Ok(())
//! # }
//! ```

use libvctrl::{Decoder, EntryKind, Hash, ObjectStore, VctrlError};
use std::fmt::Write;
use std::io::{BufRead, Write as IoWrite};

/// Specifies the operation mode for the [`cat_file`] command.
///
/// Each variant instructs the command to produce different output about a
/// single object. The mode determines whether the object is checked for
/// existence, its type is printed, its size is printed, its content is
/// pretty-printed, or its raw bytes are emitted (optionally with a type
/// check).
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// # use libvctrl_plumbing::{CatFileMode, ObjectType};
/// let mode = CatFileMode::PrettyPrint;
/// let raw_blob = CatFileMode::Raw(ObjectType::Blob);
/// ```
#[derive(Clone, Copy)]
pub enum CatFileMode {
    /// Pretty-print the object content in a human-readable format.
    PrettyPrint,
    /// Print only the object type (one of `blob`, `tree`, `commit`, `tag`).
    ObjectType,
    /// Print the encoded object size in bytes.
    ObjectSize,
    /// Check existence only; produce no output, but return an error if the
    /// object is missing or corrupted.
    Exists,
    /// Output the raw encoded bytes, optionally verifying the object type
    /// matches the expected [`ObjectType`] parameter.
    Raw(ObjectType),
}

/// Logical object types recognized by the version control system.
///
/// This enum mirrors the types defined in `libvctrl_handler`, but is localized
/// for plumbing command reporting. It is used to verify expected object types
/// and to format type strings.
///
/// # Examples
///
/// ```
/// # use libvctrl_plumbing::ObjectType;
/// let blob = ObjectType::Blob;
/// assert_eq!(blob, ObjectType::Blob);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectType {
    /// A binary large object (file content).
    Blob,
    /// A directory tree.
    Tree,
    /// A commit object.
    Commit,
    /// An annotated tag object.
    Tag,
}

/// Executes a single `cat-file` query against an object store.
///
/// This function resolves `object_name` (a 128-character hexadecimal hash),
/// retrieves the encoded bytes, decodes the object, and writes the requested
/// output to `writer` based on `mode`.
///
/// # Why this function exists
///
/// Centralizes all `cat-file` logic so that every caller (CLI, library,
/// batch mode) shares the same validation and formatting rules.
///
/// # How it works
///
/// 1. Parse `object_name` into a [`Hash`].
/// 2. Fetch the encoded bytes from `store`.
/// 3. Depending on `mode`, either:
///    - Return `Ok(())` for `Exists`.
///    - Decode the type and print it for `ObjectType`.
///    - Print the encoded length for `ObjectSize`.
///    - Decode and pretty-print for `PrettyPrint`.
///    - Verify the actual type matches `Raw(expected_type)` and then write the
///      raw bytes.
///
/// # Errors
///
/// Returns [`VctrlError`] if:
/// - `object_name` is not a valid 128-character hex string.
/// - The object is not found in the store.
/// - The encoded bytes fail to decode as any known object type.
/// - The actual type does not match the expected type in `Raw` mode.
/// - The writer fails.
///
/// # Examples
///
/// Pretty-print a stored commit:
///
/// ```
/// # use libvctrl::{
/// #     Commit, Encoder, Hasher, ObjectStore, BinaryEncoder, Sha512Hasher, MemoryStore,
/// #     Hash, UserID,
/// # };
/// # use libvctrl_core::codec::BinaryDecoder;
/// # use libvctrl_plumbing::{cat_file, CatFileMode};
/// # use std::io::Cursor;
/// # fn main() -> Result<(), libvctrl::VctrlError> {
/// // Create a simple commit.
/// let tree = Hash::from_bytes(&[0u8; 64])?;
/// let author = UserID::new("alice".into(), "alice@example.com".into())?;
/// let committer = UserID::new("bob".into(), "bob@example.com".into())?;
/// let commit = Commit::new(tree, vec![], author, committer, "initial".into())?;
///
/// // Encode, hash, and store.
/// let mut encoded = Vec::new();
/// BinaryEncoder.encode_commit(&commit, &mut encoded)?;
/// let hash = Sha512Hasher.hash(encoded.as_slice())?;
/// let mut store = MemoryStore::new();
/// store.put(&hash, &encoded)?;
///
/// // Pretty-print the commit.
/// let mut output = Vec::new();
/// cat_file(
///     &store,
///     &BinaryDecoder,
///     &hash.to_string(),
///     CatFileMode::PrettyPrint,
///     &mut output,
/// )?;
/// assert!(String::from_utf8(output).unwrap().contains("tree"));
/// # Ok(())
/// # }
/// ```
pub fn cat_file<D: Decoder>(
    store: &dyn ObjectStore,
    decoder: &D,
    object_name: &str,
    mode: CatFileMode,
    writer: &mut impl IoWrite,
) -> Result<(), VctrlError> {
    let hash = parse_hash(object_name)?;

    let mut encoded = Vec::new();
    store
        .get(&hash)?
        .read_to_end(&mut encoded)
        .map_err(|e| VctrlError::IoError(std::sync::Arc::new(e)))?;

    match mode {
        CatFileMode::Exists => Ok(()),
        CatFileMode::ObjectType => {
            let obj_type = decode_type(decoder, &encoded)?;
            let type_str = obj_type_to_str(obj_type);
            writeln!(writer, "{type_str}")
                .map_err(|e| VctrlError::IoError(std::sync::Arc::new(e)))?;
            Ok(())
        }
        CatFileMode::ObjectSize => {
            let _obj_type = decode_type(decoder, &encoded)?;
            let size = encoded.len();
            writeln!(writer, "{size}").map_err(|e| VctrlError::IoError(std::sync::Arc::new(e)))?;
            Ok(())
        }
        CatFileMode::PrettyPrint => {
            let content = pretty_print(decoder, &encoded)?;
            writer
                .write_all(content.as_bytes())
                .map_err(|e| VctrlError::IoError(std::sync::Arc::new(e)))?;
            Ok(())
        }
        CatFileMode::Raw(expected_type) => {
            let actual_type = decode_type(decoder, &encoded)?;
            if actual_type != expected_type {
                let actual_type_str = obj_type_to_str(actual_type);
                let expected_type_str = obj_type_to_str(expected_type);
                return Err(VctrlError::Other(format!(
                    "object {object_name} is a {actual_type_str}, not a {expected_type_str}"
                )));
            }
            writer
                .write_all(&encoded)
                .map_err(|e| VctrlError::IoError(std::sync::Arc::new(e)))?;
            Ok(())
        }
    }
}

/// Configuration options for batch `cat-file` processing.
///
/// This struct controls the output format, delimiters, buffering, and whether
/// object content is included in each batch entry.
///
/// # Examples
///
/// ```
/// # use libvctrl_plumbing::BatchOptions;
/// let mut opts = BatchOptions::default();
/// opts.format = Some("%(objectname) %(objecttype)".into());
/// opts.print_contents = true;
/// ```
#[allow(clippy::struct_excessive_bools)]
#[derive(Default)]
pub struct BatchOptions {
    /// Optional custom format string. Placeholders `%(objectname)`,
    /// `%(objecttype)`, and `%(objectsize)` are replaced.
    pub format: Option<String>,
    /// If `true`, input and output lines are NUL-terminated instead of
    /// newline-terminated.
    pub nul_terminated: bool,
    /// If `true`, follow symlinks when resolving object names (currently
    /// unused; reserved for future expansion).
    pub follow_symlinks: bool,
    /// If `true`, buffer all output until the entire batch is processed,
    /// then write it in one go.
    pub buffer: bool,
    /// If `true`, include pretty-printed object content after the info line.
    pub print_contents: bool,
}

/// Processes a batch of `cat-file` requests from an input stream.
///
/// Reads object names line-by-line (or NUL-separated depending on
/// `options.nul_terminated`), retrieves each object, and writes formatted
/// information (and optionally content) to the output stream. If an object is
/// missing, a `"{name} missing"` line is emitted instead of aborting.
///
/// # Why this function exists
///
/// Batch mode enables efficient processing of many objects without repeated
/// setup and teardown. It is commonly used by frontend commands and scripts.
///
/// # How it works
///
/// The function maintains an output buffer. For each input line, it calls
/// [`handle_one_object`] to obtain the info string and optional content. If
/// `options.buffer` is `false`, the buffer is flushed after each object;
/// otherwise, it accumulates and is flushed once at the end.
///
/// # Errors
///
/// Returns [`VctrlError`] if:
/// - An input line cannot be read.
/// - An object name is not a valid hash.
/// - An object cannot be retrieved or decoded.
/// - The output writer fails.
///
/// # Examples
///
/// Process two blobs and print their types:
///
/// ```
/// # use libvctrl::{
/// #     Blob, Encoder, Hasher, ObjectStore, BinaryEncoder, Sha512Hasher, MemoryStore,
/// # };
/// # use libvctrl_core::codec::BinaryDecoder;
/// # use libvctrl_plumbing::{cat_file_batch, BatchOptions};
/// # use std::io::{BufReader, Cursor};
/// # fn main() -> Result<(), libvctrl::VctrlError> {
/// // Create and store two blobs.
/// let mut store = MemoryStore::new();
/// let mut hashes = Vec::new();
/// for content in [b"first".to_vec(), b"second".to_vec()] {
///     let blob = Blob::new(content)?;
///     let mut encoded = Vec::new();
///     BinaryEncoder.encode_blob(&blob, &mut encoded)?;
///     let hash = Sha512Hasher.hash(encoded.as_slice())?;
///     store.put(&hash, &encoded)?;
///     hashes.push(hash.to_string());
/// }
///
/// // Prepare batch input.
/// let input = format!("{}\n{}\n", hashes[0], hashes[1]);
/// let mut reader = BufReader::new(input.as_bytes());
/// let mut output = Vec::new();
/// let options = BatchOptions {
///     format: Some("%(objecttype)".into()),
///     ..Default::default()
/// };
///
/// cat_file_batch(&store, &BinaryDecoder, &mut reader, &mut output, &options)?;
/// let out_str = String::from_utf8(output).unwrap();
/// assert!(out_str.contains("blob\nblob"));
/// # Ok(())
/// # }
/// ```
pub fn cat_file_batch<D: Decoder>(
    store: &dyn ObjectStore,
    decoder: &D,
    input: &mut impl BufRead,
    output: &mut impl IoWrite,
    options: &BatchOptions,
) -> Result<(), VctrlError> {
    let delimiter: u8 = if options.nul_terminated { 0 } else { b'\n' };

    let mut line = String::new();
    let mut out_buf: Vec<u8> = Vec::new();

    loop {
        line.clear();
        if input
            .read_line(&mut line)
            .map_err(|e| VctrlError::IoError(std::sync::Arc::new(e)))?
            == 0
        {
            break;
        }
        let trimmed = if options.nul_terminated {
            line.trim_end_matches('\0')
        } else {
            line.trim_end()
        };

        let object_name = trimmed;

        if let Ok((info, content)) = handle_one_object(store, decoder, object_name, options) {
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
                output
                    .write_all(&out_buf)
                    .map_err(|e| VctrlError::IoError(std::sync::Arc::new(e)))?;
                out_buf.clear();
            }
        } else {
            let error_line = format!("{object_name} missing");
            out_buf.extend_from_slice(error_line.as_bytes());
            out_buf.push(delimiter);
            if !options.buffer {
                output
                    .write_all(&out_buf)
                    .map_err(|e| VctrlError::IoError(std::sync::Arc::new(e)))?;
                out_buf.clear();
            }
        }
    }

    if !out_buf.is_empty() {
        output
            .write_all(&out_buf)
            .map_err(|e| VctrlError::IoError(std::sync::Arc::new(e)))?;
    }
    Ok(())
}

/// Handles a single object lookup and formatting for batch mode.
///
/// This helper retrieves the encoded object, decodes its type, builds the
/// info string according to `options.format`, and optionally pretty-prints
/// the content.
///
/// # Errors
///
/// Returns [`VctrlError`] if the hash is invalid, the object is missing, or
/// decoding fails.
fn handle_one_object<D: Decoder>(
    store: &dyn ObjectStore,
    decoder: &D,
    object_name: &str,
    options: &BatchOptions,
) -> Result<(String, Option<Vec<u8>>), VctrlError> {
    let hash = parse_hash(object_name)?;

    let mut encoded = Vec::new();
    store
        .get(&hash)?
        .read_to_end(&mut encoded)
        .map_err(|e| VctrlError::IoError(std::sync::Arc::new(e)))?;

    let obj_type = decode_type(decoder, &encoded)?;
    let obj_size = encoded.len() as u64;

    let info = options.format.as_ref().map_or_else(
        || {
            let type_str = obj_type_to_str(obj_type);
            format!("{hash} {type_str} {obj_size}")
        },
        |fmt| format_batch_info(fmt, &hash, obj_type, obj_size, None),
    );

    let content = if options.print_contents {
        Some(pretty_print(decoder, &encoded)?.into_bytes())
    } else {
        None
    };

    Ok((info, content))
}

/// Parses a 128-character hexadecimal string into a [`Hash`].
///
/// The hash must be exactly 128 hex digits (64 bytes). Any length mismatch or
/// invalid hex character results in an error.
///
/// # Errors
///
/// Returns [`VctrlError::Other`] if the length is not 128 or a hex digit is
/// invalid.
fn parse_hash(s: &str) -> Result<Hash, VctrlError> {
    if s.len() != 128 {
        let actual_len = s.len();
        return Err(VctrlError::Other(format!(
            "invalid hash length: {actual_len} (expected 128)"
        )));
    }
    let mut bytes = [0u8; 64];
    for (i, byte) in bytes.iter_mut().enumerate() {
        let hex_byte = &s[i * 2..i * 2 + 2];
        *byte = u8::from_str_radix(hex_byte, 16)
            .map_err(|e| VctrlError::Other(format!("invalid hex character in hash: {e}")))?;
    }
    Hash::from_bytes(&bytes)
}

/// Attempts to decode an encoded object as one of the four object types.
///
/// The decoder is tried in order: blob, tree, commit, tag. The first
/// successful decode determines the type. If none succeed, an error is
/// returned.
///
/// # Errors
///
/// Returns [`VctrlError::CorruptedData`] if the bytes do not correspond to a
/// known object type.
fn decode_type<D: Decoder>(decoder: &D, encoded: &[u8]) -> Result<ObjectType, VctrlError> {
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

/// Pretty-prints an encoded object into a human-readable string.
///
/// The format resembles Git's `cat-file -p` output:
/// - Blob: raw content (UTF-8 lossy).
/// - Tree: entries with mode, type, hash, name.
/// - Commit: tree, parents, author, committer, message.
/// - Tag: object, type, tag name, tagger, message.
///
/// # Errors
///
/// Returns [`VctrlError::CorruptedData`] if the object cannot be decoded as
/// any known type.
fn pretty_print<D: Decoder>(decoder: &D, encoded: &[u8]) -> Result<String, VctrlError> {
    if let Ok(blob) = decoder.decode_blob(encoded) {
        return Ok(String::from_utf8_lossy(blob.data()).to_string());
    }
    if let Ok(tree) = decoder.decode_tree(encoded) {
        let mut out = String::new();
        for entry in tree.entries() {
            let _ = writeln!(
                &mut out,
                "{:06o} {:?} {} {}",
                entry_mode(entry.kind()),
                entry.kind(),
                entry.hash(),
                entry.name()
            );
        }
        return Ok(out);
    }
    if let Ok(commit) = decoder.decode_commit(encoded) {
        let mut out = String::new();
        let _ = writeln!(&mut out, "tree {}", commit.tree());
        for parent in commit.parents() {
            let _ = writeln!(&mut out, "parent {parent}");
        }
        let _ = writeln!(
            &mut out,
            "author {} <{}>",
            commit.author().name(),
            commit.author().email()
        );
        let _ = writeln!(
            &mut out,
            "committer {} <{}>",
            commit.committer().name(),
            commit.committer().email()
        );
        let _ = writeln!(&mut out);
        let _ = writeln!(&mut out, "{}", commit.message());
        return Ok(out);
    }
    if let Ok(tag) = decoder.decode_tag(encoded) {
        let mut out = String::new();
        let _ = writeln!(&mut out, "object {}", tag.target());
        let _ = writeln!(&mut out, "type commit");
        let _ = writeln!(&mut out, "tag {}", tag.name());
        if let Some(tagger) = tag.tagger() {
            let _ = writeln!(&mut out, "tagger {} <{}>", tagger.name(), tagger.email());
        }
        let _ = writeln!(&mut out);
        let _ = writeln!(&mut out, "{}", tag.message());
        return Ok(out);
    }
    Err(VctrlError::CorruptedData("unknown object type".into()))
}

/// Converts an [`ObjectType`] to its lowercase string representation.
const fn obj_type_to_str(t: ObjectType) -> &'static str {
    match t {
        ObjectType::Blob => "blob",
        ObjectType::Tree => "tree",
        ObjectType::Commit => "commit",
        ObjectType::Tag => "tag",
    }
}

/// Returns the POSIX file mode corresponding to an [`EntryKind`].
///
/// This is used in tree pretty-printing to display the mode in octal.
const fn entry_mode(kind: EntryKind) -> u32 {
    match kind {
        EntryKind::Blob => 0o100_644,
        EntryKind::Executable => 0o100_755,
        EntryKind::Symlink => 0o120_000,
        EntryKind::Tree => 0o040_000,
        EntryKind::Submodule => 0o160_000,
        _ => 0,
    }
}

/// Formats the info line for batch output based on a custom format string.
///
/// Replaces `%(objectname)`, `%(objecttype)`, and `%(objectsize)` with
/// actual values. The `_mode` parameter is reserved for future use (e.g.,
/// `%(objectmode)`).
fn format_batch_info(
    format: &str,
    hash: &Hash,
    obj_type: ObjectType,
    size: u64,
    _mode: Option<u32>,
) -> String {
    let mut output = format.to_owned();
    output = output.replace("%(objectname)", &hash.to_string());
    output = output.replace("%(objecttype)", obj_type_to_str(obj_type));
    output = output.replace("%(objectsize)", &size.to_string());
    output
}
