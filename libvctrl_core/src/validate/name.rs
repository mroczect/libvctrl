use libvctrl_handler::{MAX_NAME_LENGTH, VctrlError};

pub fn validate_name(name: &str) -> Result<(), VctrlError> {
    if name.is_empty() {
        return Err(VctrlError::InvalidName("name is empty".into()));
    }
    if name.len() > MAX_NAME_LENGTH as usize {
        return Err(VctrlError::InvalidName(format!(
            "name exceeds maximum length {MAX_NAME_LENGTH}: '{name}'"
        )));
    }
    if name.contains('/') {
        return Err(VctrlError::InvalidName("name must not contain '/'".into()));
    }
    if name == "." || name == ".." {
        return Err(VctrlError::InvalidName(
            "name must not be '.' or '..'".into(),
        ));
    }
    Ok(())
}
