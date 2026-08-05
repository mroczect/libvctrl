use crate::error::VctrlError;
use serde::{Deserialize, Serialize};

const MAX_NAME_LEN: usize = 255;
const MAX_EMAIL_LEN: usize = 255;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserID {
    pub name: String,
    pub email: String,
}

impl UserID {
    pub fn new(name: String, email: String) -> Result<Self, VctrlError> {
        if name.is_empty() || name.len() > MAX_NAME_LEN {
            return Err(VctrlError::Other(format!(
                "name must be 1..{} characters",
                MAX_NAME_LEN
            )));
        }
        if email.is_empty() || email.len() > MAX_EMAIL_LEN {
            return Err(VctrlError::Other(format!(
                "email must be 1..{} characters",
                MAX_EMAIL_LEN
            )));
        }
        Ok(Self { name, email })
    }
}
