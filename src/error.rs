use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileTinderError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Directory not found: {path}")]
    DirectoryNotFound { path: std::path::PathBuf },

    #[error("File not found: {path}")]
    FileNotFound { path: std::path::PathBuf },

    #[error("Invalid file index: {index} (max: {max})")]
    InvalidIndex { index: usize, max: usize },

    #[error("No decisions to undo")]
    NothingToUndo,

    #[error("Preview generation failed: {reason}")]
    PreviewError { reason: String },

    #[error("Trash operation failed: {0}")]
    TrashError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Failed to open file: {0}")]
    OpenFileError(String),
}

pub type Result<T> = std::result::Result<T, FileTinderError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_error_display_messages() {
        let err = FileTinderError::DirectoryNotFound {
            path: std::path::PathBuf::from("/test/path"),
        };
        assert_eq!(err.to_string(), "Directory not found: /test/path");

        let err = FileTinderError::InvalidIndex { index: 5, max: 3 };
        assert_eq!(err.to_string(), "Invalid file index: 5 (max: 3)");

        let err = FileTinderError::NothingToUndo;
        assert_eq!(err.to_string(), "No decisions to undo");
    }

    #[test]
    fn test_error_from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "test error");
        let err: FileTinderError = io_err.into();
        assert!(matches!(err, FileTinderError::Io(_)));
    }
}
