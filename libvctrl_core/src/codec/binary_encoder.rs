use libvctrl_handler::{
    Blob, Commit, Encoder, EntryKind, MAX_MESSAGE_LENGTH, Tag, Tree, VctrlError,
};
use std::io::Write;

pub const VERSION: u8 = 2;

pub struct BinaryEncoder;

impl Encoder for BinaryEncoder {
    fn encode_blob<W: Write + Send>(&self, blob: &Blob, writer: &mut W) -> Result<(), VctrlError> {
        let data = blob.data();
        writer.write_all(&[VERSION]).map_err(io_err)?;
        writer
            .write_all(&(data.len() as u64).to_le_bytes())
            .map_err(io_err)?;
        writer.write_all(data).map_err(io_err)?;
        Ok(())
    }

    fn encode_tree<W: Write + Send>(&self, tree: &Tree, writer: &mut W) -> Result<(), VctrlError> {
        let entries = tree.entries();
        writer.write_all(&[VERSION]).map_err(io_err)?;
        let entry_count = u32::try_from(entries.len())
            .map_err(|_| VctrlError::SerializationError("too many entries".into()))?;
        writer
            .write_all(&entry_count.to_le_bytes())
            .map_err(io_err)?;

        for entry in entries {
            let name = entry.name();
            let name_len = u8::try_from(name.len())
                .map_err(|_| VctrlError::SerializationError("name too long".into()))?;
            writer.write_all(&[name_len]).map_err(io_err)?;
            writer.write_all(name.as_bytes()).map_err(io_err)?;

            let kind_byte = match entry.kind() {
                EntryKind::Blob => 0,
                EntryKind::Executable => 1,
                EntryKind::Symlink => 2,
                EntryKind::Tree => 3,
                EntryKind::Submodule => 4,
                _ => return Err(VctrlError::SerializationError("unknown entry kind".into())),
            };
            writer.write_all(&[kind_byte]).map_err(io_err)?;
            writer.write_all(entry.hash().as_bytes()).map_err(io_err)?;
        }
        Ok(())
    }

    fn encode_commit<W: Write + Send>(
        &self,
        commit: &Commit,
        writer: &mut W,
    ) -> Result<(), VctrlError> {
        writer.write_all(&[VERSION]).map_err(io_err)?;
        writer.write_all(commit.tree().as_bytes()).map_err(io_err)?;

        let parents = commit.parents();
        let parent_count = u8::try_from(parents.len())
            .map_err(|_| VctrlError::SerializationError("too many parents".into()))?;
        writer.write_all(&[parent_count]).map_err(io_err)?;

        for p in parents {
            writer.write_all(p.as_bytes()).map_err(io_err)?;
        }

        let author_name = commit.author().name();
        writer
            .write_all(&[u8::try_from(author_name.len())
                .map_err(|_| VctrlError::SerializationError("author name too long".into()))?])
            .map_err(io_err)?;
        writer.write_all(author_name.as_bytes()).map_err(io_err)?;

        let author_email = commit.author().email();
        writer
            .write_all(&[u8::try_from(author_email.len())
                .map_err(|_| VctrlError::SerializationError("author email too long".into()))?])
            .map_err(io_err)?;
        writer.write_all(author_email.as_bytes()).map_err(io_err)?;

        let committer_name = commit.committer().name();
        writer
            .write_all(&[u8::try_from(committer_name.len())
                .map_err(|_| VctrlError::SerializationError("committer name too long".into()))?])
            .map_err(io_err)?;
        writer
            .write_all(committer_name.as_bytes())
            .map_err(io_err)?;

        let committer_email = commit.committer().email();
        writer
            .write_all(&[u8::try_from(committer_email.len())
                .map_err(|_| VctrlError::SerializationError("committer email too long".into()))?])
            .map_err(io_err)?;
        writer
            .write_all(committer_email.as_bytes())
            .map_err(io_err)?;

        let msg = commit.message();
        let msg_len = u32::try_from(msg.len())
            .map_err(|_| VctrlError::SerializationError("message too long".into()))?;
        if msg_len as usize > usize::try_from(MAX_MESSAGE_LENGTH).unwrap_or(usize::MAX) {
            return Err(VctrlError::SerializationError(
                "commit message exceeds size limit".into(),
            ));
        }
        writer.write_all(&msg_len.to_le_bytes()).map_err(io_err)?;
        writer.write_all(msg.as_bytes()).map_err(io_err)?;

        writer
            .write_all(&commit.meta().timestamp().to_le_bytes())
            .map_err(io_err)?;
        writer
            .write_all(&commit.meta().timezone_offset().to_le_bytes())
            .map_err(io_err)?;

        match commit.meta().encoding() {
            Some(enc) => {
                let len = u8::try_from(enc.len())
                    .map_err(|_| VctrlError::SerializationError("encoding too long".into()))?;
                writer.write_all(&[len]).map_err(io_err)?;
                writer.write_all(enc.as_bytes()).map_err(io_err)?;
            }
            None => writer.write_all(&[0u8]).map_err(io_err)?,
        }
        Ok(())
    }

    fn encode_tag<W: Write + Send>(&self, tag: &Tag, writer: &mut W) -> Result<(), VctrlError> {
        writer.write_all(&[VERSION]).map_err(io_err)?;

        let name = tag.name();
        let name_len = u8::try_from(name.len())
            .map_err(|_| VctrlError::SerializationError("tag name too long".into()))?;
        writer.write_all(&[name_len]).map_err(io_err)?;
        writer.write_all(name.as_bytes()).map_err(io_err)?;

        writer.write_all(tag.target().as_bytes()).map_err(io_err)?;

        match tag.tagger() {
            Some(tagger) => {
                writer.write_all(&[1u8]).map_err(io_err)?;

                let tagger_name = tagger.name();
                writer
                    .write_all(&[u8::try_from(tagger_name.len()).map_err(|_| {
                        VctrlError::SerializationError("tagger name too long".into())
                    })?])
                    .map_err(io_err)?;
                writer.write_all(tagger_name.as_bytes()).map_err(io_err)?;

                let tagger_email = tagger.email();
                writer
                    .write_all(&[u8::try_from(tagger_email.len()).map_err(|_| {
                        VctrlError::SerializationError("tagger email too long".into())
                    })?])
                    .map_err(io_err)?;
                writer.write_all(tagger_email.as_bytes()).map_err(io_err)?;
            }
            None => writer.write_all(&[0u8]).map_err(io_err)?,
        }

        let msg = tag.message();
        let msg_len = u32::try_from(msg.len())
            .map_err(|_| VctrlError::SerializationError("message too long".into()))?;
        if msg_len as usize > usize::try_from(MAX_MESSAGE_LENGTH).unwrap_or(usize::MAX) {
            return Err(VctrlError::SerializationError(
                "tag message exceeds size limit".into(),
            ));
        }
        writer.write_all(&msg_len.to_le_bytes()).map_err(io_err)?;
        writer.write_all(msg.as_bytes()).map_err(io_err)?;

        writer
            .write_all(&tag.meta().timestamp().to_le_bytes())
            .map_err(io_err)?;
        writer
            .write_all(&tag.meta().timezone_offset().to_le_bytes())
            .map_err(io_err)?;

        match tag.meta().encoding() {
            Some(enc) => {
                let len = u8::try_from(enc.len())
                    .map_err(|_| VctrlError::SerializationError("encoding too long".into()))?;
                writer.write_all(&[len]).map_err(io_err)?;
                writer.write_all(enc.as_bytes()).map_err(io_err)?;
            }
            None => writer.write_all(&[0u8]).map_err(io_err)?,
        }
        Ok(())
    }
}

fn io_err(e: std::io::Error) -> VctrlError {
    VctrlError::IoError(std::sync::Arc::new(e))
}
