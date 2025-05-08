use crate::errors::AppResult;
use chrono::{DateTime, Datelike, Local, NaiveDate};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

#[cfg(test)]
mod tests;

pub trait JournalIO {
    fn ensure_journal_dir(&self) -> AppResult<()>;
    fn generate_path_for_date(&self, date: DateTime<Local>) -> AppResult<String>;
    fn generate_path_for_naive_date(&self, date: NaiveDate) -> AppResult<String>;
    fn file_exists(&self, path: &str) -> bool;
    fn create_or_open_file(&self, path: &str) -> AppResult<File>;
    fn read_file_content(&self, path: &str) -> AppResult<String>;
    fn append_to_file(&self, file: &mut File, content: &str) -> AppResult<()>;
}

pub struct FileSystemIO {
    pub journal_dir: String,
}

impl JournalIO for FileSystemIO {
    fn ensure_journal_dir(&self) -> AppResult<()> {
        let path = Path::new(&self.journal_dir);
        if !path.exists() {
            std::fs::create_dir_all(path)?;
        }
        Ok(())
    }

    fn generate_path_for_date(&self, date: DateTime<Local>) -> AppResult<String> {
        Ok(format!("{}/{}.md", self.journal_dir, date.format("%Y%m%d")))
    }

    fn generate_path_for_naive_date(&self, date: NaiveDate) -> AppResult<String> {
        Ok(format!(
            "{}/{:04}{:02}{:02}.md",
            self.journal_dir,
            date.year(),
            date.month(),
            date.day()
        ))
    }

    fn file_exists(&self, path: &str) -> bool {
        Path::new(path).exists()
    }

    fn create_or_open_file(&self, path: &str) -> AppResult<File> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
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