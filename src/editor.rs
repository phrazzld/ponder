use crate::errors::{AppError, AppResult};
use std::process::Command;

pub trait Editor {
    fn open_files(&self, paths: &[String]) -> AppResult<()>;
}

pub struct SystemEditor {
    pub editor_cmd: String,
}

impl Editor for SystemEditor {
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
        let editor = SystemEditor { editor_cmd: "vim".to_string() };
        let paths: Vec<String> = Vec::new();
        
        // Should succeed with empty paths
        let result = editor.open_files(&paths);
        assert!(result.is_ok());
    }
}