use crate::handler::error::VctrlError;
use crate::handler::types::{Commit, Hash, Object, ObjectStore, RefStore, UserInfo};

pub fn create_commit(
    store: &mut dyn ObjectStore,
    tree_hash: Hash,
    parents: Vec<Hash>,
    author: UserInfo,
    committer: UserInfo,
    message: String,
) -> Result<Hash, VctrlError> {
    let commit = Commit::new(tree_hash, parents, author, committer, message, None)?;
    store.put(&Object::Commit(Box::new(commit)))
}

pub fn get_commit(store: &dyn ObjectStore, hash: &Hash) -> Result<Option<Commit>, VctrlError> {
    match store.get(hash)? {
        Some(Object::Commit(c)) => Ok(Some(*c)),
        _ => Ok(None),
    }
}

pub fn log(store: &dyn ObjectStore, ref_store: &dyn RefStore) -> Result<Vec<Commit>, VctrlError> {
    let head_hash = match ref_store.head()? {
        Some(h) => h,
        None => return Ok(Vec::new()),
    };

    let mut commits = Vec::with_capacity(16);
    let mut current = Some(head_hash);
    while let Some(hash) = current {
        if let Some(commit) = get_commit(store, &hash)? {
            current = commit.parents().first().copied();
            commits.push(commit);
        } else {
            break;
        }
    }
    Ok(commits)
}
