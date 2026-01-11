// Allow dead code for now since we're building incrementally with TDD
#![allow(dead_code)]

use chrono::{DateTime, Utc};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileType {
    Text,
    Image,
    Pdf,
    Binary,
}

impl FileType {
    pub fn from_extension(ext: &str) -> Self {
        let ext = ext.to_lowercase();
        match ext.as_str() {
            // Text files
            "txt" | "md" | "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | "json" | "yaml" | "yml"
            | "toml" | "xml" | "html" | "css" | "sh" | "bash" | "c" | "cpp" | "h" | "hpp"
            | "java" | "go" | "rb" | "php" | "swift" | "kt" | "cs" | "sql" => FileType::Text,

            // Image files
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "svg" | "ico" => FileType::Image,

            // PDF files
            "pdf" => FileType::Pdf,

            // Everything else is binary
            _ => FileType::Binary,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    Keep,
    Trash,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub modified_date: DateTime<Utc>,
    pub file_type: FileType,
}

impl FileEntry {
    pub fn from_path(path: &Path) -> io::Result<Self> {
        let metadata = fs::metadata(path)?;
        let modified = metadata.modified()?;
        let modified_date: DateTime<Utc> = modified.into();

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let file_type = FileType::from_extension(extension);

        Ok(FileEntry {
            path: path.to_path_buf(),
            name,
            size: metadata.len(),
            modified_date,
            file_type,
        })
    }
}

#[derive(Debug)]
pub struct AppState {
    pub files: Vec<FileEntry>,
    pub current_index: usize,
    pub decisions_stack: Vec<(usize, Decision)>,
}

impl AppState {
    pub fn new(files: Vec<FileEntry>) -> Self {
        Self {
            files,
            current_index: 0,
            decisions_stack: Vec::new(),
        }
    }

    pub fn next(&mut self) {
        if self.current_index < self.files.len().saturating_sub(1) {
            self.current_index += 1;
        }
    }

    pub fn previous(&mut self) {
        if self.current_index > 0 {
            self.current_index -= 1;
        }
    }

    pub fn current_file(&self) -> Option<&FileEntry> {
        self.files.get(self.current_index)
    }

    pub fn record_decision(&mut self, decision: Decision) {
        self.decisions_stack.push((self.current_index, decision));
    }

    pub fn undo(&mut self) -> Option<(usize, Decision)> {
        self.decisions_stack.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod file_type_tests {
        use super::*;

        #[test]
        fn test_file_type_from_extension_text() {
            assert_eq!(FileType::from_extension("txt"), FileType::Text);
            assert_eq!(FileType::from_extension("rs"), FileType::Text);
            assert_eq!(FileType::from_extension("py"), FileType::Text);
            assert_eq!(FileType::from_extension("js"), FileType::Text);
            assert_eq!(FileType::from_extension("md"), FileType::Text);
        }

        #[test]
        fn test_file_type_from_extension_image() {
            assert_eq!(FileType::from_extension("png"), FileType::Image);
            assert_eq!(FileType::from_extension("jpg"), FileType::Image);
            assert_eq!(FileType::from_extension("jpeg"), FileType::Image);
            assert_eq!(FileType::from_extension("gif"), FileType::Image);
            assert_eq!(FileType::from_extension("webp"), FileType::Image);
        }

        #[test]
        fn test_file_type_from_extension_pdf() {
            assert_eq!(FileType::from_extension("pdf"), FileType::Pdf);
        }

        #[test]
        fn test_file_type_from_extension_binary() {
            assert_eq!(FileType::from_extension("exe"), FileType::Binary);
            assert_eq!(FileType::from_extension("bin"), FileType::Binary);
            assert_eq!(FileType::from_extension("unknown"), FileType::Binary);
            assert_eq!(FileType::from_extension(""), FileType::Binary);
        }

        #[test]
        fn test_file_type_case_insensitive() {
            assert_eq!(FileType::from_extension("PNG"), FileType::Image);
            assert_eq!(FileType::from_extension("TXT"), FileType::Text);
            assert_eq!(FileType::from_extension("PDF"), FileType::Pdf);
        }
    }

    mod file_entry_tests {
        use super::*;
        use std::fs;
        use tempfile::NamedTempFile;

        #[test]
        fn test_file_entry_from_path() {
            let temp_file = NamedTempFile::new().unwrap();
            let path = temp_file.path();
            fs::write(path, b"test content").unwrap();

            let entry = FileEntry::from_path(path).unwrap();

            assert_eq!(entry.path, path);
            assert!(entry.name.len() > 0);
            assert_eq!(entry.size, 12);
            assert_eq!(entry.file_type, FileType::Binary);
        }

        #[test]
        fn test_file_entry_from_path_with_extension() {
            let temp_file = NamedTempFile::new().unwrap();
            let path = temp_file.path();
            let txt_path = path.with_extension("txt");
            fs::write(&txt_path, b"hello").unwrap();

            let entry = FileEntry::from_path(&txt_path).unwrap();

            assert_eq!(entry.file_type, FileType::Text);
            assert_eq!(entry.size, 5);

            fs::remove_file(&txt_path).ok();
        }

        #[test]
        fn test_file_entry_nonexistent_file() {
            let result = FileEntry::from_path(Path::new("/nonexistent/file.txt"));
            assert!(result.is_err());
        }
    }

    mod app_state_tests {
        use super::*;

        fn create_test_entry(name: &str) -> FileEntry {
            FileEntry {
                path: PathBuf::from(name),
                name: name.to_string(),
                size: 0,
                modified_date: Utc::now(),
                file_type: FileType::Text,
            }
        }

        #[test]
        fn test_app_state_new() {
            let files = vec![
                create_test_entry("file1.txt"),
                create_test_entry("file2.txt"),
            ];
            let state = AppState::new(files.clone());

            assert_eq!(state.files.len(), 2);
            assert_eq!(state.current_index, 0);
            assert_eq!(state.decisions_stack.len(), 0);
        }

        #[test]
        fn test_app_state_next() {
            let files = vec![
                create_test_entry("file1.txt"),
                create_test_entry("file2.txt"),
            ];
            let mut state = AppState::new(files);

            assert_eq!(state.current_index, 0);

            state.next();
            assert_eq!(state.current_index, 1);

            state.next();
            assert_eq!(state.current_index, 1); // Should stay at last item
        }

        #[test]
        fn test_app_state_previous() {
            let files = vec![
                create_test_entry("file1.txt"),
                create_test_entry("file2.txt"),
            ];
            let mut state = AppState::new(files);
            state.current_index = 1;

            state.previous();
            assert_eq!(state.current_index, 0);

            state.previous();
            assert_eq!(state.current_index, 0); // Should stay at first item
        }

        #[test]
        fn test_app_state_current_file() {
            let files = vec![
                create_test_entry("file1.txt"),
                create_test_entry("file2.txt"),
            ];
            let state = AppState::new(files);

            let current = state.current_file();
            assert!(current.is_some());
            assert_eq!(current.unwrap().name, "file1.txt");
        }

        #[test]
        fn test_app_state_current_file_empty() {
            let state = AppState::new(vec![]);
            assert!(state.current_file().is_none());
        }

        #[test]
        fn test_app_state_record_decision() {
            let files = vec![create_test_entry("file1.txt")];
            let mut state = AppState::new(files);

            state.record_decision(Decision::Trash);

            assert_eq!(state.decisions_stack.len(), 1);
            assert_eq!(state.decisions_stack[0], (0, Decision::Trash));
        }

        #[test]
        fn test_app_state_undo() {
            let files = vec![
                create_test_entry("file1.txt"),
                create_test_entry("file2.txt"),
            ];
            let mut state = AppState::new(files);

            state.record_decision(Decision::Keep);
            state.next();
            state.record_decision(Decision::Trash);

            assert_eq!(state.current_index, 1);
            assert_eq!(state.decisions_stack.len(), 2);

            let undone = state.undo();
            assert!(undone.is_some());
            assert_eq!(undone.unwrap(), (1, Decision::Trash));
            assert_eq!(state.current_index, 1);
            assert_eq!(state.decisions_stack.len(), 1);
        }

        #[test]
        fn test_app_state_undo_empty() {
            let files = vec![create_test_entry("file1.txt")];
            let mut state = AppState::new(files);

            let undone = state.undo();
            assert!(undone.is_none());
        }
    }
}
