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
commands, diffing, and three-way merging needed to build version control
functionality directly into applications -- without shelling out to an external
VCS or depending on a CLI tool. It is a library only and does not ship a
binary.

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
  - [UserInfo](#userinfo)
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
- [Error Handling](#error-handling)
  - [VctrlError](#vctrlerror)
  - [HashError](#hasherror)
  - [TreeError](#treeerror)
- [Module Reference](#module-reference)
- [Testing](#testing)
- [Build and Lint](#build-and-lint)
- [Security Considerations](#security-considerations)
- [Limitations](#limitations)
- [Roadmap](#roadmap)
- [License](#license)

---

## Overview

libvctrl implements a content-addressed version control engine similar in
principle to Git's object model, but with key differences:

- **SHA-512 hashes** instead of SHA-1, providing 256 bits of collision
  resistance.
- **Trait-based abstractions** for storage, hashing, encoding, diffing, and
  merging, allowing custom backends and algorithms without modifying the core.
- **Command pattern** for all operations, providing a uniform interface that
  accepts mutable references to an `ObjectStore` and a `RefStore`.
- **Embedded design** -- no CLI, no subprocess calls, no filesystem
  assumptions beyond what the storage backend requires. The provided
  `MemoryStore` and `MemoryRefStore` require no filesystem at all.

---

## Architecture

```

src/
lib.rs Crate root, re-exports all modules
error.rs VctrlError enum
codec/ Encoding format
mod.rs Encoder trait and re-exports
binary.rs BinaryEncoder implementation
command/ Command pattern operations
mod.rs Command trait and re-exports
branch.rs CreateBranch, DeleteBranch, GetBranch, SetHead
checkout.rs Checkout (recursive tree materialization)
create_commit.rs CreateCommit
log.rs Log (commit history traversal)
merge.rs MergeCommand
diff/ Tree diffing
mod.rs TreeDiff trait, DiffKind, DiffEntry
tree_diff.rs TreeDiffer implementation
domain/ Core domain types
mod.rs Re-exports all domain types
blob.rs Blob (content-addressed data)
commit.rs Commit (snapshot record)
hash.rs Hash (64-byte SHA-512), HashError
object.rs Object enum (Blob, Tree, Commit)
tree.rs Tree, TreeEntry, EntryKind, TreeError
user.rs UserInfo (name + email)
hashing/ Hashing trait and implementation
mod.rs Hasher trait and re-exports
sha512.rs Sha512Hasher
merge/ Three-way merge
mod.rs ThreeWayMerge trait
resolver.rs ConflictResolver trait
three_way.rs ThreeWayMerger implementation
storage/ Storage backends
mod.rs Re-exports
traits.rs ObjectStore, RefStore traits
memory.rs MemoryStore, MemoryRefStore

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

| Crate      | Version | Purpose                                       |
| ---------- | ------- | --------------------------------------------- |
| chrono     | 0.4.45  | Timestamps for commits (with serde feature)   |
| serde      | 1.0.229 | Serialization framework (with derive feature) |
| serde_json | 1.0.151 | JSON serialization                            |
| sha2       | 0.11.0  | SHA-512 digest computation                    |
| thiserror  | 2.0.19  | Error derive macro                            |

---

## Quick Start

```rust
use libvctrl::{
    BinaryEncoder, Blob, Commit, CreateBranch, CreateCommit, Hash, Hasher,
    MemoryRefStore, MemoryStore, Object, ObjectStore, RefStore, SetHead,
    Sha512Hasher, Tree, TreeEntry, EntryKind, UserInfo, Command,
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
let author = UserInfo::new("Alice".into(), "alice@example.com".into());
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

println!("Commit created: {}", commit_hash);
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

**Methods:**

| Method       | Signature                 | Description                        |
| ------------ | ------------------------- | ---------------------------------- |
| `new`        | `(data: Vec<u8>) -> Self` | Construct from raw bytes           |
| `as_bytes`   | `(&self) -> &[u8]`        | Borrow the inner bytes             |
| `into_bytes` | `(self) -> Vec<u8>`       | Consume and return the inner bytes |

`Blob` implements `Debug`, `Clone`, `Serialize`, and `Deserialize`.

---

### Hash

```rust
pub struct Hash([u8; 64]);
```

A 64-byte (512-bit) SHA-512 hash value. The inner array is private. `Hash`
is `Copy` because 64 bytes is small enough for stack allocation.

**Construction methods:**

| Method       | Signature                            | Description                                       |
| ------------ | ------------------------------------ | ------------------------------------------------- |
| `from_bytes` | `(bytes: [u8; 64]) -> Self`          | Construct from a fixed-size array (const)         |
| `from_slice` | `(&[u8]) -> Result<Self, HashError>` | Construct from a slice; fails if length is not 64 |
| `from_hex`   | `(&str) -> Result<Self, HashError>`  | Construct from a 128-character hex string         |

**Access methods:**

| Method     | Signature              | Description                                  |
| ---------- | ---------------------- | -------------------------------------------- |
| `as_bytes` | `(&self) -> &[u8; 64]` | Borrow the inner byte array                  |
| `to_hex`   | `(&self) -> String`    | Produce a 128-character lowercase hex string |

**Trait implementations:**

| Trait                                      | Behavior                                       |
| ------------------------------------------ | ---------------------------------------------- |
| `Debug`                                    | Formats as `Hash(<hex>)`                       |
| `Display`                                  | Formats as the bare hex string                 |
| `FromStr`                                  | Parses from hex via `from_hex`                 |
| `Serialize`                                | Serializes as a hex string                     |
| `Deserialize`                              | Deserializes from a hex string with validation |
| `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash` | Standard                                       |

---

### Tree and TreeEntry

```rust
pub struct Tree {
    entries: Vec<TreeEntry>,  // private, sorted by name
}

pub struct TreeEntry {
    pub name: String,
    pub kind: EntryKind,
    pub hash: Hash,
}

pub enum EntryKind {
    Blob,
    Tree,
}
```

A `Tree` represents a directory listing: an ordered collection of named
references to child objects (blobs or subtrees). Entries are sorted
lexicographically by name on construction and duplicate names are rejected.

**Tree methods:**

| Method         | Signature                                              | Description                                |
| -------------- | ------------------------------------------------------ | ------------------------------------------ |
| `new`          | `(entries: Vec<TreeEntry>) -> Result<Self, TreeError>` | Construct, sort by name, detect duplicates |
| `entries`      | `(&self) -> &[TreeEntry]`                              | Borrow the sorted entries                  |
| `into_entries` | `(self) -> Vec<TreeEntry>`                             | Consume and return the entries             |
| `is_empty`     | `(&self) -> bool`                                      | Check if there are no entries              |

**TreeEntry::new:**

```rust
pub fn new(name: String, kind: EntryKind, hash: Hash) -> Self
```

**TreeError:**

```rust
pub enum TreeError {
    DuplicateEntry(String),  // the duplicated name
}
```

Because entries are sorted on construction, the hash of a tree is
deterministic regardless of the input order. This is critical for
content-addressed storage: two trees with the same entries in different
orders produce the same hash.

---

### Commit

```rust
pub struct Commit {
    pub tree: Hash,
    pub parents: Vec<Hash>,
    pub author: UserInfo,
    pub committer: UserInfo,
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub signature: Option<Vec<u8>>,
}
```

A snapshot record linking a tree to metadata.

**Fields:**

| Field       | Type              | Description                                                |
| ----------- | ----------------- | ---------------------------------------------------------- |
| `tree`      | `Hash`            | Hash of the root tree this commit captures                 |
| `parents`   | `Vec<Hash>`       | Zero or more parent commit hashes (empty for root commits) |
| `author`    | `UserInfo`        | The original author of the change                          |
| `committer` | `UserInfo`        | The identity that created this commit object               |
| `timestamp` | `DateTime<Utc>`   | Automatically set to `Utc::now()` on construction          |
| `message`   | `String`          | Commit message                                             |
| `signature` | `Option<Vec<u8>>` | Optional cryptographic signature bytes                     |

**Commit::new:**

```rust
pub fn new(
    tree: Hash,
    parents: Vec<Hash>,
    author: UserInfo,
    committer: UserInfo,
    message: String,
    signature: Option<Vec<u8>>,
) -> Self
```

The timestamp is always `Utc::now()` at construction time. There is no way to
set a custom timestamp through `new`. This ensures that commits created
through the API always have a valid, recent timestamp.

---

### UserInfo

```rust
pub struct UserInfo {
    pub name: String,
    pub email: String,
}
```

Author or committer identity. No validation is performed on the `name` or
`email` fields. The struct is a plain data holder.

---

### Object

```rust
pub enum Object {
    Blob(Blob),
    Tree(Tree),
    Commit(Box<Commit>),
}
```

Tagged union of all storable object types. `Commit` is boxed to avoid
infinite type recursion (a `Commit` contains `UserInfo` which contains
`String`, and the overall size is large enough that boxing reduces stack
pressure).

**Object::obj_type:**

```rust
pub fn obj_type(&self) -> &str
```

Returns `"blob"`, `"tree"`, or `"commit"`.

---

## Storage

### ObjectStore Trait

```rust
pub trait ObjectStore {
    fn put(&mut self, hash: &Hash, obj: &Object) -> Result<(), VctrlError>;
    fn get(&self, hash: &Hash) -> Result<Option<Object>, VctrlError>;
    fn exists(&self, hash: &Hash) -> Result<bool, VctrlError>;
}
```

Trait for content-addressed object storage. Implementations store and
retrieve `Object` values keyed by their `Hash`.

- `put` -- Store an object. If an object with the same hash already exists,
  the implementation may overwrite it or silently ignore the duplicate (the
  memory backend overwrites).
- `get` -- Retrieve an object. Returns `Ok(None)` if the hash is not present.
- `exists` -- Check whether a hash is present without retrieving the object.

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

Trait for named reference and HEAD management.

- `set_ref` / `get_ref` / `delete_ref` -- Manage named references (branches,
  tags, etc.) that map string names to `Hash` values.
- `set_head` -- Set the HEAD pointer. The `target` is a string that can be
  either a symbolic reference (starting with `refs/`) or a direct hex hash.
- `head` -- Resolve HEAD to a `Hash`. If HEAD is a symbolic reference, the
  implementation resolves it through `get_ref`. If HEAD is a direct hash,
  it is parsed from hex.
- `head_ref_name` -- Return the symbolic reference name that HEAD points to,
  or `None` if HEAD is a direct hash or unset. Used by `CreateCommit` to
  update the current branch after creating a commit.

### MemoryStore

```rust
pub struct MemoryStore {
    // private: HashMap<Hash, Object>
}
```

In-memory `ObjectStore` backed by `HashMap<Hash, Object>`. Provides `new()`
and implements `Default`. All objects are kept in memory for the lifetime of
the store. Suitable for testing, prototyping, and transient computations.

### MemoryRefStore

```rust
pub struct MemoryRefStore {
    // private: HashMap<String, Hash>, Option<String> for HEAD
}
```

In-memory `RefStore` backed by `HashMap<String, Hash>` for references and
`Option<String>` for HEAD.

**HEAD resolution logic in `head()`:**

1. If `head` is `None`, return `Ok(None)`.
2. If `head` is `Some(target)` and `target` starts with `"refs/"`, resolve
   by calling `get_ref(target)`.
3. If `head` is `Some(target)` and does not start with `"refs/"`, parse it
   as a 128-character hex hash via `Hash::from_hex`. Returns
   `Err(VctrlError::Hash)` if the hex is invalid.

**`head_ref_name()` logic:**

1. If `head` is `Some(target)` and `target` starts with `"refs/"`, return
   `Ok(Some(target.clone()))`.
2. Otherwise, return `Ok(None)`.

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

Trait for content-addressed hashing. Each method takes raw or encoded data
and produces a `Hash`. The separate methods allow implementations to include
a type prefix in the hash input, preventing cross-type hash collisions.

### Sha512Hasher

```rust
pub struct Sha512Hasher;
```

SHA-512 implementation of `Hasher`.

**Hash format:** Each method computes `SHA-512(prefix || length_be || 0x00 || data)`:

| Method                | Prefix      | length_be                       |
| --------------------- | ----------- | ------------------------------- |
| `hash_blob`           | `"blob "`   | `data.len() as u64`, big-endian |
| `hash_tree_encoded`   | `"tree "`   | `data.len() as u64`, big-endian |
| `hash_commit_encoded` | `"commit "` | `data.len() as u64`, big-endian |

The prefix includes a trailing space (for example, `b"blob "`). The null
byte `0x00` separates the header from the data. This format is modeled after
Git's object hashing and ensures that two different object types with
identical content produce different hashes.

**Example:**

```rust
let hasher = Sha512Hasher;
let blob_hash = hasher.hash_blob(b"hello world");
let other_hash = hasher.hash_blob(b"hello world");
assert_eq!(blob_hash, other_hash);  // deterministic

let different_hash = hasher.hash_blob(b"goodbye");
assert_ne!(blob_hash, different_hash);
```

---

## Encoding

### Encoder Trait

```rust
pub trait Encoder {
    fn encode_tree(&self, tree: &Tree, buf: &mut Vec<u8>);
    fn encode_commit(&self, commit: &Commit, buf: &mut Vec<u8>);
}
```

Trait for serializing domain objects to a byte buffer. The buffer is appended
to (not cleared), allowing multiple objects to be encoded into the same
buffer.

### BinaryEncoder

```rust
pub struct BinaryEncoder;
```

Binary format implementation of `Encoder`.

### Binary Format Specification

**Tree encoding:**

| Offset | Size | Value                        |
| ------ | ---- | ---------------------------- |
| 0      | 1    | Version byte: `0x01`         |
| 1      | 4    | Entry count (big-endian u32) |
| 5      | ...  | For each entry:              |

Per entry:

| Offset      | Size     | Value                              |
| ----------- | -------- | ---------------------------------- |
| +0          | 2        | Name length (big-endian u16)       |
| +2          | name_len | Name bytes (UTF-8)                 |
| +2+name_len | 1        | Kind: `0x00` = Blob, `0x01` = Tree |
| +3+name_len | 64       | Hash bytes (raw, 64 bytes)         |

**Commit encoding:**

| Offset | Size    | Value                                             |
| ------ | ------- | ------------------------------------------------- |
| 0      | 1       | Version byte: `0x01`                              |
| 1      | 64      | Tree hash                                         |
| 65     | 4       | Parent count (big-endian u32)                     |
| 69     | 64 * n  | Parent hashes                                     |
| ...    | ...     | Author (see below)                                |
| ...    | ...     | Committer (see below)                             |
| ...    | 8       | Timestamp seconds (big-endian i64)                |
| ...    | 4       | Timestamp sub-second nanoseconds (big-endian u32) |
| ...    | 4       | Message length (big-endian u32)                   |
| ...    | msg_len | Message bytes (UTF-8)                             |
| ...    | 4       | Signature length (big-endian u32); 0 if None      |
| ...    | sig_len | Signature bytes (if present)                      |

**User (author/committer) encoding:**

| Offset      | Size      | Value                         |
| ----------- | --------- | ----------------------------- |
| +0          | 2         | Name length (big-endian u16)  |
| +2          | name_len  | Name bytes (UTF-8)            |
| +2+name_len | 2         | Email length (big-endian u16) |
| +4+name_len | email_len | Email bytes (UTF-8)           |

---

## Commands

### Command Trait

```rust
pub trait Command {
    type Output;
    fn execute(
        &self,
        store: &mut dyn ObjectStore,
        refs: &mut dyn RefStore,
    ) -> Result<Self::Output, VctrlError>;
}
```

All operations implement this trait. Each command takes mutable references to
an object store and a reference store, and returns a typed output or a
`VctrlError`. The command itself is consumed by reference (`&self`), not by
value, allowing it to be reused.

### Branch Operations

#### CreateBranch

```rust
pub struct CreateBranch {
    pub name: String,
    pub hash: Hash,
}
```

Creates or updates a named reference. The `name` must start with
`"refs/heads/"`. Returns `Err(VctrlError::InvalidRef)` otherwise. On
success, returns `Ok(())`.

#### DeleteBranch

```rust
pub struct DeleteBranch {
    pub name: String,
}
```

Deletes a named reference. The `name` must start with `"refs/heads/"`.
Returns `Err(VctrlError::InvalidRef)` otherwise. On success, returns
`Ok(())`. Deleting a nonexistent reference silently succeeds (the memory
backend's `HashMap::remove` behavior).

#### GetBranch

```rust
pub struct GetBranch {
    pub name: String,
}
```

Retrieves the hash associated with a named reference. The `name` must start
with `"refs/heads/"`. Returns `Ok(Some(hash))` if the reference exists,
`Ok(None)` if it does not, or `Err(VctrlError::InvalidRef)` if the name is
invalid.

### SetHead

```rust
pub struct SetHead {
    pub target: String,
}
```

Sets the HEAD pointer. The `target` must be either:

- A branch reference starting with `"refs/heads/"`, or
- A valid 128-character hexadecimal hash.

Returns `Err(VctrlError::InvalidRef)` if the target is neither. On success,
returns `Ok(())`.

### CreateCommit

```rust
pub struct CreateCommit {
    pub tree_hash: Hash,
    pub parents: Vec<Hash>,
    pub author: UserInfo,
    pub committer: UserInfo,
    pub message: String,
    pub encoder: Box<dyn Encoder>,
    pub hasher: Box<dyn Hasher>,
}
```

Creates a commit object, stores it, and updates the current branch reference.

**Execution steps:**

1. Construct a `Commit` with `timestamp: Utc::now()` and `signature: None`.
2. Encode the commit using the provided `encoder`.
3. Hash the encoded bytes using the provided `hasher`.
4. Store the commit as `Object::Commit(Box::new(commit))`.
5. If HEAD points to a symbolic reference (via `refs.head_ref_name()`),
   update that reference to the new commit hash.
6. Return the commit hash.

The encoder and hasher are boxed trait objects, allowing the caller to
inject custom implementations.

### Log

```rust
pub struct Log;
```

Traverses the commit history starting from HEAD, following the first parent
of each commit.

**Execution steps:**

1. Resolve HEAD. If HEAD is unset, return an empty vector.
2. Starting from the HEAD hash, look up the commit object.
3. Push the commit into the result vector.
4. Follow `commit.parents[0]` to the next commit.
5. Repeat until a commit with no parents is reached or an object is not
   found.

The result is ordered from most recent to oldest (newest commit first).

**Limitation:** Only follows the first parent. Merge commits' second and
subsequent parents are ignored. This produces a linear history view.

### Checkout

```rust
pub struct Checkout {
    pub tree_hash: Hash,
}
```

Recursively materializes a tree into a flat list of file paths and their
contents.

**Output:** `Vec<(String, Vec<u8>)>` where each tuple is `(path, data)`.

**Execution steps:**

1. Look up the tree object by `tree_hash`.
2. For each entry in the tree:
   - If `EntryKind::Blob`: look up the blob, prepend the current path
     prefix, and add `(path, blob.into_bytes())` to the result.
   - If `EntryKind::Tree`: recurse into the subtree, extending the path
     prefix with `"{prefix}/{name}"`.
3. Return the flat list.

**Depth limit:** Recursion is capped at 1000 levels. If exceeded, returns
`Err(VctrlError::Other("max checkout depth exceeded"))`.

**Error:** Returns `Err(VctrlError::NotFound("tree not found"))` if the
hash does not resolve to a tree object.

### MergeCommand

```rust
pub struct MergeCommand {
    pub base: Hash,
    pub ours: Hash,
    pub theirs: Hash,
    pub merger: Box<dyn ThreeWayMerge>,
    pub resolver: Box<dyn ConflictResolver>,
    pub encoder: Box<dyn Encoder>,
    pub hasher: Box<dyn Hasher>,
}
```

Executes a three-way merge. Delegates to the provided `ThreeWayMerge`
implementation with the `base`, `ours`, and `theirs` tree hashes. Returns
the hash of the merged tree on success, or `Err(VctrlError::MergeConflict)`
on unresolvable conflicts.

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

pub struct DiffEntry {
    pub name: String,
    pub kind: DiffKind,
}
```

| Variant    | Meaning                                                                          |
| ---------- | -------------------------------------------------------------------------------- |
| `Added`    | Entry exists in `new_tree` but not in `old_tree`                                 |
| `Removed`  | Entry exists in `old_tree` but not in `new_tree`                                 |
| `Modified` | Entry exists in both but with different hashes; captures both old and new hashes |

### TreeDiffer

```rust
pub struct TreeDiffer;
```

Implementation of `TreeDiff`. Converts both trees to `BTreeMap<String, TreeEntry>`,
collects the union of keys, and classifies each entry by comparing presence
and hash equality. Entries present in both trees with the same hash are
omitted from the result (no diff). The output is ordered by key name because
`BTreeSet` iteration is sorted.

---

## Merging

### ThreeWayMerge Trait

```rust
pub trait ThreeWayMerge {
    fn merge(
        &self,
        store: &mut dyn ObjectStore,
        base: &Hash,
        ours: &Hash,
        theirs: &Hash,
        resolver: &dyn ConflictResolver,
        encoder: &dyn Encoder,
        hasher: &dyn Hasher,
    ) -> Result<Hash, VctrlError>;
}
```

### ConflictResolver Trait

```rust
pub trait ConflictResolver {
    fn resolve(&self, base: &[u8], ours: &[u8], theirs: &[u8]) -> Option<Vec<u8>>;
}
```

Called when both `ours` and `theirs` have modified the same blob relative to
`base`. Returns `Some(resolved_data)` to resolve the conflict, or `None` to
fail with `VctrlError::MergeConflict`.

The resolver receives the raw bytes of the base, ours, and theirs blobs. It
does not receive the entry name or path. Resolvers that need path context
must capture it through other means (closures, environment variables, etc.).

### ThreeWayMerger

```rust
pub struct ThreeWayMerger;
```

Full three-way merge implementation. Handles all nine combinations of entry
presence in (base, ours, theirs):

| base | ours | theirs         | Result                                      |
| ---- | ---- | -------------- | ------------------------------------------- |
| -    | O    | -              | Added by ours: keep O                       |
| -    | -    | T              | Added by theirs: keep T                     |
| B    | -    | T              | Removed by ours, modified by theirs: keep T |
| B    | O    | -              | Modified by ours, removed by theirs: keep O |
| B    | -    | -              | Removed by both: omit                       |
| B    | O    | T (O==T)       | Same modification: keep O                   |
| B    | O    | T (O==B)       | Ours unchanged, theirs modified: keep T     |
| B    | O    | T (T==B)       | Theirs unchanged, ours modified: keep O     |
| B    | O    | T (all differ) | Conflict: call resolver                     |

When all three versions differ and the entries are blobs, the
`ConflictResolver` is called. If it returns `Some(data)`, a new blob is
created and stored. If it returns `None`, `VctrlError::MergeConflict` is
returned.

When all three versions differ and the entries are trees, the merger
recurses into the subtrees. When the entry kinds differ (one is Blob, the
other is Tree), `VctrlError::MergeConflict` is returned with reason
`"type mismatch"`.

**Depth limit:** Recursion is capped at 1000 levels. If exceeded, returns
`Err(VctrlError::Other("max merge depth exceeded"))`.

**After merge:** The merged tree is constructed via `Tree::new` (which sorts
and validates entries), encoded, hashed, and stored. The hash of the merged
tree is returned.

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

| Variant         | Source        | When produced                                                     |
| --------------- | ------------- | ----------------------------------------------------------------- |
| `Hash`          | `HashError`   | Invalid hash length or hex string                                 |
| `Tree`          | `TreeError`   | Duplicate tree entry name                                         |
| `NotFound`      | String        | Object or tree not found in store                                 |
| `InvalidRef`    | String        | Branch name lacks `refs/heads/` prefix, or HEAD target is invalid |
| `MergeConflict` | entry, reason | Unresolvable conflict during three-way merge                      |
| 5               | `Io`          | `std::io::Error`                                                  | I/O failure (reserved for filesystem backends) |
| `Serialization` | String        | Serialization failure                                             |
| `Backend`       | String        | Backend-specific error                                            |
| `Other`         | String        | Catch-all for uncategorized errors                                |

`VctrlError` implements `std::error::Error` (via thiserror), `Debug`, and
`Display`. The `Hash`, `Tree`, and `Io` variants implement `From` for
automatic conversion with the `?` operator.

### HashError

```rust
pub enum HashError {
    InvalidLength(usize),   // actual length (expected 64)
    InvalidHex,             // non-hex characters
}
```

### TreeError

```rust
pub enum TreeError {
    DuplicateEntry(String),  // the duplicated entry name
}
```

---

## Module Reference

| Module    | Status      | Description                                                  |
| --------- | ----------- | ------------------------------------------------------------ |
| `codec`   | Implemented | Encoder trait and BinaryEncoder                              |
| `command` | Implemented | Command trait and all command implementations                |
| `diff`    | Implemented | TreeDiff trait, DiffKind, DiffEntry, TreeDiffer              |
| `domain`  | Implemented | Blob, Hash, Tree, TreeEntry, Commit, UserInfo, Object        |
| `error`   | Implemented | VctrlError, HashError, TreeError                             |
| `hashing` | Implemented | Hasher trait and Sha512Hasher                                |
| `merge`   | Implemented | ThreeWayMerge trait, ConflictResolver trait, ThreeWayMerger  |
| `storage` | Implemented | ObjectStore and RefStore traits, MemoryStore, MemoryRefStore |

---

## Testing

22 tests across 7 test files:

### blob_test (2 tests)

| Test                  | Verifies                                |
| --------------------- | --------------------------------------- |
| `blob_new_and_access` | `Blob::new` and `as_bytes()` round-trip |
| `blob_into_bytes`     | `into_bytes()` returns original data    |

### branch_test (3 tests)

| Test                       | Verifies                                               |
| -------------------------- | ------------------------------------------------------ |
| `branch_create_get_delete` | Create, get, and delete branch round-trip              |
| `branch_invalid_name`      | Name without `refs/heads/` prefix returns `InvalidRef` |
| `set_head_works`           | `SetHead` resolves HEAD to the branch's hash           |

### checkout_test (4 tests)

| Test                              | Verifies                                       |
| --------------------------------- | ---------------------------------------------- |
| `checkout_flat_tree`              | Flat tree materializes to expected file paths  |
| `checkout_recursive`              | Nested tree produces paths with `/` separators |
| `checkout_empty_tree`             | Empty tree produces empty file list            |
| `checkout_nonexistent_tree_error` | Non-existent tree returns `NotFound`           |

### commit_test (3 tests)

| Test                    | Verifies                                         |
| ----------------------- | ------------------------------------------------ |
| `create_commit_and_log` | Create a commit and retrieve it via Log          |
| `commit_chain_log`      | Two-commit chain produces correct history order  |
| `commit_getters`        | Commit struct field access and default timestamp |

### diff_test (2 tests)

| Test                          | Verifies                                      |
| ----------------------------- | --------------------------------------------- |
| `diff_added_removed_modified` | Added, removed, and modified entries detected |
| `diff_no_changes`             | Identical trees produce empty diff            |

### merge_test (3 tests)

| Test                  | Verifies                                                         |
| --------------------- | ---------------------------------------------------------------- |
| `merge_no_conflict`   | Non-conflicting changes merge correctly                          |
| `merge_conflict_blob` | Blob conflict returns `MergeConflict`                            |
| `merge_resolved`      | `KeepOursResolver` resolves conflict and produces correct result |

### tree_test (5 tests)

| Test                           | Verifies                                           |
| ------------------------------ | -------------------------------------------------- |
| `tree_new_sorts_entries`       | Entries are sorted by name on construction         |
| `tree_duplicate_entries_error` | Duplicate names return `TreeError::DuplicateEntry` |
| `tree_hash_deterministic`      | Different input order produces same hash           |
| `tree_empty`                   | Empty tree is valid and reports `is_empty()`       |
| `tree_into_entries`            | `into_entries()` returns the entries               |

Run the test suite:

```bash
cargo test
```

---

## Build and Lint

The project includes a Makefile. Run `make ci` for the full CI pipeline
(format check, clippy, and tests).

---

## Security Considerations

- **SHA-512 collision resistance.** The hashing scheme uses SHA-512, which
  provides 256 bits of collision resistance. This is sufficient for all
  practical purposes and is stronger than Git's SHA-1.
- **Type-prefixed hashing.** Each hash includes the object type (`blob`,
  `tree`, `commit`) as a prefix. This prevents cross-type hash collisions
  where a blob and a tree with the same content would produce the same hash.
- **Content-addressed integrity.** Objects are stored and retrieved by their
  content hash. Any corruption of the stored data will result in a hash
  mismatch when the object is re-read and verified by the caller.
- **No cryptographic signing.** The `Commit::signature` field is an opaque
  `Option<Vec<u8>>`. libvctrl does not create, verify, or interpret
  signatures. Applications must implement signing and verification
  themselves.
- **No encryption.** libvctrl does not encrypt objects. If confidentiality
  is required, the application must encrypt data before storing it as a blob.
- **Depth limits.** `Checkout` and `ThreeWayMerger` enforce a maximum
  recursion depth of 1000. This prevents stack overflow from maliciously
  crafted deeply nested trees.
- **Memory store has no persistence.** `MemoryStore` and `MemoryRefStore`
  exist only in RAM. Data is lost when the process exits. Applications
  requiring persistence must implement a filesystem or database backend.

---

## Limitations

- Only an in-memory storage backend is provided.
- `Log` only follows the first parent, producing linear history.
- `Checkout` produces in-memory file lists, not filesystem writes.
- Branch names must start with `refs/heads/`. Tags and remote references
  are not supported.
- `MergeCommand` produces a merged tree but does not create a merge commit.
- `ConflictResolver` receives blob data only, without path context.
- `Commit::new` always sets `timestamp` to `Utc::now()`. Custom timestamps
  are not supported.
- The binary encoding does not include a CRC or integrity checksum beyond
  the SHA-512 content hash.

---

## Roadmap

- Filesystem storage backend.
- Tag support (`refs/tags/`).
- Remote reference namespace (`refs/remotes/`).
- Full ancestry traversal (all parents, not just first).
- Merge commit creation as part of `MergeCommand`.
- Path-aware conflict resolver.
- Custom commit timestamps.
- Streaming encoding and decoding for large objects.
- Pack format for efficient storage.
- crates.io publication.

---

## License

This project is licensed under the MIT License. See the LICENSE file in the
repository for the full text.
