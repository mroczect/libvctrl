use crate::codec::Encoder;
use crate::domain::commit::Commit;
use crate::domain::tag::Tag;
use crate::domain::tree::{EntryKind, Tree};
use crate::domain::user::UserID;
use crate::error::VctrlError;

pub struct BinaryEncoder;

impl Encoder for BinaryEncoder {
    fn encode_tree(&self, tree: &Tree, buf: &mut Vec<u8>) -> Result<(), VctrlError> {
        buf.push(1u8);
        let n = u32::try_from(tree.entries().len())
            .map_err(|_| VctrlError::Other("tree has too many entries to encode".into()))?;
        buf.extend_from_slice(&n.to_be_bytes());
        for entry in tree.entries() {
            let name = entry.name.as_bytes();
            let name_len = u16::try_from(name.len()).map_err(|_| {
                VctrlError::Other(format!("entry name '{}' too long to encode", entry.name))
            })?;
            buf.extend_from_slice(&name_len.to_be_bytes());
            buf.extend_from_slice(name);
            match entry.kind {
                EntryKind::Blob => buf.push(0u8),
                EntryKind::Tree => buf.push(1u8),
            }
            buf.extend_from_slice(entry.hash.as_bytes());
        }
        Ok(())
    }

    fn encode_commit(&self, commit: &Commit, buf: &mut Vec<u8>) -> Result<(), VctrlError> {
        buf.push(1u8);
        buf.extend_from_slice(commit.tree.as_bytes());
        let np = u32::try_from(commit.parents.len())
            .map_err(|_| VctrlError::Other("commit has too many parents".into()))?;
        buf.extend_from_slice(&np.to_be_bytes());
        for p in &commit.parents {
            buf.extend_from_slice(p.as_bytes());
        }
        write_user(&commit.author, buf)?;
        write_user(&commit.committer, buf)?;
        let ts = commit.timestamp.timestamp();
        let ts_ns = commit.timestamp.timestamp_subsec_nanos();
        buf.extend_from_slice(&ts.to_be_bytes());
        buf.extend_from_slice(&ts_ns.to_be_bytes());
        let msg = commit.message.as_bytes();
        let msg_len = u32::try_from(msg.len())
            .map_err(|_| VctrlError::Other("commit message too long".into()))?;
        buf.extend_from_slice(&msg_len.to_be_bytes());
        buf.extend_from_slice(msg);
        if let Some(sig) = &commit.signature {
            let sig_len = u32::try_from(sig.len())
                .map_err(|_| VctrlError::Other("signature too long".into()))?;
            buf.extend_from_slice(&sig_len.to_be_bytes());
            buf.extend_from_slice(sig);
        } else {
            buf.extend_from_slice(&0u32.to_be_bytes());
        }
        Ok(())
    }

    fn encode_tag(&self, tag: &Tag, buf: &mut Vec<u8>) -> Result<(), VctrlError> {
        buf.push(1u8);
        buf.extend_from_slice(tag.target.as_bytes());
        write_user(&tag.tagger, buf)?;
        let ts = tag.timestamp.timestamp();
        let ts_ns = tag.timestamp.timestamp_subsec_nanos();
        buf.extend_from_slice(&ts.to_be_bytes());
        buf.extend_from_slice(&ts_ns.to_be_bytes());
        let msg = tag.message.as_bytes();
        let msg_len = u32::try_from(msg.len())
            .map_err(|_| VctrlError::Other("tag message too long".into()))?;
        buf.extend_from_slice(&msg_len.to_be_bytes());
        buf.extend_from_slice(msg);
        Ok(())
    }
}

fn write_user(user: &UserID, buf: &mut Vec<u8>) -> Result<(), VctrlError> {
    let name = user.name.as_bytes();
    let name_len = u16::try_from(name.len())
        .map_err(|_| VctrlError::Other(format!("name '{}' too long", user.name)))?;
    buf.extend_from_slice(&name_len.to_be_bytes());
    buf.extend_from_slice(name);

    let email = user.email.as_bytes();
    let email_len = u16::try_from(email.len())
        .map_err(|_| VctrlError::Other(format!("email '{}' too long", user.email)))?;
    buf.extend_from_slice(&email_len.to_be_bytes());
    buf.extend_from_slice(email);
    Ok(())
}
