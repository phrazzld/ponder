#[cfg(test)]
mod io_tests {
    use crate::errors::AppResult;
    use crate::journal::io::JournalIO;
    use chrono::{DateTime, Datelike, Local, NaiveDate};
    use std::fs::{self, File, OpenOptions};
    use std::io::{Read, Write};
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    struct TestFileSystemIO {
        pub journal_dir: PathBuf,
    }

    impl JournalIO for TestFileSystemIO {
        fn ensure_journal_dir(&self) -> AppResult<()> {
            if !self.journal_dir.exists() {
                fs::create_dir_all(&self.journal_dir)?;
            }
            Ok(())
        }

        fn generate_path_for_date(&self, date: DateTime<Local>) -> AppResult<String> {
            let filename = format!("{}.md", date.format("%Y%m%d"));
            let filepath = self.journal_dir.join(filename);
            Ok(filepath.to_string_lossy().to_string())
        }

        fn generate_path_for_naive_date(&self, date: NaiveDate) -> AppResult<String> {
            let filename = format!("{:04}{:02}{:02}.md", date.year(), date.month(), date.day());
            let filepath = self.journal_dir.join(filename);
            Ok(filepath.to_string_lossy().to_string())
        }

        fn file_exists(&self, path: &str) -> bool {
            Path::new(path).exists()
        }

        fn create_or_open_file(&self, path: &str) -> AppResult<File> {
            let file = OpenOptions::new()
                .read(true)
                .create(true)
                .append(true)
                .open(path)?;
            Ok(file)
        }

        fn read_file_content(&self, path: &str) -> AppResult<String> {
            let mut content = String::new();
            let mut file = File::open(path)?;
            file.read_to_string(&mut content)?;
            Ok(content)
        }

        fn append_to_file(&self, file: &mut File, content: &str) -> AppResult<()> {
            file.write_all(content.as_bytes())?;
            Ok(())
        }
    }

    #[test]
    fn test_file_operations() -> AppResult<()> {
        let temp_dir = tempdir()?;
        let journal_dir = temp_dir.path().to_path_buf();

        let io = TestFileSystemIO { journal_dir };
        io.ensure_journal_dir()?;

        let test_path = io.journal_dir.join("test.md").to_string_lossy().to_string();
        let mut file = io.create_or_open_file(&test_path)?;

        io.append_to_file(&mut file, "Test content")?;

        let content = io.read_file_content(&test_path)?;
        assert_eq!(content, "Test content");

        assert!(io.file_exists(&test_path));
        assert!(!io.file_exists(
            &io.journal_dir
                .join("nonexistent.md")
                .to_string_lossy()
                .to_string()
        ));

        Ok(())
    }
}
