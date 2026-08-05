use crate::domain::hash::Hash;
use crate::error::VctrlError;

#[derive(Debug, Clone)]
pub struct ReflogEntry {
    pub ref_name: String,
    pub old_hash: Option<Hash>,
    pub new_hash: Hash,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub message: String,
}

pub trait ReflogStore {
    fn log_ref_update(
        &mut self,
        ref_name: &str,
        old_hash: Option<Hash>,
        new_hash: Hash,
        message: &str,
    ) -> Result<(), VctrlError>;
    fn reflog(&self, ref_name: &str) -> Result<Vec<ReflogEntry>, VctrlError>;
}
