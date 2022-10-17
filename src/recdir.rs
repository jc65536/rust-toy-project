use std::fs;
use std::fs::ReadDir;
use std::path::PathBuf;

/// Iterates through a directory recursively
pub struct RecDir {
    stack: Vec<PathBuf>, // Stack of directories to look at later
    entries: ReadDir, // Iterates through the current directory
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
            let path = entry.expect("Error reading directory entry").path();
            if path.is_dir() {
                self.stack.push(path);
            } else {
                return Some(path);
            }
        } else if let Some(dir) = self.stack.pop() {
            self.entries = fs::read_dir(dir).expect("Error reading directory.");
        } else {
            return None;
        }
        // Tail recursion ftw
        self.next()
    }
}
