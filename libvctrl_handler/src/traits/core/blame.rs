use crate::errors::VctrlError;
use crate::types::Hash;

/// A single line range in a file attributed to a commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlameEntry {
    commit_id: Hash,
    start_line: usize,
    line_count: usize,
    path: String,
    summary: Option<String>,
}

impl BlameEntry {
    /// Creates a new `BlameEntry`.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError::InvalidBlameRange`] if `start_line` is 0 or `line_count` is 0.
    pub fn new(
        commit_id: Hash,
        start_line: usize,
        line_count: usize,
        path: String,
        summary: Option<String>,
    ) -> Result<Self, VctrlError> {
        if start_line == 0 || line_count == 0 {
            return Err(VctrlError::InvalidBlameRange);
        }
        Ok(Self {
            commit_id,
            start_line,
            line_count,
            path,
            summary,
        })
    }

    /// Returns the commit that last modified these lines.
    #[must_use]
    pub const fn commit_id(&self) -> Hash {
        self.commit_id
    }

    /// Returns the first line number (1-based).
    #[must_use]
    pub const fn start_line(&self) -> usize {
        self.start_line
    }

    /// Returns the number of lines in this range.
    #[must_use]
    pub const fn line_count(&self) -> usize {
        self.line_count
    }

    /// Returns the path of the file.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns an optional summary of the commit message.
    #[must_use]
    pub fn summary(&self) -> Option<&str> {
        self.summary.as_deref()
    }
}

/// Trait for computing blame information for files.
pub trait Blame: Send + Sync {
    /// Returns blame entries for the given file path.
    ///
    /// # Errors
    ///
    /// Returns [`VctrlError`] if the file cannot be found or the blame calculation fails.
    fn blame_file(&self, path: &str) -> Result<Vec<BlameEntry>, VctrlError>;
}
