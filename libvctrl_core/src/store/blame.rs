use alloc::string::String;
use alloc::vec::Vec;

use libvctrl_handler::{Blame, BlameHunk, Hash, UserID, VctrlError};

#[derive(Debug, Clone)]
pub struct MemoryBlame {
    path: String,
    hunks: Vec<BlameHunk>,
}

impl MemoryBlame {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            hunks: Vec::new(),
        }
    }

    pub fn with_hunks(path: impl Into<String>, hunks: Vec<BlameHunk>) -> Self {
        Self {
            path: path.into(),
            hunks,
        }
    }

    pub fn blame_single_commit(
        path: impl Into<String>,
        content: &str,
        commit_hash: Hash,
        author: UserID,
        committer: UserID,
        message: &str,
    ) -> Result<Self, VctrlError> {
        let lines: Vec<&str> = content.lines().collect();

        let hunk = BlameHunk::new(
            1,
            lines.len(),
            commit_hash,
            Some(author),
            Some(committer),
            Some(message.to_string()),
        )?;

        Ok(Self {
            path: path.into(),
            hunks: vec![hunk],
        })
    }

    pub fn blame_with_ranges(
        path: impl Into<String>,
        ranges: Vec<BlameRange>,
    ) -> Result<Self, VctrlError> {
        let hunks = ranges
            .into_iter()
            .map(|range| {
                BlameHunk::new(
                    range.start_line,
                    range.line_count,
                    range.commit_hash,
                    range.author,
                    range.committer,
                    range.message,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            path: path.into(),
            hunks,
        })
    }
}

pub struct BlameRange {
    start_line: usize,
    line_count: usize,
    commit_hash: Hash,
    author: Option<UserID>,
    committer: Option<UserID>,
    message: Option<String>,
}

impl BlameRange {
    pub fn new(start_line: usize, line_count: usize, commit_hash: Hash) -> Self {
        Self {
            start_line,
            line_count,
            commit_hash,
            author: None,
            committer: None,
            message: None,
        }
    }

    pub fn author(mut self, author: UserID) -> Self {
        self.author = Some(author);
        self
    }

    pub fn committer(mut self, committer: UserID) -> Self {
        self.committer = Some(committer);
        self
    }

    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }
}

impl Blame for MemoryBlame {
    fn path(&self) -> &str {
        &self.path
    }

    fn hunks(&self) -> &[BlameHunk] {
        &self.hunks
    }

    fn get_hunk_at_line(&self, line: usize) -> Option<&BlameHunk> {
        let mut current_line = 1usize;
        for hunk in &self.hunks {
            let hunk_end = current_line + hunk.lines_in_hunk();
            if line >= current_line && line < hunk_end {
                return Some(hunk);
            }
            current_line = hunk_end;
        }
        None
    }

    fn get_hunk_by_index(&self, index: usize) -> Option<&BlameHunk> {
        self.hunks.get(index)
    }

    fn hunk_count(&self) -> usize {
        self.hunks.len()
    }

    fn total_lines(&self) -> usize {
        self.hunks.iter().map(|h| h.lines_in_hunk()).sum()
    }

    fn iter(&self) -> impl Iterator<Item = &BlameHunk> {
        self.hunks.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash_byte(byte: u8) -> Hash {
        Hash::from_bytes(&[byte; 64]).unwrap()
    }

    fn user(name: &str, email: &str) -> UserID {
        UserID::new(name.to_string(), email.to_string()).unwrap()
    }

    #[test]
    fn blame_single_commit() {
        let content = "line1\nline2\nline3\n";
        let blame = MemoryBlame::blame_single_commit(
            "test.txt",
            content,
            hash_byte(0x01),
            user("Alice", "alice@example.com"),
            user("Alice", "alice@example.com"),
            "initial commit",
        )
        .unwrap();

        assert_eq!(blame.path(), "test.txt");
        assert_eq!(blame.hunk_count(), 1);
        assert_eq!(blame.total_lines(), 3);
    }

    #[test]
    fn get_hunk_at_line() {
        let blame = MemoryBlame::blame_with_ranges(
            "file.txt",
            vec![
                BlameRange::new(1, 5, hash_byte(0x01))
                    .author(user("Alice", "a@b.com"))
                    .message("first"),
                BlameRange::new(6, 3, hash_byte(0x02))
                    .author(user("Bob", "b@c.com"))
                    .message("second"),
            ],
        )
        .unwrap();

        let hunk1 = blame.get_hunk_at_line(1).unwrap();
        assert_eq!(hunk1.final_commit_id(), &hash_byte(0x01));

        let hunk3 = blame.get_hunk_at_line(3).unwrap();
        assert_eq!(hunk3.final_commit_id(), &hash_byte(0x01));

        let hunk6 = blame.get_hunk_at_line(6).unwrap();
        assert_eq!(hunk6.final_commit_id(), &hash_byte(0x02));

        let hunk8 = blame.get_hunk_at_line(8).unwrap();
        assert_eq!(hunk8.final_commit_id(), &hash_byte(0x02));

        assert!(blame.get_hunk_at_line(9).is_none());
    }

    #[test]
    fn get_hunk_by_index() {
        let blame = MemoryBlame::blame_with_ranges(
            "file.txt",
            vec![
                BlameRange::new(1, 2, hash_byte(0x01)),
                BlameRange::new(3, 2, hash_byte(0x02)),
                BlameRange::new(5, 2, hash_byte(0x03)),
            ],
        )
        .unwrap();

        assert_eq!(
            blame.get_hunk_by_index(0).unwrap().final_commit_id(),
            &hash_byte(0x01)
        );
        assert_eq!(
            blame.get_hunk_by_index(1).unwrap().final_commit_id(),
            &hash_byte(0x02)
        );
        assert_eq!(
            blame.get_hunk_by_index(2).unwrap().final_commit_id(),
            &hash_byte(0x03)
        );
        assert!(blame.get_hunk_by_index(3).is_none());
    }

    #[test]
    fn iter_hunks() {
        let blame = MemoryBlame::blame_with_ranges(
            "file.txt",
            vec![
                BlameRange::new(1, 1, hash_byte(0x01)),
                BlameRange::new(2, 1, hash_byte(0x02)),
            ],
        )
        .unwrap();

        let hashes: Vec<&Hash> = blame.iter().map(|h| h.final_commit_id()).collect();
        assert_eq!(hashes.len(), 2);
    }

    #[test]
    fn total_lines() {
        let blame = MemoryBlame::blame_with_ranges(
            "file.txt",
            vec![
                BlameRange::new(1, 10, hash_byte(0x01)),
                BlameRange::new(11, 5, hash_byte(0x02)),
            ],
        )
        .unwrap();

        assert_eq!(blame.total_lines(), 15);
    }
}
