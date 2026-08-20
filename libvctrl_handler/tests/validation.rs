use criterion as _;
use libvctrl_handler::{
    HASH_LENGTH, MAX_NAME_LENGTH, VctrlError, validate_hash_bytes, validate_name,
    validate_ref_name, validate_tree_entry_name,
};
mod common;

#[test]
fn test_validate_hash_bytes_valid() {
    let bytes = [0_u8; HASH_LENGTH];
    assert!(validate_hash_bytes(&bytes).is_ok());
}

#[test]
fn test_validate_hash_bytes_invalid() {
    let result = validate_hash_bytes(&[0_u8; 10]);
    assert!(result.is_err());
    assert_eq!(common::err(result), VctrlError::InvalidHashLength(10));
}

#[test]
fn test_validate_name_valid() {
    assert!(validate_name("file.txt").is_ok());
    assert!(validate_name("a").is_ok());
}

#[test]
fn test_validate_name_invalid_empty() {
    let result = validate_name("");
    assert!(result.is_err());
    assert_eq!(
        common::err(result),
        VctrlError::InvalidName("name is empty".to_string())
    );
}

#[test]
fn test_validate_name_invalid_too_long() {
    let max_len = usize::try_from(MAX_NAME_LENGTH).unwrap_or(usize::MAX);
    let name = "a".repeat(max_len + 1);
    let result = validate_name(&name);
    assert!(result.is_err());
    assert_eq!(
        common::err(result),
        VctrlError::InvalidName(format!(
            "name exceeds maximum length {MAX_NAME_LENGTH}: '{name}'"
        ))
    );
}

#[test]
fn test_validate_name_invalid_control_chars() {
    let name = "a\nb";
    let result = validate_name(name);
    assert!(result.is_err());
    assert_eq!(
        common::err(result),
        VctrlError::InvalidName(format!("name contains control characters: '{name}'"))
    );
}

#[test]
fn test_validate_ref_name_valid() {
    assert!(validate_ref_name("refs/heads/main").is_ok());
    assert!(validate_ref_name("v1.0.0").is_ok());
}

#[test]
fn test_validate_ref_name_invalid_cases() {
    let invalid_names = [
        "@",
        "/leading",
        "trailing/",
        "double//slash",
        "refs/.hidden",
        "refs/heads/main.lock",
        "refs/heads/main..",
        "refs/heads/main~1",
        "refs/heads/main^",
        "refs/heads/main:",
        "refs/heads/main?",
        "refs/heads/main*",
        "refs/heads/main[",
        "refs/heads/main\\",
        "refs/heads/main ",
        "refs/heads/main@{",
        "refs/heads/main<",
        "refs/heads/main>",
        "refs/heads/main|",
        "refs/heads/main\"",
    ];

    for name in invalid_names {
        assert!(
            validate_ref_name(name).is_err(),
            "expected invalid: '{name}'"
        );
    }
}

#[test]
fn test_validate_tree_entry_name_valid() {
    assert!(validate_tree_entry_name("file.txt").is_ok());
}

#[test]
fn test_validate_tree_entry_name_invalid() {
    let invalid_names = ["a/b", "a\\b", ".", ".."];

    for name in invalid_names {
        assert!(
            validate_tree_entry_name(name).is_err(),
            "expected invalid: '{name}'"
        );
    }
}
