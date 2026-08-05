mod common;
use common::{blob_hash, setup_refs, setup_store};
use libvctrl::*;
use libvctrl::{Command, CreateBranch, DeleteBranch, GetBranch, RefStore, SetHead};

#[test]
fn branch_create_get_delete() {
    let mut store = setup_store();
    let mut refs = setup_refs();
    let hash = blob_hash(b"data");

    let create = CreateBranch {
        name: "refs/heads/feature".into(),
        hash,
    };
    create.execute(&mut store, &mut refs).unwrap();

    let get = GetBranch {
        name: "refs/heads/feature".into(),
    };
    assert_eq!(get.execute(&mut store, &mut refs).unwrap(), Some(hash));

    let delete = DeleteBranch {
        name: "refs/heads/feature".into(),
    };
    delete.execute(&mut store, &mut refs).unwrap();

    let get = GetBranch {
        name: "refs/heads/feature".into(),
    };
    assert!(get.execute(&mut store, &mut refs).unwrap().is_none());
}

#[test]
fn branch_invalid_name() {
    let mut store = setup_store();
    let mut refs = setup_refs();
    let hash = blob_hash(b"data");

    let create = CreateBranch {
        name: "invalid_name".into(),
        hash,
    };
    let err = create.execute(&mut store, &mut refs).unwrap_err();
    match err {
        libvctrl::VctrlError::InvalidRef(_) => {}
        _ => panic!("expected InvalidRef"),
    }
}

#[test]
fn set_head_works() {
    let mut store = setup_store();
    let mut refs = setup_refs();
    let hash = blob_hash(b"data");

    let create = CreateBranch {
        name: "refs/heads/main".into(),
        hash,
    };
    create.execute(&mut store, &mut refs).unwrap();

    let set_head = SetHead {
        target: "refs/heads/main".into(),
    };
    set_head.execute(&mut store, &mut refs).unwrap();

    assert_eq!(refs.head().unwrap(), Some(hash));
}

#[test]
fn set_head_to_nonexistent_branch() {
    let mut store = setup_store();
    let mut refs = setup_refs();
    let cmd = SetHead {
        target: "refs/heads/ghost".into(),
    };
    let err = cmd.execute(&mut store, &mut refs).unwrap_err();
    assert!(matches!(err, VctrlError::InvalidRef(_)));
}

#[test]
fn list_branches() {
    let mut store = setup_store();
    let mut refs = setup_refs();
    let hash = blob_hash(b"x");
    CreateBranch {
        name: "refs/heads/a".into(),
        hash,
    }
    .execute(&mut store, &mut refs)
    .unwrap();
    CreateBranch {
        name: "refs/heads/b".into(),
        hash,
    }
    .execute(&mut store, &mut refs)
    .unwrap();
    SetHead {
        target: "refs/heads/a".into(),
    }
    .execute(&mut store, &mut refs)
    .unwrap();

    let list = ListBranches.execute(&mut store, &mut refs).unwrap();
    assert_eq!(list.len(), 2);
    let a = list.iter().find(|(name, _, _)| name == "a").unwrap();
    assert!(a.2);
    let b = list.iter().find(|(name, _, _)| name == "b").unwrap();
    assert!(!b.2);
}
