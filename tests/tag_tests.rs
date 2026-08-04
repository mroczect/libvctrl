mod common;
use common::{alice, blob_hash, encoder, hasher, setup_refs, setup_store};
use libvctrl::ObjectStore;
use libvctrl::RefStore;
use libvctrl::{Command, CreateAnnotatedTag, CreateLightweightTag, DeleteTag, ListTags};

#[test]
fn test_lightweight_tag() {
    let mut store = setup_store();
    let mut refs = setup_refs();
    let commit_hash = blob_hash(b"some commit");

    let cmd = CreateLightweightTag {
        name: "v1.0".into(),
        target: commit_hash,
    };
    cmd.execute(&mut store, &mut refs).unwrap();

    let ref_name = "refs/tags/v1.0";
    assert_eq!(refs.get_ref(ref_name).unwrap(), Some(commit_hash));
}

#[test]
fn test_annotated_tag() {
    let mut store = setup_store();
    let mut refs = setup_refs();
    let commit_hash = blob_hash(b"some commit");

    let cmd = CreateAnnotatedTag {
        name: "v2.0".into(),
        target: commit_hash,
        tagger: alice(),
        message: "Release v2.0".into(),
        encoder: Box::new(encoder()),
        hasher: Box::new(hasher()),
    };
    let tag_hash = cmd.execute(&mut store, &mut refs).unwrap();
    assert!(store.exists(&tag_hash).unwrap());

    let ref_name = "refs/tags/v2.0";
    assert_eq!(refs.get_ref(ref_name).unwrap(), Some(tag_hash));
}

#[test]
fn test_delete_tag() {
    let mut store = setup_store();
    let mut refs = setup_refs();
    let commit_hash = blob_hash(b"some commit");

    CreateLightweightTag {
        name: "v1.0".into(),
        target: commit_hash,
    }
    .execute(&mut store, &mut refs)
    .unwrap();

    DeleteTag {
        name: "v1.0".into(),
    }
    .execute(&mut store, &mut refs)
    .unwrap();

    let ref_name = "refs/tags/v1.0";
    assert!(refs.get_ref(ref_name).unwrap().is_none());
}

#[test]
fn test_list_tags() {
    let mut store = setup_store();
    let mut refs = setup_refs();
    let hash = blob_hash(b"commit");

    CreateLightweightTag {
        name: "v1".into(),
        target: hash,
    }
    .execute(&mut store, &mut refs)
    .unwrap();
    CreateLightweightTag {
        name: "v2".into(),
        target: hash,
    }
    .execute(&mut store, &mut refs)
    .unwrap();

    let tags = ListTags.execute(&mut store, &mut refs).unwrap();
    assert!(tags.contains(&"v1".to_string()));
    assert!(tags.contains(&"v2".to_string()));
}
