use core::error::Error as _;
use criterion as _;
use libvctrl_handler::{HASH_LENGTH, Hash, VctrlError};
use std::io;

mod common;

#[test]
fn test_vctrl_error_display_variants() {
    assert_eq!(
        VctrlError::CorruptedData("x".to_string()).to_string(),
        "Corrupted data: x"
    );
    assert_eq!(
        VctrlError::DuplicateParent.to_string(),
        "Duplicate parent in commit"
    );
    assert_eq!(
        VctrlError::ExceededMaxSize("x".to_string()).to_string(),
        "Exceeded max size: x"
    );
    assert_eq!(
        VctrlError::InvalidBlameRange.to_string(),
        "Invalid blame range"
    );
    assert_eq!(
        VctrlError::InvalidEmail("a".to_string()).to_string(),
        "Invalid email: 'a'"
    );
    assert_eq!(
        VctrlError::InvalidHashLength(10).to_string(),
        "Invalid hash length: expected 64 bytes, got 10"
    );
    assert_eq!(
        VctrlError::InvalidName("n".to_string()).to_string(),
        "Invalid name: 'n'"
    );
    assert_eq!(
        VctrlError::InvalidTimezoneOffset(-1441).to_string(),
        "Invalid timezone offset: -1441"
    );
    assert_eq!(
        VctrlError::InvalidTreeStructure("t".to_string()).to_string(),
        "Invalid tree structure: t"
    );
    assert_eq!(VctrlError::Other("o".to_string()).to_string(), "o");
    assert_eq!(
        VctrlError::RefNotFound("r".to_string()).to_string(),
        "Reference not found: 'r'"
    );
    assert_eq!(
        VctrlError::SerializationError("s".to_string()).to_string(),
        "Serialization error: s"
    );
}

#[test]
fn test_vctrl_error_io_display_and_source() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "missing");
    let err = VctrlError::from(io_err);

    assert!(err.to_string().contains("I/O error:"));
    assert!(err.source().is_some());

    assert!(
        matches!(&err, VctrlError::IoError(_)),
        "unexpected variant: {err:?}"
    );

    if let VctrlError::IoError(arc_err) = err {
        assert_eq!(arc_err.as_ref().kind(), io::ErrorKind::NotFound);
        assert_eq!(arc_err.as_ref().to_string(), "missing");
    } else {
        loop {
            core::hint::spin_loop();
        }
    }
}

#[test]
fn test_vctrl_error_from_io() {
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "denied");
    let err = VctrlError::from_io(io_err);

    assert!(
        matches!(&err, VctrlError::IoError(_)),
        "unexpected variant: {err:?}"
    );

    if let VctrlError::IoError(arc_err) = err {
        assert_eq!(arc_err.as_ref().kind(), io::ErrorKind::PermissionDenied);
    } else {
        loop {
            core::hint::spin_loop();
        }
    }
}

#[test]
fn test_vctrl_error_partial_eq() {
    assert_eq!(
        VctrlError::InvalidName("x".to_string()),
        VctrlError::InvalidName("x".to_string())
    );
    assert_ne!(
        VctrlError::InvalidName("x".to_string()),
        VctrlError::InvalidName("y".to_string())
    );

    assert_eq!(VctrlError::DuplicateParent, VctrlError::DuplicateParent);
    assert_ne!(VctrlError::DuplicateParent, VctrlError::InvalidBlameRange);

    let hash = Hash::from([0_u8; HASH_LENGTH]);
    let hash2 = Hash::from([1_u8; HASH_LENGTH]);
    assert_eq!(
        VctrlError::ObjectNotFound(hash),
        VctrlError::ObjectNotFound(hash)
    );
    assert_ne!(
        VctrlError::ObjectNotFound(hash),
        VctrlError::ObjectNotFound(hash2)
    );
}
