














use crate::errors::VctrlError;
use crate::types::Hash;





























#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlameEntry {
    commit_id: Hash,
    start_line: usize,
    line_count: usize,
    path: String,
    summary: Option<String>,
}

impl BlameEntry {
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
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

    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    #[must_use]
    pub const fn commit_id(&self) -> Hash {
        self.commit_id
    }

    
    
    
    
    
    
    
    
    
    
    
    #[must_use]
    pub const fn start_line(&self) -> usize {
        self.start_line
    }

    
    
    
    
    
    
    
    
    
    
    
    #[must_use]
    pub const fn line_count(&self) -> usize {
        self.line_count
    }

    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    #[must_use]
    pub fn summary(&self) -> Option<&str> {
        self.summary.as_deref()
    }
}





































pub trait Blame: Send + Sync {
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    fn blame_file(&self, path: &str) -> Result<Vec<BlameEntry>, VctrlError>;
}
