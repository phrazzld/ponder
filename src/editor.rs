//! Editor abstraction for opening journal files.
//!
//! This module provides an abstraction for opening files in an external editor,
//! allowing the application to work with different editors and to be testable
//! by mocking the editor functionality.

use crate::errors::{AppError, AppResult};
use std::process::Command;

/// Trait defining the interface for an editor component.
///
/// This trait abstracts the functionality of opening files in an editor,
/// allowing different implementations for different use cases (e.g.,
/// a real system editor or a mock editor for testing).
///
/// # Examples
///
/// ```
/// use ponder::editor::Editor;
/// use ponder::errors::AppResult;
///
/// struct DummyEditor;
///
/// impl Editor for DummyEditor {
///     fn open_files(&self, paths: &[String]) -> AppResult<()> {
///         println!("Would open files: {:?}", paths);
///         Ok(())
///     }
/// }
///
/// let editor = DummyEditor;
/// let files = vec!["file1.md".to_string(), "file2.md".to_string()];
/// editor.open_files(&files).unwrap();
/// ```
pub trait Editor {
    /// Opens one or more files in the editor.
    ///
    /// # Parameters
    ///
    /// * `paths` - A slice of file paths to open in the editor.
    ///
    /// # Returns
    ///
    /// A Result that is Ok(()) if the files were opened successfully,
    /// or an AppError if there was a problem opening the files.
    ///
    /// # Errors
    ///
    /// Different implementations may return different errors when file opening fails.
    fn open_files(&self, paths: &[String]) -> AppResult<()>;
}

/// An implementation of the Editor trait that uses a system command to open files.
///
/// This implementation launches an external editor process with the specified command
/// and passes the file paths as arguments.
///
/// # Examples
///
/// ```no_run
/// use ponder::editor::SystemEditor;
/// use ponder::editor::Editor;
///
/// let editor = SystemEditor {
///     editor_cmd: "vim".to_string(),
/// };
///
/// let files = vec!["file1.md".to_string(), "file2.md".to_string()];
/// editor.open_files(&files).expect("Failed to open files");
/// ```
pub struct SystemEditor {
    /// The command to use for opening files (e.g., "vim", "code", "nano").
    pub editor_cmd: String,
}

impl Editor for SystemEditor {
    /// Opens the specified files with the configured editor command.
    ///
    /// Uses `std::process::Command` to launch the editor with the file paths as arguments.
    /// If the list of paths is empty, the method returns immediately with Ok(()).
    ///
    /// # Parameters
    ///
    /// * `paths` - A slice of file paths to open in the editor.
    ///
    /// # Returns
    ///
    /// A Result that is Ok(()) if the editor was launched successfully,
    /// or an AppError if there was a problem launching the editor.
    ///
    /// # Errors
    ///
    /// Returns `AppError::Editor` if the editor command failed to execute.
    /// This could happen if:
    /// - The editor command is not found
    /// - The process cannot be spawned
    /// - One of the file paths is invalid
    fn open_files(&self, paths: &[String]) -> AppResult<()> {
        if paths.is_empty() {
            return Ok(());
        }

        Command::new(&self.editor_cmd)
            .args(paths)
            .status()
            .map_err(|e| AppError::Editor(format!("Failed to open files: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct MockEditor {
        pub opened_files: Arc<Mutex<Vec<String>>>,
    }

    impl MockEditor {
        fn new() -> Self {
            MockEditor {
                opened_files: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    impl Editor for MockEditor {
        fn open_files(&self, paths: &[String]) -> AppResult<()> {
            let mut opened = self.opened_files.lock().unwrap();
            for path in paths {
                opened.push(path.clone());
            }
            Ok(())
        }
    }

    #[test]
    fn test_mock_editor_open_files() {
        let editor = MockEditor::new();
        let paths = vec!["file1.md".to_string(), "file2.md".to_string()];

        // Open files
        editor.open_files(&paths).unwrap();

        // Check that the files were recorded
        let opened = editor.opened_files.lock().unwrap();
        assert_eq!(opened.len(), 2);
        assert_eq!(opened[0], "file1.md");
        assert_eq!(opened[1], "file2.md");
    }

    #[test]
    fn test_system_editor_empty_paths() {
        let editor = SystemEditor {
            editor_cmd: "vim".to_string(),
        };
        let paths: Vec<String> = Vec::new();

        // Should succeed with empty paths
        let result = editor.open_files(&paths);
        assert!(result.is_ok());
    }
}
