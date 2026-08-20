use criterion as _;
use libvctrl_handler::constants::HASH_LENGTH;
use libvctrl_handler::{Hash, VctrlError};
mod common;

fn valid_hex() -> String {
    use core::fmt::Write;

    let mut s = String::with_capacity(HASH_LENGTH * 2);
    for b in 0..HASH_LENGTH {
        let _ = write!(s, "{b:02x}");
    }
    s
}

#[test]
fn test_hash_from_bytes_valid() {
    let bytes = [7_u8; HASH_LENGTH];
    let hash = common::ok(Hash::from_bytes(&bytes));
    assert_eq!(&hash.as_bytes()[..], &bytes[..]);
}

#[test]
fn test_hash_from_bytes_invalid_length() {
    let result = Hash::from_bytes(&[0_u8; 10]);
    assert!(result.is_err());
    assert_eq!(common::err(result), VctrlError::InvalidHashLength(10));
}

#[test]
fn test_hash_from_array() {
    let arr = [1_u8; HASH_LENGTH];
    let hash = Hash::from(arr);
    assert_eq!(&hash.as_bytes()[..], &arr[..]);
}

#[test]
fn test_hash_try_from_slice_valid() {
    let arr = [2_u8; HASH_LENGTH];
    let hash: Hash = common::ok(Hash::try_from(&arr[..]));
    assert_eq!(&hash.as_bytes()[..], &arr[..]);
}

#[test]
fn test_hash_try_from_slice_invalid() {
    let result: Result<Hash, _> = Hash::try_from(&[0_u8; 3][..]);
    assert!(result.is_err());
}

#[test]
fn test_hash_as_ref() {
    let arr = [3_u8; HASH_LENGTH];
    let hash = Hash::from(arr);
    assert_eq!(hash.as_ref(), &arr[..]);
}

#[test]
fn test_hash_from_str_valid() {
    let s = valid_hex();
    let expected: Vec<u8> = (0..HASH_LENGTH)
        .map(|i| u8::try_from(i).unwrap_or(0))
        .collect();
    let hash = common::ok(s.parse::<Hash>());
    assert_eq!(&hash.as_bytes()[..], expected.as_slice());
}

#[test]
fn test_hash_from_str_invalid_length() {
    let result = "abc".parse::<Hash>();
    assert!(result.is_err());
    assert_eq!(common::err(result), VctrlError::InvalidHashLength(3));
}

#[test]
fn test_hash_from_str_invalid_hex() {
    let s = "zz".repeat(HASH_LENGTH);
    let result = s.parse::<Hash>();
    assert!(result.is_err());

    let err = common::err(result);
    assert!(
        matches!(&err, VctrlError::CorruptedData(_)),
        "unexpected error: {err:?}"
    );

    if let VctrlError::CorruptedData(msg) = err {
        assert!(msg.contains("invalid hex char in hash"));
    } else {
        loop {
            core::hint::spin_loop();
        }
    }
}

#[test]
fn test_hash_display() {
    let s = valid_hex();
    let hash = common::ok(s.parse::<Hash>());
    assert_eq!(hash.to_string(), s);
}

#[test]
fn test_hash_debug() {
    let s = valid_hex();
    let hash = common::ok(s.parse::<Hash>());
    let dbg = format!("{hash:?}");
    assert!(dbg.starts_with("Hash("));
    assert!(dbg.contains("..."));
    assert!(dbg.ends_with(')'));
}
