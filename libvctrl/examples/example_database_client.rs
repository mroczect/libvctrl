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
    fn fetch_object(&self, hash: &libvctrl::Hash) -> Result<Vec<u8>, VctrlError> {
        self.server
            .objects
            .get(hash)
            .cloned()
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

    let blob = libvctrl::Blob::new(b"Hello, Server!".to_vec());
    let encoded_blob = encoder.encode_blob(&blob)?;
    let blob_hash = hasher.hash(&encoded_blob);
    server.store(&blob_hash, &encoded_blob);

    let entry = TreeEntry::new("hello.txt".into(), EntryKind::Blob, blob_hash)?;
    let tree = Tree::new(vec![entry])?;
    let encoded_tree = encoder.encode_tree(&tree)?;
    let tree_hash = hasher.hash(&encoded_tree);
    server.store(&tree_hash, &encoded_tree);

    let commit = Commit::new(
        tree_hash,
        vec![],
        alice.clone(),
        alice,
        "Server commit".into(),
    );
    let encoded_commit = encoder.encode_commit(&commit)?;
    let commit_hash = hasher.hash(&encoded_commit);
    server.store(&commit_hash, &encoded_commit);

    // Client connects to server
    let transport = ClientTransport { server: &server };
    let decoder = BinaryDecoder;

    println!("=== Fetching commit from server ===");
    let encoded_commit = transport.fetch_object(&commit_hash)?;
    let commit = decoder.decode_commit(&encoded_commit)?;
    println!("Commit message: {}", commit.message());

    println!("\n=== Fetching tree from server ===");
    let encoded_tree = transport.fetch_object(commit.tree())?;
    let tree = decoder.decode_tree(&encoded_tree)?;
    for entry in tree.entries() {
        println!("  {:?} {}", entry.kind(), entry.name());
        if entry.kind() == EntryKind::Blob {
            let encoded_blob = transport.fetch_object(entry.hash())?;
            let blob = decoder.decode_blob(&encoded_blob)?;
            let content = String::from_utf8_lossy(blob.data());
            println!("    Content: {content}");
        }
    }

    Ok(())
}
