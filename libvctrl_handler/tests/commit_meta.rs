use criterion as _;
use libvctrl_handler::{CommitMeta, VctrlError};
mod common;

#[test]
fn test_commit_meta_valid_boundaries() {
    let meta_min = common::ok(CommitMeta::new(123, -1440, None));
    assert_eq!(meta_min.timestamp(), 123);
    assert_eq!(meta_min.timezone_offset(), -1440);
    assert_eq!(meta_min.encoding(), None);

    let meta_zero = common::ok(CommitMeta::new(0, 0, Some("utf-8".into())));
    assert_eq!(meta_zero.timestamp(), 0);
    assert_eq!(meta_zero.timezone_offset(), 0);
    assert_eq!(meta_zero.encoding(), Some("utf-8"));

    let meta_max = common::ok(CommitMeta::new(456, 1440, Some("iso-8859-1".into())));
    assert_eq!(meta_max.timestamp(), 456);
    assert_eq!(meta_max.timezone_offset(), 1440);
    assert_eq!(meta_max.encoding(), Some("iso-8859-1"));
}

#[test]
fn test_commit_meta_invalid_timezone() {
    let result = CommitMeta::new(0, -1441, None);
    assert!(result.is_err());
    assert_eq!(
        common::err(result),
        VctrlError::InvalidTimezoneOffset(-1441)
    );

    let result = CommitMeta::new(0, 1441, None);
    assert!(result.is_err());
    assert_eq!(common::err(result), VctrlError::InvalidTimezoneOffset(1441));
}
