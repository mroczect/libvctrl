//! Builder for [`Tree`] and [`TreeEntry`] objects.

use libvctrl_handler::{EntryKind, Hash, Tree, TreeEntry, VctrlError};

/// Builder for [`Tree`] objects.
///
/// Entries are collected and then validated upon [`build`](TreeBuilder::build).
#[derive(Debug, Default)]
pub struct TreeBuilder {
    entries: Vec<TreeEntry>,
}

impl TreeBuilder {
    /// Creates an empty builder.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Adds a pre‑built entry.
    #[must_use]
    pub fn entry(mut self, entry: TreeEntry) -> Self {
        self.entries.push(entry);
        self
    }

    /// Convenience method to add an entry from parts.
    ///
    /// # Errors
    /// Returns [`VctrlError::InvalidName`] if the name is invalid.
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

    /// Builds the tree, ensuring entries are sorted and unique.
    ///
    /// # Errors
    /// Returns an error if entries are not sorted or contain duplicates.
    pub fn build(self) -> Result<Tree, VctrlError> {
        Tree::new(self.entries)
    }
}

/// Builder for a single [`TreeEntry`].
#[derive(Debug)]
pub struct TreeEntryBuilder {
    name: String,
    kind: EntryKind,
    hash: Hash,
}

impl TreeEntryBuilder {
    /// Starts a new entry builder.
    #[must_use]
    pub const fn new(name: String, kind: EntryKind, hash: Hash) -> Self {
        Self { name, kind, hash }
    }

    /// Builds the entry.
    ///
    /// # Errors
    /// Returns [`VctrlError::InvalidName`] if the name is invalid.
    pub fn build(self) -> Result<TreeEntry, VctrlError> {
        TreeEntry::new(self.name, self.kind, self.hash)
    }
}
