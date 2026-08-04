use super::hash::Hash;
use super::user::UserID;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub target: Hash,
    pub tagger: UserID,
    pub timestamp: DateTime<Utc>,
    pub message: String,
}

impl Tag {
    pub fn new(target: Hash, tagger: UserID, message: String) -> Self {
        Self {
            target,
            tagger,
            timestamp: Utc::now(),
            message,
        }
    }
}
