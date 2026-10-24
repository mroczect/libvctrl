use std::fs;
use std::path::{Path, PathBuf};

use libvctrl_core as _;
use libvctrl_handler as _;
use libvctrl_sha512 as _;
use proptest as _;

use libvctrl::{
    BinaryDecoder, BinaryEncoder, Blob, Commit, CommitMeta, Decoder, Encoder, EntryKind, Hash,
    Hasher, Sha512Hasher, Tree, TreeEntry, UserID, VctrlError,
};

const REPO_ROOT: &str = "walkthrough-repo";

fn main() -> Result<(), VctrlError> {
    let root = Path::new(REPO_ROOT);
    let vctrl = root.join(".vctrl");

    if root.exists() {
        fs::remove_dir_all(root)?;
    }
    fs::create_dir_all(vctrl.join("objects"))?;
    fs::create_dir_all(vctrl.join("refs/heads"))?;
    fs::write(vctrl.join("HEAD"), "ref: refs/heads/main\n")?;

    let blob = write_blob(&vctrl, b"hello, vcs!\n")?;
    let tree = write_tree(&vctrl, "hello.txt", blob)?;
    let commit = write_commit(&vctrl, tree, "first commit")?;
    fs::write(vctrl.join("refs/heads/main"), format!("{commit}\n"))?;

    println!("wrote repo under {}\n", root.display());
    list_tree(root, 0)?;
    println!();

    let head = read_ref(&vctrl, "refs/heads/main")?;
    let decoded_commit = read_commit(&vctrl, &head)?;
    let decoded_tree = read_tree(&vctrl, decoded_commit.tree())?;

    print_commit(&decoded_commit, &head);
    print_tree(&decoded_tree, decoded_commit.tree());

    Ok(())
}

fn object_path(vctrl: &Path, hash: &Hash) -> PathBuf {
    let hex = hash.to_string();
    let (dir, file) = hex.split_at(2);
    vctrl.join("objects").join(dir).join(file)
}

fn write_object(vctrl: &Path, hash: &Hash, bytes: &[u8]) -> Result<(), VctrlError> {
    let path = object_path(vctrl, hash);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, bytes)?;
    Ok(())
}

fn read_object(vctrl: &Path, hash: &Hash) -> Result<Vec<u8>, VctrlError> {
    Ok(fs::read(object_path(vctrl, hash))?)
}

fn read_ref(vctrl: &Path, name: &str) -> Result<Hash, VctrlError> {
    let raw = fs::read_to_string(vctrl.join(name))?;
    raw.trim().parse()
}

fn write_blob(vctrl: &Path, data: &[u8]) -> Result<Hash, VctrlError> {
    let blob = Blob::new(data.to_vec())?;
    let mut bytes = Vec::new();
    BinaryEncoder.encode_blob(&blob, &mut bytes)?;
    let hash = Sha512Hasher.hash(bytes.as_slice())?;
    write_object(vctrl, &hash, &bytes)?;
    Ok(hash)
}

fn write_tree(vctrl: &Path, name: &str, blob: Hash) -> Result<Hash, VctrlError> {
    let entry = TreeEntry::new(name.into(), EntryKind::Blob, blob)?;
    let tree = Tree::new(vec![entry])?;
    let mut bytes = Vec::new();
    BinaryEncoder.encode_tree(&tree, &mut bytes)?;
    let hash = Sha512Hasher.hash(bytes.as_slice())?;
    write_object(vctrl, &hash, &bytes)?;
    Ok(hash)
}

fn write_commit(vctrl: &Path, tree: Hash, message: &str) -> Result<Hash, VctrlError> {
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
    write_object(vctrl, &hash, &bytes)?;
    Ok(hash)
}

fn read_tree(vctrl: &Path, hash: &Hash) -> Result<Tree, VctrlError> {
    let bytes = read_object(vctrl, hash)?;
    BinaryDecoder.decode_tree(bytes.as_slice())
}

fn read_commit(vctrl: &Path, hash: &Hash) -> Result<Commit, VctrlError> {
    let bytes = read_object(vctrl, hash)?;
    BinaryDecoder.decode_commit(bytes.as_slice())
}

fn list_tree(dir: &Path, indent: usize) -> Result<(), VctrlError> {
    let mut entries: Vec<_> = fs::read_dir(dir)?.filter_map(Result::ok).collect();
    entries.sort_by_key(fs::DirEntry::path);
    for entry in entries {
        let path = entry.path();
        println!(
            "{:indent$}{}",
            "",
            entry.file_name().to_string_lossy(),
            indent = indent,
        );
        if path.is_dir() {
            list_tree(&path, indent.saturating_add(2))?;
        }
    }
    Ok(())
}

fn print_commit(commit: &Commit, hash: &Hash) {
    println!("commit {hash}");
    println!(
        "Author: {} <{}>",
        commit.author().name(),
        commit.author().email(),
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
            entry.name(),
        );
    }
}
