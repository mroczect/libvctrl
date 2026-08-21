use std::io::Read;

use libvctrl_sha512 as _;
use proptest as _;

use libvctrl_core::store::{MemoryRefStore, MemoryStore};
use libvctrl_handler::{ObjectStore, RefStore, VctrlError};

pub mod common;

#[test]
fn memory_store_put_get_delete_exists() -> Result<(), VctrlError> {
    let mut store = MemoryStore::new();
    let hash = common::make_hash(0xAA)?;
    let data = vec![1_u8, 2, 3, 4];

    store.put(&hash, &data)?;

    {
        let mut reader = store.get(&hash)?;
        let mut buf = Vec::new();
        let _ = reader.read_to_end(&mut buf)?;
        assert_eq!(buf, data);
    }

    assert!(store.exists(&hash)?);
    store.delete(&hash)?;
    assert!(!store.exists(&hash)?);

    Ok(())
}

#[test]
fn memory_store_get_missing_errors() -> Result<(), VctrlError> {
    let store = MemoryStore::new();
    let hash = common::make_hash(0xBB)?;
    let result = store.get(&hash);
    assert!(matches!(result, Err(VctrlError::ObjectNotFound(_))));
    Ok(())
}

#[test]
fn memory_ref_store_roundtrip_and_list() -> Result<(), VctrlError> {
    let mut store = MemoryRefStore::new();
    let h1 = common::make_hash(0x01)?;
    let h2 = common::make_hash(0x02)?;

    store.set_ref("refs/heads/main", &h1)?;
    store.set_ref("refs/heads/dev", &h2)?;

    assert_eq!(store.get_ref("refs/heads/main")?, h1);

    let names: Vec<String> = store.list_refs()?.collect::<Result<_, _>>()?;
    assert_eq!(
        names,
        vec!["refs/heads/dev".to_string(), "refs/heads/main".to_string()]
    );

    store.delete_ref("refs/heads/dev")?;
    assert!(store.get_ref("refs/heads/dev").is_err());

    Ok(())
}

#[test]
fn memory_ref_store_invalid_name_errors() -> Result<(), VctrlError> {
    let mut store = MemoryRefStore::new();
    let hash = common::make_hash(0x03)?;
    let result = store.set_ref("bad name", &hash);
    assert!(result.is_err());
    Ok(())
}
