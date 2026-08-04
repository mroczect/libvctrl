pub mod branch;
pub mod checkout;
pub mod create_commit;
pub mod log;
pub mod merge;

pub use branch::*;
pub use checkout::*;
pub use create_commit::*;
pub use log::*;
pub use merge::*;

use crate::error::VctrlError;
use crate::storage::traits::{ObjectStore, RefStore};

pub trait Command {
    type Output;
    fn execute(
        &self,
        store: &mut dyn ObjectStore,
        refs: &mut dyn RefStore,
    ) -> Result<Self::Output, VctrlError>;
}
