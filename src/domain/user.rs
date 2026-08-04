use crate::error::VctrlError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserID {
    pub name: String,
    pub email: String,
}

impl UserID {
    pub fn new(name: String, email: String) -> Result<Self, VctrlError> {
        if name.is_empty() || email.is_empty() {
            return Err(VctrlError::Other("name and email must not be empty".into()));
        }
        Ok(Self { name, email })
    }
}
