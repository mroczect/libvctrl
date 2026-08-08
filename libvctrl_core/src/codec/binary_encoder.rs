//! Binary encoder – serializes objects into a deterministic byte format.

use libvctrl_handler::{Blob, Commit, Encoder, EntryKind, Tag, Tree, VctrlError};

/// Encodes objects into a deterministic binary format.
///
/// # Format (informal)
/// - **Blob**: 8‑byte little‑endian length prefix, followed by the raw data.
/// - **Tree**: 4‑byte little‑endian entry count, then for each entry:
///     - 1‑byte name length
///     - name bytes (UTF‑8)
///     - 1‑byte kind (`0` = Blob, `1` = Tree)
///     - 64‑byte hash
/// - **Commit**: tree hash (64), parent count (1), each parent (64), author
///   name length (1) + bytes, author email length (1) + bytes, committer
///   name length (1) + bytes, committer email length (1) + bytes, message
///   length (4 little‑endian) + UTF‑8 bytes.
/// - **Tag**: name length (1) + bytes, target hash (64), tagger presence flag
///   (1, `0` or `1`), optionally tagger name/email, message length (4) + UTF‑8.
///
/// # Round‑trip guarantee
/// When paired with [`BinaryDecoder`](super::binary_decoder::BinaryDecoder),
/// the following must hold for any valid object:
/// ```text
/// decode(encode(obj)) == obj
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
