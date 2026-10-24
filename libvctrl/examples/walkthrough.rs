use libvctrl_core as _;
use libvctrl_handler as _;
use libvctrl_sha512 as _;
use proptest as _;

use libvctrl::{
    BinaryDecoder, BinaryEncoder, Blob, Commit, CommitMeta, Decoder, Encoder, EntryKind, Hash,
    Hasher, MemoryRefStore, MemoryStore, ObjectStore, RefStore, Sha512Hasher, Tree, TreeEntry,
    UserID, VctrlError,
};

fn main() -> Result<(), VctrlError> {
    let mut objects = MemoryStore::new();
    let mut refs = MemoryRefStore::new();

    let blob = write_blob(&mut objects, b"hello, vcs?\n")?;
    let tree = write_tree(&mut objects, "hello.txt", blob)?;
    let commit = write_commit(&mut objects, tree, "first commit")?;

    refs.set_ref("refs/heads/main", &commit)?;

    print_tree(&objects, &tree)?;
    print_commit(&objects, &commit)?;
    println!("\nHEAD -> {}", refs.get_ref("refs/heads/main")?);

    Ok(())
}

fn write_blob(store: &mut MemoryStore, data: &[u8]) -> Result<Hash, VctrlError> {
    let blob = Blob::new(data.to_vec())?;
    let mut bytes = Vec::new();
    BinaryEncoder.encode_blob(&blob, &mut bytes)?;
    let hash = Sha512Hasher.hash(bytes.as_slice())?;
    store.put(&hash, &bytes)?;
    Ok(hash)
}

fn write_tree(store: &mut MemoryStore, name: &str, blob: Hash) -> Result<Hash, VctrlError> {
    let entry = TreeEntry::new(name.into(), EntryKind::Blob, blob)?;
    let tree = Tree::new(vec![entry])?;
    let mut bytes = Vec::new();
    BinaryEncoder.encode_tree(&tree, &mut bytes)?;
    let hash = Sha512Hasher.hash(bytes.as_slice())?;
    store.put(&hash, &bytes)?;
    Ok(hash)
}

fn write_commit(store: &mut MemoryStore, tree: Hash, message: &str) -> Result<Hash, VctrlError> {
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
    store.put(&hash, &bytes)?;
    Ok(hash)
}

fn print_tree(store: &MemoryStore, tree: &Hash) -> Result<(), VctrlError> {
    let decoded = BinaryDecoder.decode_tree(store.get(tree)?)?;
    println!("tree {tree}");
    for entry in decoded.entries() {
        println!(
            "  {:06o} {} {}",
            entry.kind().mode(),
            entry.hash(),
            entry.name(),
        );
    }
    Ok(())
}

fn print_commit(store: &MemoryStore, commit: &Hash) -> Result<(), VctrlError> {
    let decoded = BinaryDecoder.decode_commit(store.get(commit)?)?;
    println!("commit {commit}");
    println!(
        "Author: {} <{}>",
        decoded.author().name(),
        decoded.author().email(),
    );
    println!("Date:   {}", decoded.meta().timestamp());
    println!();
    for line in decoded.message().lines() {
        println!("    {line}");
    }
    Ok(())
}
