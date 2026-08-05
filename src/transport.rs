use crate::error::VctrlError;
use crate::storage::traits::{ObjectStore, RefStore};

pub trait Transport {
    fn fetch(
        &mut self,
        store: &mut dyn ObjectStore,
        want: &[crate::domain::hash::Hash],
    ) -> Result<(), VctrlError>;
    fn push(
        &mut self,
        store: &dyn ObjectStore,
        refs: &dyn RefStore,
        ref_names: &[String],
    ) -> Result<(), VctrlError>;
}
