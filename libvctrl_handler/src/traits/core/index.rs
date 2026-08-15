use crate::VctrlError;

pub trait Index {
    type Entry;

    type Path;

    type TreeId;

    fn add(&mut self, entry: Self::Entry) -> Result<(), VctrlError>;

    fn remove(&mut self, path: &Self::Path) -> Result<(), VctrlError>;

    fn clear(&mut self) -> Result<(), VctrlError>;

    fn write_tree(&self) -> Result<Self::TreeId, VctrlError>;

    fn read_tree(&mut self, tree: &Self::TreeId) -> Result<(), VctrlError>;
}
