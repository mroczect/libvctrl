pub use age_credentials::UserID;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub name: String,
    pub email: String,
}

impl UserInfo {
    pub fn new(name: String, email: String) -> Self {
        Self { name, email }
    }
}
