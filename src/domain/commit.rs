use super::hash::Hash;
use super::user::UserInfo;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub tree: Hash,
    pub parents: Vec<Hash>,
    pub author: UserInfo,
    pub committer: UserInfo,
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub signature: Option<Vec<u8>>,
}

impl Commit {
    pub fn new(
        tree: Hash,
        parents: Vec<Hash>,
        author: UserInfo,
        committer: UserInfo,
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
        }
    }
}
