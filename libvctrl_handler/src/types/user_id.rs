use crate::errors::VctrlError;

use super::validate_name;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserID {
    name: String,
    email: String,
}

impl UserID {
    pub fn new(name: String, email: String) -> Result<Self, VctrlError> {
        validate_name(&name)?;
        if email.is_empty() {
            return Err(VctrlError::InvalidName("email is empty".into()));
        }
        Ok(Self { name, email })
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn email(&self) -> &str {
        &self.email
    }
}
