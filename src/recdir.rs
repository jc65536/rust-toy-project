use std::fs;
use std::fs::ReadDir;
use std::path::PathBuf;

pub struct RecDir {
    stack: Vec<PathBuf>,
    entries: ReadDir,
}

impl RecDir {
    pub fn new(dir: &str) -> Self {
        RecDir {
            stack: Vec::new(),
            entries: fs::read_dir(dir).expect("Error reading directory."),
        }
    }
}

impl Iterator for RecDir {
    type Item = PathBuf;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entry) = self.entries.next() {
            let entry = entry.expect("Error reading directory entry");
            let file_type = entry.file_type().expect("Error reading directory entry.");
            if file_type.is_dir() {
                self.stack.push(entry.path());
            } else {
                return Some(entry.path());
            }
        } else if let Some(dir) = self.stack.pop() {
            self.entries = fs::read_dir(dir).expect("Error reading directory.");
        } else {
            return None;
        }
        self.next()
    }
}
