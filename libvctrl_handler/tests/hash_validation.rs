#![allow(missing_docs)]

use libvctrl_handler::*;
use std::error::Error as StdError;

// ---------------------------------------------------------------------------
fn make_hash(byte: u8) -> Hash {
    Hash::from_bytes(&[byte; HASH_LENGTH]).unwrap()
}

#[test]
fn from_bytes_valid_length() {
    let hash = make_hash(0xAA);
    assert_eq!(hash.as_bytes(), &[0xAA; HASH_LENGTH]);
}

#[test]
fn from_bytes_too_short() {
    let err = Hash::from_bytes(&[0; 10]).unwrap_err();
    assert_eq!(err, VctrlError::InvalidHashLength(10));
}

#[test]
fn from_bytes_too_long() {
    let err = Hash::from_bytes(&[0; 100]).unwrap_err();
    assert_eq!(err, VctrlError::InvalidHashLength(100));
}

#[test]
fn as_bytes_matches_input() {
    let hash = make_hash(0x11);
    assert_eq!(hash.as_bytes(), &[0x11; HASH_LENGTH]);
}

#[test]
fn display_is_full_hex() {
    let hash = make_hash(0xAB);
    let s = hash.to_string();
    assert_eq!(s.len(), HASH_LENGTH * 2);
    assert!(s.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn debug_format_contains_short_hex_and_ellipsis() {
    let hash = make_hash(0xCD);
    let dbg = format!("{hash:?}");
    assert!(dbg.starts_with("Hash("));
    assert!(dbg.ends_with("…)"));
    assert!(dbg.contains("cdcd"));
}

#[test]
fn hash_is_copy_and_clone() {
    let h1 = make_hash(1);
    let h2 = h1;
    let _ = h2;
    assert_eq!(h1, h2);
}

#[test]
fn hash_partial_eq_and_eq() {
    let a = make_hash(5);
    let b = make_hash(5);
    let c = make_hash(6);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn hash_ord_and_sort() {
    let low = make_hash(0);
    let high = make_hash(1);
    assert!(low < high);
    let mut v = vec![high, low];
    v.sort();
    assert_eq!(v, vec![low, high]);
}

#[test]
fn hash_std_hash_usable_in_hashset() {
    use std::collections::HashSet;
    let a = make_hash(10);
    let b = make_hash(10);
    let mut set = HashSet::new();
    assert!(set.insert(a));
    assert!(!set.insert(b));
    assert!(set.contains(&a));
}

#[test]
fn vctrl_error_display_includes_message() {
    let e = VctrlError::InvalidName("test-name".into());
    let msg = e.to_string();
    assert!(msg.contains("Invalid name"));
    assert!(msg.contains("test-name"));
}

#[test]
fn vctrl_error_debug_and_clone() {
    let e = VctrlError::ObjectNotFound(make_hash(0xFF));
    let cloned = e.clone();
    assert_eq!(e, cloned);
    let dbg = format!("{cloned:?}");
    assert!(dbg.contains("ObjectNotFound"));
}

#[test]
fn vctrl_error_is_std_error_and_source_none() {
    let e = VctrlError::Other("oops".into());
    assert!(e.source().is_none());
}

#[test]
fn constants_have_expected_values() {
    assert_eq!(HASH_LENGTH, 64);
    assert_eq!(MAX_NAME_LENGTH, 255);
}
