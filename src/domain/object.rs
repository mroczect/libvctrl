use super::blob::Blob;
use super::commit::Commit;
use super::tree::Tree;

#[derive(Debug, Clone)]
pub enum Object {
    Blob(Blob),
    Tree(Tree),
    Commit(Box<Commit>),
}

impl Object {
    pub fn obj_type(&self) -> &str {
        match self {
            Object::Blob(_) => "blob",
            Object::Tree(_) => "tree",
            Object::Commit(_) => "commit",
        }
    }
}
