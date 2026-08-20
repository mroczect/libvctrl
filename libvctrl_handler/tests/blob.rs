use criterion as _;
use libvctrl_handler::{Blob, MAX_BLOB_SIZE, VctrlError};
mod common;

#[test]
fn test_blob_valid_empty() {
    let blob = common::ok(Blob::new(Vec::new()));
    let empty: &[u8] = &[];
    assert!(blob.is_empty());
    assert_eq!(blob.size(), 0);
    assert_eq!(blob.data(), empty);
}

#[test]
fn test_blob_valid_small() {
    let data = vec![1, 2, 3, 4];
    let blob = common::ok(Blob::new(data.clone()));
    assert!(!blob.is_empty());
    assert_eq!(blob.size(), 4);
    assert_eq!(blob.data(), data.as_slice());
}

#[test]
fn test_blob_exceeds_max_size() {
    let max_len = usize::try_from(MAX_BLOB_SIZE).unwrap_or(usize::MAX);
    let data = vec![0_u8; max_len + 1];
    let result = Blob::new(data);
    assert!(result.is_err());

    let expected_msg = format!(
        "blob size {} exceeds maximum allowed size {}",
        max_len + 1,
        MAX_BLOB_SIZE
    );
    assert_eq!(
        common::err(result),
        VctrlError::ExceededMaxSize(expected_msg)
    );
}
