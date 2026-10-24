use alloc::collections::{BTreeSet, VecDeque};
use alloc::vec::Vec;
use std::cmp::Ordering;

use libvctrl_handler::{Hash, RevWalk, SortMode, VctrlError};

#[derive(Debug, Clone)]
pub struct MemoryRevWalk {
    pending: VecDeque<Hash>,

    visited: BTreeSet<[u8; 64]>,

    sort_mode: SortMode,

    hidden: BTreeSet<[u8; 64]>,
}

impl Default for MemoryRevWalk {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryRevWalk {
    pub fn new() -> Self {
        Self {
            pending: VecDeque::new(),
            visited: BTreeSet::new(),
            sort_mode: SortMode::None,
            hidden: BTreeSet::new(),
        }
    }

    pub fn with_sort_mode(mode: SortMode) -> Self {
        let mut walk = Self::new();
        walk.sort_mode = mode;
        walk
    }

    fn is_hidden(&self, hash: &Hash) -> bool {
        self.hidden.contains(hash.as_bytes())
    }

    fn mark_visited(&mut self, hash: &Hash) {
        self.visited.insert(*hash.as_bytes());
    }

    fn was_visited(&self, hash: &Hash) -> bool {
        self.visited.contains(hash.as_bytes())
    }
}

impl RevWalk for MemoryRevWalk {
    fn push(&mut self, hash: Hash) -> Result<(), VctrlError> {
        if !self.was_visited(&hash) && !self.is_hidden(&hash) {
            self.pending.push_back(hash);
        }
        Ok(())
    }

    fn push_range(&mut self, from: &Hash, to: &Hash) -> Result<(), VctrlError> {
        self.push(*to)?;
        self.hide(*from)?;
        Ok(())
    }

    fn push_head(&mut self) -> Result<(), VctrlError> {
        Err(VctrlError::Other(
            "push_head requires repository context".into(),
        ))
    }

    fn hide(&mut self, hash: Hash) -> Result<(), VctrlError> {
        self.hidden.insert(*hash.as_bytes());
        Ok(())
    }

    fn next<F>(&mut self, parent_fn: F) -> Result<Option<Hash>, VctrlError>
    where
        F: Fn(&Hash) -> Result<Vec<Hash>, VctrlError>,
    {
        while let Some(hash) = self.pending.pop_front() {
            if self.was_visited(&hash) || self.is_hidden(&hash) {
                continue;
            }

            self.mark_visited(&hash);

            match parent_fn(&hash) {
                Ok(parents) => {
                    for parent in parents {
                        if !self.was_visited(&parent) && !self.is_hidden(&parent) {
                            match self.sort_mode {
                                SortMode::Time => {
                                    self.pending.push_back(parent);
                                }
                                SortMode::Topological | SortMode::Reverse => {
                                    self.pending.push_front(parent);
                                }
                                SortMode::None => {
                                    self.pending.push_back(parent);
                                }
                            }
                        }
                    }
                }
                Err(VctrlError::NotFound(_)) => {
                    continue;
                }
                Err(e) => return Err(e),
            }

            return Ok(Some(hash));
        }

        Ok(None)
    }

    fn reset(&mut self) {
        self.pending.clear();
        self.visited.clear();
    }

    fn count<F>(&mut self, parent_fn: F) -> Result<usize, VctrlError>
    where
        F: Fn(&Hash) -> Result<Vec<Hash>, VctrlError>,
    {
        let mut count = 0;
        while self.next(&parent_fn)?.is_some() {
            count += 1;
        }
        Ok(count)
    }

    fn simplify_first_parent(&mut self) {}

    fn sort_mode(&self) -> SortMode {
        self.sort_mode
    }

    fn set_sort_mode(&mut self, mode: SortMode) {
        self.sort_mode = mode;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash_byte(byte: u8) -> Hash {
        Hash::from_bytes(&[byte; 64]).unwrap()
    }

    fn linear_parents(hash: &Hash) -> Result<Vec<Hash>, VctrlError> {
        match hash.as_bytes()[0] {
            0x04 => Ok(vec![hash_byte(0x03)]),
            0x03 => Ok(vec![hash_byte(0x02)]),
            0x02 => Ok(vec![hash_byte(0x01)]),
            0x01 => Ok(vec![]),
            _ => Err(VctrlError::NotFound(hash.to_string())),
        }
    }

    #[test]
    fn walk_linear_history() {
        let mut walk = MemoryRevWalk::new();
        walk.push(hash_byte(0x04)).unwrap();

        let mut visited = Vec::new();
        while let Some(hash) = walk.next(&linear_parents).unwrap() {
            visited.push(hash);
        }

        assert_eq!(visited.len(), 4);
        assert_eq!(visited[0], hash_byte(0x04));
        assert_eq!(visited[3], hash_byte(0x01));
    }

    #[test]
    fn hide_commit() {
        let mut walk = MemoryRevWalk::new();
        walk.push(hash_byte(0x04)).unwrap();
        walk.hide(hash_byte(0x02)).unwrap();

        let visited: Vec<_> = (0..10)
            .filter_map(|_| walk.next(&linear_parents).transpose())
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(visited.len(), 2);
        assert_eq!(visited[0], hash_byte(0x04));
        assert_eq!(visited[1], hash_byte(0x03));
    }

    #[test]
    fn count_commits() {
        let mut walk = MemoryRevWalk::new();
        walk.push(hash_byte(0x04)).unwrap();

        let count = walk.count(&linear_parents).unwrap();
        assert_eq!(count, 4);
    }

    #[test]
    fn reset_walk() {
        let mut walk = MemoryRevWalk::new();
        walk.push(hash_byte(0x04)).unwrap();

        walk.next(&linear_parents).unwrap();
        walk.next(&linear_parents).unwrap();

        walk.reset();
        walk.push(hash_byte(0x04)).unwrap();

        let count = walk.count(&linear_parents).unwrap();
        assert_eq!(count, 4);
    }

    #[test]
    fn push_range() {
        let mut walk = MemoryRevWalk::new();
        walk.push_range(&hash_byte(0x02), &hash_byte(0x04)).unwrap();

        let visited: Vec<_> = (0..10)
            .filter_map(|_| walk.next(&linear_parents).transpose())
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(visited.len(), 2);
        assert_eq!(visited[0], hash_byte(0x04));
        assert_eq!(visited[1], hash_byte(0x03));
    }

    #[test]
    fn sort_mode() {
        let mut walk = MemoryRevWalk::with_sort_mode(SortMode::Topological);
        assert_eq!(walk.sort_mode(), SortMode::Topological);

        walk.set_sort_mode(SortMode::Time);
        assert_eq!(walk.sort_mode(), SortMode::Time);
    }
}
