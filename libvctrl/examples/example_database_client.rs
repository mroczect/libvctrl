//! Simulates a client fetching objects from a remote server via the Transport trait.

use libvctrl::{
    BinaryDecoder, BinaryEncoder, Commit, Decoder, Encoder, EntryKind, Hasher, Sha512Hasher,
    Transport, Tree, TreeEntry, UserID, VctrlError,
};
use std::collections::HashMap;

// A simple server that holds objects in memory
struct Server {
    objects: HashMap<libvctrl::Hash, Vec<u8>>,
}

impl Server {
    fn new() -> Self {
        Self {
            objects: HashMap::new(),
        }
    }

    fn store(&mut self, hash: &libvctrl::Hash, data: &[u8]) {
        self.objects.insert(*hash, data.to_vec());
    }
}

// Transport implementation that "connects" to the server
struct ClientTransport<'a> {
    server: &'a Server,
}

impl Transport for ClientTransport<'_> {
    fn fetch_object(
        &self,
        hash: &libvctrl::Hash,
    ) -> Result<Box<dyn std::io::Read + Send + '_>, VctrlError> {
        self.server
            .objects
            .get(hash)
            .map(|v| {
                Box::new(std::io::Cursor::new(v.clone())) as Box<dyn std::io::Read + Send + '_>
            })
            .ok_or(VctrlError::ObjectNotFound(*hash))
    }

    fn push_object(&mut self, _hash: &libvctrl::Hash, _data: &[u8]) -> Result<(), VctrlError> {
        Ok(())
    }
}

fn main() -> Result<(), VctrlError> {
    // Setup: build a server with some objects
    let mut server = Server::new();
    let encoder = BinaryEncoder;
    let hasher = Sha512Hasher;
    let alice = UserID::new("Alice".into(), "alice@example.com".into())?;

    let blob = libvctrl::Blob::new(b"Hello, Server!".to_vec())?;
    let mut encoded_blob = Vec::new();
    encoder.encode_blob(&blob, &mut encoded_blob)?;
    let blob_hash = hasher.hash(&encoded_blob[..])?;
    server.store(&blob_hash, &encoded_blob);

    let entry = TreeEntry::new("hello.txt".into(), EntryKind::Blob, blob_hash)?;
    let tree = Tree::new(vec![entry])?;
    let mut encoded_tree = Vec::new();
    encoder.encode_tree(&tree, &mut encoded_tree)?;
    let tree_hash = hasher.hash(&encoded_tree[..])?;
    server.store(&tree_hash, &encoded_tree);

    let commit = Commit::new(
        tree_hash,
        vec![],
        alice.clone(),
        alice,
        "Server commit".into(),
    )?;
    let mut encoded_commit = Vec::new();
    encoder.encode_commit(&commit, &mut encoded_commit)?;
    let commit_hash = hasher.hash(&encoded_commit[..])?;
    server.store(&commit_hash, &encoded_commit);

    // Client connects to server
    let transport = ClientTransport { server: &server };
    let decoder = BinaryDecoder;

    println!("=== Fetching commit from server ===");
    let mut encoded_commit = Vec::new();
    transport
        .fetch_object(&commit_hash)?
        .read_to_end(&mut encoded_commit)
        .map_err(|e| VctrlError::IoError(std::sync::Arc::new(e)))?;
    let commit = decoder.decode_commit(&encoded_commit[..])?;
    println!("Commit message: {}", commit.message());

    println!("\n=== Fetching tree from server ===");
    let mut encoded_tree = Vec::new();
    transport
        .fetch_object(commit.tree())?
        .read_to_end(&mut encoded_tree)
        .map_err(|e| VctrlError::IoError(std::sync::Arc::new(e)))?;
    let tree = decoder.decode_tree(&encoded_tree[..])?;
    for entry in tree.entries() {
        println!("  {:?} {}", entry.kind(), entry.name());
        if entry.kind() == EntryKind::Blob {
            let mut encoded_blob = Vec::new();
            transport
                .fetch_object(entry.hash())?
                .read_to_end(&mut encoded_blob)
                .map_err(|e| VctrlError::IoError(std::sync::Arc::new(e)))?;
            let blob = decoder.decode_blob(&encoded_blob[..])?;
            let content = String::from_utf8_lossy(blob.data());
            println!("    Content: {content}");
        }
    }

    Ok(())
}
