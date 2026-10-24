//!   header : b"VCTRLDB\0" | u32 version | u64 object_count
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use libvctrl_core as _;
use libvctrl_handler as _;
use libvctrl_sha512 as _;
use proptest as _;

use libvctrl::{
    BinaryDecoder, BinaryEncoder, Blob, Commit, CommitMeta, Decoder, Encoder, EntryKind, Hash,
    Hasher, Sha512Hasher, Tree, TreeEntry, UserID, VctrlError,
};

const REPO_DIR: &str = "single-file-repo";
const DB_FILE: &str = "single-file-repo/.vctrl/index.db";
const HEAD_FILE: &str = "single-file-repo/.vctrl/HEAD";
const MAIN_REF: &str = "single-file-repo/.vctrl/refs/heads/main";

const MAGIC: &[u8] = b"VCTRLDB\0";
const VERSION: u32 = 1;
const HASH_LEN: usize = 64;

fn main() -> Result<(), VctrlError> {
    prepare_layout()?;

    let (blob_hash, blob_bytes) = encode_blob(b"hello, vcs!\n")?;
    let (tree_hash, tree_bytes) = encode_tree("hello.txt", blob_hash)?;
    let (commit_hash, commit_bytes) = encode_commit(tree_hash, "first commit")?;

    let objects = vec![
        (blob_hash, blob_bytes),
        (tree_hash, tree_bytes),
        (commit_hash, commit_bytes),
    ];
    write_db(Path::new(DB_FILE), &objects)?;
    fs::write(MAIN_REF, format!("{commit_hash}\n"))?;

    let size = fs::metadata(DB_FILE)?.len();
    println!("wrote {DB_FILE}");
    println!("  {size} bytes, {} objects\n", objects.len());

    let db = Db::open(Path::new(DB_FILE))?;
    println!("reopened {} objects\n", db.len());

    let head: Hash = fs::read_to_string(MAIN_REF)?.trim().parse()?;
    let commit = BinaryDecoder.decode_commit(db.get(&head)?)?;
    print_commit(&commit, &head);

    let tree = BinaryDecoder.decode_tree(db.get(commit.tree())?)?;
    print_tree(&tree, commit.tree());

    Ok(())
}

fn prepare_layout() -> Result<(), VctrlError> {
    let root = Path::new(REPO_DIR);
    if root.exists() {
        fs::remove_dir_all(root)?;
    }
    fs::create_dir_all(root.join(".vctrl/refs/heads"))?;
    fs::write(HEAD_FILE, "ref: refs/heads/main\n")?;
    Ok(())
}

fn encode_blob(data: &[u8]) -> Result<(Hash, Vec<u8>), VctrlError> {
    let blob = Blob::new(data.to_vec())?;
    let mut bytes = Vec::new();
    BinaryEncoder.encode_blob(&blob, &mut bytes)?;
    let hash = Sha512Hasher.hash(bytes.as_slice())?;
    Ok((hash, bytes))
}

fn encode_tree(name: &str, blob: Hash) -> Result<(Hash, Vec<u8>), VctrlError> {
    let entry = TreeEntry::new(name.into(), EntryKind::Blob, blob)?;
    let tree = Tree::new(vec![entry])?;
    let mut bytes = Vec::new();
    BinaryEncoder.encode_tree(&tree, &mut bytes)?;
    let hash = Sha512Hasher.hash(bytes.as_slice())?;
    Ok((hash, bytes))
}

fn encode_commit(tree: Hash, message: &str) -> Result<(Hash, Vec<u8>), VctrlError> {
    let user = UserID::new("Alice".into(), "alice@example.com".into())?;
    let meta = CommitMeta::new(1_700_000_000, 0, None)?;
    let commit = Commit::with_meta(
        tree,
        vec![],
        user.clone(),
        user,
        format!("{message}\n"),
        meta,
    )?;
    let mut bytes = Vec::new();
    BinaryEncoder.encode_commit(&commit, &mut bytes)?;
    let hash = Sha512Hasher.hash(bytes.as_slice())?;
    Ok((hash, bytes))
}

struct Db {
    objects: HashMap<Hash, Vec<u8>>,
}

impl Db {
    fn open(path: &Path) -> Result<Self, VctrlError> {
        let data = fs::read(path)?;
        let mut cur = 0usize;

        expect_magic(&data, &mut cur)?;
        expect_version(&data, &mut cur)?;
        let count = usize::try_from(read_u64(&data, &mut cur)?)
            .map_err(|e| VctrlError::CorruptedData(format!("count too large: {e}")))?;

        let mut objects = HashMap::with_capacity(count);
        for _ in 0..count {
            let hash = Hash::from_bytes(read_slice(&data, &mut cur, HASH_LEN)?)?;
            let len = usize::try_from(read_u64(&data, &mut cur)?)
                .map_err(|e| VctrlError::CorruptedData(format!("length too large: {e}")))?;
            let payload = read_slice(&data, &mut cur, len)?.to_vec();
            let _ = objects.insert(hash, payload);
        }

        if cur != data.len() {
            return Err(VctrlError::CorruptedData(format!(
                "trailing {} bytes in db",
                data.len().saturating_sub(cur)
            )));
        }
        Ok(Self { objects })
    }

    fn get(&self, hash: &Hash) -> Result<&[u8], VctrlError> {
        self.objects
            .get(hash)
            .map(Vec::as_slice)
            .ok_or(VctrlError::ObjectNotFound(*hash))
    }

    fn len(&self) -> usize {
        self.objects.len()
    }
}

fn write_db(path: &Path, objects: &[(Hash, Vec<u8>)]) -> Result<(), VctrlError> {
    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&VERSION.to_le_bytes());
    out.extend_from_slice(&(objects.len() as u64).to_le_bytes());
    for (hash, bytes) in objects {
        out.extend_from_slice(hash.as_bytes());
        out.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
        out.extend_from_slice(bytes);
    }
    fs::write(path, out)?;
    Ok(())
}

fn expect_magic(data: &[u8], cur: &mut usize) -> Result<(), VctrlError> {
    let got = read_slice(data, cur, MAGIC.len())?;
    if got != MAGIC {
        return Err(VctrlError::CorruptedData("bad magic".into()));
    }
    Ok(())
}

fn expect_version(data: &[u8], cur: &mut usize) -> Result<(), VctrlError> {
    let bytes: [u8; 4] = read_slice(data, cur, 4)?
        .try_into()
        .map_err(|e| VctrlError::CorruptedData(format!("version: {e}")))?;
    let v = u32::from_le_bytes(bytes);
    if v != VERSION {
        return Err(VctrlError::CorruptedData(format!(
            "unsupported version: {v} (expected {VERSION})"
        )));
    }
    Ok(())
}

fn read_slice<'a>(data: &'a [u8], cur: &mut usize, n: usize) -> Result<&'a [u8], VctrlError> {
    let end = cur
        .checked_add(n)
        .ok_or_else(|| VctrlError::CorruptedData("cursor overflow".into()))?;
    let slice = data
        .get(*cur..end)
        .ok_or_else(|| VctrlError::CorruptedData(format!("truncated at offset {cur}")))?;
    *cur = end;
    Ok(slice)
}

fn read_u64(data: &[u8], cur: &mut usize) -> Result<u64, VctrlError> {
    let bytes: [u8; 8] = read_slice(data, cur, 8)?
        .try_into()
        .map_err(|e| VctrlError::CorruptedData(format!("u64: {e}")))?;
    Ok(u64::from_le_bytes(bytes))
}

fn print_commit(commit: &Commit, hash: &Hash) {
    println!("commit {hash}");
    println!(
        "Author: {} <{}>",
        commit.author().name(),
        commit.author().email()
    );
    println!("Date:   {}", commit.meta().timestamp());
    println!();
    for line in commit.message().lines() {
        println!("    {line}");
    }
}

fn print_tree(tree: &Tree, hash: &Hash) {
    println!("tree {hash}");
    for entry in tree.entries() {
        println!(
            "  {:06o} {} {}",
            entry.kind().mode(),
            entry.hash(),
            entry.name()
        );
    }
}
