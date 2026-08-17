//! Index (staging area) trait.
//!
//! # Architecture
//! This module defines the abstract contract for managing the Git index, commonly
//! known as the staging area. The index acts as the crucial intermediate state
//! between the working directory and the object database, tracking planned changes
//! for the next commit.
//!
//! # Design Rationale: Associated Types over Generics
//! The trait uses associated types (`type Entry`, `type Path`, `type TreeId`)
//! rather than generic parameters. This design ties the data representations
//! directly to the specific `Index` implementation. An in-memory index might use
//! `Rc<TreeEntry>` and `String`, while a disk-backed index might use `TreeEntry`
//! and `PathBuf`. This prevents type mismatches at compile time and simplifies
//! the API by removing the need for verbose generic annotations at every call site.

use crate::errors::VctrlError;

/// A trait for managing a Git index (staging area).
///
/// # Why this exists
/// The staging area allows users to stage partial changes (hunks) before committing
/// them to history. By abstracting this into a trait, the crate allows the core
/// engine to orchestrate commits, diffs, and merges without being tied to a specific
/// binary format (like the `.git/index` file) or an in-memory representation.
///
/// # How it works
/// The index maintains a mapping between file paths and their staged object entries.
/// It supports adding, removing, and querying entries. The `write_tree` method
/// serializes the current state into one or more tree objects in the object database,
/// returning the root tree identifier. `read_tree` performs the inverse, populating
/// the index from an existing tree.
///
/// # Design Rationale: `&self` on `write_tree`
/// Note that `write_tree` takes `&self` instead of `&mut self`. This is because
/// writing a tree does not mutate the logical state of the index itself. The
/// implementor is responsible for handling any necessary interior mutability
/// (e.g., using `RefCell` or `Mutex`) when interacting with the underlying
/// `ObjectStore` to persist the tree objects.
///
/// # Examples
///
/// Implementing the trait for a mock in-memory store:
///
/// ```
/// # use libvctrl_handler::traits::core::index::Index;
/// # use libvctrl_handler::VctrlError;
/// # use std::collections::HashMap;
/// #
/// #[derive(Default)]
/// struct MockIndex {
///     data: HashMap<String, String>,
/// }
///
/// impl Index for MockIndex {
///     type Entry = String;
///     type Path = String;
///     type TreeId = u32;
///
///     fn add(&mut self, entry: Self::Entry) -> Result<(), VctrlError> {
///         self.data.insert(entry.clone(), entry);
///         Ok(())
///     }
///
///     fn remove(&mut self, path: &Self::Path) -> Result<(), VctrlError> {
///         self.data.remove(path);
///         Ok(())
///     }
///
///     fn clear(&mut self) -> Result<(), VctrlError> {
///         self.data.clear();
///         Ok(())
///     }
///
///     fn get(&self, path: &Self::Path) -> Result<Option<Self::Entry>, VctrlError> {
///         Ok(self.data.get(path).cloned())
///     }
///
///     fn contains(&self, path: &Self::Path) -> Result<bool, VctrlError> {
///         Ok(self.data.contains_key(path))
///     }
///
///     fn len(&self) -> Result<usize, VctrlError> {
///         Ok(self.data.len())
///     }
///
///     fn entries(&self) -> Result<Vec<Self::Entry>, VctrlError> {
///         Ok(self.data.values().cloned().collect())
///     }
///
///     fn write_tree(&self) -> Result<Self::TreeId, VctrlError> {
///         // In a real impl, this would write to an ObjectStore.
///         Ok(1)
///     }
///
///     fn read_tree(&mut self, _tree: &Self::TreeId) -> Result<(), VctrlError> {
///         // Mock implementation
///         Ok(())
///     }
/// }
///
/// let mut index = MockIndex::default();
/// index.add("file.txt".to_string())?;
/// assert_eq!(index.len()?, 1);
/// assert!(index.contains(&"file.txt".to_string())?);
/// # Ok::<(), VctrlError>(())
/// ```
pub trait Index: Send + Sync {
    /// The entry type used by the index.
    ///
    /// # Why this exists
    /// Allows the backend to define its own representation of a staged file, which
    /// might include mode bits, object hashes, and filesystem stat data (mtime, ctime)
    /// for optimization.
    type Entry: Send + Sync;

    /// The path type used by the index.
    ///
    /// # Why this exists
    /// Decouples the path representation. While typically a `String` or `PathBuf`,
    /// this allows backends to use interned strings or OS-specific paths.
    type Path: Send + Sync;

    /// The tree identifier type.
    ///
    /// # Why this exists
    /// Matches the identifier type used by the backend's `ObjectStore` or `TreeDiffer`,
    /// ensuring seamless interoperability when writing or reading trees.
    type TreeId: Send + Sync;

    /// Adds an entry to the index.
    ///
    /// # How it works
    /// Inserts or updates the entry in the index. If an entry with the same path already
    /// exists, it is overwritten. Requires `&mut self` as it mutates the logical state
    /// of the staging area.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the underlying storage fails to persist the update
    /// or if the entry is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::index::Index;
    /// # use libvctrl_handler::VctrlError;
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct MockIndex { data: HashMap<String, String> }
    /// # impl Index for MockIndex {
    /// #     type Entry = String; type Path = String; type TreeId = u32;
    /// #     fn add(&mut self, e: Self::Entry) -> Result<(), VctrlError> { self.data.insert(e.clone(), e); Ok(()) }
    /// #     fn remove(&mut self, p: &Self::Path) -> Result<(), VctrlError> { self.data.remove(p); Ok(()) }
    /// #     fn clear(&mut self) -> Result<(), VctrlError> { self.data.clear(); Ok(()) }
    /// #     fn get(&self, p: &Self::Path) -> Result<Option<Self::Entry>, VctrlError> { Ok(self.data.get(p).cloned()) }
    /// #     fn contains(&self, p: &Self::Path) -> Result<bool, VctrlError> { Ok(self.data.contains_key(p)) }
    /// #     fn len(&self) -> Result<usize, VctrlError> { Ok(self.data.len()) }
    /// #     fn entries(&self) -> Result<Vec<Self::Entry>, VctrlError> { Ok(self.data.values().cloned().collect()) }
    /// #     fn write_tree(&self) -> Result<Self::TreeId, VctrlError> { Ok(1) }
    /// #     fn read_tree(&mut self, _t: &Self::TreeId) -> Result<(), VctrlError> { Ok(()) }
    /// # }
    /// let mut index = MockIndex::default();
    /// index.add("new_file.txt".to_string())?;
    /// assert_eq!(index.len()?, 1);
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn add(&mut self, entry: Self::Entry) -> Result<(), VctrlError>;

    /// Removes an entry from the index by path.
    ///
    /// # How it works
    /// Locates the entry by its path and removes it. If the path does not exist,
    /// this operation is typically idempotent and returns `Ok(())`.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the underlying storage fails to persist the deletion.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::index::Index;
    /// # use libvctrl_handler::VctrlError;
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct MockIndex { data: HashMap<String, String> }
    /// # impl Index for MockIndex {
    /// #     type Entry = String; type Path = String; type TreeId = u32;
    /// #     fn add(&mut self, e: Self::Entry) -> Result<(), VctrlError> { self.data.insert(e.clone(), e); Ok(()) }
    /// #     fn remove(&mut self, p: &Self::Path) -> Result<(), VctrlError> { self.data.remove(p); Ok(()) }
    /// #     fn clear(&mut self) -> Result<(), VctrlError> { self.data.clear(); Ok(()) }
    /// #     fn get(&self, p: &Self::Path) -> Result<Option<Self::Entry>, VctrlError> { Ok(self.data.get(p).cloned()) }
    /// #     fn contains(&self, p: &Self::Path) -> Result<bool, VctrlError> { Ok(self.data.contains_key(p)) }
    /// #     fn len(&self) -> Result<usize, VctrlError> { Ok(self.data.len()) }
    /// #     fn entries(&self) -> Result<Vec<Self::Entry>, VctrlError> { Ok(self.data.values().cloned().collect()) }
    /// #     fn write_tree(&self) -> Result<Self::TreeId, VctrlError> { Ok(1) }
    /// #     fn read_tree(&mut self, _t: &Self::TreeId) -> Result<(), VctrlError> { Ok(()) }
    /// # }
    /// let mut index = MockIndex::default();
    /// index.add("file.txt".to_string())?;
    /// index.remove(&"file.txt".to_string())?;
    /// assert!(index.is_empty()?);
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn remove(&mut self, path: &Self::Path) -> Result<(), VctrlError>;

    /// Clears all entries from the index.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the underlying storage cannot be cleared.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::index::Index;
    /// # use libvctrl_handler::VctrlError;
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct MockIndex { data: HashMap<String, String> }
    /// # impl Index for MockIndex {
    /// #     type Entry = String; type Path = String; type TreeId = u32;
    /// #     fn add(&mut self, e: Self::Entry) -> Result<(), VctrlError> { self.data.insert(e.clone(), e); Ok(()) }
    /// #     fn remove(&mut self, p: &Self::Path) -> Result<(), VctrlError> { self.data.remove(p); Ok(()) }
    /// #     fn clear(&mut self) -> Result<(), VctrlError> { self.data.clear(); Ok(()) }
    /// #     fn get(&self, p: &Self::Path) -> Result<Option<Self::Entry>, VctrlError> { Ok(self.data.get(p).cloned()) }
    /// #     fn contains(&self, p: &Self::Path) -> Result<bool, VctrlError> { Ok(self.data.contains_key(p)) }
    /// #     fn len(&self) -> Result<usize, VctrlError> { Ok(self.data.len()) }
    /// #     fn entries(&self) -> Result<Vec<Self::Entry>, VctrlError> { Ok(self.data.values().cloned().collect()) }
    /// #     fn write_tree(&self) -> Result<Self::TreeId, VctrlError> { Ok(1) }
    /// #     fn read_tree(&mut self, _t: &Self::TreeId) -> Result<(), VctrlError> { Ok(()) }
    /// # }
    /// let mut index = MockIndex::default();
    /// index.add("a".to_string())?;
    /// index.clear()?;
    /// assert_eq!(index.len()?, 0);
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn clear(&mut self) -> Result<(), VctrlError>;

    /// Retrieves an entry by path.
    ///
    /// # How it works
    /// Performs a lookup. Returns `Ok(None)` if the path is not staged, maintaining
    /// a clear distinction between "not staged" and "I/O error".
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the underlying storage cannot be read.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::index::Index;
    /// # use libvctrl_handler::VctrlError;
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct MockIndex { data: HashMap<String, String> }
    /// # impl Index for MockIndex {
    /// #     type Entry = String; type Path = String; type TreeId = u32;
    /// #     fn add(&mut self, e: Self::Entry) -> Result<(), VctrlError> { self.data.insert(e.clone(), e); Ok(()) }
    /// #     fn remove(&mut self, p: &Self::Path) -> Result<(), VctrlError> { self.data.remove(p); Ok(()) }
    /// #     fn clear(&mut self) -> Result<(), VctrlError> { self.data.clear(); Ok(()) }
    /// #     fn get(&self, p: &Self::Path) -> Result<Option<Self::Entry>, VctrlError> { Ok(self.data.get(p).cloned()) }
    /// #     fn contains(&self, p: &Self::Path) -> Result<bool, VctrlError> { Ok(self.data.contains_key(p)) }
    /// #     fn len(&self) -> Result<usize, VctrlError> { Ok(self.data.len()) }
    /// #     fn entries(&self) -> Result<Vec<Self::Entry>, VctrlError> { Ok(self.data.values().cloned().collect()) }
    /// #     fn write_tree(&self) -> Result<Self::TreeId, VctrlError> { Ok(1) }
    /// #     fn read_tree(&mut self, _t: &Self::TreeId) -> Result<(), VctrlError> { Ok(()) }
    /// # }
    /// let mut index = MockIndex::default();
    /// index.add("file.txt".to_string())?;
    /// assert!(index.get(&"file.txt".to_string())?.is_some());
    /// assert!(index.get(&"missing.txt".to_string())?.is_none());
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn get(&self, path: &Self::Path) -> Result<Option<Self::Entry>, VctrlError>;

    /// Checks if an entry exists by path.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the underlying storage cannot be read.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::index::Index;
    /// # use libvctrl_handler::VctrlError;
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct MockIndex { data: HashMap<String, String> }
    /// # impl Index for MockIndex {
    /// #     type Entry = String; type Path = String; type TreeId = u32;
    /// #     fn add(&mut self, e: Self::Entry) -> Result<(), VctrlError> { self.data.insert(e.clone(), e); Ok(()) }
    /// #     fn remove(&mut self, p: &Self::Path) -> Result<(), VctrlError> { self.data.remove(p); Ok(()) }
    /// #     fn clear(&mut self) -> Result<(), VctrlError> { self.data.clear(); Ok(()) }
    /// #     fn get(&self, p: &Self::Path) -> Result<Option<Self::Entry>, VctrlError> { Ok(self.data.get(p).cloned()) }
    /// #     fn contains(&self, p: &Self::Path) -> Result<bool, VctrlError> { Ok(self.data.contains_key(p)) }
    /// #     fn len(&self) -> Result<usize, VctrlError> { Ok(self.data.len()) }
    /// #     fn entries(&self) -> Result<Vec<Self::Entry>, VctrlError> { Ok(self.data.values().cloned().collect()) }
    /// #     fn write_tree(&self) -> Result<Self::TreeId, VctrlError> { Ok(1) }
    /// #     fn read_tree(&mut self, _t: &Self::TreeId) -> Result<(), VctrlError> { Ok(()) }
    /// # }
    /// let mut index = MockIndex::default();
    /// index.add("file.txt".to_string())?;
    /// assert!(index.contains(&"file.txt".to_string())?);
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn contains(&self, path: &Self::Path) -> Result<bool, VctrlError>;

    /// Returns the number of entries in the index.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the underlying storage cannot be read.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::index::Index;
    /// # use libvctrl_handler::VctrlError;
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct MockIndex { data: HashMap<String, String> }
    /// # impl Index for MockIndex {
    /// #     type Entry = String; type Path = String; type TreeId = u32;
    /// #     fn add(&mut self, e: Self::Entry) -> Result<(), VctrlError> { self.data.insert(e.clone(), e); Ok(()) }
    /// #     fn remove(&mut self, p: &Self::Path) -> Result<(), VctrlError> { self.data.remove(p); Ok(()) }
    /// #     fn clear(&mut self) -> Result<(), VctrlError> { self.data.clear(); Ok(()) }
    /// #     fn get(&self, p: &Self::Path) -> Result<Option<Self::Entry>, VctrlError> { Ok(self.data.get(p).cloned()) }
    /// #     fn contains(&self, p: &Self::Path) -> Result<bool, VctrlError> { Ok(self.data.contains_key(p)) }
    /// #     fn len(&self) -> Result<usize, VctrlError> { Ok(self.data.len()) }
    /// #     fn entries(&self) -> Result<Vec<Self::Entry>, VctrlError> { Ok(self.data.values().cloned().collect()) }
    /// #     fn write_tree(&self) -> Result<Self::TreeId, VctrlError> { Ok(1) }
    /// #     fn read_tree(&mut self, _t: &Self::TreeId) -> Result<(), VctrlError> { Ok(()) }
    /// # }
    /// let mut index = MockIndex::default();
    /// index.add("a".to_string())?;
    /// index.add("b".to_string())?;
    /// assert_eq!(index.len()?, 2);
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn len(&self) -> Result<usize, VctrlError>;

    /// Returns `true` if the index is empty.
    ///
    /// # How it works
    /// This is a provided method that default-implements by calling `len()`. It
    /// exists to provide ergonomic, self-documenting code at call sites.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the underlying storage cannot be read.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::index::Index;
    /// # use libvctrl_handler::VctrlError;
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct MockIndex { data: HashMap<String, String> }
    /// # impl Index for MockIndex {
    /// #     type Entry = String; type Path = String; type TreeId = u32;
    /// #     fn add(&mut self, e: Self::Entry) -> Result<(), VctrlError> { self.data.insert(e.clone(), e); Ok(()) }
    /// #     fn remove(&mut self, p: &Self::Path) -> Result<(), VctrlError> { self.data.remove(p); Ok(()) }
    /// #     fn clear(&mut self) -> Result<(), VctrlError> { self.data.clear(); Ok(()) }
    /// #     fn get(&self, p: &Self::Path) -> Result<Option<Self::Entry>, VctrlError> { Ok(self.data.get(p).cloned()) }
    /// #     fn contains(&self, p: &Self::Path) -> Result<bool, VctrlError> { Ok(self.data.contains_key(p)) }
    /// #     fn len(&self) -> Result<usize, VctrlError> { Ok(self.data.len()) }
    /// #     fn entries(&self) -> Result<Vec<Self::Entry>, VctrlError> { Ok(self.data.values().cloned().collect()) }
    /// #     fn write_tree(&self) -> Result<Self::TreeId, VctrlError> { Ok(1) }
    /// #     fn read_tree(&mut self, _t: &Self::TreeId) -> Result<(), VctrlError> { Ok(()) }
    /// # }
    /// let index = MockIndex::default();
    /// assert!(index.is_empty()?);
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn is_empty(&self) -> Result<bool, VctrlError> {
        Ok(self.len()? == 0)
    }

    /// Returns all entries in the index.
    ///
    /// # How it works
    /// Collects all staged entries into a `Vec`. This requires heap allocation.
    /// Callers should prefer `get` or `contains` if they only need to query a
    /// specific path, to avoid the overhead of collecting the entire index.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the underlying storage cannot be read.
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::index::Index;
    /// # use libvctrl_handler::VctrlError;
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct MockIndex { data: HashMap<String, String> }
    /// # impl Index for MockIndex {
    /// #     type Entry = String; type Path = String; type TreeId = u32;
    /// #     fn add(&mut self, e: Self::Entry) -> Result<(), VctrlError> { self.data.insert(e.clone(), e); Ok(()) }
    /// #     fn remove(&mut self, p: &Self::Path) -> Result<(), VctrlError> { self.data.remove(p); Ok(()) }
    /// #     fn clear(&mut self) -> Result<(), VctrlError> { self.data.clear(); Ok(()) }
    /// #     fn get(&self, p: &Self::Path) -> Result<Option<Self::Entry>, VctrlError> { Ok(self.data.get(p).cloned()) }
    /// #     fn contains(&self, p: &Self::Path) -> Result<bool, VctrlError> { Ok(self.data.contains_key(p)) }
    /// #     fn len(&self) -> Result<usize, VctrlError> { Ok(self.data.len()) }
    /// #     fn entries(&self) -> Result<Vec<Self::Entry>, VctrlError> { Ok(self.data.values().cloned().collect()) }
    /// #     fn write_tree(&self) -> Result<Self::TreeId, VctrlError> { Ok(1) }
    /// #     fn read_tree(&mut self, _t: &Self::TreeId) -> Result<(), VctrlError> { Ok(()) }
    /// # }
    /// let mut index = MockIndex::default();
    /// index.add("a".to_string())?;
    /// let entries = index.entries()?;
    /// assert_eq!(entries.len(), 1);
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn entries(&self) -> Result<Vec<Self::Entry>, VctrlError>;

    /// Writes the current index to a tree object and returns its identifier.
    ///
    /// # How it works
    /// Traverses the staged entries, recursively building tree objects for directories.
    /// It persists these trees to the `ObjectStore` (handled internally by the implementor)
    /// and returns the hash (or ID) of the root tree. This is the final step before
    /// creating a commit object.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the tree cannot be constructed or persisted, typically
    /// due to I/O failures or invalid index states (e.g., unsorted entries).
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::index::Index;
    /// # use libvctrl_handler::VctrlError;
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct MockIndex { data: HashMap<String, String> }
    /// # impl Index for MockIndex {
    /// #     type Entry = String; type Path = String; type TreeId = u32;
    /// #     fn add(&mut self, e: Self::Entry) -> Result<(), VctrlError> { self.data.insert(e.clone(), e); Ok(()) }
    /// #     fn remove(&mut self, p: &Self::Path) -> Result<(), VctrlError> { self.data.remove(p); Ok(()) }
    /// #     fn clear(&mut self) -> Result<(), VctrlError> { self.data.clear(); Ok(()) }
    /// #     fn get(&self, p: &Self::Path) -> Result<Option<Self::Entry>, VctrlError> { Ok(self.data.get(p).cloned()) }
    /// #     fn contains(&self, p: &Self::Path) -> Result<bool, VctrlError> { Ok(self.data.contains_key(p)) }
    /// #     fn len(&self) -> Result<usize, VctrlError> { Ok(self.data.len()) }
    /// #     fn entries(&self) -> Result<Vec<Self::Entry>, VctrlError> { Ok(self.data.values().cloned().collect()) }
    /// #     fn write_tree(&self) -> Result<Self::TreeId, VctrlError> { Ok(42) }
    /// #     fn read_tree(&mut self, _t: &Self::TreeId) -> Result<(), VctrlError> { Ok(()) }
    /// # }
    /// let mut index = MockIndex::default();
    /// index.add("file.txt".to_string())?;
    /// let tree_id = index.write_tree()?;
    /// assert_eq!(tree_id, 42);
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn write_tree(&self) -> Result<Self::TreeId, VctrlError>;

    /// Reads a tree into the index.
    ///
    /// # How it works
    /// Clears the current index state and populates it with the entries from the
    /// specified tree object. This is commonly used during `checkout` or `reset`
    /// operations to synchronize the staging area with a specific commit's state.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the tree cannot be found or if the index cannot be
    /// mutated (e.g., I/O errors).
    ///
    /// # Examples
    ///
    /// ```
    /// # use libvctrl_handler::traits::core::index::Index;
    /// # use libvctrl_handler::VctrlError;
    /// # use std::collections::HashMap;
    /// # #[derive(Default)]
    /// # struct MockIndex { data: HashMap<String, String> }
    /// # impl Index for MockIndex {
    /// #     type Entry = String; type Path = String; type TreeId = u32;
    /// #     fn add(&mut self, e: Self::Entry) -> Result<(), VctrlError> { self.data.insert(e.clone(), e); Ok(()) }
    /// #     fn remove(&mut self, p: &Self::Path) -> Result<(), VctrlError> { self.data.remove(p); Ok(()) }
    /// #     fn clear(&mut self) -> Result<(), VctrlError> { self.data.clear(); Ok(()) }
    /// #     fn get(&self, p: &Self::Path) -> Result<Option<Self::Entry>, VctrlError> { Ok(self.data.get(p).cloned()) }
    /// #     fn contains(&self, p: &Self::Path) -> Result<bool, VctrlError> { Ok(self.data.contains_key(p)) }
    /// #     fn len(&self) -> Result<usize, VctrlError> { Ok(self.data.len()) }
    /// #     fn entries(&self) -> Result<Vec<Self::Entry>, VctrlError> { Ok(self.data.values().cloned().collect()) }
    /// #     fn write_tree(&self) -> Result<Self::TreeId, VctrlError> { Ok(1) }
    /// #     fn read_tree(&mut self, _t: &Self::TreeId) -> Result<(), VctrlError> { Ok(()) }
    /// # }
    /// let mut index = MockIndex::default();
    /// index.read_tree(&99)?;
    /// assert!(index.is_empty()?); // Mock implementation does not populate
    /// # Ok::<(), VctrlError>(())
    /// ```
    fn read_tree(&mut self, tree: &Self::TreeId) -> Result<(), VctrlError>;
}
