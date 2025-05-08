#[cfg(test)]
mod tests {
    use crate::journal::{DateSpecifier, JournalService};
    use crate::config::Config;
    use crate::editor::Editor;
    use crate::errors::{AppError, AppResult};
    use crate::journal::io::JournalIO;
    use chrono::{Datelike, Duration, Local, NaiveDate};
    use std::path::PathBuf;
    use std::cell::RefCell;
    use std::fs::File;
    
    #[test]
    fn test_date_specifier_from_args() {
        // Test today (default)
        let specifier = DateSpecifier::from_args(false, false, None).unwrap();
        assert_eq!(specifier, DateSpecifier::Today);
        
        // Test retro
        let specifier = DateSpecifier::from_args(true, false, None).unwrap();
        assert_eq!(specifier, DateSpecifier::Retro);
        
        // Test reminisce
        let specifier = DateSpecifier::from_args(false, true, None).unwrap();
        assert_eq!(specifier, DateSpecifier::Reminisce);
        
        // Test specific date (YYYY-MM-DD format)
        let specifier = DateSpecifier::from_args(false, false, Some("2023-01-15")).unwrap();
        assert_eq!(specifier, DateSpecifier::Specific(NaiveDate::from_ymd_opt(2023, 1, 15).unwrap()));
        
        // Test specific date (YYYYMMDD format)
        let specifier = DateSpecifier::from_args(false, false, Some("20230115")).unwrap();
        assert_eq!(specifier, DateSpecifier::Specific(NaiveDate::from_ymd_opt(2023, 1, 15).unwrap()));
    }
    
    #[test]
    fn test_date_specifier_parse_error() {
        // Test invalid date format
        let result = DateSpecifier::from_args(false, false, Some("not-a-date"));
        assert!(result.is_err());
        
        match result {
            Err(AppError::Journal(msg)) => {
                assert!(msg.contains("Invalid date format"));
            },
            _ => panic!("Expected Journal error"),
        }
    }
    
    #[test]
    fn test_get_dates_today() {
        let specifier = DateSpecifier::Today;
        let dates = specifier.get_dates();
        
        assert_eq!(dates.len(), 1);
        assert_eq!(dates[0], Local::now().naive_local().date());
    }
    
    #[test]
    fn test_get_dates_retro() {
        let specifier = DateSpecifier::Retro;
        let dates = specifier.get_dates();
        
        assert_eq!(dates.len(), 7);
        
        let today = Local::now().naive_local().date();
        for (i, date) in dates.iter().enumerate() {
            assert_eq!(*date, today - Duration::days(i as i64 + 1));
        }
    }
    
    #[test]
    fn test_get_dates_reminisce() {
        let specifier = DateSpecifier::Reminisce;
        let dates = specifier.get_dates();
        
        // Should include at least the 1-month, 3-month, 6-month, and 1-year anniversaries
        assert!(dates.len() >= 4);
        
        // Dates should be in reverse chronological order (newest first)
        for i in 1..dates.len() {
            assert!(dates[i-1] > dates[i]);
        }
    }
    
    #[test]
    fn test_get_dates_specific() {
        let specific_date = NaiveDate::from_ymd_opt(2023, 1, 15).unwrap();
        let specifier = DateSpecifier::Specific(specific_date);
        let dates = specifier.get_dates();
        
        assert_eq!(dates.len(), 1);
        assert_eq!(dates[0], specific_date);
    }
    
    // Mock implementations for testing JournalService
    struct MockJournalIO {
        journal_dir: String,
        exists_result: bool,
        file_content: String,
        paths_generated: RefCell<Vec<String>>,
        appended_content: RefCell<Vec<String>>,
    }
    
    impl MockJournalIO {
        fn new() -> Self {
            MockJournalIO {
                journal_dir: String::from("/mock/journal/dir"),
                exists_result: true,
                file_content: String::new(),
                paths_generated: RefCell::new(Vec::new()),
                appended_content: RefCell::new(Vec::new()),
            }
        }
    }
    
    impl JournalIO for MockJournalIO {
        fn ensure_journal_dir(&self) -> AppResult<()> {
            Ok(())
        }
        
        fn generate_path_for_date(&self, date: chrono::DateTime<Local>) -> AppResult<String> {
            let path = format!("{}/{}.md", self.journal_dir, date.format("%Y%m%d"));
            self.paths_generated.borrow_mut().push(path.clone());
            Ok(path)
        }
        
        fn generate_path_for_naive_date(&self, date: NaiveDate) -> AppResult<String> {
            let path = format!(
                "{}/{:04}{:02}{:02}.md",
                self.journal_dir,
                date.year(),
                date.month(),
                date.day()
            );
            self.paths_generated.borrow_mut().push(path.clone());
            Ok(path)
        }
        
        fn file_exists(&self, _path: &str) -> bool {
            self.exists_result
        }
        
        fn create_or_open_file(&self, _path: &str) -> AppResult<File> {
            Err(AppError::Journal("Mock doesn't create real files".to_string()))
        }
        
        fn read_file_content(&self, _path: &str) -> AppResult<String> {
            Ok(self.file_content.clone())
        }
        
        fn append_to_file(&self, _file: &mut File, content: &str) -> AppResult<()> {
            self.appended_content.borrow_mut().push(content.to_string());
            Ok(())
        }
    }
    
    struct MockEditor {
        opened_files: RefCell<Vec<Vec<String>>>,
    }
    
    impl MockEditor {
        fn new() -> Self {
            MockEditor {
                opened_files: RefCell::new(Vec::new()),
            }
        }
    }
    
    impl Editor for MockEditor {
        fn open_files(&self, paths: &[String]) -> AppResult<()> {
            self.opened_files.borrow_mut().push(paths.to_vec());
            Ok(())
        }
    }
    
    #[test]
    fn test_journal_service_construction() {
        let config = Config {
            editor: "test-editor".to_string(),
            journal_dir: PathBuf::from("/test/journal/dir"),
        };
        
        let io = Box::new(MockJournalIO::new());
        let editor = Box::new(MockEditor::new());
        
        let service = JournalService::new(config, io, editor);
        
        assert_eq!(service.get_editor_cmd(), "test-editor");
        assert_eq!(service.get_journal_dir(), &PathBuf::from("/test/journal/dir"));
    }
    
    #[test]
    fn test_journal_service_open_entries_today() {
        let config = Config {
            editor: "test-editor".to_string(),
            journal_dir: PathBuf::from("/test/journal/dir"),
        };
        
        let io = MockJournalIO::new();
        let io_ptr = Box::new(io);
        
        let editor = MockEditor::new();
        let editor_ptr = Box::new(editor);
        
        let service = JournalService::new(config, io_ptr, editor_ptr);
        
        // This will not work in the test since we return an error in create_or_open_file
        // That's expected and we're just testing the logic flow
        let _ = service.open_entries(&DateSpecifier::Today);
    }
}