use libvctrl_core::object::{BlobBuilder, CommitBuilder, TagBuilder, TreeBuilder};

mod common;

#[test]
fn test_blob_builder_build_success_via_public_api() {
    let result = BlobBuilder::new().with_data(vec![1, 2, 3]).build();
    assert!(
        result.is_ok(),
        "BlobBuilder should succeed with valid data via public API"
    );
}

#[test]
fn test_commit_builder_missing_tree_via_public_api() {
    let result = CommitBuilder::new().build();
    assert!(
        result.is_err(),
        "CommitBuilder should fail without tree via public API"
    );
}

#[test]
fn test_tag_builder_missing_name_via_public_api() {
    let result = TagBuilder::new().build();
    assert!(
        result.is_err(),
        "TagBuilder should fail without name via public API"
    );
}

#[test]
fn test_tag_builder_missing_target_via_public_api() {
    let result = TagBuilder::new().name("v1.0").build();
    assert!(
        result.is_err(),
        "TagBuilder should fail without target via public API"
    );
}

#[test]
fn test_tree_builder_build_empty_via_public_api() {
    let result = TreeBuilder::new().build();
    assert!(
        result.is_ok(),
        "TreeBuilder should succeed with empty entries via public API"
    );
}
