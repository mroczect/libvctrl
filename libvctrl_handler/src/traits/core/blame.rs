//! Blame trait and entry type.

use crate::VctrlError;
use crate::types::Hash;

/// A single line range in a file attributed to a commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlameEntry {
    /// The commit that last modified these lines.
    pub commit_id: Hash,
    /// The first line number (1-based).
    pub start_line: usize,
    /// The number of lines in this range.
    pub line_count: usize,
    /// The path of the file.
    pub path: String,
    /// An optional summary of the commit message.
    pub summary: Option<String>,
}

/// Trait for computing blame information for files.
pub trait Blame {
    /// Returns blame entries for the given file path.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the file cannot be found or the blame calculation fails.
    fn blame_file(&self, path: &str) -> Result<Vec<BlameEntry>, VctrlError>;
}
