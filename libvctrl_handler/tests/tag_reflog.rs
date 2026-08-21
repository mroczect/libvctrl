use criterion as _;
use libvctrl_handler::{
    CommitMeta, HASH_LENGTH, Hash, MAX_MESSAGE_LENGTH, ReflogEntry, Tag, UserID, VctrlError,
};
mod common;

fn h(byte: u8) -> Hash {
    Hash::from([byte; HASH_LENGTH])
}

fn tagger() -> UserID {
    common::ok(UserID::new(
        "Tagger".to_string(),
        "tagger@example.com".to_string(),
    ))
}

#[test]
fn test_tag_valid_with_meta() {
    let target = h(1);
    let tagger = tagger();
    let meta = common::ok(CommitMeta::new(1_700_000_000, 300, None));

    let tag = common::ok(Tag::with_meta(
        "v1.0.0".to_string(),
        target,
        Some(tagger.clone()),
        "release 1.0.0".to_string(),
        meta,
    ));

    assert_eq!(tag.name(), "v1.0.0");
    assert_eq!(tag.target(), &target);
    assert_eq!(tag.tagger(), Some(&tagger));
    assert_eq!(tag.message(), "release 1.0.0");
    assert_eq!(tag.meta().timestamp(), 1_700_000_000);
    assert_eq!(tag.meta().timezone_offset(), 300);
}

#[test]
fn test_tag_invalid_ref_name() {
    let target = h(1);
    let result = Tag::new("bad name".to_string(), target, None, "message".to_string());
    assert!(result.is_err());
}

#[test]
fn test_tag_message_too_long() {
    let target = h(1);
    let max_msg = usize::try_from(MAX_MESSAGE_LENGTH).unwrap_or(usize::MAX);
    let message = "a".repeat(max_msg + 1);

    let result = Tag::new("v1.0.0".to_string(), target, None, message);
    assert!(result.is_err());

    let err = common::err(result);
    assert!(
        matches!(&err, VctrlError::ExceededMaxSize(_)),
        "unexpected error: {err:?}"
    );
}

#[test]
fn test_reflog_entry_valid() {
    let old = Some(h(1));
    let new = Some(h(2));
    let entry = common::ok(ReflogEntry::new(
        old,
        new,
        "update".to_string(),
        1_700_000_000,
        120,
    ));

    assert_eq!(entry.old_id(), old);
    assert_eq!(entry.new_id(), new);
    assert_eq!(entry.reason(), "update");
    assert_eq!(entry.timestamp(), 1_700_000_000);
    assert_eq!(entry.timezone_offset(), 120);
}

#[test]
fn test_reflog_entry_invalid_timezone() {
    let result = ReflogEntry::new(None, None, "update".to_string(), 1_700_000_000, -2000);
    assert!(result.is_err());
    assert_eq!(
        common::err(result),
        VctrlError::InvalidTimezoneOffset(-2000)
    );
}
