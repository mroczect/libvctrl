//! Binary encoder – serializes objects into the deterministic binary format.
//!
//! This module provides [`BinaryEncoder`], which implements the [`Encoder`]
//! trait. The format is designed to be simple, predictable, and easy to
//! implement in any language.

use libvctrl_handler::{Blob, Commit, Encoder, EntryKind, Tag, Tree, VctrlError};

/// Encodes objects into a deterministic binary format.
///
/// # Format specification
///
/// The encoder produces a byte sequence that can be parsed by
/// [`BinaryDecoder`](super::binary_decoder::BinaryDecoder). Every multi‑byte
/// integer is encoded as little‑endian.
///
/// ## Blob
/// ```text
/// [ 8 bytes data_length | data_bytes... ]
/// ```
///
/// ## Tree
/// ```text
/// [ 4 bytes entry_count ]
/// for each entry:
///     [ 1 byte name_length | name_bytes... | 1 byte kind | 64 bytes hash ]
/// ```
/// where `kind` is `0` for Blob and `1` for Tree.
///
/// ## Commit
/// ```text
/// [ 64 bytes tree_hash ]
/// [ 1 byte parent_count ]
/// [ for each parent: 64 bytes parent_hash ]
/// [ 1 byte author_name_length | author_name_bytes... ]
/// [ 1 byte author_email_length | author_email_bytes... ]
/// [ 1 byte committer_name_length | committer_name_bytes... ]
/// [ 1 byte committer_email_length | committer_email_bytes... ]
/// [ 4 bytes message_length | message_bytes... ]
/// ```
///
/// ## Tag
/// ```text
/// [ 1 byte name_length | name_bytes... ]
/// [ 64 bytes target_hash ]
/// [ 1 byte has_tagger ]
/// if has_tagger == 1:
///     [ 1 byte tagger_name_length | tagger_name_bytes... ]
///     [ 1 byte tagger_email_length | tagger_email_bytes... ]
/// [ 4 bytes message_length | message_bytes... ]
/// ```
///
/// # Error handling
///
/// All methods return [`VctrlError::SerializationError`] if a field exceeds
/// the maximum allowed size (e.g., a name longer than 255 bytes). This is
/// a safety measure to ensure the encoding remains valid.
///
/// # Round‑trip guarantee
///
/// When paired with [`BinaryDecoder`](super::binary_decoder::BinaryDecoder),
/// encoding and then decoding any valid object must yield the original object.
///
/// # Example
///
/// ```rust
/// use libvctrl_core::codec::BinaryEncoder;
/// use libvctrl_handler::{Blob, Encoder};
///
/// let encoder = BinaryEncoder;
/// let blob = Blob::new(b"example".to_vec());
/// let encoded = encoder.encode_blob(&blob).expect("encode should succeed");
/// // encoded is now a Vec<u8> ready for storage.
/// ```
pub struct BinaryEncoder;

impl Encoder for BinaryEncoder {
    fn encode_blob(&self, blob: &Blob) -> Result<Vec<u8>, VctrlError> {
        let data = blob.data();
        let mut out = Vec::with_capacity(8 + data.len());
        out.extend_from_slice(&(data.len() as u64).to_le_bytes());
        out.extend_from_slice(data);
        Ok(out)
    }

    fn encode_tree(&self, tree: &Tree) -> Result<Vec<u8>, VctrlError> {
        let entries = tree.entries();
        let mut out = Vec::new();
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

    fn encode_commit(&self, commit: &Commit) -> Result<Vec<u8>, VctrlError> {
        let mut out = Vec::new();
        out.extend_from_slice(commit.tree().as_bytes());
        let parents = commit.parents();
        let parent_count = u8::try_from(parents.len())
            .map_err(|_| VctrlError::SerializationError("too many parents".into()))?;
        out.push(parent_count);
        for p in parents {
            out.extend_from_slice(p.as_bytes());
        }
        // Author
        let author_name_len = u8::try_from(commit.author().name().len())
            .map_err(|_| VctrlError::SerializationError("author name too long".into()))?;
        out.push(author_name_len);
        out.extend_from_slice(commit.author().name().as_bytes());
        let author_email_len = u8::try_from(commit.author().email().len())
            .map_err(|_| VctrlError::SerializationError("author email too long".into()))?;
        out.push(author_email_len);
        out.extend_from_slice(commit.author().email().as_bytes());
        // Committer
        let committer_name_len = u8::try_from(commit.committer().name().len())
            .map_err(|_| VctrlError::SerializationError("committer name too long".into()))?;
        out.push(committer_name_len);
        out.extend_from_slice(commit.committer().name().as_bytes());
        let committer_email_len = u8::try_from(commit.committer().email().len())
            .map_err(|_| VctrlError::SerializationError("committer email too long".into()))?;
        out.push(committer_email_len);
        out.extend_from_slice(commit.committer().email().as_bytes());
        // Message
        let msg = commit.message();
        let msg_len = u32::try_from(msg.len())
            .map_err(|_| VctrlError::SerializationError("message too long".into()))?;
        out.extend_from_slice(&msg_len.to_le_bytes());
        out.extend_from_slice(msg.as_bytes());
        Ok(out)
    }

    fn encode_tag(&self, tag: &Tag) -> Result<Vec<u8>, VctrlError> {
        let mut out = Vec::new();
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
        Ok(out)
    }
}
