//! # Binary Decoder
//!
//! This module provides a strict, bounds-checked decoder for the binary
//! serialization format defined by the sibling encoder. It is the inverse of
//! the encoder: every byte sequence produced by the encoder is accepted by
//! this decoder, and every decoded object is guaranteed to satisfy the
//! invariants of the corresponding `libvctrl_handler` types.
//!
//! ## Design rationale
//!
//! Decoding untrusted input is one of the most dangerous operations in a
//! version control system. A naive implementation might trust length prefixes
//! and parse out of bounds. This decoder therefore follows a "defense in
//! depth" strategy:
//!
//! - The stream is first bounded by a conservative maximum size.
//! - Every offset is checked before slicing.
//! - Every string is validated as UTF-8.
//! - System limits are re-checked after numeric conversion.
//!
//! ## How it works
//!
//! Each `decode_*` method first calls [`read_bounded`] to slurp the input into
//! a bounded `Vec<u8>`, then calls [`check_version`] to strip and validate the
//! version byte, and finally parses the remaining bytes with explicit offset
//! checks. No slice indexing is performed without a preceding bounds check.

use libvctrl_handler::{
    Blob, Commit, CommitMeta, Decoder, EntryKind, HASH_LENGTH, Hash, MAX_BLOB_SIZE,
    MAX_MESSAGE_LENGTH, MAX_TREE_ENTRIES, Tag, Tree, TreeEntry, UserID, VctrlError,
};
use std::str;

/// The binary format version this decoder accepts.
const EXPECTED_VERSION: u8 = 3;

/// Decodes the binary format for Git objects.
///
/// `BinaryDecoder` is a zero-sized type that implements [`Decoder`]. It accepts
/// any [`std::io::Read`] source and verifies the version byte, length prefixes,
/// and all system limits before constructing the object.
///
/// # Why this struct exists
///
/// The encoder/decoder split separates serialization concerns. `BinaryDecoder`
/// ensures that reading data from external sources is as safe as constructing
/// objects directly through the handler types.
///
/// # How it works
///
/// Each `decode_*` method first calls [`read_bounded`] to slurp the input into
/// a bounded `Vec<u8>`, then calls [`check_version`] to strip the version byte,
/// and finally parses the remaining bytes with explicit offset checks.
///
/// # Examples
///
/// ```
/// use libvctrl_handler::Decoder;
/// use libvctrl_core::codec::BinaryDecoder;
///
/// let decoder = BinaryDecoder;
/// // Decoding methods require an encoded byte stream; see the individual
/// // `decode_blob`, `decode_tree`, `decode_commit`, and `decode_tag` examples.
/// ```
pub struct BinaryDecoder;

impl BinaryDecoder {
    /// Strips and validates the version byte.
    ///
    /// The first byte of every encoded object must equal [`EXPECTED_VERSION`].
    /// Returns the remaining bytes if valid, otherwise a
    /// [`VctrlError::CorruptedData`].
    fn check_version(data: &[u8]) -> Result<&[u8], VctrlError> {
        if data.is_empty() {
            return Err(VctrlError::CorruptedData("missing version byte".into()));
        }
        if data[0] != EXPECTED_VERSION {
            return Err(VctrlError::CorruptedData(format!(
                "unsupported version: {} (expected {})",
                data[0], EXPECTED_VERSION
            )));
        }
        Ok(&data[1..])
    }

    /// Reads the reader into memory while enforcing a hard size bound.
    ///
    /// This helper prevents denial-of-service attacks by refusing to allocate
    /// more than `max_size` bytes. It uses a fixed 4 KiB buffer to avoid
    /// reallocation on each byte and returns [`VctrlError::IoError`] if the
    /// underlying reader fails.
    fn read_bounded<R: std::io::Read>(
        reader: &mut R,
        max_size: usize,
    ) -> Result<Vec<u8>, VctrlError> {
        let mut buf = Vec::new();
        let mut chunk = [0u8; 4096];
        loop {
            let n = reader
                .read(&mut chunk)
                .map_err(|e| VctrlError::IoError(std::sync::Arc::new(e)))?;
            if n == 0 {
                break;
            }
            if buf.len() + n > max_size {
                return Err(VctrlError::CorruptedData(
                    "stream exceeds maximum allowed size".into(),
                ));
            }
            buf.extend_from_slice(&chunk[..n]);
        }
        Ok(buf)
    }
}

impl Decoder for BinaryDecoder {
    /// Decodes a binary blob.
    ///
    /// # Format
    ///
    /// The encoded blob starts with a version byte (currently `3`), followed by
    /// an 8-byte little-endian length prefix and exactly that many data bytes.
    /// The declared byte length is re-checked against [`MAX_BLOB_SIZE`].
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::CorruptedData`] if the version is wrong, the length
    /// prefix is truncated, the blob exceeds the limit, or the declared length
    /// does not match the remaining bytes. Returns [`VctrlError::IoError`] if the
    /// reader fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io::Cursor;
    /// # use libvctrl_handler::{Blob, Decoder, Encoder};
    /// # use libvctrl_core::codec::{BinaryDecoder, BinaryEncoder};
    /// let original = Blob::new(b"hello world".to_vec()).unwrap();
    ///
    /// let mut encoded = Vec::new();
    /// BinaryEncoder.encode_blob(&original, &mut encoded).unwrap();
    ///
    /// let decoded = BinaryDecoder.decode_blob(Cursor::new(encoded.as_slice())).unwrap();
    /// assert_eq!(decoded, original);
    /// ```
    fn decode_blob<R: std::io::Read + Send>(&self, mut reader: R) -> Result<Blob, VctrlError> {
        let max_size = usize::try_from(MAX_BLOB_SIZE).unwrap_or(usize::MAX) + 16;
        let data = Self::read_bounded(&mut reader, max_size)?;

        let data = Self::check_version(&data)?;
        if data.len() < 8 {
            return Err(VctrlError::CorruptedData(
                "blob too short for length prefix".into(),
            ));
        }
        let len_bytes: [u8; 8] = data[..8].try_into().unwrap();
        let data_len = usize::try_from(u64::from_le_bytes(len_bytes))
            .map_err(|_| VctrlError::CorruptedData("blob length out of range".into()))?;
        if data_len > usize::try_from(MAX_BLOB_SIZE).unwrap_or(usize::MAX) {
            return Err(VctrlError::CorruptedData("blob exceeds size limit".into()));
        }
        if data.len() != 8 + data_len {
            return Err(VctrlError::CorruptedData("blob length mismatch".into()));
        }
        Blob::new(data[8..].to_vec())
    }

    /// Decodes a binary tree.
    ///
    /// # Format
    ///
    /// After the version byte, a 4-byte little-endian count is followed by that
    /// many entries. Each entry starts with a one-byte name length, a UTF-8 name,
    /// a one-byte kind tag, and a 64-byte hash.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::CorruptedData`] if any prefix is truncated, the entry
    /// count exceeds [`MAX_TREE_ENTRIES`], the name is not valid UTF-8, the kind
    /// byte is unknown, the hash is invalid, or the final parsed position does not
    /// equal the total byte length. Also returns validation errors from
    /// [`Tree::new`] and [`TreeEntry::new`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io::Cursor;
    /// # use libvctrl_handler::{Decoder, Encoder, EntryKind, Hash, Tree, TreeEntry};
    /// # use libvctrl_core::codec::{BinaryDecoder, BinaryEncoder};
    /// let hash = Hash::from_bytes(&[0u8; 64]).unwrap();
    /// let entry = TreeEntry::new("a.txt".to_owned(), EntryKind::Blob, hash).unwrap();
    /// let original = Tree::new(vec![entry]).unwrap();
    ///
    /// let mut encoded = Vec::new();
    /// BinaryEncoder.encode_tree(&original, &mut encoded).unwrap();
    ///
    /// let decoded = BinaryDecoder.decode_tree(Cursor::new(encoded.as_slice())).unwrap();
    /// assert_eq!(decoded, original);
    /// ```
    fn decode_tree<R: std::io::Read + Send>(&self, mut reader: R) -> Result<Tree, VctrlError> {
        let max_size = usize::try_from(MAX_TREE_ENTRIES).unwrap_or(usize::MAX) * 321 + 5;
        let data = Self::read_bounded(&mut reader, max_size)?;

        let data = Self::check_version(&data)?;
        if data.len() < 4 {
            return Err(VctrlError::CorruptedData("tree too short".into()));
        }
        let count_bytes: [u8; 4] = data[..4].try_into().unwrap();
        let count = u32::from_le_bytes(count_bytes) as usize;
        if count > usize::try_from(MAX_TREE_ENTRIES).unwrap_or(usize::MAX) {
            return Err(VctrlError::CorruptedData(
                "tree entry count exceeds limit".into(),
            ));
        }
        let mut pos = 4;
        let mut entries = Vec::with_capacity(count);
        for _ in 0..count {
            if pos >= data.len() {
                return Err(VctrlError::CorruptedData("unexpected end of tree".into()));
            }
            let name_len = data[pos] as usize;
            pos += 1;
            if pos + name_len > data.len() {
                return Err(VctrlError::CorruptedData("name exceeds data".into()));
            }
            let name = str::from_utf8(&data[pos..pos + name_len])
                .map_err(|_| VctrlError::CorruptedData("invalid UTF-8 in name".into()))?
                .to_string();
            pos += name_len;
            if pos >= data.len() {
                return Err(VctrlError::CorruptedData("missing kind".into()));
            }
            let kind = match data[pos] {
                0 => EntryKind::Blob,
                1 => EntryKind::Executable,
                2 => EntryKind::Symlink,
                3 => EntryKind::Tree,
                4 => EntryKind::Submodule,
                _ => return Err(VctrlError::CorruptedData("unknown entry kind".into())),
            };
            pos += 1;
            if pos + HASH_LENGTH > data.len() {
                return Err(VctrlError::CorruptedData("hash truncated".into()));
            }
            let hash = Hash::from_bytes(&data[pos..pos + HASH_LENGTH])?;
            pos += HASH_LENGTH;
            entries.push(TreeEntry::new(name, kind, hash)?);
        }
        if pos != data.len() {
            return Err(VctrlError::CorruptedData("trailing bytes in tree".into()));
        }
        Tree::new(entries)
    }

    /// Decodes a binary commit.
    ///
    /// # Format
    ///
    /// The commit layout is fixed: tree hash, u16 parent count, parent hashes,
    /// author name/email with u8 length prefixes, committer name/email, u32
    /// message length, message bytes, i64 timestamp, i16 timezone offset, and an
    /// optional encoding string. All integer fields are little-endian.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::CorruptedData`] for structural issues and
    /// [`VctrlError::SerializationError`] if the message exceeds
    /// [`MAX_MESSAGE_LENGTH`]. Also returns validation errors from
    /// [`Commit::with_meta`] and [`UserID::new`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io::Cursor;
    /// # use libvctrl_handler::{Commit, Decoder, Encoder, Hash, UserID};
    /// # use libvctrl_core::codec::{BinaryDecoder, BinaryEncoder};
    /// let tree = Hash::from_bytes(&[1u8; 64]).unwrap();
    /// let author = UserID::new("Alice".to_owned(), "alice@example.com".to_owned()).unwrap();
    /// let committer = UserID::new("Bob".to_owned(), "bob@example.com".to_owned()).unwrap();
    /// let original = Commit::new(
    ///     tree,
    ///     vec![],
    ///     author,
    ///     committer,
    ///     "Initial commit".to_owned(),
    /// )
    /// .unwrap();
    ///
    /// let mut encoded = Vec::new();
    /// BinaryEncoder.encode_commit(&original, &mut encoded).unwrap();
    ///
    /// let decoded = BinaryDecoder.decode_commit(Cursor::new(encoded.as_slice())).unwrap();
    /// assert_eq!(decoded, original);
    /// ```
    #[allow(clippy::too_many_lines)]
    fn decode_commit<R: std::io::Read + Send>(&self, mut reader: R) -> Result<Commit, VctrlError> {
        let max_size = usize::try_from(MAX_MESSAGE_LENGTH).unwrap_or(usize::MAX) + 1024;
        let data = Self::read_bounded(&mut reader, max_size)?;

        let data = Self::check_version(&data)?;
        if data.len() < HASH_LENGTH + 2 {
            return Err(VctrlError::CorruptedData("commit too short".into()));
        }
        let tree = Hash::from_bytes(&data[..HASH_LENGTH])?;
        let parent_count_bytes: [u8; 2] = data[HASH_LENGTH..HASH_LENGTH + 2].try_into().unwrap();
        let parent_count = u16::from_le_bytes(parent_count_bytes) as usize;
        let mut pos = HASH_LENGTH + 2;
        let mut parents = Vec::with_capacity(parent_count);
        for _ in 0..parent_count {
            if pos + HASH_LENGTH > data.len() {
                return Err(VctrlError::CorruptedData("parent hash truncated".into()));
            }
            parents.push(Hash::from_bytes(&data[pos..pos + HASH_LENGTH])?);
            pos += HASH_LENGTH;
        }
        if pos >= data.len() {
            return Err(VctrlError::CorruptedData("missing author name".into()));
        }
        let author_name_len = data[pos] as usize;
        pos += 1;
        if pos + author_name_len > data.len() {
            return Err(VctrlError::CorruptedData("author name truncated".into()));
        }
        let author_name = str::from_utf8(&data[pos..pos + author_name_len])
            .map_err(|_| VctrlError::CorruptedData("invalid UTF-8 in author name".into()))?
            .to_string();
        pos += author_name_len;
        if pos >= data.len() {
            return Err(VctrlError::CorruptedData("missing author email".into()));
        }
        let author_email_len = data[pos] as usize;
        pos += 1;
        if pos + author_email_len > data.len() {
            return Err(VctrlError::CorruptedData("author email truncated".into()));
        }
        let author_email = str::from_utf8(&data[pos..pos + author_email_len])
            .map_err(|_| VctrlError::CorruptedData("invalid UTF-8 in author email".into()))?
            .to_string();
        pos += author_email_len;
        let author = UserID::new(author_name, author_email)?;
        if pos >= data.len() {
            return Err(VctrlError::CorruptedData("missing committer name".into()));
        }
        let committer_name_len = data[pos] as usize;
        pos += 1;
        if pos + committer_name_len > data.len() {
            return Err(VctrlError::CorruptedData("committer name truncated".into()));
        }
        let committer_name = str::from_utf8(&data[pos..pos + committer_name_len])
            .map_err(|_| VctrlError::CorruptedData("invalid UTF-8 in committer name".into()))?
            .to_string();
        pos += committer_name_len;
        if pos >= data.len() {
            return Err(VctrlError::CorruptedData("missing committer email".into()));
        }
        let committer_email_len = data[pos] as usize;
        pos += 1;
        if pos + committer_email_len > data.len() {
            return Err(VctrlError::CorruptedData(
                "committer email truncated".into(),
            ));
        }
        let committer_email = str::from_utf8(&data[pos..pos + committer_email_len])
            .map_err(|_| VctrlError::CorruptedData("invalid UTF-8 in committer email".into()))?
            .to_string();
        pos += committer_email_len;
        let committer = UserID::new(committer_name, committer_email)?;
        if pos + 4 > data.len() {
            return Err(VctrlError::CorruptedData("missing message length".into()));
        }
        let msg_len_bytes: [u8; 4] = data[pos..pos + 4].try_into().unwrap();
        let msg_len = u32::from_le_bytes(msg_len_bytes) as usize;
        pos += 4;
        if msg_len > usize::try_from(MAX_MESSAGE_LENGTH).unwrap_or(usize::MAX) {
            return Err(VctrlError::CorruptedData(
                "commit message exceeds size limit".into(),
            ));
        }
        if pos + msg_len > data.len() {
            return Err(VctrlError::CorruptedData("message truncated".into()));
        }
        let message = str::from_utf8(&data[pos..pos + msg_len])
            .map_err(|_| VctrlError::CorruptedData("invalid UTF-8 in message".into()))?
            .to_string();
        pos += msg_len;

        if pos + 8 > data.len() {
            return Err(VctrlError::CorruptedData("missing timestamp".into()));
        }
        let timestamp = i64::from_le_bytes(data[pos..pos + 8].try_into().unwrap());
        pos += 8;
        if pos + 2 > data.len() {
            return Err(VctrlError::CorruptedData("missing timezone offset".into()));
        }
        let timezone_offset = i16::from_le_bytes(data[pos..pos + 2].try_into().unwrap());
        pos += 2;
        if pos >= data.len() {
            return Err(VctrlError::CorruptedData("missing encoding length".into()));
        }
        let encoding_len = data[pos] as usize;
        pos += 1;
        let encoding = if encoding_len > 0 {
            if pos + encoding_len > data.len() {
                return Err(VctrlError::CorruptedData("encoding truncated".into()));
            }
            let enc = str::from_utf8(&data[pos..pos + encoding_len])
                .map_err(|_| VctrlError::CorruptedData("invalid UTF-8 in encoding".into()))?
                .to_string();
            pos += encoding_len;
            Some(enc)
        } else {
            None
        };

        let meta = CommitMeta::new(timestamp, timezone_offset, encoding)?;
        if pos != data.len() {
            return Err(VctrlError::CorruptedData("trailing bytes in commit".into()));
        }
        Commit::with_meta(tree, parents, author, committer, message, meta)
    }

    /// Decodes a binary tag.
    ///
    /// # Format
    ///
    /// Tag starts with a one-byte name length and name, a 64-byte target hash, a
    /// tagger presence byte, optional tagger name/email, u32 message length,
    /// message, timestamp, timezone offset, and optional encoding.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::CorruptedData`] for structural issues and
    /// [`VctrlError::SerializationError`] if the message exceeds
    /// [`MAX_MESSAGE_LENGTH`]. Also returns validation errors from
    /// [`Tag::with_meta`] and [`UserID::new`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io::Cursor;
    /// # use libvctrl_handler::{Decoder, Encoder, Hash, Tag, UserID};
    /// # use libvctrl_core::codec::{BinaryDecoder, BinaryEncoder};
    /// let target = Hash::from_bytes(&[2u8; 64]).unwrap();
    /// let tagger = UserID::new("Tagger".to_owned(), "tagger@example.com".to_owned()).unwrap();
    /// let original = Tag::new(
    ///     "v1.0.0".to_owned(),
    ///     target,
    ///     Some(tagger),
    ///     "Release".to_owned(),
    /// )
    /// .unwrap();
    ///
    /// let mut encoded = Vec::new();
    /// BinaryEncoder.encode_tag(&original, &mut encoded).unwrap();
    ///
    /// let decoded = BinaryDecoder.decode_tag(Cursor::new(encoded.as_slice())).unwrap();
    /// assert_eq!(decoded, original);
    /// ```
    #[allow(clippy::too_many_lines)]
    fn decode_tag<R: std::io::Read + Send>(&self, mut reader: R) -> Result<Tag, VctrlError> {
        let max_size = usize::try_from(MAX_MESSAGE_LENGTH).unwrap_or(usize::MAX) + 1024;
        let data = Self::read_bounded(&mut reader, max_size)?;

        let data = Self::check_version(&data)?;
        if data.is_empty() {
            return Err(VctrlError::CorruptedData("tag too short".into()));
        }
        let name_len = data[0] as usize;
        let mut pos = 1;
        if pos + name_len > data.len() {
            return Err(VctrlError::CorruptedData("tag name truncated".into()));
        }
        let name = str::from_utf8(&data[pos..pos + name_len])
            .map_err(|_| VctrlError::CorruptedData("invalid UTF-8 in tag name".into()))?
            .to_string();
        pos += name_len;
        if pos + HASH_LENGTH > data.len() {
            return Err(VctrlError::CorruptedData("target hash truncated".into()));
        }
        let target = Hash::from_bytes(&data[pos..pos + HASH_LENGTH])?;
        pos += HASH_LENGTH;
        if pos >= data.len() {
            return Err(VctrlError::CorruptedData(
                "missing tagger presence byte".into(),
            ));
        }
        let has_tagger = match data[pos] {
            0 => false,
            1 => true,
            _ => {
                return Err(VctrlError::CorruptedData(
                    "invalid tagger presence byte".into(),
                ));
            }
        };
        pos += 1;
        let tagger = if has_tagger {
            if pos >= data.len() {
                return Err(VctrlError::CorruptedData("missing tagger name".into()));
            }
            let tagger_name_len = data[pos] as usize;
            pos += 1;
            if pos + tagger_name_len > data.len() {
                return Err(VctrlError::CorruptedData("tagger name truncated".into()));
            }
            let tagger_name = str::from_utf8(&data[pos..pos + tagger_name_len])
                .map_err(|_| VctrlError::CorruptedData("invalid UTF-8 in tagger name".into()))?
                .to_string();
            pos += tagger_name_len;
            if pos >= data.len() {
                return Err(VctrlError::CorruptedData("missing tagger email".into()));
            }
            let tagger_email_len = data[pos] as usize;
            pos += 1;
            if pos + tagger_email_len > data.len() {
                return Err(VctrlError::CorruptedData("tagger email truncated".into()));
            }
            let tagger_email = str::from_utf8(&data[pos..pos + tagger_email_len])
                .map_err(|_| VctrlError::CorruptedData("invalid UTF-8 in tagger email".into()))?
                .to_string();
            pos += tagger_email_len;
            Some(UserID::new(tagger_name, tagger_email)?)
        } else {
            None
        };
        if pos + 4 > data.len() {
            return Err(VctrlError::CorruptedData("missing message length".into()));
        }
        let msg_len_bytes: [u8; 4] = data[pos..pos + 4].try_into().unwrap();
        let msg_len = u32::from_le_bytes(msg_len_bytes) as usize;
        pos += 4;
        if msg_len > usize::try_from(MAX_MESSAGE_LENGTH).unwrap_or(usize::MAX) {
            return Err(VctrlError::SerializationError(
                "tag message exceeds size limit".into(),
            ));
        }
        if pos + msg_len > data.len() {
            return Err(VctrlError::CorruptedData("message truncated".into()));
        }
        let message = str::from_utf8(&data[pos..pos + msg_len])
            .map_err(|_| VctrlError::CorruptedData("invalid UTF-8 in message".into()))?
            .to_string();
        pos += msg_len;

        if pos + 8 > data.len() {
            return Err(VctrlError::CorruptedData("missing timestamp".into()));
        }
        let timestamp = i64::from_le_bytes(data[pos..pos + 8].try_into().unwrap());
        pos += 8;
        if pos + 2 > data.len() {
            return Err(VctrlError::CorruptedData("missing timezone offset".into()));
        }
        let timezone_offset = i16::from_le_bytes(data[pos..pos + 2].try_into().unwrap());
        pos += 2;
        if pos >= data.len() {
            return Err(VctrlError::CorruptedData("missing encoding length".into()));
        }
        let encoding_len = data[pos] as usize;
        pos += 1;
        let encoding = if encoding_len > 0 {
            if pos + encoding_len > data.len() {
                return Err(VctrlError::CorruptedData("encoding truncated".into()));
            }
            let enc = str::from_utf8(&data[pos..pos + encoding_len])
                .map_err(|_| VctrlError::CorruptedData("invalid UTF-8 in encoding".into()))?
                .to_string();
            pos += encoding_len;
            Some(enc)
        } else {
            None
        };

        let meta = CommitMeta::new(timestamp, timezone_offset, encoding)?;
        if pos != data.len() {
            return Err(VctrlError::CorruptedData("trailing bytes in tag".into()));
        }
        Tag::with_meta(name, target, tagger, message, meta)
    }
}
