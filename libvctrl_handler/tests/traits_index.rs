use criterion as _;
use libvctrl_handler::{Index, VctrlError};
mod common;

#[derive(Debug)]
struct MockIndex {
    len: usize,
}

impl Index for MockIndex {
    type Entry = i32;
    type Path = String;
    type TreeId = ();

    fn add(&mut self, _entry: Self::Entry) -> Result<(), VctrlError> {
        Ok(())
    }

    fn remove(&mut self, _path: &Self::Path) -> Result<(), VctrlError> {
        Ok(())
    }

    fn clear(&mut self) -> Result<(), VctrlError> {
        Ok(())
    }

    fn get(&self, _path: &Self::Path) -> Result<Option<Self::Entry>, VctrlError> {
        Ok(None)
    }

    fn contains(&self, _path: &Self::Path) -> Result<bool, VctrlError> {
        Ok(false)
    }

    fn len(&self) -> Result<usize, VctrlError> {
        Ok(self.len)
    }

    fn entries(&self) -> Result<Vec<Self::Entry>, VctrlError> {
        Ok(Vec::new())
    }

    fn write_tree(&self) -> Result<Self::TreeId, VctrlError> {
        Ok(())
    }

    fn read_tree(&mut self, _tree: &Self::TreeId) -> Result<(), VctrlError> {
        Ok(())
    }
}

#[test]
fn test_index_is_empty_default_implementation() {
    let empty = MockIndex { len: 0 };
    let empty_result = empty.is_empty();
    assert_eq!(empty_result, Ok(true));

    let non_empty = MockIndex { len: 2 };
    let non_empty_result = non_empty.is_empty();
    assert_eq!(non_empty_result, Ok(false));
}
