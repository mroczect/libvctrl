use crate::errors::VctrlError;
use crate::types::validate_name;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserID {
    name: String,
    email: String,
}

impl UserID {
    pub fn new(name: String, email: String) -> Result<Self, VctrlError> {
        validate_name(&name)?;
        if email.is_empty()
            || !email.contains('@')
            || email.starts_with('@')
            || email.ends_with('@')
            || email.contains(' ')
        {
            return Err(VctrlError::InvalidEmail(format!(
                "invalid email: '{email}'"
            )));
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
