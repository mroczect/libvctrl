use crate::codec::Encoder;
use crate::domain::commit::Commit;
use crate::domain::tree::{EntryKind, Tree};
use crate::domain::user::UserInfo;

pub struct BinaryEncoder;

impl Encoder for BinaryEncoder {
    fn encode_tree(&self, tree: &Tree, buf: &mut Vec<u8>) {
        buf.push(1u8); // version
        let n = tree.entries().len() as u32;
        buf.extend_from_slice(&n.to_be_bytes());
        for entry in tree.entries() {
            let name = entry.name.as_bytes();
            let name_len = name.len() as u16;
            buf.extend_from_slice(&name_len.to_be_bytes());
            buf.extend_from_slice(name);
            match entry.kind {
                EntryKind::Blob => buf.push(0u8),
                EntryKind::Tree => buf.push(1u8),
            }
            buf.extend_from_slice(entry.hash.as_bytes());
        }
    }

    fn encode_commit(&self, commit: &Commit, buf: &mut Vec<u8>) {
        buf.push(1u8);
        buf.extend_from_slice(commit.tree.as_bytes());
        let np = commit.parents.len() as u32;
        buf.extend_from_slice(&np.to_be_bytes());
        for p in &commit.parents {
            buf.extend_from_slice(p.as_bytes());
        }
        write_user(&commit.author, buf);
        write_user(&commit.committer, buf);
        let ts = commit.timestamp.timestamp();
        let ts_ns = commit.timestamp.timestamp_subsec_nanos();
        buf.extend_from_slice(&ts.to_be_bytes());
        buf.extend_from_slice(&ts_ns.to_be_bytes());
        let msg = commit.message.as_bytes();
        let msg_len = msg.len() as u32;
        buf.extend_from_slice(&msg_len.to_be_bytes());
        buf.extend_from_slice(msg);
        if let Some(sig) = &commit.signature {
            let sig_len = sig.len() as u32;
            buf.extend_from_slice(&sig_len.to_be_bytes());
            buf.extend_from_slice(sig);
        } else {
            buf.extend_from_slice(&0u32.to_be_bytes());
        }
    }
}

fn write_user(user: &UserInfo, buf: &mut Vec<u8>) {
    let name = user.name.as_bytes();
    let name_len = name.len() as u16;
    buf.extend_from_slice(&name_len.to_be_bytes());
    buf.extend_from_slice(name);
    let email = user.email.as_bytes();
    let email_len = email.len() as u16;
    buf.extend_from_slice(&email_len.to_be_bytes());
    buf.extend_from_slice(email);
}
