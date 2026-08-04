pub mod branch;
pub mod checkout;
pub mod cherry_pick;
pub mod create_commit;
pub mod log;
pub mod merge;
pub mod revert;
pub mod tag_cmd;
pub mod verify_commit;

pub use branch::*;
pub use checkout::*;
pub use cherry_pick::*;
pub use create_commit::*;
pub use log::*;
pub use merge::*;
pub use revert::*;
pub use tag_cmd::*;
pub use verify_commit::*;

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
