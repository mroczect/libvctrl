pub mod resolver;
pub mod three_way;
pub use resolver::*;
pub use three_way::*;
pub mod base;
pub use base::*;
pub mod strategy;
use crate::codec::Encoder;
use crate::domain::hash::Hash;
use crate::error::VctrlError;
use crate::hashing::Hasher;
use crate::storage::traits::ObjectStore;
pub use strategy::*;

#[allow(clippy::too_many_arguments)]
pub trait ThreeWayMerge {
    fn merge(
        &self,
        store: &mut dyn ObjectStore,
        base: &Hash,
        ours: &Hash,
        theirs: &Hash,
        resolver: &dyn ConflictResolver,
        encoder: &dyn Encoder,
        hasher: &dyn Hasher,
    ) -> Result<Hash, VctrlError>;
}
