//! Editor abstraction for opening journal files.
//!
//! This module provides an abstraction for opening files in an external editor,
//! allowing the application to work with different editors and to be testable
//! by mocking the editor functionality.

use crate::errors::{AppError, AppResult};
use std::path::Path;
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
/// use std::path::Path;
///
/// struct DummyEditor;
///
/// impl Editor for DummyEditor {
///     fn open_files(&self, paths: &[&Path]) -> AppResult<()> {
///         println!("Would open files: {:?}", paths);
///         Ok(())
///     }
/// }
///
/// let editor = DummyEditor;
/// let file1 = Path::new("file1.md");
/// let file2 = Path::new("file2.md");
/// let paths = vec![file1, file2];
/// editor.open_files(&paths).unwrap();
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
    fn open_files(&self, paths: &[&Path]) -> AppResult<()>;
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
/// let file1 = Path::new("file1.md");
/// let file2 = Path::new("file2.md");
/// let paths = vec![file1, file2];
/// editor.open_files(&paths).expect("Failed to open files");
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
    fn open_files(&self, paths: &[&Path]) -> AppResult<()> {
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
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    /// Mock implementation of Editor for testing purposes.
    ///
    /// This mock captures opened files and can be configured to succeed or fail.
    /// It allows testing both success and error paths in code that uses an Editor.
    pub struct MockEditor {
        /// Records file paths that were opened through this editor.
        pub opened_files: Arc<Mutex<Vec<PathBuf>>>,
        /// Controls whether the editor should fail when opening files.
        pub should_fail: bool,
        /// The error to return when `should_fail` is true.
        pub failure_error: Option<AppError>,
    }

    impl MockEditor {
        /// Creates a new MockEditor instance that succeeds by default.
        fn new() -> Self {
            MockEditor {
                opened_files: Arc::new(Mutex::new(Vec::new())),
                should_fail: false,
                failure_error: None,
            }
        }
        
        /// Configures the MockEditor to fail with the specified error.
        fn with_failure(error: AppError) -> Self {
            MockEditor {
                opened_files: Arc::new(Mutex::new(Vec::new())),
                should_fail: true,
                failure_error: Some(error),
            }
        }
        
        /// Configures whether this editor should fail when opening files.
        fn set_should_fail(&mut self, should_fail: bool) {
            self.should_fail = should_fail;
        }
        
        /// Sets the error to return when `should_fail` is true.
        fn set_failure_error(&mut self, error: AppError) {
            self.failure_error = Some(error);
        }
    }

    impl Editor for MockEditor {
        fn open_files(&self, paths: &[&Path]) -> AppResult<()> {
            // Record the files that were attempted to be opened
            let mut opened = self.opened_files.lock().unwrap();
            for &path in paths {
                opened.push(path.to_path_buf());
            }
            
            // If configured to fail, return the specified error or a default one
            if self.should_fail {
                return match &self.failure_error {
                    Some(error) => Err(error.clone()),
                    None => Err(AppError::Editor("Mock editor failed by configuration".to_string())),
                };
            }
            
            Ok(())
        }
    }

    #[test]
    fn test_mock_editor_open_files_success() {
        let editor = MockEditor::new();
        let file1 = Path::new("file1.md");
        let file2 = Path::new("file2.md");
        let paths = [file1, file2];

        // Open files - should succeed by default
        let result = editor.open_files(&paths);
        assert!(result.is_ok());

        // Check that the files were recorded
        let opened = editor.opened_files.lock().unwrap();
        assert_eq!(opened.len(), 2);
        assert_eq!(opened[0], Path::new("file1.md"));
        assert_eq!(opened[1], Path::new("file2.md"));
    }
    
    #[test]
    fn test_mock_editor_open_files_failure() {
        // Create a MockEditor configured to fail
        let error = AppError::Editor("Test error".to_string());
        let editor = MockEditor::with_failure(error);
        
        let file = Path::new("file.md");
        let paths = [file];
        
        // Attempt to open files - should fail
        let result = editor.open_files(&paths);
        assert!(result.is_err());
        
        // The error should match our configured error
        match result {
            Err(AppError::Editor(msg)) => assert!(msg.contains("Test error")),
            _ => panic!("Unexpected error type"),
        }
        
        // Even though it failed, the files should still be recorded
        let opened = editor.opened_files.lock().unwrap();
        assert_eq!(opened.len(), 1);
        assert_eq!(opened[0], Path::new("file.md"));
    }
    
    #[test]
    fn test_mock_editor_set_failure_behavior() {
        // Start with a successful editor
        let mut editor = MockEditor::new();
        let file = Path::new("file.md");
        let paths = [file];
        
        // Should succeed initially
        assert!(editor.open_files(&paths).is_ok());
        
        // Configure to fail
        editor.set_should_fail(true);
        editor.set_failure_error(AppError::Editor("Configured failure".to_string()));
        
        // Should fail now
        let result = editor.open_files(&paths);
        assert!(result.is_err());
        match result {
            Err(AppError::Editor(msg)) => assert!(msg.contains("Configured failure")),
            _ => panic!("Unexpected error type"),
        }
        
        // Reset to success
        editor.set_should_fail(false);
        
        // Should succeed again
        assert!(editor.open_files(&paths).is_ok());
    }

    #[test]
    fn test_system_editor_empty_paths() {
        let editor = SystemEditor {
            editor_cmd: "vim".to_string(),
        };
        let paths: Vec<&Path> = Vec::new();

        // Should succeed with empty paths
        let result = editor.open_files(&paths);
        assert!(result.is_ok());
    }
}
