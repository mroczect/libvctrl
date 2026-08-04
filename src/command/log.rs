use crate::command::Command;
use crate::domain::commit::Commit;
use crate::domain::object::Object;
use crate::error::VctrlError;
use crate::storage::traits::{ObjectStore, RefStore};

pub struct Log;
impl Command for Log {
    type Output = Vec<Commit>;
    fn execute(
        &self,
        store: &mut dyn ObjectStore,
        refs: &mut dyn RefStore,
    ) -> Result<Vec<Commit>, VctrlError> {
        let head = match refs.head()? {
            Some(h) => h,
            None => return Ok(Vec::new()),
        };
        let mut commits = Vec::with_capacity(16);
        let mut current = Some(head);
        while let Some(h) = current {
            match store.get(&h)? {
                Some(Object::Commit(c)) => {
                    current = c.parents.first().copied();
                    commits.push(*c);
                }
                _ => break,
            }
        }
        Ok(commits)
    }
}
