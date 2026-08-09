//! Binary serialization format encoder for `libvctrl_core`.
//!
//! # Purpose
//! This module provides the [`BinaryEncoder`], a concrete implementation of the
//! [`Encoder`](libvctrl_handler::Encoder) trait. It serializes version control
//! objects ([`Blob`](libvctrl_handler::Blob), [`Tree`](libvctrl_handler::Tree),
//! [`Commit`](libvctrl_handler::Commit), [`Tag`](libvctrl_handler::Tag)) into a
//! compact, little-endian binary format suitable for storage or network transmission.
//!
//! # Design rationale
//! - **Little-Endian Integers**: All integer fields (lengths, timestamps) are
//!   encoded in little-endian format. This is consistent with modern CPU
//!   architectures (x86, ARM) and avoids byte-swapping overhead.
//! - **Length-Prefixed Strings**: Variable-length data (names, messages) are
//!   prefixed by their length. This allows the corresponding
//!   [`BinaryDecoder`](crate::codec::BinaryDecoder) to pre-allocate buffers
//!   efficiently and avoid reading until EOF.
//! - **Versioning**: Every serialized payload begins with a version byte
//!   ([`VERSION`]). This ensures forward/backward compatibility; if the format
//!   changes, the version can be bumped, and decoders can reject unsupported
//!   versions cleanly.
//! - **Zero-Copy Where Possible**: The encoder uses `extend_from_slice` to
//!   copy data directly into the output `Vec<u8>`, leveraging LLVM's
//!   `memcpy` intrinsics for fast bulk copies.

use libvctrl_handler::{
    Blob, Commit, Encoder, EntryKind, MAX_MESSAGE_LENGTH, Tag, Tree, VctrlError,
};

/// The binary format version number.
///
/// # Purpose
/// This constant is prepended to every serialized object. It allows the
/// [`BinaryDecoder`](crate::codec::BinaryDecoder) to verify that the data
/// was produced by a compatible encoder.
///
/// # Design rationale
/// Bumping this version allows breaking changes to the wire format in the
/// future. A decoder reading an unexpected version can fail gracefully
/// instead of attempting to parse incompatible data.
///
/// # Examples
///
/// ```
/// use libvctrl_core::codec::binary_encoder::VERSION;
/// assert_eq!(VERSION, 2);
/// ```
pub const VERSION: u8 = 2;

/// A binary encoder that serializes version control objects into a compact byte format.
///
/// # Purpose
/// Implements the [`Encoder`](libvctrl_handler::Encoder) trait to convert
/// in-memory objects into a deterministic binary representation.
///
/// # Design rationale
/// This encoder is stateless (a unit struct) because encoding does not require
/// external configuration or state. It can be instantiated cheaply anywhere.
///
/// # Internal mechanism
/// The encoder pre-allocates a `Vec<u8>` based on the estimated size of the
/// object to minimize reallocations. It then pushes the version byte, followed
/// by length-prefixed fields, using `extend_from_slice` for fast memory copies.
///
/// # Examples
///
/// Encoding a simple `Blob`:
///
/// ```
/// use libvctrl_handler::{Blob, Encoder};
/// use libvctrl_core::codec::BinaryEncoder;
///
/// let encoder = BinaryEncoder;
/// let blob = Blob::new(b"hello".to_vec());
/// let bytes = encoder.encode_blob(&blob).unwrap();
///
/// // The first byte is the version
/// assert_eq!(bytes[0], 2);
/// ```
pub struct BinaryEncoder;

impl Encoder for BinaryEncoder {
    /// Encodes a [`Blob`](libvctrl_handler::Blob) into a byte vector.
    ///
    /// # Format
    /// 1. `VERSION` (1 byte, u8)
    /// 2. `data_len` (8 bytes, u64 LE)
    /// 3. `data` (`data_len` bytes)
    ///
    /// # Errors
    /// This method is currently infallible for valid `Blob`s, but returns a
    /// `Result` to satisfy the [`Encoder`](libvctrl_handler::Encoder) trait.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::{Blob, Encoder};
    /// use libvctrl_core::codec::BinaryEncoder;
    ///
    /// let encoder = BinaryEncoder;
    /// let blob = Blob::new(vec![1, 2, 3]);
    /// let encoded = encoder.encode_blob(&blob).unwrap();
    ///
    /// assert_eq!(encoded.len(), 1 + 8 + 3);
    /// ```
    fn encode_blob(&self, blob: &Blob) -> Result<Vec<u8>, VctrlError> {
        let data = blob.data();
        let mut out = Vec::with_capacity(1 + 8 + data.len());
        out.push(VERSION);
        out.extend_from_slice(&(data.len() as u64).to_le_bytes());
        out.extend_from_slice(data);
        Ok(out)
    }

    /// Encodes a [`Tree`](libvctrl_handler::Tree) into a byte vector.
    ///
    /// # Format
    /// 1. `VERSION` (1 byte, u8)
    /// 2. `entry_count` (4 bytes, u32 LE)
    /// 3. For each entry:
    ///    a. `name_len` (1 byte, u8)
    ///    b. `name` (`name_len` bytes, UTF-8)
    ///    c. `kind` (1 byte, u8: 0 = Blob, 1 = Tree)
    ///    d. `hash` (64 bytes)
    ///
    /// # Errors
    /// Returns [`VctrlError::SerializationError`](libvctrl_handler::VctrlError::SerializationError)
    /// if the number of entries exceeds `u32::MAX`, or if a name length exceeds
    /// `u8::MAX` (255 bytes).
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::{Encoder, EntryKind, Hash, Tree, TreeEntry};
    /// use libvctrl_core::codec::BinaryEncoder;
    ///
    /// let encoder = BinaryEncoder;
    /// let hash = Hash::from_bytes(&[0; 64]).unwrap();
    /// let entry = TreeEntry::new("file.txt".to_string(), EntryKind::Blob, hash).unwrap();
    /// let tree = Tree::new(vec![entry]).unwrap();
    ///
    /// let encoded = encoder.encode_tree(&tree).unwrap();
    /// assert!(!encoded.is_empty());
    /// ```
    fn encode_tree(&self, tree: &Tree) -> Result<Vec<u8>, VctrlError> {
        let entries = tree.entries();
        let mut out = vec![VERSION];
        let entry_count = u32::try_from(entries.len())
            .map_err(|_| VctrlError::SerializationError("too many entries".into()))?;
        out.extend_from_slice(&entry_count.to_le_bytes());
        for entry in entries {
            let name = entry.name();
            let name_len = u8::try_from(name.len())
                .map_err(|_| VctrlError::SerializationError("name too long".into()))?;
            out.push(name_len);
            out.extend_from_slice(name.as_bytes());
            out.push(match entry.kind() {
                EntryKind::Blob => 0,
                EntryKind::Tree => 1,
                _ => return Err(VctrlError::SerializationError("unknown entry kind".into())),
            });
            out.extend_from_slice(entry.hash().as_bytes());
        }
        Ok(out)
    }

    /// Encodes a [`Commit`](libvctrl_handler::Commit) into a byte vector.
    ///
    /// # Format
    /// 1. `VERSION` (1 byte, u8)
    /// 2. `tree_hash` (64 bytes)
    /// 3. `parent_count` (1 byte, u8)
    /// 4. `parent_hashes` (64 bytes * `parent_count`)
    /// 5. `author_name_len` (1 byte, u8) + `author_name`
    /// 6. `author_email_len` (1 byte, u8) + `author_email`
    /// 7. `committer_name_len` (1 byte, u8) + `committer_name`
    /// 8. `committer_email_len` (1 byte, u8) + `committer_email`
    /// 9. `msg_len` (4 bytes, u32 LE) + `msg`
    /// 10. `timestamp` (8 bytes, i64 LE)
    /// 11. `timezone_offset` (2 bytes, i16 LE)
    /// 12. `encoding_len` (1 byte, u8) + `encoding`
    ///
    /// # Errors
    /// Returns [`VctrlError::SerializationError`](libvctrl_handler::VctrlError::SerializationError)
    /// if string lengths exceed their prefix limits (u8 or u32), if the message
    /// exceeds `MAX_MESSAGE_LENGTH`, or if there are more than 255 parents.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::{Commit, Encoder, Hash, UserID};
    /// use libvctrl_core::codec::BinaryEncoder;
    ///
    /// let encoder = BinaryEncoder;
    /// let tree = Hash::from_bytes(&[0; 64]).unwrap();
    /// let user = UserID::new("Alice".to_string(), "alice@example.com".to_string()).unwrap();
    /// let commit = Commit::new(tree, Vec::new(), user.clone(), user, "Initial".to_string());
    ///
    /// let encoded = encoder.encode_commit(&commit).unwrap();
    /// assert!(!encoded.is_empty());
    /// ```
    fn encode_commit(&self, commit: &Commit) -> Result<Vec<u8>, VctrlError> {
        let mut out = vec![VERSION];
        out.extend_from_slice(commit.tree().as_bytes());
        let parents = commit.parents();
        let parent_count = u8::try_from(parents.len())
            .map_err(|_| VctrlError::SerializationError("too many parents".into()))?;
        out.push(parent_count);
        for p in parents {
            out.extend_from_slice(p.as_bytes());
        }
        let author_name_len = u8::try_from(commit.author().name().len())
            .map_err(|_| VctrlError::SerializationError("author name too long".into()))?;
        out.push(author_name_len);
        out.extend_from_slice(commit.author().name().as_bytes());
        let author_email_len = u8::try_from(commit.author().email().len())
            .map_err(|_| VctrlError::SerializationError("author email too long".into()))?;
        out.push(author_email_len);
        out.extend_from_slice(commit.author().email().as_bytes());
        let committer_name_len = u8::try_from(commit.committer().name().len())
            .map_err(|_| VctrlError::SerializationError("committer name too long".into()))?;
        out.push(committer_name_len);
        out.extend_from_slice(commit.committer().name().as_bytes());
        let committer_email_len = u8::try_from(commit.committer().email().len())
            .map_err(|_| VctrlError::SerializationError("committer email too long".into()))?;
        out.push(committer_email_len);
        out.extend_from_slice(commit.committer().email().as_bytes());
        let msg = commit.message();
        let msg_len = u32::try_from(msg.len())
            .map_err(|_| VctrlError::SerializationError("message too long".into()))?;
        if msg_len as usize > MAX_MESSAGE_LENGTH {
            return Err(VctrlError::SerializationError(
                "commit message exceeds size limit".into(),
            ));
        }
        out.extend_from_slice(&msg_len.to_le_bytes());
        out.extend_from_slice(msg.as_bytes());
        out.extend_from_slice(&commit.timestamp().to_le_bytes());
        out.extend_from_slice(&commit.timezone_offset().to_le_bytes());
        match commit.encoding() {
            Some(enc) => {
                let len = u8::try_from(enc.len())
                    .map_err(|_| VctrlError::SerializationError("encoding too long".into()))?;
                out.push(len);
                out.extend_from_slice(enc.as_bytes());
            }
            None => out.push(0u8),
        }
        Ok(out)
    }

    /// Encodes a [`Tag`](libvctrl_handler::Tag) into a byte vector.
    ///
    /// # Format
    /// 1. `VERSION` (1 byte, u8)
    /// 2. `name_len` (1 byte, u8) + `name`
    /// 3. `target_hash` (64 bytes)
    /// 4. `has_tagger` (1 byte, u8: 0 = false, 1 = true)
    /// 5. If `has_tagger` is 1:
    ///    a. `tagger_name_len` (1 byte, u8) + `tagger_name`
    ///    b. `tagger_email_len` (1 byte, u8) + `tagger_email`
    /// 6. `msg_len` (4 bytes, u32 LE) + `msg`
    /// 7. `timestamp` (8 bytes, i64 LE)
    /// 8. `timezone_offset` (2 bytes, i16 LE)
    /// 9. `encoding_len` (1 byte, u8) + `encoding`
    ///
    /// # Errors
    /// Returns [`VctrlError::SerializationError`](libvctrl_handler::VctrlError::SerializationError)
    /// if string lengths exceed their prefix limits (u8 or u32), or if the
    /// message exceeds `MAX_MESSAGE_LENGTH`.
    ///
    /// # Examples
    ///
    /// ```
    /// use libvctrl_handler::{Encoder, Hash, Tag, UserID};
    /// use libvctrl_core::codec::BinaryEncoder;
    ///
    /// let encoder = BinaryEncoder;
    /// let target = Hash::from_bytes(&[0; 64]).unwrap();
    /// let tagger = UserID::new("Bob".to_string(), "bob@example.com".to_string()).unwrap();
    /// let tag = Tag::new("v1.0".to_string(), target, Some(tagger), "Release".to_string()).unwrap();
    ///
    /// let encoded = encoder.encode_tag(&tag).unwrap();
    /// assert!(!encoded.is_empty());
    /// ```
    fn encode_tag(&self, tag: &Tag) -> Result<Vec<u8>, VctrlError> {
        let mut out = vec![VERSION];
        let name_len = u8::try_from(tag.name().len())
            .map_err(|_| VctrlError::SerializationError("tag name too long".into()))?;
        out.push(name_len);
        out.extend_from_slice(tag.name().as_bytes());
        out.extend_from_slice(tag.target().as_bytes());
        match tag.tagger() {
            Some(tagger) => {
                out.push(1u8);
                let tagger_name_len = u8::try_from(tagger.name().len())
                    .map_err(|_| VctrlError::SerializationError("tagger name too long".into()))?;
                out.push(tagger_name_len);
                out.extend_from_slice(tagger.name().as_bytes());
                let tagger_email_len = u8::try_from(tagger.email().len())
                    .map_err(|_| VctrlError::SerializationError("tagger email too long".into()))?;
                out.push(tagger_email_len);
                out.extend_from_slice(tagger.email().as_bytes());
            }
            None => out.push(0u8),
        }
        let msg = tag.message();
        let msg_len = u32::try_from(msg.len())
            .map_err(|_| VctrlError::SerializationError("message too long".into()))?;
        out.extend_from_slice(&msg_len.to_le_bytes());
        out.extend_from_slice(msg.as_bytes());
        out.extend_from_slice(&tag.timestamp().to_le_bytes());
        out.extend_from_slice(&tag.timezone_offset().to_le_bytes());
        match tag.encoding() {
            Some(enc) => {
                let len = u8::try_from(enc.len())
                    .map_err(|_| VctrlError::SerializationError("encoding too long".into()))?;
                out.push(len);
                out.extend_from_slice(enc.as_bytes());
            }
            None => out.push(0u8),
        }
        Ok(out)
    }
}
