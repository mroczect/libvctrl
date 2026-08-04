# libvctrl

[![Crates.io](https://img.shields.io/crates/v/libvctrl)](https://crates.io/crates/libvctrl)
[![Downloads](https://img.shields.io/crates/d/libvctrl?label=downloads)](https://crates.io/crates/libvctrl)
[![License: MIT](https://img.shields.io/crates/l/libvctrl)](#license)
[![Docs](https://docs.rs/libvctrl/badge.svg)](https://docs.rs/libvctrl)
[![CI](https://img.shields.io/github/actions/workflow/status/mroczect/libvctrl/rust.yml?branch=master)](https://github.com/mroczect/libvctrl/actions)
[![MSRV](https://img.shields.io/badge/MSRV-1.85.0-blue)](#installation)
[![LoC](https://img.shields.io/tokei/lines/github/mroczect/libvctrl)](https://github.com/mroczect/libvctrl)
[![Last Commit](https://img.shields.io/github/last-commit/mroczect/libvctrl)](https://github.com/mroczect/libvctrl/commits/master)
[![Repo Size](https://img.shields.io/github/repo-size/mroczect/libvctrl)](https://github.com/mroczect/libvctrl)

A robust, content-addressed version control engine for arbitrary data, designed
for embedding into applications.

libvctrl provides the core data model, storage abstractions, hashing, encoding,
commands, diffing, three-way merging, and cryptographic signing needed to build
version control functionality directly into applications -- without shelling out
to an external VCS or depending on a CLI tool. It is a library only and does
not ship a binary.

---

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Installation](#installation)
- [Dependencies](#dependencies)
- [Quick Start](#quick-start)
- [Domain Model](#domain-model)
  - [Blob](#blob)
  - [Hash](#hash)
  - [Tree and TreeEntry](#tree-and-treeentry)
  - [Commit](#commit)
  - [UserID and UserInfo](#userid-and-userinfo)
  - [Object](#object)
- [Storage](#storage)
  - [ObjectStore Trait](#objectstore-trait)
  - [RefStore Trait](#refstore-trait)
  - [MemoryStore](#memorystore)
  - [MemoryRefStore](#memoryrefstore)
- [Hashing](#hashing-1)
  - [Hasher Trait](#hasher-trait)
  - [Sha512Hasher](#sha512hasher)
- [Encoding](#encoding)
  - [Encoder Trait](#encoder-trait)
  - [BinaryEncoder](#binaryencoder)
  - [Binary Format Specification](#binary-format-specification)
- [Commands](#commands)
  - [Command Trait](#command-trait)
  - [Branch Operations](#branch-operations)
  - [SetHead](#sethead)
  - [CreateCommit](#createcommit)
  - [Log](#log)
  - [Checkout](#checkout)
  - [MergeCommand](#mergecommand)
- [Diffing](#diffing)
  - [TreeDiff Trait](#treediff-trait)
  - [TreeDiffer](#treediffer)
  - [DiffKind and DiffEntry](#diffkind-and-diffentry)
- [Merging](#merging)
  - [ThreeWayMerge Trait](#threewaymerge-trait)
  - [ConflictResolver Trait](#conflictresolver-trait)
  - [ThreeWayMerger](#threewaymerger)
- [Cryptographic Signing](#cryptographic-signing)
  - [Signer Trait](#signer-trait)
  - [LibrageSigner](#libragesigner)
  - [Signing and Verification Workflow](#signing-and-verification-workflow)
- [Error Handling](#error-handling)
  - [VctrlError](#vctrlerror)
  - [HashError](#hasherror)
  - [TreeError](#treeerror)
- [Module Reference](#module-reference)
- [Testing](#testing)
- [Build and Lint](#build-and-lint)
- [Security Considerations](#security-considerations)
- [Limitations](#limitations)
- [Migration from v0.1.0](#migration-from-v010)
- [Roadmap](#roadmap)
- [License](#license)

---

## Overview

libvctrl implements a content-addressed version control engine similar in
principle to Git's object model, but with key differences:

- **SHA-512 hashes** instead of SHA-1, providing 256 bits of collision
  resistance.
- **Ed25519 signing** for commit integrity verification, using the
  `ed25519-dalek` crate.
- **Validated user identity** through the `UserID` type from the

# age-credentials crate, enforcing name and email format rules.

- **Trait-based abstractions** for storage, hashing, encoding, signing,
  diffing, and merging, allowing custom backends and algorithms without
  modifying the core.
- **Command pattern** for all operations, providing a uniform interface that
  accepts mutable references to an `ObjectStore` and a `RefStore`.
- **Embedded design** -- no CLI, no subprocess calls, no filesystem
  assumptions beyond what the storage backend requires. The provided
  `MemoryStore` and `MemoryRefStore` require no filesystem at all.

---

## Architecture

```
src/
  lib.rs              Crate root, re-exports all modules
  error.rs            VctrlError enum
  codec/              Encoding format
    mod.rs              Encoder trait and re-exports
    binary.rs           BinaryEncoder implementation
  command/            Command pattern operations
    mod.rs              Command trait and re-exports
    branch.rs           CreateBranch, DeleteBranch, GetBranch, SetHead
    checkout.rs         Checkout (recursive tree materialization)
    create_commit.rs    CreateCommit
    log.rs              Log (commit history traversal)
    merge.rs            MergeCommand
  crypto/             Cryptographic signing
    mod.rs              Signer trait and re-exports
    signer.rs           LibrageSigner (Ed25519)
  diff/               Tree diffing
    mod.rs              TreeDiff trait, DiffKind, DiffEntry
    tree_diff.rs        TreeDiffer implementation
  domain/             Core domain types
    mod.rs              Re-exports all domain types
    blob.rs             Blob (content-addressed data)
    commit.rs           Commit (snapshot record)
    hash.rs             Hash (64-byte SHA-512), HashError
    object.rs           Object enum (Blob, Tree, Commit)
    tree.rs             Tree, TreeEntry, EntryKind, TreeError
    user.rs             UserID (from age-credentials), UserInfo
  hashing/            Hashing trait and implementation
    mod.rs              Hasher trait and re-exports
    sha512.rs           Sha512Hasher
  merge/              Three-way merge
    mod.rs              ThreeWayMerge trait
    resolver.rs         ConflictResolver trait
    three_way.rs        ThreeWayMerger implementation
  storage/            Storage backends
    mod.rs              Re-exports
    traits.rs           ObjectStore, RefStore traits
    memory.rs           MemoryStore, MemoryRefStore
```

---

## Installation

There are several ways to add `libvctrl` to your Rust project:

### 1. Using `cargo add`

```sh
cargo add libvctrl
```

This command will automatically add the dependency line to `Cargo.toml`.

### 2. Adding Manually in `Cargo.toml`

Add the following line to the `[dependencies]` section:

```toml
[dependencies]
libvctrl = { git = "https://github.com/mroczect/libvctrl.git" }
```

### 3. Clone the repository and use it as a local dependency (path)

If you want to develop or modify the library alongside your project, clone the repository first:

```sh
git clone https://github.com/mroczect/libvctrl.git
```

Then, in your project's `Cargo.toml`, navigate to the cloned path:

```toml
[dependencies]
libvctrl = { path = "../libvctrl" } # adjust the directory location
```

With this method, changes you make to the library will be immediately reflected in the main project upon compilation.

### 4. Fork and use it as a Git dependency from your fork

You can fork the repository to your own GitHub account, then use it the same way as method 1 or 2, just replace the URL to your fork repository:

```toml
[dependencies]
libvctrl = { git = "https://github.com/your-username/libvctrl.git" }
```

### Toolchain Requirements

This library uses **Rust edition 2024**. Make sure your toolchain supports that edition (Rust **1.85.0** or later). To check the installed Rust version:

```sh
rustc --version
```

If your toolchain is older, update it with:

```sh
rustup update stable
```

If for some reason you need to use an older edition (e.g., 2021), you can change the `edition` line in `libvctrl`'s `Cargo.toml` from `"2024"` to `"2021"`. However, keep in mind that some syntactic features may not be available.

### Dependencies

`libvctrl` depends on the following crates (handled automatically by Cargo):

- `chrono` 0.4.45 (`serde` feature)
- `serde` 1.0.229 (`derive` feature)
- `serde_json` 1.0.151
- `sha2` 0.11.0
- `thiserror` 2.0.19

All public types and traits are exported directly in the crate root, so you can import them easily:

```rust
use libvctrl::{Blob, MemoryStore, Command, ...};
```

See the `tests/` directory in the repository for complete usage examples.

---

## Dependencies

| Crate           | Version | Purpose                                           |
| --------------- | ------- | ------------------------------------------------- |
| chrono          | 0.4.45  | Timestamps for commits (with serde feature)       |
| serde           | 1.0.229 | Serialization framework (with derive feature)     |
| serde_json      | 1.0.151 | JSON serialization                                |
| sha2            | 0.11.0  | SHA-512 digest computation                        |
| thiserror       | 2.0.19  | Error derive macro                                |
| ed25519-dalek   | 3.0.0   | Ed25519 signing and verification                  |
| rand            | 0.8.7   | CSPRNG for signing key generation                 |
| age-credentials | 0.3.0   | Validated UserID type                             |
| librage         | 1.1.0   | age encryption backend                            |
| age             | 0.12.1  | age library (transitive, used by age-credentials) |
| tempfile        | 3.27.0  | Atomic file writes (used by age-credentials)      |

---

## Quick Start

```rust
use libvctrl::{
    BinaryEncoder, Blob, Command, CreateBranch, CreateCommit, Hash,
    Hasher, LibrageSigner, MemoryRefStore, MemoryStore, Object,
    ObjectStore, RefStore, SetHead, Sha512Hasher, Signer, Tree,
    TreeEntry, EntryKind, UserID,
};

// Set up storage
let mut store = MemoryStore::new();
let mut refs = MemoryRefStore::new();

// Create a blob and store it
let blob = Blob::new(b"hello world".to_vec());
let hasher = Sha512Hasher;
let blob_hash = hasher.hash_blob(blob.as_bytes());
store.put(&blob_hash, &Object::Blob(blob)).unwrap();

// Create a tree with one entry
let entry = TreeEntry::new("hello.txt".into(), EntryKind::Blob, blob_hash);
let tree = Tree::new(vec![entry]).unwrap();
let encoder = BinaryEncoder;
let mut buf = Vec::new();
encoder.encode_tree(&tree, &mut buf);
let tree_hash = hasher.hash_tree_encoded(&buf);
store.put(&tree_hash, &Object::Tree(tree)).unwrap();

// Create a branch and set HEAD
let author = UserID::new("Alice Example", "alice@example.com").unwrap();
CreateBranch { name: "refs/heads/main".into(), hash: tree_hash }
    .execute(&mut store, &mut refs).unwrap();
SetHead { target: "refs/heads/main".into() }
    .execute(&mut store, &mut refs).unwrap();

// Create a commit
let commit_hash = CreateCommit {
    tree_hash,
    parents: vec![],
    author: author.clone(),
    committer: author,
    message: "initial commit".into(),
    encoder: Box::new(BinaryEncoder),
    hasher: Box::new(Sha512Hasher),
}.execute(&mut store, &mut refs).unwrap();

// Sign the commit
let signer = LibrageSigner::generate();
let signature = signer.sign(commit_hash.as_bytes()).unwrap();
let verifying_key = signer.verifying_key();
println!("Commit: {}", commit_hash);
println!("Signature length: {} bytes", signature.len());
```

---

## Domain Model

### Blob

```rust
pub struct Blob {
    data: Vec<u8>,  // private
}
```

A content-addressed data container. The inner `data` field is private to
enforce encapsulation.

| Method       | Signature                 | Description                        |
| ------------ | ------------------------- | ---------------------------------- |
| `new`        | `(data: Vec<u8>) -> Self` | Construct from raw bytes           |
| `as_bytes`   | `(&self) -> &[u8]`        | Borrow the inner bytes             |
| `into_bytes` | `(self) -> Vec<u8>`       | Consume and return the inner bytes |

Implements `Debug`, `Clone`, `Serialize`, `Deserialize`.

---

### Hash

```rust
pub struct Hash([u8; 64]);
```

A 64-byte (512-bit) SHA-512 hash value. `Copy` because 64 bytes fits on
the stack.

| Method       | Signature                            | Description                                       |
| ------------ | ------------------------------------ | ------------------------------------------------- |
| `from_bytes` | `(bytes: [u8; 64]) -> Self`          | Construct from a fixed-size array (const)         |
| `from_slice` | `(&[u8]) -> Result<Self, HashError>` | Construct from a slice; fails if length is not 64 |
| `from_hex`   | `(&str) -> Result<Self, HashError>`  | Construct from a 128-character hex string         |
| `as_bytes`   | `(&self) -> &[u8; 64]`               | Borrow the inner byte array                       |
| `to_hex`     | `(&self) -> String`                  | Produce a 128-character lowercase hex string      |

Implements `Debug`, `Display`, `FromStr`, `Serialize` (as hex),
`Deserialize` (from hex with validation), `Clone`, `Copy`, `PartialEq`,
`Eq`, `Hash`.

---

### Tree and TreeEntry

```rust
pub struct Tree { entries: Vec<TreeEntry> }  // private, sorted by name

pub struct TreeEntry {
    pub name: String,
    pub kind: EntryKind,
    pub hash: Hash,
}

pub enum EntryKind { Blob, Tree }
```

| Tree method    | Signature                                              | Description                                |
| -------------- | ------------------------------------------------------ | ------------------------------------------ |
| `new`          | `(entries: Vec<TreeEntry>) -> Result<Self, TreeError>` | Construct, sort by name, detect duplicates |
| `entries`      | `(&self) -> &[TreeEntry]`                              | Borrow the sorted entries                  |
| `into_entries` | `(self) -> Vec<TreeEntry>`                             | Consume and return the entries             |
| `is_empty`     | `(&self) -> bool`                                      | Check if there are no entries              |

Entries are sorted on construction, making tree hashes deterministic
regardless of input order.

---

### Commit

```rust
pub struct Commit {
    pub tree: Hash,
    pub parents: Vec<Hash>,
    pub author: UserID,
    pub committer: UserID,
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub signature: Option<Vec<u8>>,
}
```

| Field       | Type              | Description                                       |
| ----------- | ----------------- | ------------------------------------------------- |
| `tree`      | `Hash`            | Hash of the root tree this commit captures        |
| `parents`   | `Vec<Hash>`       | Zero or more parent commit hashes                 |
| `author`    | `UserID`          | The original author (validated)                   |
| `committer` | `UserID`          | The identity that created this commit (validated) |
| `timestamp` | `DateTime<Utc>`   | Automatically set to `Utc::now()` on construction |
| `message`   | `String`          | Commit message                                    |
| `signature` | `Option<Vec<u8>>` | Optional cryptographic signature bytes            |

The timestamp is always `Utc::now()` at construction time. There is no way to
set a custom timestamp through `Commit::new`.

---

### UserID and UserInfo

**UserID** is re-exported from the `age-credentials` crate. It is a validated
user identity with the following rules:

- **Name:** non-empty after trimming, minimum 2 characters, maximum 255
  characters, only alphabetic, numeric, space, hyphen, apostrophe, or period
  characters.
- **Email:** non-empty after trimming, exactly one `@`, non-empty local part
  and domain, maximum 254 characters, only alphanumeric, period, hyphen,
  underscore, `@`, or plus characters.

```rust
let uid = UserID::new("Alice Smith", "alice@example.com")?;
assert_eq!(uid.to_formatted(), "Alice Smith <alice@example.com>");
```

**UserInfo** is a plain, unvalidated struct retained for backward
compatibility:

```rust
pub struct UserInfo {
    pub name: String,
    pub email: String,
}
```

`Commit` and `CreateCommit` use `UserID`. Use `UserInfo` only if you need
an unvalidated identity (for example, when reading data that has already
been validated elsewhere).

---

### Object

```rust
pub enum Object {
    Blob(Blob),
    Tree(Tree),
    Commit(Box<Commit>),
}
```

Tagged union of all storable types. `Commit` is boxed to avoid infinite
type recursion. `obj_type()` returns `"blob"`, `"tree"`, or `"commit"`.

---

## Storage

### ObjectStore Trait

```rust
pub trait ObjectStore {
    fn put(&mut self, hash: &Hash, obj: &Object) ->FResult<(), VctrlError>;
    fn get(&self, hash: &Hash) -> Result<Option<Object>, VctrlError>;
    fn exists(&self, hash: &Hash) -> Result<bool, VctrlError>;
}
```

Trait for content-addressed object storage.

### RefStore Trait

```rust
pub trait RefStore {
    fn set_ref(&mut self, name: &str, hash: &Hash) -> Result<(), VctrlError>;
    fn get_ref(&self, name: &str) -> Result<Option<Hash>, VctrlError>;
    fn delete_ref(&mut self, name: &str) -> Result<(), VctrlError>;
    fn set_head(&mut self, target: &str) -> Result<(), VctrlError>;
    fn head(&self) -> Result<Option<Hash>, VctrlError>;
    fn head_ref_name(&self) -> Result<Option<String>, VctrlError>;
}
```

Trait for named reference and HEAD management. `head()` resolves HEAD to a
`Hash` (through symbolic reference or direct hex). `head_ref_name()` returns
the symbolic reference name, or `None` if HEAD is a direct hash or unset.

### MemoryStore

In-memory `ObjectStore` backed by `HashMap<Hash, Object>`. Implements `Default`.

### MemoryRefStore

In-memory `RefStore` backed by `HashMap<String, Hash>` and `Option<String>`
for HEAD. HEAD can be a symbolic reference (starting with `refs/`) or a
direct hex hash. Implements `Default`.

---

## Hashing

### Hasher Trait

```rust
pub trait Hasher {
    fn hash_blob(&self, data: &[u8]) -> Hash;
    fn hash_tree_encoded(&self, data: &[u8]) -> Hash;
    fn hash_commit_encoded(&self, data: &[u8]) -> Hash;
}
```

### Sha512Hasher

Computes `SHA-512(prefix || length_u64_be || 0x00 || data)`.

| Method                | Prefix      |
| --------------------- | ----------- |
| `hash_blob`           | `"blob "`   |
| `hash_tree_encoded`   | `"tree "`   |
| `hash_commit_encoded` | `"commit "` |

The type prefix and length header prevent cross-type hash collisions.

---

## Encoding

### Encoder Trait

```rust
pub trait Encoder {
    fn encode_tree(&self, tree: &Tree, buf: &mut Vec<u8>);
    fn encode_commit(&self, commit: &Commit, buf: &mut Vec<u8>);
}
```

### BinaryEncoder

Binary format implementation. Appends to the provided buffer.

### Binary Format Specification

**Tree:** version byte (0x01), entry count (u32 BE), per-entry: name
length (u16 BE), name bytes, kind byte (0x00=Blob, 0x01=Tree), hash (64
bytes).

**Commit:** version byte (0x01), tree hash (64 bytes), parent count (u32
BE), parent hashes, author, committer, timestamp seconds (i64 BE), timestamp
nanoseconds (u32 BE), message length (u32 BE"BE), message bytes, signature
length (u32 BE), signature bytes.

**User (author/committer):** name length (u16 BE), name bytes, email length
(u16 BE), email bytes.

---

## Commands

### Command Trait

```rust
pub trait Command {
    type Output;
    fn execute(&self, store: &mut dyn ObjectStore, refs: &mut dyn RefStore)
        -> Result<Self::Output, VctrlError>;
}
```

### Branch Operations

| Command                       | Output         | Description                                                   |
| ----------------------------- | -------------- | ------------------------------------------------------------- |
| `CreateBranch { name, hash }` | `()`           | Create/update a reference; name must start with `refs/heads/` |
| `DeleteBranch { name }`       | `()`           | Delete a reference; name must start with `refs/heads/`        |
| `GetBranch { name }`          | `Option<Hash>` | Look up a reference; name must start with `refs/heads/`       |

### SetHead

```rust
pub struct SetHead { pub target: String }
```

Sets HEAD. Target must start with `refs/` or be a valid 128-char hex hash.

### CreateCommit

```rust
pub struct CreateCommit {
    pub tree_hash: Hash,
    pub parents: Vec<Hash>,
    pub author: UserID,
    pub committer: UserID,
    pub message: String,
    pub encoder: Box<dyn Encoder>,
    pub hasher: Box<dyn Hasher>,
}
```

Creates a commit with `timestamp: Utc::now()` and `signature: None`, stores
it, and updates the current branch reference if HEAD is symbolic. Returns
the commit hash.

### Log

```rust
pub struct Log;
```

Traverses commit history from HEAD, following the first parent. Returns
commits newest-first. Returns empty vector if HEAD is unset.

### Checkout

```rust
pub struct Checkout { pub tree_hash: Hash }
```

Recursively materializes a tree into `Vec<(String, Vec<u8>)>` (path, data).
Depth capped at 1000.

### MergeCommand

```rust
pub struct MergeCommand {
    pub base: Hash, pub ours: Hash, pub theirs: Hash,
    pub merger: Box<dyn ThreeWayMerge>,
    pub resolver: Box<dyn ConflictResolver>,
    pub encoder: Box<dyn Encoder>,
    pub hasher: Box<dyn Hasher>,
}
```

Executes three-way merge, returning the merged tree hash.

---

## Diffing

### TreeDiff Trait

```rust
pub trait TreeDiff {
    fn diff(&self, old_tree: &Tree, new_tree: &Tree) -> Result<Vec<DiffEntry>, VctrlError>;
}
```

### DiffKind and DiffEntry

```rust
pub enum DiffKind {
    Added,
    Removed,
    Modified { old_hash: Hash, new_hash: Hash },
}

pub struct DiffEntry { pub name: String, pub kind: DiffKind }
```

### TreeDiffer

Converts both trees to `BTreeMap`, compares keys, and classifies entries
as Added, Removed, or Modified. Output is sorted by name.

---

## Merging

### ThreeWayMerge Trait

```rust
pub trait ThreeWayMerge {
    fn merge(&self, store: &mut dyn ObjectStore,
        base: &Hash, ours: &Hash, theirs: &Hash,
        resolver: &dyn ConflictResolver,
        encoder: &dyn Encoder, hasher: &dyn Hasher)
    -> Result<Hash, VctrlError>;
}
```

### ConflictResolver Trait

```rust
pub trait ConflictResolver {
    fn resolve(&self, base: &[u8], ours: &[u8], theirs: &[u8]) -> Option<Vec<u8>>;
}
```

Returns `Some(resolved_data)` to resolve a conflict, or `None` to fail.

### ThreeWayMerger

Handles all nine combinations of (base, ours, theirs) presence/absence.
Recurses into subtrees when both sides are trees. Calls ConflictResolver
when both sides modified the same blob. Depth capped at 1000.

---

## Cryptographic Signing

### Signer Trait

```rust
pub trait Signer {
    fn sign(&self, commit_hash: &[u8]) -> Result<Vec<u8>, VctrlError>;
}
```

A trait for signing arbitrary byte slices, intended for signing commit
hashes. The input should be the 64-byte `Hash` of the commit. The output
is a signature as raw bytes.

### LibrageSigner

```rust
pub struct LibrageSigner { /* private: SigningKey */ }
```

An Ed25519 signing implementation using `ed25519_dalek::SigningKey`.

#### LibrageSigner::generate

```rust
pub fn generate() -> Self
```

Generates a new Ed25519 signing key using the operating system CSPRNG
(`rand::rngs::OsRng`). The signing key is created from a random 32-byte
seed. This method cannot fail because `OsRng` is infallible in the
`rand` 0.8 API.

#### LibrageSigner::from_seed_file

```rust
pub fn from_seed_file(path: impl AsRef<Path>) -> Result<Self, VctrlError>
```

Loads a signing key from a 32-byte seed file on disk.

- Reads the file using `std::fs::read`.
- If the file cannot be read, returns `Err(VctrlError::Io)`.
- If the file is not exactly 32 bytes, returns `Err(VctrlError::Other)`
  with message `"seed file must be exactly 32 bytes"`.
- Constructs the `SigningKey` from the seed and returns `Ok(LrageSigner)`.

#### LibrageSigner::verifying_key

```rust
pub fn verifying_key(&self) -> VerifyingKey
```

Returns the `ed25519_dalek::VerifyingKey` (public key) corresponding to
the signing key. The verifying key can be used to verify signatures
produced by this signer.

#### Signer trait implementation

```rust
impl Signer for LibrageSigner {
    fn sign(&self, commit_hash: &[u8]) -> Result<Vec<u8>, VctrlError>
}
```

Signs the provided bytes using the Ed25519 signing key. Returns the
64-byte signature as a `Vec<u8>`. This method cannot fail because
Ed25519 signing is inf(ally infallible for any input length.

### Signing and Verification Workflow

libvctrl provides the building blocks for signing and verification but
does not automatically sign commits during `CreateCommit`. Applications
must integrate signing explicitly.

**Signing a commit:**

```rust
use libvctrl::{LibrageSigner, Signer};

let signer = LibrageSigner::generate();
let commit_hash_bytes = commit_hash.as_bytes();
let signature = signer.sign(commit_hash_bytes).unwrap();
// Store signature[;]signature bytes, e.g. in Commit::signature
```

**Verifying a commit signature:**

```rust
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

let verifying_key = signer.verifying_key();
let sig = Signature::try_from(signature.as_slice()).expect("valid signature");
assert!(verifying_key.verify(commit_hash.as_bytes(), &sig).is_ok());
```

**Persisting and loading a signing key:**

```rust
// Save the seed (32 bytes) to a file
use ed25519_dalek::SigningKey;
// Access the seed through the SigningKey's internal bytes
// (application-specific; LibrageSigner wraps this)

// Later, load from file
let signer = LibrageSigner::from_seed_file("/path/to/seed.bin")?;
```

**Important:** The seed file contains the private signing key. Protect it
with appropriate filesystem permissions (0600 on POSIX systems).

---

## Error Handling

### VctrlError

```rust
pub enum VctrlError {
    Hash(HashError),
    Tree(TreeError),
    NotFound(String),
    InvalidRef(String),
    MergeConflict { entry: String, reason: String },
    Io(std::io::Error),
    Serialization(String),
    Backend(String),
    Other(String),
}
```

| Variant         | When produced                                          |
| --------------- | ------------------------------------------------------ |
| `Hash`          | Invalid hash length or hex string                      |
| `Tree`          | Duplicate tree entry name                              |
| `NotFound`      | Object or tree not found                               |
| `InvalidRef`    | Invalid reference name or HEAD target                  |
| `MergeConflict` | Unresolvable merge conflict                            |
| `Io`            | I/O failure                                            |
| `Serialization` | Serialization failure                                  |
| `Backend`       | Backend-specific error                                 |
| `Other`         | Catch-all (e.g., seed file wrong size, depth exceeded) |

### HashError

```rust
pub enum HashError {
    InvalidLength(usize),
    InvalidHex,
}
```

### TreeError

```rust
pub enum TreeError {
    DuplicateEntry(String),
}
```

---

## Module Reference

| Module    | Status      | Description                                                   |
| --------- | ----------- | ------------------------------------------------------------- |
| `codec`   | Implemented | Encoder trait and BinaryEncoder                               |
| `command` | Implemented | Command trait and all command implementations                 |
| `crypto`  | Implemented | Signer trait and LibrageSigner                                |
| `diff`    | Implemented | TreeDiff trait, DiffKind, DiffEntry, TreeDiffer               |
| `domain`  | Implemented | Blob, Hash, Tree, TreeEntry, Commit, UserID, UserInfo, Object |
| `error`   | Implemented | VctrlError, HashError, TreeError                              |
| `hashing` | Implemented | Hasher trait and Sha512Hasher                                 |
| `merge`   | Implemented | ThreeWayMerge trait, ConflictResolver trait, ThreeWayMerger   |
| `storage` | Implemented | ObjectStore and RefStore traits, MemoryStore, MemoryRefStore  |

---

## Testing

29 tests across 8 test files:

| Test file     | Count | Verifies                                                                                             |
| ------------- | ----- | ---------------------------------------------------------------------------------------------------- |
| blob_test     | 2     | Blob construction and access                                                                         |
| branch_test   | 3     | Branch create/get/delete, invalid name, SetHead                                                      |
| checkout_test | 4     | Flat tree, recursive, empty tree, nonexistent tree                                                   |
| commit_test   | 3     | Create+log, commit chain, field access                                                               |
| diff_test     | 2     | Added/removed/modified, no changes                                                                   |
| merge_test    | 3     | No conflict, blob conflict, resolved conflict                                                        |
| sign_tests    | 7     | Generate+sign, deterministic, different hashes, wrong hash, empty hash, seed file, invalid seed file |
| tree_test     | 5     | Sort, duplicate error, hash determinism, empty, into_entries                                         |

Run the test suite:

```bash
cargo test
```

---

## Build and Lint

The project includes a Makefile. Run `make ci` for the full CI pipeline
(format check, clippy with all targets and features, and tests<stests).

---

## Security Considerations

- **SHA-512 collision resistance.** 256 bits of collision resistance, stronger
  than Git's SHA-1.
- **Type-prefixed hashing.** Prevents cross-type hash collisions.
- **Content-addressed integrity.** Objects are stored and retrieved by hash.
- **Ed25519 signing.** Provides 128-bit security level. Signatures are
  deterministic for the same key and message.
- **Seed file protection.** The 32-byte seed file contains the private
  signing key. Applications must protect it with filesystem permissions.
- **CSPRNG.** `LibrageSigner::generate` uses `OsRng`, which draws from the
  operating system's secure random number generator.
- **No encryption.** libvctrl does not encrypt objects. Applications must
  encrypt data before storing as blobs if confidentiality is required.
- **Depth limits.** Checkout and ThreeWayMerger cap recursion at 1000 levels.
- **Memory store has no persistence.** Data is lost on process exit.

---

## Limitations

- Only in-memory storage backend.
- `Log` only follows first parent (linear history).
- `Checkout` produces in-memory file lists, not filesystem writes.
- Branch names must start with `refs/heads/`. Tags and remotes not
  supported.
- `MergeCommand` produces merged tree but not merge commit.
- `ConflictResolver` has no path context.
- `Commit::new` always sets `timestamp` to `Utc::now()`.
- `CreateCommit` does not automatically sign commits.
- `LibrageSigner` does not persist the signing key.

---

## Migration from v0.1.0

### Commit and CreateCommit use UserID instead of UserInfo

The `author` and `committer` fields of `Commit` and `CreateCommit` now use
`UserID` (validated, from age-credentials) instead of `UserInfo`
(unvalidated). Update all construction sites:

```rust
// Before (v0.1.0):
let author = UserInfo::new("Alice".into(), "alice@example.com".into());

// After (v0.2.0):
let author = UserID::new("Alice Example", "alice@example.com")?;
```

`UserID::new` can return an error if the name or email fails validation.
Ensure your code handles the `Result`.

### SetHead accepts any refs/ prefix

In v0.1.0, `SetHead` required the target to start with `"refs/heads/"`.
In v0.2.0, it accepts any target starting with `"refs/"`. Code that relied
on `"refs/heads/"` being the only accepted prefix still works but can now
also use `"refs/tags/"` and other namespaces.

---

## Roadmap

- Filesystem storage backend.
- Tag support.
- Remote reference namespace.
- Full ancestry traversal (all parents).
- Merge commit creation as part of MergeCommand.
- Path-aware conflict resolver.
- Custom commit timestamps.
- Automatic commit signing in CreateCommit.
- Signing key persistence helpers.
- Streaming encoding and decoding.
- Pack format for efficient storage.
- crates.io publication.

---

## License

This project is licensed under the MIT License. See the LICENSE file in the
repository for the full text.
