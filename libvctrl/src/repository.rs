//! High-level façade for embedding libvctrl in an application.
//!
//! ```ignore
//! let mut repo = Repository::new(
//!     Box::new(MemoryStore::new()),
//!     Box::new(MemoryRefStore::new()),
//! );
//! let commit = repo.commit(
//!     "refs/heads/main",
//!     [FileToCommit::blob("notes.md", b"hello\n".to_vec())],
//!     "first save",
//!     UserID::new("Alice".into(), "alice@example.com".into())?,
//! )?;
//! ``

use crate::{
    BinaryDecoder, BinaryEncoder, Blob, Commit, CommitMeta, Decoder, Encoder, EntryKind, Hash,
    Hasher, ObjectStore, RefStore, Sha512Hasher, Tree, TreeEntry, UserID, VctrlError,
};

#[derive(Debug)]
pub struct FileToCommit {
    pub path: String,
    pub kind: EntryKind,
    pub data: Vec<u8>,
}

impl FileToCommit {
    pub fn blob(path: impl Into<String>, data: impl Into<Vec<u8>>) -> Self {
        Self {
            path: path.into(),
            kind: EntryKind::Blob,
            data: data.into(),
        }
    }
    pub fn executable(path: impl Into<String>, data: impl Into<Vec<u8>>) -> Self {
        Self {
            path: path.into(),
            kind: EntryKind::Executable,
            data: data.into(),
        }
    }
    pub fn symlink(path: impl Into<String>, target: impl Into<Vec<u8>>) -> Self {
        Self {
            path: path.into(),
            kind: EntryKind::Symlink,
            data: target.into(),
        }
    }
}

pub struct Repository {
    objects: Box<dyn ObjectStore>,
    refs: Box<dyn RefStore>,
}

impl core::fmt::Debug for Repository {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Repository").finish_non_exhaustive()
    }
}

impl Repository {
    #[must_use]
    pub fn new(objects: Box<dyn ObjectStore>, refs: Box<dyn RefStore>) -> Self {
        Self { objects, refs }
    }

    pub fn commit(
        &mut self,
        branch: &str,
        files: impl IntoIterator<Item = FileToCommit>,
        message: impl Into<String>,
        author: UserID,
    ) -> Result<Hash, VctrlError> {
        let message = message.into();

        let mut entries = Vec::new();
        for f in files {
            let blob = Blob::new(f.data)?;
            let mut buf = Vec::new();
            BinaryEncoder.encode_blob(&blob, &mut buf)?;
            let hash = Sha512Hasher.hash(buf.as_slice())?;
            self.objects.put(&hash, &buf)?;
            entries.push(TreeEntry::new(f.path, f.kind, hash)?);
        }

        let tree = Tree::new(entries)?;
        let mut buf = Vec::new();
        BinaryEncoder.encode_tree(&tree, &mut buf)?;
        let tree_hash = Sha512Hasher.hash(buf.as_slice())?;
        self.objects.put(&tree_hash, &buf)?;

        let parent = self.refs.get_ref(branch).ok();
        let meta = CommitMeta::new(now_unix(), 0, None)?;
        let commit = Commit::with_meta(
            tree_hash,
            parent.into_iter().collect(),
            author.clone(),
            author,
            message,
            meta,
        )?;
        let mut buf = Vec::new();
        BinaryEncoder.encode_commit(&commit, &mut buf)?;
        let commit_hash = Sha512Hasher.hash(buf.as_slice())?;
        self.objects.put(&commit_hash, &buf)?;

        self.refs.set_ref(branch, &commit_hash)?;
        Ok(commit_hash)
    }

    pub fn snapshot(
        &mut self,
        files: impl IntoIterator<Item = FileToCommit>,
    ) -> Result<Hash, VctrlError> {
        let mut entries = Vec::new();
        for f in files {
            let blob = Blob::new(f.data)?;
            let mut buf = Vec::new();
            BinaryEncoder.encode_blob(&blob, &mut buf)?;
            let hash = Sha512Hasher.hash(buf.as_slice())?;
            self.objects.put(&hash, &buf)?;
            entries.push(TreeEntry::new(f.path, f.kind, hash)?);
        }
        let tree = Tree::new(entries)?;
        let mut buf = Vec::new();
        BinaryEncoder.encode_tree(&tree, &mut buf)?;
        let tree_hash = Sha512Hasher.hash(buf.as_slice())?;
        self.objects.put(&tree_hash, &buf)?;
        Ok(tree_hash)
    }

    pub fn read_blob(&self, hash: &Hash) -> Result<Blob, VctrlError> {
        BinaryDecoder.decode_blob(self.objects.get(hash)?)
    }

    pub fn read_tree(&self, hash: &Hash) -> Result<Tree, VctrlError> {
        BinaryDecoder.decode_tree(self.objects.get(hash)?)
    }

    pub fn read_commit(&self, hash: &Hash) -> Result<Commit, VctrlError> {
        BinaryDecoder.decode_commit(self.objects.get(hash)?)
    }

    pub fn head(&self, branch: &str) -> Result<Hash, VctrlError> {
        self.refs.get_ref(branch)
    }

    pub fn branches(&self) -> Result<Vec<String>, VctrlError> {
        self.refs.list_refs()
    }

    pub fn log(&self, branch: &str) -> Result<Vec<(Hash, Commit)>, VctrlError> {
        let mut out = Vec::new();
        let mut cursor = match self.refs.get_ref(branch) {
            Ok(h) => Some(h),
            Err(VctrlError::RefNotFound(_)) => None,
            Err(e) => return Err(e),
        };
        let mut guard = 0usize;
        while let Some(hash) = cursor {
            guard = guard.saturating_add(1);
            if guard > 100_000 {
                return Err(VctrlError::Other("log walk exceeded 100k commits".into()));
            }
            let commit = self.read_commit(&hash)?;
            cursor = commit.parents().first().copied();
            out.push((hash, commit));
        }
        Ok(out)
    }

    pub fn file_at(&self, commit: &Hash, path: &str) -> Result<Option<Vec<u8>>, VctrlError> {
        let commit = self.read_commit(commit)?;
        let tree = self.read_tree(commit.tree())?;
        match tree.get(path) {
            Some(entry) => Ok(Some(self.read_blob(entry.hash())?.data().to_vec())),
            None => Ok(None),
        }
    }

    pub fn create_branch(&mut self, branch: &str, from: &Hash) -> Result<(), VctrlError> {
        self.refs.set_ref(branch, from)
    }

    pub fn delete_branch(&mut self, branch: &str) -> Result<(), VctrlError> {
        self.refs.delete_ref(branch)
    }
}

fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(0))
}
