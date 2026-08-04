pub trait ConflictResolver {
    fn resolve(&self, base: &[u8], ours: &[u8], theirs: &[u8]) -> Option<Vec<u8>>;
}
