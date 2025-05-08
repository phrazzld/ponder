use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Journal logic error: {0}")]
    Journal(String),
    
    #[error("Editor interaction error: {0}")]
    Editor(String),
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    
    #[test]
    fn test_app_error_from_io_error() {
        // Create an IO error
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        
        // Convert to AppError
        let app_error: AppError = io_error.into();
        
        // Verify conversion
        match app_error {
            AppError::Io(inner) => {
                assert_eq!(inner.kind(), io::ErrorKind::NotFound);
            },
            _ => panic!("Expected AppError::Io variant"),
        }
    }
    
    #[test]
    fn test_app_error_display() {
        // Test Config error
        let config_error = AppError::Config("Invalid configuration".to_string());
        assert_eq!(format!("{}", config_error), "Configuration error: Invalid configuration");
        
        // Test Io error
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
        let app_io_error = AppError::Io(io_error);
        assert_eq!(format!("{}", app_io_error), "I/O error: permission denied");
        
        // Test Journal error
        let journal_error = AppError::Journal("Invalid date".to_string());
        assert_eq!(format!("{}", journal_error), "Journal logic error: Invalid date");
        
        // Test Editor error
        let editor_error = AppError::Editor("Failed to launch".to_string());
        assert_eq!(format!("{}", editor_error), "Editor interaction error: Failed to launch");
    }
    
    #[test]
    fn test_result_combinators() {
        // Test using map_err with AppResult
        let io_result: Result<(), io::Error> = Err(io::Error::new(io::ErrorKind::Other, "test error"));
        let app_result: AppResult<()> = io_result.map_err(AppError::Io);
        
        assert!(app_result.is_err());
        match app_result {
            Err(AppError::Io(inner)) => {
                assert_eq!(inner.kind(), io::ErrorKind::Other);
            },
            _ => panic!("Expected AppError::Io variant"),
        }
    }
}