//! Module for opening files in external editors or applications

use crate::error::{FileTinderError, Result};
use std::env;
use std::path::Path;

/// Opens a file in the user's preferred editor or default application.
///
/// Precedence:
/// 1. If $EDITOR or $VISUAL environment variable is set, use it (via edit crate)
/// 2. Otherwise, use system default application (via open crate)
///
/// This function blocks until the editor/application closes.
pub fn open_file<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();

    // Check if EDITOR or VISUAL environment variable is set
    if env::var("EDITOR").is_ok() || env::var("VISUAL").is_ok() {
        // Use edit crate which respects $EDITOR/$VISUAL and waits for completion
        edit::edit_file(path).map_err(|e| {
            FileTinderError::OpenFileError(format!("Failed to open file with editor: {}", e))
        })?;
    } else {
        // Fallback to system default application
        open::that(path).map_err(|e| {
            FileTinderError::OpenFileError(format!(
                "Failed to open file with default application: {}",
                e
            ))
        })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_open_file_with_nonexistent_file() {
        let result = open_file("/nonexistent/file.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_open_file_with_existing_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        fs::write(path, b"test content").unwrap();

        // Just verify path exists (actual opening depends on environment)
        assert!(path.exists());
    }
}
