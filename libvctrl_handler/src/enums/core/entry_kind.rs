#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EntryKind {
    Blob,

    Executable,

    Symlink,

    Tree,

    Submodule,
}
