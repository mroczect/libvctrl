use crate::codec::BinaryEncoder;
use crate::codec::Encoder;
use crate::command::Command;
use crate::domain::commit::Commit;
use crate::domain::hash::Hash;
use crate::domain::object::Object;
use crate::error::VctrlError;
use crate::hashing::Hasher;
use crate::hashing::Sha512Hasher;
use crate::storage::traits::{ObjectStore, RefStore};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

pub struct VerifyCommit {
    pub commit_hash: Hash,
    pub verifying_key: VerifyingKey,
}

impl Command for VerifyCommit {
    type Output = bool;

    fn execute(
        &self,
        store: &mut dyn ObjectStore,
        _refs: &mut dyn RefStore,
    ) -> Result<bool, VctrlError> {
        let commit = match store.get(&self.commit_hash)? {
            Some(Object::Commit(c)) => *c,
            _ => return Err(VctrlError::NotFound("commit not found".into())),
        };

        let sig_bytes = match &commit.signature {
            Some(s) => s.clone(),
            None => return Ok(false),
        };

        let pre_sig_commit = Commit {
            tree: commit.tree,
            parents: commit.parents.clone(),
            author: commit.author.clone(),
            committer: commit.committer.clone(),
            timestamp: commit.timestamp,
            message: commit.message.clone(),
            signature: None,
        };

        let encoder = BinaryEncoder;
        let hasher = Sha512Hasher;
        let mut buf = Vec::new();
        encoder.encode_commit(&pre_sig_commit, &mut buf);
        let pre_sig_hash = hasher.hash_commit_encoded(&buf);

        let signature = Signature::try_from(sig_bytes.as_slice())
            .map_err(|_| VctrlError::Other("invalid signature format".into()))?;

        self.verifying_key
            .verify(pre_sig_hash.as_bytes(), &signature)
            .map_err(|_| VctrlError::Other("signature verification failed".into()))?;

        Ok(true)
    }
}
