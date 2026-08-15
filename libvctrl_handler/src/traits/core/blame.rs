use crate::{Hash, VctrlError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlameEntry {
    pub commit_id: Hash,

    pub start_line: usize,

    pub line_count: usize,

    pub path: String,

    pub summary: Option<String>,
}

pub trait Blame {
    type CommitId;

    type Path;

    fn blame_file(&self, path: &Self::Path) -> Result<Vec<BlameEntry>, VctrlError>;
}
