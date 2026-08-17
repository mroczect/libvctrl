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
        let version = data
            .first()
            .copied()
            .ok_or_else(|| VctrlError::CorruptedData("missing version byte".into()))?;
        if version != EXPECTED_VERSION {
            return Err(VctrlError::CorruptedData(format!(
                "unsupported version: {} (expected {})",
                version, EXPECTED_VERSION
            )));
        }
        data.get(1..)
            .ok_or_else(|| VctrlError::CorruptedData("missing payload after version".into()))
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
            buf.extend_from_slice(chunk.get(..n).unwrap_or(&[]));
        }
        Ok(buf)
    }

    /// Returns a single byte at `pos`, or a structured error.
    fn require_byte(data: &[u8], pos: usize, what: &str) -> Result<u8, VctrlError> {
        data.get(pos)
            .copied()
            .ok_or_else(|| VctrlError::CorruptedData(format!("missing {what}")))
    }

    /// Returns a slice `data[start..start+len]`, with overflow and bounds checks.
    fn require_slice<'a>(
        data: &'a [u8],
        start: usize,
        len: usize,
        what: &str,
    ) -> Result<&'a [u8], VctrlError> {
        let end = start
            .checked_add(len)
            .ok_or_else(|| VctrlError::CorruptedData(format!("invalid {what} length")))?;
        data.get(start..end)
            .ok_or_else(|| VctrlError::CorruptedData(format!("{what} truncated")))
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
        let len_bytes: [u8; 8] = Self::require_slice(data, 0, 8, "blob length prefix")?
            .try_into()
            .map_err(|e| VctrlError::CorruptedData(format!("invalid blob length prefix: {e}")))?;

        let data_len = usize::try_from(u64::from_le_bytes(len_bytes))
            .map_err(|e| VctrlError::CorruptedData(format!("blob length out of range: {e}")))?;

        if data_len > usize::try_from(MAX_BLOB_SIZE).unwrap_or(usize::MAX) {
            return Err(VctrlError::CorruptedData("blob exceeds size limit".into()));
        }

        let total_len = 8usize
            .checked_add(data_len)
            .ok_or_else(|| VctrlError::CorruptedData("blob length overflow".into()))?;
        if data.len() != total_len {
            return Err(VctrlError::CorruptedData("blob length mismatch".into()));
        }

        let payload = Self::require_slice(data, 8, data_len, "blob data")?;
        Blob::new(payload.to_vec())
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
        let count_bytes: [u8; 4] = Self::require_slice(data, 0, 4, "tree entry count")?
            .try_into()
            .map_err(|e| VctrlError::CorruptedData(format!("invalid tree count: {e}")))?;
        let count = u32::from_le_bytes(count_bytes) as usize;

        if count > usize::try_from(MAX_TREE_ENTRIES).unwrap_or(usize::MAX) {
            return Err(VctrlError::CorruptedData(
                "tree entry count exceeds limit".into(),
            ));
        }

        let mut pos = 4usize;
        let mut entries = Vec::with_capacity(count);

        for _ in 0..count {
            let name_len = Self::require_byte(data, pos, "tree entry name length")? as usize;
            pos += 1;

            let name_bytes = Self::require_slice(data, pos, name_len, "tree entry name")?;
            let name = str::from_utf8(name_bytes)
                .map_err(|e| VctrlError::CorruptedData(format!("invalid UTF-8 in name: {e}")))?
                .to_string();
            pos += name_len;

            let kind_byte = Self::require_byte(data, pos, "tree entry kind")?;
            pos += 1;

            let kind = match kind_byte {
                0 => EntryKind::Blob,
                1 => EntryKind::Executable,
                2 => EntryKind::Symlink,
                3 => EntryKind::Tree,
                4 => EntryKind::Submodule,
                other => {
                    return Err(VctrlError::CorruptedData(format!(
                        "unknown entry kind: {other}"
                    )));
                }
            };

            let hash_bytes = Self::require_slice(data, pos, HASH_LENGTH, "tree entry hash")?;
            let hash = Hash::from_bytes(hash_bytes)?;
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

        // Tree hash
        let tree_hash = Self::require_slice(data, 0, HASH_LENGTH, "commit tree hash")?;
        let tree = Hash::from_bytes(tree_hash)?;

        // Parent count and parents
        let parent_count_bytes = Self::require_slice(data, HASH_LENGTH, 2, "commit parent count")?;
        let parent_count = u16::from_le_bytes(
            parent_count_bytes
                .try_into()
                .map_err(|e| VctrlError::CorruptedData(format!("invalid parent count: {e}")))?,
        ) as usize;

        let mut pos = HASH_LENGTH + 2;
        let mut parents = Vec::with_capacity(parent_count);
        for _ in 0..parent_count {
            let parent_bytes = Self::require_slice(data, pos, HASH_LENGTH, "parent hash")?;
            parents.push(Hash::from_bytes(parent_bytes)?);
            pos += HASH_LENGTH;
        }

        // Author name
        let author_name_len = Self::require_byte(data, pos, "author name length")? as usize;
        pos += 1;
        let author_name_bytes = Self::require_slice(data, pos, author_name_len, "author name")?;
        let author_name = str::from_utf8(author_name_bytes)
            .map_err(|e| VctrlError::CorruptedData(format!("invalid UTF-8 in author name: {e}")))?
            .to_string();
        pos += author_name_len;

        // Author email
        let author_email_len = Self::require_byte(data, pos, "author email length")? as usize;
        pos += 1;
        let author_email_bytes = Self::require_slice(data, pos, author_email_len, "author email")?;
        let author_email = str::from_utf8(author_email_bytes)
            .map_err(|e| VctrlError::CorruptedData(format!("invalid UTF-8 in author email: {e}")))?
            .to_string();
        pos += author_email_len;

        let author = UserID::new(author_name, author_email)?;

        // Committer name
        let committer_name_len = Self::require_byte(data, pos, "committer name length")? as usize;
        pos += 1;
        let committer_name_bytes =
            Self::require_slice(data, pos, committer_name_len, "committer name")?;
        let committer_name = str::from_utf8(committer_name_bytes)
            .map_err(|e| {
                VctrlError::CorruptedData(format!("invalid UTF-8 in committer name: {e}"))
            })?
            .to_string();
        pos += committer_name_len;

        // Committer email
        let committer_email_len = Self::require_byte(data, pos, "committer email length")? as usize;
        pos += 1;
        let committer_email_bytes =
            Self::require_slice(data, pos, committer_email_len, "committer email")?;
        let committer_email = str::from_utf8(committer_email_bytes)
            .map_err(|e| {
                VctrlError::CorruptedData(format!("invalid UTF-8 in committer email: {e}"))
            })?
            .to_string();
        pos += committer_email_len;

        let committer = UserID::new(committer_name, committer_email)?;

        // Message
        let msg_len_bytes = Self::require_slice(data, pos, 4, "commit message length")?;
        let msg_len = u32::from_le_bytes(
            msg_len_bytes
                .try_into()
                .map_err(|e| VctrlError::CorruptedData(format!("invalid message length: {e}")))?,
        ) as usize;
        pos += 4;

        if msg_len > usize::try_from(MAX_MESSAGE_LENGTH).unwrap_or(usize::MAX) {
            return Err(VctrlError::CorruptedData(
                "commit message exceeds size limit".into(),
            ));
        }

        let msg_bytes = Self::require_slice(data, pos, msg_len, "commit message")?;
        let message = str::from_utf8(msg_bytes)
            .map_err(|e| VctrlError::CorruptedData(format!("invalid UTF-8 in message: {e}")))?
            .to_string();
        pos += msg_len;

        // Timestamp and timezone
        let timestamp_bytes = Self::require_slice(data, pos, 8, "commit timestamp")?;
        let timestamp = i64::from_le_bytes(
            timestamp_bytes
                .try_into()
                .map_err(|e| VctrlError::CorruptedData(format!("invalid timestamp: {e}")))?,
        );
        pos += 8;

        let tz_bytes = Self::require_slice(data, pos, 2, "commit timezone offset")?;
        let timezone_offset = i16::from_le_bytes(
            tz_bytes
                .try_into()
                .map_err(|e| VctrlError::CorruptedData(format!("invalid timezone offset: {e}")))?,
        );
        pos += 2;

        // Optional encoding
        let encoding_len = Self::require_byte(data, pos, "commit encoding length")? as usize;
        pos += 1;
        let encoding = if encoding_len > 0 {
            let enc_bytes = Self::require_slice(data, pos, encoding_len, "commit encoding")?;
            let enc = str::from_utf8(enc_bytes)
                .map_err(|e| VctrlError::CorruptedData(format!("invalid UTF-8 in encoding: {e}")))?
                .to_string();
            pos += encoding_len;
            Some(enc)
        } else {
            None
        };

        if pos != data.len() {
            return Err(VctrlError::CorruptedData("trailing bytes in commit".into()));
        }

        let meta = CommitMeta::new(timestamp, timezone_offset, encoding)?;
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

        // Tag name
        let name_len = Self::require_byte(data, 0, "tag name length")? as usize;
        let name_bytes = Self::require_slice(data, 1, name_len, "tag name")?;
        let name = str::from_utf8(name_bytes)
            .map_err(|e| VctrlError::CorruptedData(format!("invalid UTF-8 in tag name: {e}")))?
            .to_string();
        let mut pos = 1 + name_len;

        // Target hash
        let target_bytes = Self::require_slice(data, pos, HASH_LENGTH, "tag target hash")?;
        let target = Hash::from_bytes(target_bytes)?;
        pos += HASH_LENGTH;

        // Tagger presence
        let has_tagger = match Self::require_byte(data, pos, "tagger presence byte")? {
            0 => false,
            1 => true,
            other => {
                return Err(VctrlError::CorruptedData(format!(
                    "invalid tagger presence byte: {other}"
                )));
            }
        };
        pos += 1;

        // Optional tagger
        let tagger = if has_tagger {
            let tagger_name_len = Self::require_byte(data, pos, "tagger name length")? as usize;
            pos += 1;
            let tagger_name_bytes = Self::require_slice(data, pos, tagger_name_len, "tagger name")?;
            let tagger_name = str::from_utf8(tagger_name_bytes)
                .map_err(|e| {
                    VctrlError::CorruptedData(format!("invalid UTF-8 in tagger name: {e}"))
                })?
                .to_string();
            pos += tagger_name_len;

            let tagger_email_len = Self::require_byte(data, pos, "tagger email length")? as usize;
            pos += 1;
            let tagger_email_bytes =
                Self::require_slice(data, pos, tagger_email_len, "tagger email")?;
            let tagger_email = str::from_utf8(tagger_email_bytes)
                .map_err(|e| {
                    VctrlError::CorruptedData(format!("invalid UTF-8 in tagger email: {e}"))
                })?
                .to_string();
            pos += tagger_email_len;

            Some(UserID::new(tagger_name, tagger_email)?)
        } else {
            None
        };

        // Message
        let msg_len_bytes = Self::require_slice(data, pos, 4, "tag message length")?;
        let msg_len = u32::from_le_bytes(
            msg_len_bytes
                .try_into()
                .map_err(|e| VctrlError::CorruptedData(format!("invalid message length: {e}")))?,
        ) as usize;
        pos += 4;

        if msg_len > usize::try_from(MAX_MESSAGE_LENGTH).unwrap_or(usize::MAX) {
            return Err(VctrlError::SerializationError(
                "tag message exceeds size limit".into(),
            ));
        }

        let msg_bytes = Self::require_slice(data, pos, msg_len, "tag message")?;
        let message = str::from_utf8(msg_bytes)
            .map_err(|e| VctrlError::CorruptedData(format!("invalid UTF-8 in message: {e}")))?
            .to_string();
        pos += msg_len;

        // Timestamp and timezone
        let timestamp_bytes = Self::require_slice(data, pos, 8, "tag timestamp")?;
        let timestamp = i64::from_le_bytes(
            timestamp_bytes
                .try_into()
                .map_err(|e| VctrlError::CorruptedData(format!("invalid timestamp: {e}")))?,
        );
        pos += 8;

        let tz_bytes = Self::require_slice(data, pos, 2, "tag timezone offset")?;
        let timezone_offset = i16::from_le_bytes(
            tz_bytes
                .try_into()
                .map_err(|e| VctrlError::CorruptedData(format!("invalid timezone offset: {e}")))?,
        );
        pos += 2;

        // Optional encoding
        let encoding_len = Self::require_byte(data, pos, "tag encoding length")? as usize;
        pos += 1;
        let encoding = if encoding_len > 0 {
            let enc_bytes = Self::require_slice(data, pos, encoding_len, "tag encoding")?;
            let enc = str::from_utf8(enc_bytes)
                .map_err(|e| VctrlError::CorruptedData(format!("invalid UTF-8 in encoding: {e}")))?
                .to_string();
            pos += encoding_len;
            Some(enc)
        } else {
            None
        };

        if pos != data.len() {
            return Err(VctrlError::CorruptedData("trailing bytes in tag".into()));
        }

        let meta = CommitMeta::new(timestamp, timezone_offset, encoding)?;
        Tag::with_meta(name, target, tagger, message, meta)
    }
}
