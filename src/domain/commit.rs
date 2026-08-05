use super::hash::Hash;
use super::user::UserID;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub tree: Hash,
    pub parents: Vec<Hash>,
    pub author: UserID,
    pub committer: UserID,
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub signature: Option<Vec<u8>>,
    pub headers: Vec<(String, String)>,
}

impl Commit {
    pub fn new(
        tree: Hash,
        parents: Vec<Hash>,
        author: UserID,
        committer: UserID,
        message: String,
        signature: Option<Vec<u8>>,
    ) -> Self {
        Self {
            tree,
            parents,
            author,
            committer,
            timestamp: Utc::now(),
            message,
            signature,
            headers: Vec::new(),
        }
    }
}
