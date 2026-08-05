use crate::command::Command;
use crate::domain::hash::Hash;
use crate::domain::object::Object;
use crate::error::VctrlError;
use crate::storage::traits::{ObjectStore, RefStore};
use std::collections::{HashSet, VecDeque};

pub struct Describe {
    pub commit_hash: Hash,
    pub max_commits_to_search: usize,
}

impl Command for Describe {
    type Output = Option<String>;

    fn execute(
        &self,
        store: &mut dyn ObjectStore,
        refs: &mut dyn RefStore,
    ) -> Result<Option<String>, VctrlError> {
        let tag_refs = refs.list_refs("refs/tags/")?;
        let mut tag_map: Vec<(Hash, String)> = Vec::new();
        for ref_name in &tag_refs {
            if let Some(hash) = refs.get_ref(ref_name)? {
                let commit_hash = match store.get(&hash)? {
                    Some(Object::Tag(t)) => t.target,
                    Some(Object::Commit(_)) => hash,
                    _ => continue,
                };
                let tag_name = ref_name.trim_start_matches("refs/tags/").to_string();
                tag_map.push((commit_hash, tag_name));
            }
        }

        if tag_map.is_empty() {
            return Ok(None);
        }

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back((self.commit_hash, 0usize));
        visited.insert(self.commit_hash);

        let mut found: Option<(String, usize)> = None;

        while let Some((hash, dist)) = queue.pop_front() {
            for (commit_hash, tag_name) in &tag_map {
                if *commit_hash == hash && (found.is_none() || dist < found.as_ref().unwrap().1) {
                    found = Some((tag_name.clone(), dist));
                }
            }
            if found.is_some() {
                break;
            }

            if dist >= self.max_commits_to_search {
                continue;
            }

            if let Some(Object::Commit(commit)) = store.get(&hash)? {
                for parent in &commit.parents {
                    if visited.insert(*parent) {
                        queue.push_back((*parent, dist + 1));
                    }
                }
            }
        }

        if let Some((tag_name, dist)) = found {
            let mut desc = tag_name;
            if dist > 0 {
                desc.push_str(&format!("-{}", dist));
            }
            let short_hash = self.commit_hash.to_hex();
            let short = if short_hash.len() >= 8 {
                &short_hash[..8]
            } else {
                &short_hash
            };
            desc.push_str(&format!("-g{}", short));
            Ok(Some(desc))
        } else {
            Ok(None)
        }
    }
}
