use crate::codec::Encoder;
use crate::command::Command;
use crate::crypto::Signer;
use crate::domain::commit::Commit;
use crate::domain::hash::Hash;
use crate::domain::object::Object;
use crate::domain::user::UserID;
use crate::error::VctrlError;
use crate::hashing::Hasher;
use crate::storage::traits::{ObjectStore, RefStore};

pub struct CreateCommit {
    pub tree_hash: Hash,
    pub parents: Vec<Hash>,
    pub author: UserID,
    pub committer: UserID,
    pub message: String,
    pub transaction_id: Option<String>,
    pub encoder: Box<dyn Encoder>,
    pub hasher: Box<dyn Hasher>,
    pub signer: Option<Box<dyn Signer>>,
}

impl Command for CreateCommit {
    type Output = Hash;

    fn execute(
        &self,
        store: &mut dyn ObjectStore,
        refs: &mut dyn RefStore,
    ) -> Result<Hash, VctrlError> {
        let final_message = match &self.transaction_id {
            Some(id) if !id.is_empty() => format!("[tx-{}] {}", id, self.message),
            _ => self.message.clone(),
        };

        let mut commit = Commit::new(
            self.tree_hash,
            self.parents.clone(),
            self.author.clone(),
            self.committer.clone(),
            final_message,
            None,
        );

        let mut buf = Vec::new();
        self.encoder.encode_commit(&commit, &mut buf)?;
        let pre_sig_hash = self.hasher.hash_commit_encoded(&buf);

        if let Some(signer) = &self.signer {
            let sig = signer.sign(pre_sig_hash.as_bytes())?;
            commit.signature = Some(sig);

            buf.clear();
            self.encoder.encode_commit(&commit, &mut buf)?;
        }

        let final_hash = self.hasher.hash_commit_encoded(&buf);
        store.put(&final_hash, &Object::Commit(Box::new(commit)))?;

        if let Some(branch_name) = refs.head_ref_name()? {
            refs.set_ref(&branch_name, &final_hash)?;
        }

        Ok(final_hash)
    }
}
