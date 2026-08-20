use criterion as _;
use libvctrl_handler::EntryKind;
use libvctrl_handler::constants::entry_mode;
mod common;

#[test]
fn test_entry_kind_mode_matches_constants() {
    assert_eq!(EntryKind::Blob.mode(), entry_mode::BLOB);
    assert_eq!(EntryKind::Executable.mode(), entry_mode::EXECUTABLE);
    assert_eq!(EntryKind::Symlink.mode(), entry_mode::SYMLINK);
    assert_eq!(EntryKind::Tree.mode(), entry_mode::TREE);
    assert_eq!(EntryKind::Submodule.mode(), entry_mode::SUBMODULE);
}

#[test]
fn test_entry_kind_from_mode_roundtrip() {
    let kinds = [
        EntryKind::Blob,
        EntryKind::Executable,
        EntryKind::Symlink,
        EntryKind::Tree,
        EntryKind::Submodule,
    ];

    for kind in kinds {
        assert_eq!(EntryKind::from_mode(kind.mode()), Some(kind));
    }
}

#[test]
fn test_entry_kind_from_mode_invalid() {
    assert_eq!(EntryKind::from_mode(0), None);
    assert_eq!(EntryKind::from_mode(u32::MAX), None);
}
