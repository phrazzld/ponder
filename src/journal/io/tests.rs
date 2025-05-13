#[cfg(test)]
mod io_tests {
    use crate::errors::AppResult;
    use crate::journal::io::{FileSystemIO, JournalIO};
    use chrono::{DateTime, Local, NaiveDate};
    use tempfile::tempdir;

    #[test]
    fn test_file_operations() -> AppResult<()> {
        let temp_dir = tempdir()?;
        let journal_dir = temp_dir.path().to_path_buf();

        let io = FileSystemIO { journal_dir };
        io.ensure_journal_dir()?;

        let test_path = io.journal_dir.join("test.md");
        let mut file = io.create_or_open_file(&test_path)?;

        io.append_to_file(&mut file, "Test content")?;

        let content = io.read_file_content(&test_path)?;
        assert_eq!(content, "Test content");

        assert!(io.file_exists(&test_path));
        assert!(!io.file_exists(&io.journal_dir.join("nonexistent.md")));

        Ok(())
    }

    #[test]
    fn test_path_generation() -> AppResult<()> {
        let temp_dir = tempdir()?;
        let journal_dir = temp_dir.path().to_path_buf();

        let io = FileSystemIO {
            journal_dir: journal_dir.clone(),
        };

        // Test DateTime path generation
        let date = DateTime::parse_from_rfc3339("2023-01-15T12:00:00+00:00")
            .unwrap()
            .with_timezone(&Local);
        let path = io.generate_path_for_date(date)?;

        // The expected filename should be 20230115.md
        let expected_path = journal_dir.join("20230115.md");
        assert_eq!(path, expected_path);

        // Test NaiveDate path generation
        let naive_date = NaiveDate::from_ymd_opt(2023, 1, 15).unwrap();
        let path = io.generate_path_for_naive_date(naive_date)?;

        // The expected filename should again be 20230115.md
        assert_eq!(path, expected_path);

        Ok(())
    }

    #[test]
    fn test_ensure_journal_dir_creates_directory() -> AppResult<()> {
        let temp_dir = tempdir()?;

        // Create a non-existent subdirectory path
        let journal_dir = temp_dir.path().join("subdir").join("journal");

        // Verify the directory doesn't exist yet
        assert!(!journal_dir.exists());

        let io = FileSystemIO {
            journal_dir: journal_dir.clone(),
        };

        // This should create the directory
        io.ensure_journal_dir()?;

        // Now verify the directory exists
        assert!(journal_dir.exists());

        Ok(())
    }
}
