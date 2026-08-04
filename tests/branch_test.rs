mod common;
use common::setup_refs;

use libvctrl::{
    Blob, RefStore, VctrlError, create_branch, delete_branch, get_branch, set_head_branch,
};

#[test]
fn branch_create_get_delete() {
    let mut refs = setup_refs();
    let hash = Blob::new(b"data".to_vec()).hash().unwrap();

    create_branch(&mut refs, "refs/heads/feature", &hash).unwrap();
    let got = get_branch(&refs, "refs/heads/feature").unwrap();
    assert_eq!(got, Some(hash));

    delete_branch(&mut refs, "refs/heads/feature").unwrap();
    let got = get_branch(&refs, "refs/heads/feature").unwrap();
    assert!(got.is_none());
}

#[test]
fn branch_invalid_name() {
    let mut refs = setup_refs();
    let hash = Blob::new(b"data".to_vec()).hash().unwrap();

    let err = create_branch(&mut refs, "invalid_name", &hash).unwrap_err();
    match err {
        VctrlError::InvalidRef(_) => {}
        _ => panic!("expected InvalidRef error"),
    }
}

#[test]
fn set_head_branch_works() {
    let mut refs = setup_refs();
    let hash = Blob::new(b"data".to_vec()).hash().unwrap();

    create_branch(&mut refs, "refs/heads/main", &hash).unwrap();
    set_head_branch(&mut refs, "refs/heads/main").unwrap();

    let head = refs.head().unwrap().unwrap();
    assert_eq!(head, hash);
}
