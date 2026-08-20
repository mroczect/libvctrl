use crate::errors::VctrlError;

pub trait Index: Send + Sync {
    type Entry: Send + Sync;

    type Path: Send + Sync;

    type TreeId: Send + Sync;

    fn add(&mut self, entry: Self::Entry) -> Result<(), VctrlError>;

    fn remove(&mut self, path: &Self::Path) -> Result<(), VctrlError>;

    fn clear(&mut self) -> Result<(), VctrlError>;

    fn get(&self, path: &Self::Path) -> Result<Option<Self::Entry>, VctrlError>;

    fn contains(&self, path: &Self::Path) -> Result<bool, VctrlError>;

    fn len(&self) -> Result<usize, VctrlError>;

    fn is_empty(&self) -> Result<bool, VctrlError> {
        Ok(self.len()? == 0)
    }

    fn entries(&self) -> Result<Vec<Self::Entry>, VctrlError>;

    fn write_tree(&self) -> Result<Self::TreeId, VctrlError>;

    fn read_tree(&mut self, tree: &Self::TreeId) -> Result<(), VctrlError>;
}
