mod cli;
mod config;
mod editor;
mod errors;
mod journal;

use cli::CliArgs;
use config::Config;
use editor::SystemEditor;
use errors::AppResult;
use journal::io::FileSystemIO;
use journal::{DateSpecifier, JournalService};
use log::{info, error, debug};

fn main() -> AppResult<()> {
    // Initialize logging
    env_logger::init();
    info!("Starting ponder");
    
    // Parse command-line arguments
    let args = CliArgs::parse();
    debug!("CLI arguments: {:?}", args);
    
    // Set up verbose logging if requested
    if args.verbose {
        debug!("Verbose mode enabled");
    }
    
    // Load and validate configuration
    info!("Loading configuration");
    let config = Config::load().map_err(|e| {
        error!("Configuration error: {}", e);
        e
    })?;
    
    config.validate().map_err(|e| {
        error!("Invalid configuration: {}", e);
        e
    })?;
    
    // Ensure journal directory exists
    debug!("Journal directory: {:?}", config.journal_dir);
    config.ensure_journal_dir().map_err(|e| {
        error!("Failed to create journal directory: {}", e);
        e
    })?;
    
    // Initialize I/O, editor, and journal service
    info!("Initializing journal service");
    let io = Box::new(FileSystemIO {
        journal_dir: config.journal_dir.to_string_lossy().to_string(),
    });
    
    let editor = Box::new(SystemEditor {
        editor_cmd: config.editor.clone(),
    });
    
    let journal_service = JournalService::new(config, io, editor);
    
    // Determine which entry type to open based on CLI arguments
    let date_spec = get_date_specifier_from_args(&args)?;
    
    // Open the appropriate journal entries
    info!("Opening journal entries");
    journal_service.open_entries(&date_spec).map_err(|e| {
        error!("Failed to open journal entries: {}", e);
        e
    })?;
    
    info!("Journal entries opened successfully");
    Ok(())
}

/// Converts CLI arguments to a DateSpecifier
fn get_date_specifier_from_args(args: &CliArgs) -> AppResult<DateSpecifier> {
    if args.retro {
        Ok(DateSpecifier::Retro)
    } else if args.reminisce {
        Ok(DateSpecifier::Reminisce)
    } else if let Some(date_str) = &args.date {
        // Parse the date string
        match DateSpecifier::from_args(false, false, Some(date_str)) {
            Ok(date_spec) => Ok(date_spec),
            Err(e) => {
                error!("Invalid date format: {}", e);
                Err(e)
            }
        }
    } else {
        // Default to today if no options are specified
        Ok(DateSpecifier::Today)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use errors::AppError;
    use chrono::Datelike;
    
    #[test]
    fn test_get_date_specifier_from_retro_args() {
        let args = CliArgs {
            retro: true,
            reminisce: false,
            date: None,
            verbose: false,
        };
        
        let date_spec = get_date_specifier_from_args(&args).unwrap();
        assert_eq!(date_spec, DateSpecifier::Retro);
    }
    
    #[test]
    fn test_get_date_specifier_from_reminisce_args() {
        let args = CliArgs {
            retro: false,
            reminisce: true,
            date: None,
            verbose: false,
        };
        
        let date_spec = get_date_specifier_from_args(&args).unwrap();
        assert_eq!(date_spec, DateSpecifier::Reminisce);
    }
    
    #[test]
    fn test_get_date_specifier_from_date_args() {
        let args = CliArgs {
            retro: false,
            reminisce: false,
            date: Some("2023-01-15".to_string()),
            verbose: false,
        };
        
        let date_spec = get_date_specifier_from_args(&args).unwrap();
        if let DateSpecifier::Specific(date) = date_spec {
            assert_eq!(date.year(), 2023);
            assert_eq!(date.month(), 1);
            assert_eq!(date.day(), 15);
        } else {
            panic!("Expected DateSpecifier::Specific");
        }
    }
    
    #[test]
    fn test_get_date_specifier_from_invalid_date_args() {
        let args = CliArgs {
            retro: false,
            reminisce: false,
            date: Some("invalid-date".to_string()),
            verbose: false,
        };
        
        let result = get_date_specifier_from_args(&args);
        assert!(result.is_err());
        
        match result {
            Err(AppError::Journal(msg)) => {
                assert!(msg.contains("Invalid date format"));
            },
            _ => panic!("Expected Journal error"),
        }
    }
    
    #[test]
    fn test_get_date_specifier_from_default_args() {
        let args = CliArgs {
            retro: false,
            reminisce: false,
            date: None,
            verbose: false,
        };
        
        let date_spec = get_date_specifier_from_args(&args).unwrap();
        assert_eq!(date_spec, DateSpecifier::Today);
    }
}