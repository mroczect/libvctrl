use libvctrl_sha512 as _;
use proptest as _;

use libvctrl_core::object::{
    BlobBuilder, CommitBuilder, TagBuilder, TreeBuilder, TreeEntryBuilder,
};
use libvctrl_handler::{EntryKind, UserID, VctrlError};

pub mod common;

fn make_user(name: &str, email: &str) -> Result<UserID, VctrlError> {
    UserID::new(name.to_string(), email.to_string())
}

#[test]
fn builder_chain_public_api() -> Result<(), VctrlError> {
    let hash = common::make_hash(0x77)?;
    let entry = TreeEntryBuilder::new("file".to_string(), EntryKind::Blob, hash).build()?;
    let _tree = TreeBuilder::new().entry(entry).build()?;

    let blob = BlobBuilder::new().with_data(vec![1_u8, 2]).build()?;
    assert_eq!(blob.data(), &[1_u8, 2]);

    let commit = CommitBuilder::new()
        .tree(common::make_hash(0x78)?)
        .author(make_user("Alice", "alice@example.com")?)
        .committer(make_user("Bob", "bob@example.com")?)
        .message("builder commit")
        .build()?;
    assert_eq!(commit.message(), "builder commit");

    let tag = TagBuilder::new()
        .name("v1")
        .target(common::make_hash(0x79)?)
        .message("builder tag")
        .build()?;
    assert_eq!(tag.name(), "v1");

    Ok(())
}
