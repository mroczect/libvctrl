//! # Binary Encoder
//!
//! This module provides a deterministic, versioned, little-endian binary
//! encoder for every core object type defined by `libvctrl_handler`.
//!
//! The encoder is the counterpart to [`BinaryDecoder`](super::binary_decoder::BinaryDecoder).
//! Data written by this encoder can always be decoded back into an equivalent
//! object, provided the same system limits and version are used.
//!
//! ## Design rationale
//!
//! Version control objects are content-addressed. Deterministic serialization
//! is therefore critical: the same object must always produce exactly the same
//! bytes, otherwise the hash changes and the object becomes unreachable.
//!
//! The encoder achieves determinism by:
//!
//! - Using a fixed version byte.
//! - Using little-endian integer encoding on all supported platforms.
//! - Writing fields in a strict, documented order.
//! - Never depending on platform-specific layouts.
//!
//! ## How it works
//!
//! Every `encode_*` method writes directly to the supplied writer. Length
//! prefixes are validated before conversion to prevent silent truncation.
//! All string fields are encoded as a one-byte length followed by UTF-8 bytes.
//! The writer uses [`std::io::Write::write_all`] to guarantee complete writes.

use libvctrl_handler::{
    Blob, Commit, Encoder, EntryKind, MAX_MESSAGE_LENGTH, Tag, Tree, VctrlError,
};
use std::io::Write;

/// The current version of the binary encoding format.
///
/// This version byte is written as the first byte of every encoded object.
/// The decoder rejects any input whose first byte does not equal this value.
///
/// # Examples
///
/// ```
/// # use libvctrl_core::codec::VERSION;
/// assert_eq!(VERSION, 3);
/// ```
pub const VERSION: u8 = 3;

/// An encoder for the binary format of Git objects.
///
/// `BinaryEncoder` is a stateless, zero-sized type that implements the
/// [`Encoder`] trait. It converts high-level objects such as [`Blob`],
/// [`Tree`], [`Commit`], and [`Tag`] into a compact, versioned byte stream.
///
/// # Why this struct exists
///
/// Serialization is isolated behind a trait so that different storage backends
/// can use different wire formats. `BinaryEncoder` is the reference
/// implementation and defines the canonical on-disk format for the workspace.
///
/// # How it works
///
/// Each method writes to a [`std::io::Write`] implementation. The encoder does
/// not allocate the entire payload upfront; it streams fields directly to the
/// writer. However, all length conversions are checked with `try_from`, so
/// impossible lengths are reported as [`VctrlError::SerializationError`]
/// instead of causing silent truncation.
///
/// # Examples
///
/// ```
/// # use std::io::Cursor;
/// # use libvctrl_handler::{Blob, Encoder};
/// # use libvctrl_core::codec::BinaryEncoder;
/// let blob = Blob::new(b"hello".to_vec()).unwrap();
/// let mut buf = Vec::new();
/// BinaryEncoder.encode_blob(&blob, &mut buf).unwrap();
/// assert_eq!(buf[0], 3);
/// assert_eq!(buf.len(), 1 + 8 + 5);
/// ```
pub struct BinaryEncoder;

impl Encoder for BinaryEncoder {
    /// Encodes a [`Blob`] into the binary format.
    ///
    /// The output layout is:
    ///
    /// | Offset | Size       | Field               |
    /// |--------|------------|---------------------|
    /// | 0      | 1          | Version byte        |
    /// | 1      | 8          | `data_len` (u64 LE) |
    /// | 9      | `data_len` | Raw blob data       |
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::IoError`] if the writer fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io::Cursor;
    /// # use libvctrl_handler::{Blob, Encoder};
    /// # use libvctrl_core::codec::{BinaryEncoder, VERSION};
    /// let blob = Blob::new(b"hello world".to_vec()).unwrap();
    /// let mut encoded = Vec::new();
    /// BinaryEncoder.encode_blob(&blob, &mut encoded).unwrap();
    ///
    /// assert_eq!(encoded[0], VERSION);
    /// assert_eq!(encoded.len(), 1 + 8 + blob.data().len());
    /// ```
    fn encode_blob<W: Write + Send>(&self, blob: &Blob, writer: &mut W) -> Result<(), VctrlError> {
        let data = blob.data();
        writer.write_all(&[VERSION]).map_err(VctrlError::from_io)?;
        writer
            .write_all(&(data.len() as u64).to_le_bytes())
            .map_err(VctrlError::from_io)?;
        writer.write_all(data).map_err(VctrlError::from_io)?;
        Ok(())
    }

    /// Encodes a [`Tree`] into the binary format.
    ///
    /// The output layout is:
    ///
    /// | Offset | Size       | Field                                   |
    /// |--------|------------|------------------------------------------|
    /// | 0      | 1          | Version byte                             |
    /// | 1      | 4          | `entry_count` (u32 LE)                   |
    /// | 5      | varies     | Repeated entries, each consisting of:    |
    /// |        |            | - `name_len` (u8)                        |
    /// |        |            | - `name` (UTF-8)                         |
    /// |        |            | - `kind_byte` (u8)                       |
    /// |        |            | - `hash` (64 bytes)                      |
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::SerializationError`] if:
    ///
    /// - the tree contains more than `u32::MAX` entries,
    /// - an entry name is longer than `u8::MAX` bytes,
    /// - an entry kind is unknown.
    ///
    /// Returns [`VctrlError::IoError`] if the writer fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io::Cursor;
    /// # use libvctrl_handler::{Encoder, EntryKind, Hash, Tree, TreeEntry};
    /// # use libvctrl_core::codec::{BinaryEncoder, VERSION};
    /// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let entry = TreeEntry::new("a.txt".to_owned(), EntryKind::Blob, hash).unwrap();
    /// let tree = Tree::new(vec![entry]).unwrap();
    ///
    /// let mut encoded = Vec::new();
    /// BinaryEncoder.encode_tree(&tree, &mut encoded).unwrap();
    ///
    /// assert_eq!(encoded[0], VERSION);
    /// let count = u32::from_le_bytes(encoded[1..5].try_into().unwrap());
    /// assert_eq!(count, 1);
    /// ```
    fn encode_tree<W: Write + Send>(&self, tree: &Tree, writer: &mut W) -> Result<(), VctrlError> {
        let entries = tree.entries();
        writer.write_all(&[VERSION]).map_err(VctrlError::from_io)?;
        let entry_count = u32::try_from(entries.len())
            .map_err(|e| VctrlError::SerializationError(format!("too many entries: {e}")))?;
        writer
            .write_all(&entry_count.to_le_bytes())
            .map_err(VctrlError::from_io)?;

        for entry in entries {
            let name = entry.name();
            let name_len = u8::try_from(name.len())
                .map_err(|e| VctrlError::SerializationError(format!("name too long: {e}")))?;
            writer.write_all(&[name_len]).map_err(VctrlError::from_io)?;
            writer
                .write_all(name.as_bytes())
                .map_err(VctrlError::from_io)?;

            let kind_byte = match entry.kind() {
                EntryKind::Blob => 0,
                EntryKind::Executable => 1,
                EntryKind::Symlink => 2,
                EntryKind::Tree => 3,
                EntryKind::Submodule => 4,
                _ => {
                    return Err(VctrlError::SerializationError("unknown entry kind".into()));
                }
            };
            writer
                .write_all(&[kind_byte])
                .map_err(VctrlError::from_io)?;
            writer
                .write_all(entry.hash().as_bytes())
                .map_err(VctrlError::from_io)?;
        }
        Ok(())
    }

    /// Encodes a [`Commit`] into the binary format.
    ///
    /// The output layout is fixed and ordered:
    ///
    /// | Field                 | Size          |
    /// |-----------------------|---------------|
    /// | Version               | 1             |
    /// | Tree hash             | 64            |
    /// | Parent count          | 2 (u16 LE)    |
    /// | Parent hashes         | 64 * count    |
    /// | Author name length    | 1             |
    /// | Author name           | length        |
    /// | Author email length   | 1             |
    /// | Author email          | length        |
    /// | Committer name length | 1             |
    /// | Committer name        | length        |
    /// | Committer email length| 1             |
    /// | Committer email       | length        |
    /// | Message length        | 4 (u32 LE)    |
    /// | Message               | length        |
    /// | Timestamp             | 8 (i64 LE)    |
    /// | Timezone offset       | 2 (i16 LE)    |
    /// | Encoding length       | 1             |
    /// | Encoding              | length or 0   |
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::SerializationError`] if:
    ///
    /// - the commit has more than `u16::MAX` parents,
    /// - any name or email is longer than `u8::MAX` bytes,
    /// - the message length cannot be represented as `u32`,
    /// - the message exceeds [`MAX_MESSAGE_LENGTH`],
    /// - the encoding string is longer than `u8::MAX` bytes.
    ///
    /// Returns [`VctrlError::IoError`] if the writer fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io::Cursor;
    /// # use libvctrl_handler::{Commit, Encoder, Hash, UserID};
    /// # use libvctrl_core::codec::{BinaryEncoder, VERSION};
    /// let tree = Hash::from_bytes(&[1u8; 64]).unwrap();
    /// let author = UserID::new("Alice".to_owned(), "alice@example.com".to_owned()).unwrap();
    /// let committer = UserID::new("Bob".to_owned(), "bob@example.com".to_owned()).unwrap();
    /// let commit = Commit::new(
    ///     tree,
    ///     vec![],
    ///     author,
    ///     committer,
    ///     "Initial commit".to_owned(),
    /// )
    /// .unwrap();
    ///
    /// let mut encoded = Vec::new();
    /// BinaryEncoder.encode_commit(&commit, &mut encoded).unwrap();
    ///
    /// assert_eq!(encoded[0], VERSION);
    /// assert!(encoded.len() > 1 + 64 + 2);
    /// ```
    fn encode_commit<W: Write + Send>(
        &self,
        commit: &Commit,
        writer: &mut W,
    ) -> Result<(), VctrlError> {
        writer.write_all(&[VERSION]).map_err(VctrlError::from_io)?;
        writer
            .write_all(commit.tree().as_bytes())
            .map_err(VctrlError::from_io)?;

        let parents = commit.parents();
        let parent_count = u16::try_from(parents.len())
            .map_err(|e| VctrlError::SerializationError(format!("too many parents: {e}")))?;
        writer
            .write_all(&parent_count.to_le_bytes())
            .map_err(VctrlError::from_io)?;

        for p in parents {
            writer
                .write_all(p.as_bytes())
                .map_err(VctrlError::from_io)?;
        }

        let author_name = commit.author().name();
        writer
            .write_all(&[u8::try_from(author_name.len()).map_err(|e| {
                VctrlError::SerializationError(format!("author name too long: {e}"))
            })?])
            .map_err(VctrlError::from_io)?;
        writer
            .write_all(author_name.as_bytes())
            .map_err(VctrlError::from_io)?;

        let author_email = commit.author().email();
        writer
            .write_all(&[u8::try_from(author_email.len()).map_err(|e| {
                VctrlError::SerializationError(format!("author email too long: {e}"))
            })?])
            .map_err(VctrlError::from_io)?;
        writer
            .write_all(author_email.as_bytes())
            .map_err(VctrlError::from_io)?;

        let committer_name = commit.committer().name();
        writer
            .write_all(&[u8::try_from(committer_name.len()).map_err(|e| {
                VctrlError::SerializationError(format!("committer name too long: {e}"))
            })?])
            .map_err(VctrlError::from_io)?;
        writer
            .write_all(committer_name.as_bytes())
            .map_err(VctrlError::from_io)?;

        let committer_email = commit.committer().email();
        writer
            .write_all(&[u8::try_from(committer_email.len()).map_err(|e| {
                VctrlError::SerializationError(format!("committer email too long: {e}"))
            })?])
            .map_err(VctrlError::from_io)?;
        writer
            .write_all(committer_email.as_bytes())
            .map_err(VctrlError::from_io)?;

        let msg = commit.message();
        let msg_len = u32::try_from(msg.len())
            .map_err(|e| VctrlError::SerializationError(format!("message too long: {e}")))?;
        if msg_len as usize > usize::try_from(MAX_MESSAGE_LENGTH).unwrap_or(usize::MAX) {
            return Err(VctrlError::SerializationError(
                "commit message exceeds size limit".into(),
            ));
        }
        writer
            .write_all(&msg_len.to_le_bytes())
            .map_err(VctrlError::from_io)?;
        writer
            .write_all(msg.as_bytes())
            .map_err(VctrlError::from_io)?;

        writer
            .write_all(&commit.meta().timestamp().to_le_bytes())
            .map_err(VctrlError::from_io)?;
        writer
            .write_all(&commit.meta().timezone_offset().to_le_bytes())
            .map_err(VctrlError::from_io)?;

        match commit.meta().encoding() {
            Some(enc) => {
                let len = u8::try_from(enc.len()).map_err(|e| {
                    VctrlError::SerializationError(format!("encoding too long: {e}"))
                })?;
                writer.write_all(&[len]).map_err(VctrlError::from_io)?;
                writer
                    .write_all(enc.as_bytes())
                    .map_err(VctrlError::from_io)?;
            }
            None => writer.write_all(&[0u8]).map_err(VctrlError::from_io)?,
        }
        Ok(())
    }

    /// Encodes a [`Tag`] into the binary format.
    ///
    /// The output layout is:
    ///
    /// | Field              | Size         |
    /// |--------------------|--------------|
    /// | Version            | 1            |
    /// | Name length        | 1            |
    /// | Name               | length       |
    /// | Target hash        | 64           |
    /// | Tagger presence    | 1            |
    /// | Tagger name length | 1 or omitted |
    /// | Tagger name        | length       |
    /// | Tagger email length| 1 or omitted |
    /// | Tagger email       | length       |
    /// | Message length     | 4 (u32 LE)   |
    /// | Message            | length       |
    /// | Timestamp          | 8 (i64 LE)   |
    /// | Timezone offset    | 2 (i16 LE)   |
    /// | Encoding length    | 1            |
    /// | Encoding           | length or 0  |
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::SerializationError`] if:
    ///
    /// - the tag name is longer than `u8::MAX` bytes,
    /// - a tagger name or email is longer than `u8::MAX` bytes,
    /// - the message cannot be represented as `u32`,
    /// - the message exceeds [`MAX_MESSAGE_LENGTH`],
    /// - the encoding string is longer than `u8::MAX` bytes.
    ///
    /// Returns [`VctrlError::IoError`] if the writer fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io::Cursor;
    /// # use libvctrl_handler::{Encoder, Hash, Tag, UserID};
    /// # use libvctrl_core::codec::{BinaryEncoder, VERSION};
    /// let target = Hash::from_bytes(&[2u8; 64]).unwrap();
    /// let tagger = UserID::new("Tagger".to_owned(), "tagger@example.com".to_owned()).unwrap();
    /// let tag = Tag::new(
    ///     "v1.0.0".to_owned(),
    ///     target,
    ///     Some(tagger),
    ///     "Release".to_owned(),
    /// )
    /// .unwrap();
    ///
    /// let mut encoded = Vec::new();
    /// BinaryEncoder.encode_tag(&tag, &mut encoded).unwrap();
    ///
    /// assert_eq!(encoded[0], VERSION);
    /// assert!(encoded.len() > 1 + 64 + 1);
    /// ```
    fn encode_tag<W: Write + Send>(&self, tag: &Tag, writer: &mut W) -> Result<(), VctrlError> {
        writer.write_all(&[VERSION]).map_err(VctrlError::from_io)?;

        let name = tag.name();
        let name_len = u8::try_from(name.len())
            .map_err(|e| VctrlError::SerializationError(format!("tag name too long: {e}")))?;
        writer.write_all(&[name_len]).map_err(VctrlError::from_io)?;
        writer
            .write_all(name.as_bytes())
            .map_err(VctrlError::from_io)?;

        writer
            .write_all(tag.target().as_bytes())
            .map_err(VctrlError::from_io)?;

        match tag.tagger() {
            Some(tagger) => {
                writer.write_all(&[1u8]).map_err(VctrlError::from_io)?;

                let tagger_name = tagger.name();
                writer
                    .write_all(&[u8::try_from(tagger_name.len()).map_err(|e| {
                        VctrlError::SerializationError(format!("tagger name too long: {e}"))
                    })?])
                    .map_err(VctrlError::from_io)?;
                writer
                    .write_all(tagger_name.as_bytes())
                    .map_err(VctrlError::from_io)?;

                let tagger_email = tagger.email();
                writer
                    .write_all(&[u8::try_from(tagger_email.len()).map_err(|e| {
                        VctrlError::SerializationError(format!("tagger email too long: {e}"))
                    })?])
                    .map_err(VctrlError::from_io)?;
                writer
                    .write_all(tagger_email.as_bytes())
                    .map_err(VctrlError::from_io)?;
            }
            None => writer.write_all(&[0u8]).map_err(VctrlError::from_io)?,
        }

        let msg = tag.message();
        let msg_len = u32::try_from(msg.len())
            .map_err(|e| VctrlError::SerializationError(format!("message too long: {e}")))?;
        if msg_len as usize > usize::try_from(MAX_MESSAGE_LENGTH).unwrap_or(usize::MAX) {
            return Err(VctrlError::SerializationError(
                "tag message exceeds size limit".into(),
            ));
        }
        writer
            .write_all(&msg_len.to_le_bytes())
            .map_err(VctrlError::from_io)?;
        writer
            .write_all(msg.as_bytes())
            .map_err(VctrlError::from_io)?;

        writer
            .write_all(&tag.meta().timestamp().to_le_bytes())
            .map_err(VctrlError::from_io)?;
        writer
            .write_all(&tag.meta().timezone_offset().to_le_bytes())
            .map_err(VctrlError::from_io)?;

        match tag.meta().encoding() {
            Some(enc) => {
                let len = u8::try_from(enc.len()).map_err(|e| {
                    VctrlError::SerializationError(format!("encoding too long: {e}"))
                })?;
                writer.write_all(&[len]).map_err(VctrlError::from_io)?;
                writer
                    .write_all(enc.as_bytes())
                    .map_err(VctrlError::from_io)?;
            }
            None => writer.write_all(&[0u8]).map_err(VctrlError::from_io)?,
        }
        Ok(())
    }
}
