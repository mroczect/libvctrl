use criterion as _;
use libvctrl_handler::{MAX_NAME_LENGTH, UserID, VctrlError};
mod common;

#[test]
fn test_user_id_valid() {
    let user = common::ok(UserID::new(
        "Alice".to_string(),
        "alice@example.com".to_string(),
    ));
    assert_eq!(user.name(), "Alice");
    assert_eq!(user.email(), "alice@example.com");
}

#[test]
fn test_user_id_invalid_empty_name() {
    let result = UserID::new(String::new(), "alice@example.com".to_string());
    assert!(result.is_err());
    assert_eq!(
        common::err(result),
        VctrlError::InvalidName("user name is empty".to_string())
    );
}

#[test]
fn test_user_id_invalid_name_too_long() {
    let max_len = usize::try_from(MAX_NAME_LENGTH).unwrap_or(usize::MAX);
    let name = "a".repeat(max_len + 1);
    let result = UserID::new(name, "alice@example.com".to_string());
    assert!(result.is_err());
    assert_eq!(
        common::err(result),
        VctrlError::InvalidName(format!(
            "user name exceeds maximum length {MAX_NAME_LENGTH}"
        ))
    );
}

#[test]
fn test_user_id_invalid_name_control_chars() {
    let name = "Alice\nBob".to_string();
    let result = UserID::new(name.clone(), "alice@example.com".to_string());
    assert!(result.is_err());
    assert_eq!(
        common::err(result),
        VctrlError::InvalidName(format!("user name contains control characters: '{name}'"))
    );
}

#[test]
fn test_user_id_invalid_empty_email() {
    let result = UserID::new("Alice".to_string(), String::new());
    assert!(result.is_err());
    assert_eq!(
        common::err(result),
        VctrlError::InvalidEmail("email is empty".to_string())
    );
}

#[test]
fn test_user_id_invalid_email_no_at() {
    let email = "alice.example.com".to_string();
    let result = UserID::new("Alice".to_string(), email.clone());
    assert!(result.is_err());
    assert_eq!(
        common::err(result),
        VctrlError::InvalidEmail(format!("email must contain '@': '{email}'"))
    );
}

#[test]
fn test_user_id_invalid_email_too_long() {
    let max_len = usize::try_from(MAX_NAME_LENGTH).unwrap_or(usize::MAX);
    let email = format!("{}@example.com", "a".repeat(max_len + 1));
    let result = UserID::new("Alice".to_string(), email);
    assert!(result.is_err());
    assert_eq!(
        common::err(result),
        VctrlError::InvalidEmail(format!("email exceeds maximum length {MAX_NAME_LENGTH}"))
    );
}

#[test]
fn test_user_id_invalid_email_control_chars() {
    let email = "alice@example.com\n".to_string();
    let result = UserID::new("Alice".to_string(), email.clone());
    assert!(result.is_err());
    assert_eq!(
        common::err(result),
        VctrlError::InvalidEmail(format!("email contains control characters: '{email}'"))
    );
}
