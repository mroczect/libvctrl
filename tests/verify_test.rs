mod common;
use common::*;
use ed25519_dalek::{Signer as DalekSigner, SigningKey};
use libvctrl::*;

fn test_signing_key() -> SigningKey {
    SigningKey::from_bytes(&[0u8; 32])
}

struct Ed25519Signer(SigningKey);

impl Signer for Ed25519Signer {
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>, VctrlError> {
        let signature = self.0.sign(data);
        Ok(signature.to_bytes().to_vec())
    }
}

impl Verifier for Ed25519Signer {
    fn verify(&self, data: &[u8], sig: &[u8]) -> Result<bool, VctrlError> {
        use ed25519_dalek::Verifier;
        let sig = ed25519_dalek::Signature::try_from(sig)
            .map_err(|_| VctrlError::Other("bad sig".into()))?;
        Ok(self.0.verifying_key().verify(data, &sig).is_ok())
    }
}

#[test]
fn signed_commit_verification() {
    let mut store = MemoryStore::new();
    let mut refs = MemoryRefStore::new();
    let key = test_signing_key();
    let signer = Ed25519Signer(key.clone());

    let tree_hash = put_tree(&mut store, &Tree::new(vec![]).unwrap());
    let mut commit = Commit::new(tree_hash, vec![], alice(), alice(), "signed".into(), None);
    let pre_hash = {
        let mut buf = Vec::new();
        encoder()
            .encode_commit(
                &Commit {
                    signature: None,
                    ..commit.clone()
                },
                &mut buf,
            )
            .unwrap();
        hasher().hash_commit_encoded(&buf)
    };
    commit.signature = Some(signer.sign(pre_hash.as_bytes()).unwrap());
    let commit_hash = commit_hash(&commit);
    store
        .put(&commit_hash, &Object::Commit(Box::new(commit)))
        .unwrap();

    let verify = VerifyCommit {
        commit_hash,
        verifier: Box::new(signer),
        encoder: Box::new(encoder()),
        hasher: Box::new(hasher()),
    };
    assert!(verify.execute(&mut store, &mut refs).unwrap());
}

#[test]
fn signed_tag_verification() {
    let mut store = MemoryStore::new();
    let mut refs = MemoryRefStore::new();
    let key = test_signing_key();
    let signer = Ed25519Signer(key.clone());

    let target = blob_hash(b"target");
    let mut tag = Tag::new(target, alice(), "signed".into());
    let pre_hash = {
        let mut buf = Vec::new();
        encoder()
            .encode_tag(
                &Tag {
                    signature: None,
                    ..tag.clone()
                },
                &mut buf,
            )
            .unwrap();
        hasher().hash_tag_encoded(&buf)
    };
    tag.signature = Some(signer.sign(pre_hash.as_bytes()).unwrap());
    let tag_hash = {
        let mut buf = Vec::new();
        encoder().encode_tag(&tag, &mut buf).unwrap();
        hasher().hash_tag_encoded(&buf)
    };
    store.put(&tag_hash, &Object::Tag(Box::new(tag))).unwrap();

    let verify = VerifyTag {
        tag_hash,
        verifier: Box::new(signer),
        encoder: Box::new(encoder()),
        hasher: Box::new(hasher()),
    };
    assert!(verify.execute(&mut store, &mut refs).unwrap());
}
