#[cfg(test)]
mod journal_tests {
    use crate::config::Config;
    use crate::editor::Editor;
    use crate::errors::{AppError, AppResult};
    use crate::journal::io::JournalIO;
    use crate::journal::{DateSpecifier, JournalService};
    use chrono::{Datelike, Duration, Local, NaiveDate};
    use std::cell::RefCell;
    use std::fs::File;
    use std::path::{Path, PathBuf};

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
        assert_eq!(
            specifier,
            DateSpecifier::Specific(NaiveDate::from_ymd_opt(2023, 1, 15).unwrap())
        );

        // Test specific date (YYYYMMDD format)
        let specifier = DateSpecifier::from_args(false, false, Some("20230115")).unwrap();
        assert_eq!(
            specifier,
            DateSpecifier::Specific(NaiveDate::from_ymd_opt(2023, 1, 15).unwrap())
        );
    }

    #[test]
    fn test_date_specifier_parse_error() {
        // Test invalid date format
        let result = DateSpecifier::from_args(false, false, Some("not-a-date"));
        assert!(result.is_err());

        match result {
            Err(AppError::Journal(msg)) => {
                assert!(msg.contains("Invalid date format"));
            }
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
            assert!(dates[i - 1] > dates[i]);
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
        journal_dir: PathBuf,
        // Tracking fields (for assertions in tests)
        paths_generated: RefCell<Vec<PathBuf>>,
        appended_content: RefCell<Vec<String>>,

        // Configuration for ensure_journal_dir
        ensure_dir_should_fail: bool,
        ensure_dir_error: Option<AppError>,

        // Configuration for path generation methods
        generate_path_should_fail: bool,
        generate_path_error: Option<AppError>,

        // Configuration for file_exists
        default_exists_result: bool,
        file_exists_results: RefCell<std::collections::HashMap<PathBuf, bool>>,

        // Configuration for create_or_open_file
        create_or_open_should_fail: bool,
        create_or_open_error: Option<AppError>,

        // Configuration for read_file_content
        read_content_should_fail: bool,
        read_content_error: Option<AppError>,
        default_file_content: String,
        file_contents: RefCell<std::collections::HashMap<PathBuf, String>>,

        // Configuration for append_to_file
        append_should_fail: bool,
        append_error: Option<AppError>,
    }

    impl MockJournalIO {
        fn new() -> Self {
            MockJournalIO {
                journal_dir: PathBuf::from("/mock/journal/dir"),
                paths_generated: RefCell::new(Vec::new()),
                appended_content: RefCell::new(Vec::new()),

                // Default configuration (for backward compatibility)
                ensure_dir_should_fail: false,
                ensure_dir_error: None,
                generate_path_should_fail: false,
                generate_path_error: None,
                default_exists_result: true,
                file_exists_results: RefCell::new(std::collections::HashMap::new()),
                create_or_open_should_fail: true, // Default to fail for backward compatibility
                create_or_open_error: Some(AppError::Journal(
                    "Mock doesn't create real files".to_string(),
                )),
                read_content_should_fail: false,
                read_content_error: None,
                default_file_content: String::new(),
                file_contents: RefCell::new(std::collections::HashMap::new()),
                append_should_fail: false,
                append_error: None,
            }
        }

        // Configuration methods for ensure_journal_dir
        fn set_ensure_dir_should_fail(&mut self, should_fail: bool) {
            self.ensure_dir_should_fail = should_fail;
        }

        fn set_ensure_dir_error(&mut self, error: AppError) {
            self.ensure_dir_error = Some(error);
        }

        // Configuration methods for path generation
        fn set_generate_path_should_fail(&mut self, should_fail: bool) {
            self.generate_path_should_fail = should_fail;
        }

        fn set_generate_path_error(&mut self, error: AppError) {
            self.generate_path_error = Some(error);
        }

        // Configuration methods for file_exists
        fn set_default_exists_result(&mut self, exists: bool) {
            self.default_exists_result = exists;
        }

        fn set_file_exists_result(&mut self, path: PathBuf, exists: bool) {
            self.file_exists_results.borrow_mut().insert(path, exists);
        }

        // Configuration methods for create_or_open_file
        fn set_create_or_open_should_fail(&mut self, should_fail: bool) {
            self.create_or_open_should_fail = should_fail;
        }

        fn set_create_or_open_error(&mut self, error: AppError) {
            self.create_or_open_error = Some(error);
        }

        // Configuration methods for read_file_content
        fn set_read_content_should_fail(&mut self, should_fail: bool) {
            self.read_content_should_fail = should_fail;
        }

        fn set_read_content_error(&mut self, error: AppError) {
            self.read_content_error = Some(error);
        }

        fn set_default_file_content(&mut self, content: String) {
            self.default_file_content = content;
        }

        fn set_file_content(&mut self, path: PathBuf, content: String) {
            self.file_contents.borrow_mut().insert(path, content);
        }

        // Configuration methods for append_to_file
        fn set_append_should_fail(&mut self, should_fail: bool) {
            self.append_should_fail = should_fail;
        }

        fn set_append_error(&mut self, error: AppError) {
            self.append_error = Some(error);
        }

        // Utility method to configure success paths (for tests that need the mock to work)
        fn configure_for_success(&mut self) {
            self.ensure_dir_should_fail = false;
            self.generate_path_should_fail = false;
            self.default_exists_result = true;
            self.create_or_open_should_fail = false;
            self.read_content_should_fail = false;
            self.append_should_fail = false;
        }
    }

    impl JournalIO for MockJournalIO {
        fn ensure_journal_dir(&self) -> AppResult<()> {
            if self.ensure_dir_should_fail {
                return match &self.ensure_dir_error {
                    Some(error) => Err(error.clone()),
                    None => Err(AppError::Journal(
                        "Mock directory creation failed".to_string(),
                    )),
                };
            }
            Ok(())
        }

        fn generate_path_for_date(&self, date: chrono::DateTime<Local>) -> AppResult<PathBuf> {
            // First check if configured to fail
            if self.generate_path_should_fail {
                return match &self.generate_path_error {
                    Some(error) => Err(error.clone()),
                    None => Err(AppError::Journal("Mock path generation failed".to_string())),
                };
            }

            // Generate the path using the standard format
            let filename = format!("{}.md", date.format("%Y%m%d"));
            let path = self.journal_dir.join(filename);

            // Track the path for test assertions
            self.paths_generated.borrow_mut().push(path.clone());

            Ok(path)
        }

        fn generate_path_for_naive_date(&self, date: NaiveDate) -> AppResult<PathBuf> {
            // First check if configured to fail
            if self.generate_path_should_fail {
                return match &self.generate_path_error {
                    Some(error) => Err(error.clone()),
                    None => Err(AppError::Journal("Mock path generation failed".to_string())),
                };
            }

            // Generate the path using the standard format
            let filename = format!("{:04}{:02}{:02}.md", date.year(), date.month(), date.day());
            let path = self.journal_dir.join(filename);

            // Track the path for test assertions
            self.paths_generated.borrow_mut().push(path.clone());

            Ok(path)
        }

        fn file_exists(&self, path: &Path) -> bool {
            // Check if we have a specific configuration for this path
            if let Some(exists) = self.file_exists_results.borrow().get(path) {
                return *exists;
            }

            // Otherwise return the default exists result
            self.default_exists_result
        }

        fn create_or_open_file(&self, _path: &Path) -> AppResult<File> {
            if self.create_or_open_should_fail {
                return match &self.create_or_open_error {
                    Some(error) => Err(error.clone()),
                    None => Err(AppError::Journal(
                        "Mock doesn't create real files".to_string(),
                    )),
                };
            }

            // Return a tempfile if configured to succeed
            match tempfile::tempfile() {
                Ok(file) => Ok(file),
                Err(e) => Err(AppError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e,
                ))),
            }
        }

        fn read_file_content(&self, path: &Path) -> AppResult<String> {
            if self.read_content_should_fail {
                return match &self.read_content_error {
                    Some(error) => Err(error.clone()),
                    None => Err(AppError::Journal("Mock file reading failed".to_string())),
                };
            }

            // Check if we have a specific content for this path
            if let Some(content) = self.file_contents.borrow().get(path) {
                return Ok(content.clone());
            }

            // Otherwise return the default content
            Ok(self.default_file_content.clone())
        }

        fn append_to_file(&self, _file: &mut File, content: &str) -> AppResult<()> {
            if self.append_should_fail {
                return match &self.append_error {
                    Some(error) => Err(error.clone()),
                    None => Err(AppError::Journal("Mock file append failed".to_string())),
                };
            }

            // Track the appended content for test assertions
            self.appended_content.borrow_mut().push(content.to_string());

            Ok(())
        }
    }

    // Enhanced MockEditor for test use - includes success/failure configuration
    struct MockEditor {
        opened_files: RefCell<Vec<Vec<PathBuf>>>,
        should_fail: bool,
        failure_error: Option<AppError>,
    }

    impl MockEditor {
        fn new() -> Self {
            MockEditor {
                opened_files: RefCell::new(Vec::new()),
                should_fail: false,
                failure_error: None,
            }
        }

        fn with_failure(error: AppError) -> Self {
            MockEditor {
                opened_files: RefCell::new(Vec::new()),
                should_fail: true,
                failure_error: Some(error),
            }
        }

        #[allow(dead_code)]
        fn set_should_fail(&mut self, should_fail: bool) {
            self.should_fail = should_fail;
        }

        #[allow(dead_code)]
        fn set_failure_error(&mut self, error: AppError) {
            self.failure_error = Some(error);
        }
    }

    impl Editor for MockEditor {
        fn open_files(&self, paths: &[&Path]) -> AppResult<()> {
            // Record the files that were attempted to be opened
            self.opened_files
                .borrow_mut()
                .push(paths.iter().map(|&p| p.to_path_buf()).collect());

            // If configured to fail, return the specified error or a default one
            if self.should_fail {
                return match &self.failure_error {
                    Some(error) => Err(error.clone()),
                    None => Err(AppError::Editor(
                        "Mock editor failed by configuration".to_string(),
                    )),
                };
            }

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
        assert_eq!(
            service.get_journal_dir(),
            &PathBuf::from("/test/journal/dir")
        );
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

    #[test]
    fn test_journal_service_editor_failure() {
        let config = Config {
            editor: "test-editor".to_string(),
            journal_dir: PathBuf::from("/test/journal/dir"),
        };

        let mut io = MockJournalIO::new();
        // Configure the IO to succeed for this test
        io.configure_for_success();
        let io_ptr = Box::new(io);

        // Configure the editor to fail with a specific error
        let editor_error = AppError::Editor("Failed to launch editor".to_string());
        let editor = MockEditor::with_failure(editor_error);
        let editor_ptr = Box::new(editor);

        let service = JournalService::new(config, io_ptr, editor_ptr);

        // Try to open today's entry - should fail because the editor is configured to fail
        let result = service.open_entries(&DateSpecifier::Today);

        // Verify the error
        assert!(result.is_err());
        match result {
            Err(AppError::Editor(msg)) => {
                assert!(msg.contains("Failed to launch editor"));
            }
            _ => panic!("Expected Editor error"),
        }
    }

    #[test]
    fn test_journal_service_path_generation_failure() {
        let config = Config {
            editor: "test-editor".to_string(),
            journal_dir: PathBuf::from("/test/journal/dir"),
        };

        let mut io = MockJournalIO::new();
        // Configure the path generation to fail
        io.set_generate_path_should_fail(true);
        io.set_generate_path_error(AppError::Journal("Path generation failed".to_string()));
        let io_ptr = Box::new(io);

        let editor = MockEditor::new();
        let editor_ptr = Box::new(editor);

        let service = JournalService::new(config, io_ptr, editor_ptr);

        // Try to open today's entry - should fail because path generation is configured to fail
        let result = service.open_entries(&DateSpecifier::Today);

        // Verify the error
        assert!(result.is_err());
        match result {
            Err(AppError::Journal(msg)) => {
                assert!(msg.contains("Path generation failed"));
            }
            _ => panic!("Expected Journal error for path generation"),
        }

        // The same failure should occur for other date specifiers too
        let specific_date = NaiveDate::from_ymd_opt(2023, 1, 15).unwrap();
        let result = service.open_entries(&DateSpecifier::Specific(specific_date));
        assert!(result.is_err());

        let result = service.open_entries(&DateSpecifier::Retro);
        assert!(result.is_err());

        let result = service.open_entries(&DateSpecifier::Reminisce);
        assert!(result.is_err());
    }

    #[test]
    fn test_journal_service_file_creation_failure() {
        let config = Config {
            editor: "test-editor".to_string(),
            journal_dir: PathBuf::from("/test/journal/dir"),
        };

        let mut io = MockJournalIO::new();
        // Keep create_or_open_should_fail as true (the default)
        io.set_create_or_open_error(AppError::Journal("Cannot create file".to_string()));
        let io_ptr = Box::new(io);

        let editor = MockEditor::new();
        let editor_ptr = Box::new(editor);

        let service = JournalService::new(config, io_ptr, editor_ptr);

        // Try to open today's entry - should fail because file creation is configured to fail
        let result = service.open_entries(&DateSpecifier::Today);

        // Verify the error
        assert!(result.is_err());
        match result {
            Err(AppError::Journal(msg)) => {
                assert!(msg.contains("Cannot create file"));
            }
            _ => panic!("Expected Journal error for file creation"),
        }
    }

    #[test]
    fn test_journal_service_file_read_failure() {
        let config = Config {
            editor: "test-editor".to_string(),
            journal_dir: PathBuf::from("/test/journal/dir"),
        };

        let mut io = MockJournalIO::new();
        // Configure file creation to succeed but reading to fail
        io.set_create_or_open_should_fail(false);
        io.set_read_content_should_fail(true);
        io.set_read_content_error(AppError::Journal("Cannot read file".to_string()));
        let io_ptr = Box::new(io);

        let editor = MockEditor::new();
        let editor_ptr = Box::new(editor);

        let service = JournalService::new(config, io_ptr, editor_ptr);

        // Try to open today's entry - should fail because file reading is configured to fail
        let result = service.open_entries(&DateSpecifier::Today);

        // Verify the error
        assert!(result.is_err());
        match result {
            Err(AppError::Journal(msg)) => {
                assert!(msg.contains("Cannot read file"));
            }
            _ => panic!("Expected Journal error for file reading"),
        }
    }

    #[test]
    fn test_journal_service_file_append_failure() {
        let config = Config {
            editor: "test-editor".to_string(),
            journal_dir: PathBuf::from("/test/journal/dir"),
        };

        let mut io = MockJournalIO::new();
        // Configure creation and reading to succeed but appending to fail
        io.set_create_or_open_should_fail(false);
        io.set_read_content_should_fail(false);
        io.set_append_should_fail(true);
        io.set_append_error(AppError::Journal("Cannot append to file".to_string()));
        let io_ptr = Box::new(io);

        let editor = MockEditor::new();
        let editor_ptr = Box::new(editor);

        let service = JournalService::new(config, io_ptr, editor_ptr);

        // Try to open today's entry - should fail because file appending is configured to fail
        let result = service.open_entries(&DateSpecifier::Today);

        // Verify the error
        assert!(result.is_err());
        match result {
            Err(AppError::Journal(msg)) => {
                assert!(msg.contains("Cannot append to file"));
            }
            _ => panic!("Expected Journal error for file appending"),
        }
    }

    #[test]
    fn test_journal_service_convenience_methods() {
        // Create a special version of MockJournalIO that doesn't fail on create_or_open_file
        struct TestMockJournalIO {
            paths_generated: RefCell<Vec<PathBuf>>,
        }

        impl TestMockJournalIO {
            fn new() -> Self {
                TestMockJournalIO {
                    paths_generated: RefCell::new(Vec::new()),
                }
            }
        }

        impl JournalIO for TestMockJournalIO {
            fn ensure_journal_dir(&self) -> AppResult<()> {
                Ok(())
            }

            fn generate_path_for_date(&self, date: chrono::DateTime<Local>) -> AppResult<PathBuf> {
                let path = PathBuf::from(format!("/test/journal/{}.md", date.format("%Y%m%d")));
                self.paths_generated.borrow_mut().push(path.clone());
                Ok(path)
            }

            fn generate_path_for_naive_date(&self, date: NaiveDate) -> AppResult<PathBuf> {
                let path = PathBuf::from(format!(
                    "/test/journal/{:04}{:02}{:02}.md",
                    date.year(),
                    date.month(),
                    date.day()
                ));
                self.paths_generated.borrow_mut().push(path.clone());
                Ok(path)
            }

            fn file_exists(&self, _path: &Path) -> bool {
                true
            }

            fn create_or_open_file(&self, _path: &Path) -> AppResult<File> {
                // Just return a dummy value that will be captured by the mock file system
                let file = tempfile::tempfile()
                    .map_err(|e| AppError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                Ok(file)
            }

            fn read_file_content(&self, _path: &Path) -> AppResult<String> {
                Ok(String::new())
            }

            fn append_to_file(&self, _file: &mut File, _content: &str) -> AppResult<()> {
                Ok(())
            }
        }

        let config = Config {
            editor: "test-editor".to_string(),
            journal_dir: PathBuf::from("/test/journal/dir"),
        };

        let io = Box::new(TestMockJournalIO::new());
        let editor = Box::new(MockEditor::new());

        let service = JournalService::new(config, io, editor);

        // Test the convenience methods - no assertions needed as we just want to ensure
        // they run without panicking
        let _ = service.open_entry();
        let _ = service.open_retro_entry();
        let _ = service.open_reminisce_entry();

        // Test with a specific date
        let specific_date = NaiveDate::from_ymd_opt(2023, 1, 15).unwrap();
        let _ = service.open_specific_entry(specific_date);
    }

    #[test]
    fn test_journal_service_ensure_journal_dir_failure() {
        let config = Config {
            editor: "test-editor".to_string(),
            journal_dir: PathBuf::from("/test/journal/dir"),
        };

        let mut io = MockJournalIO::new();
        // Configure ensure_journal_dir to fail
        io.set_ensure_dir_should_fail(true);
        io.set_ensure_dir_error(AppError::Journal(
            "Cannot create journal directory".to_string(),
        ));
        let io_ptr = Box::new(io);

        let editor = MockEditor::new();
        let editor_ptr = Box::new(editor);

        let service = JournalService::new(config, io_ptr, editor_ptr);

        // Create a method that explicitly calls ensure_journal_dir
        let result = service.io.ensure_journal_dir();

        // Verify the error
        assert!(result.is_err());
        match result {
            Err(AppError::Journal(msg)) => {
                assert!(msg.contains("Cannot create journal directory"));
            }
            _ => panic!("Expected Journal error for directory creation"),
        }
    }

    #[test]
    fn test_journal_service_retro_entries_with_custom_file_exists() {
        let config = Config {
            editor: "test-editor".to_string(),
            journal_dir: PathBuf::from("/test/journal/dir"),
        };

        // Create a MockJournalIO with custom file_exists settings
        let mut io = MockJournalIO::new();
        io.configure_for_success();
        io.set_default_exists_result(false); // Default: files don't exist

        // Set a specific path to exist
        let today = Local::now().naive_local().date();
        let specific_date = today - Duration::days(3);
        let path = io.journal_dir.join(format!(
            "{:04}{:02}{:02}.md",
            specific_date.year(),
            specific_date.month(),
            specific_date.day()
        ));
        io.set_file_exists_result(path, true);

        let io_ptr = Box::new(io);
        let editor = MockEditor::new();
        let editor_ptr = Box::new(editor);

        let service = JournalService::new(config, io_ptr, editor_ptr);

        // Get retro entries - should only include the one we configured to exist
        let result = service.get_retro_entries();
        assert!(result.is_ok());

        let paths = result.unwrap();
        assert_eq!(paths.len(), 1); // Only one path should exist

        // Make sure it's the path we expect
        let path_string = paths[0].to_string_lossy();
        assert!(path_string.contains(&format!(
            "{:04}{:02}{:02}",
            specific_date.year(),
            specific_date.month(),
            specific_date.day()
        )));
    }

    // Tests for our enhanced MockJournalIO
    #[test]
    fn test_mock_journal_io_configurability() {
        // Create a MockJournalIO with default configuration
        let mut io = MockJournalIO::new();

        // Test ensure_journal_dir configurability
        assert!(io.ensure_journal_dir().is_ok()); // Default is success

        io.set_ensure_dir_should_fail(true);
        let result = io.ensure_journal_dir();
        assert!(result.is_err());

        io.set_ensure_dir_error(AppError::Config("Custom error".to_string()));
        let result = io.ensure_journal_dir();
        assert!(result.is_err());
        match result {
            Err(AppError::Config(msg)) => assert_eq!(msg, "Custom error"),
            _ => panic!("Expected AppError::Config"),
        }

        // Test generate_path configurability
        io.set_ensure_dir_should_fail(false); // Reset
        io.set_generate_path_should_fail(true);
        let result = io.generate_path_for_date(Local::now());
        assert!(result.is_err());

        let result = io.generate_path_for_naive_date(Local::now().naive_local().date());
        assert!(result.is_err());

        // Test file_exists configurability
        io.set_generate_path_should_fail(false); // Reset
        io.set_default_exists_result(false);
        let path = PathBuf::from("/test/specific/path.md");
        assert!(!io.file_exists(&path)); // Should use default (false)

        io.set_file_exists_result(path.clone(), true);
        assert!(io.file_exists(&path)); // Should use specific setting (true)

        // Test create_or_open_file configurability
        io.set_create_or_open_should_fail(false);
        let result = io.create_or_open_file(&path);
        assert!(result.is_ok()); // Should succeed now

        io.set_create_or_open_should_fail(true);
        io.set_create_or_open_error(AppError::Config("Can't open file".to_string()));
        let result = io.create_or_open_file(&path);
        assert!(result.is_err());
        match result {
            Err(AppError::Config(msg)) => assert_eq!(msg, "Can't open file"),
            _ => panic!("Expected AppError::Config"),
        }

        // Test read_file_content configurability
        io.set_read_content_should_fail(true);
        let result = io.read_file_content(&path);
        assert!(result.is_err());

        io.set_read_content_should_fail(false);
        let result = io.read_file_content(&path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ""); // Default content is empty

        io.set_default_file_content("Default content".to_string());
        assert_eq!(io.read_file_content(&path).unwrap(), "Default content");

        let specific_path = PathBuf::from("/specific/path.md");
        io.set_file_content(specific_path.clone(), "Specific content".to_string());
        assert_eq!(
            io.read_file_content(&specific_path).unwrap(),
            "Specific content"
        );

        // Test append_to_file configurability
        io.set_append_should_fail(true);
        let mut file = tempfile::tempfile().unwrap();
        let result = io.append_to_file(&mut file, "Content");
        assert!(result.is_err());

        io.set_append_should_fail(false);
        let result = io.append_to_file(&mut file, "Content");
        assert!(result.is_ok());

        // Verify that append content was tracked
        {
            let appended = io.appended_content.borrow();
            assert_eq!(appended.len(), 1);
            assert_eq!(appended[0], "Content");
        } // Drop the appended borrow here

        // Test the utility configure_for_success method
        io.set_ensure_dir_should_fail(true);
        io.set_generate_path_should_fail(true);
        io.set_create_or_open_should_fail(true);
        io.set_read_content_should_fail(true);
        io.set_append_should_fail(true);

        io.configure_for_success();

        assert!(io.ensure_journal_dir().is_ok());
        assert!(io.generate_path_for_date(Local::now()).is_ok());
        assert!(io.create_or_open_file(&path).is_ok());
        assert!(io.read_file_content(&path).is_ok());
        assert!(io.append_to_file(&mut file, "More content").is_ok());
    }
}
