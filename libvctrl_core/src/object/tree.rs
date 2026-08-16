use libvctrl_handler::{EntryKind, Hash, Tree, TreeEntry, VctrlError};

/// A builder for creating [`Tree`] objects.
#[derive(Debug, Default)]
pub struct TreeBuilder {
    entries: Vec<TreeEntry>,
}

impl TreeBuilder {
    /// Creates a new `TreeBuilder`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Adds an existing [`TreeEntry`].
    #[must_use]
    pub fn entry(mut self, entry: TreeEntry) -> Self {
        self.entries.push(entry);
        self
    }

    /// Creates and adds a new [`TreeEntry`].
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the entry name is invalid.
    pub fn add_entry(
        mut self,
        name: String,
        kind: EntryKind,
        hash: Hash,
    ) -> Result<Self, VctrlError> {
        let entry = TreeEntry::new(name, kind, hash)?;
        self.entries.push(entry);
        Ok(self)
    }

    /// Builds the [`Tree`].
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the entries are not sorted or contain duplicates.
    pub fn build(self) -> Result<Tree, VctrlError> {
        Tree::new(self.entries)
    }
}

/// A builder for creating [`TreeEntry`] objects.
#[derive(Debug)]
pub struct TreeEntryBuilder {
    name: String,
    kind: EntryKind,
    hash: Hash,
}

impl TreeEntryBuilder {
    /// Creates a new `TreeEntryBuilder`.
    #[must_use]
    pub const fn new(name: String, kind: EntryKind, hash: Hash) -> Self {
        Self { name, kind, hash }
    }

    /// Builds the [`TreeEntry`].
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the entry name is invalid.
    pub fn build(self) -> Result<TreeEntry, VctrlError> {
        TreeEntry::new(self.name, self.kind, self.hash)
    }
}
