pub mod io;

#[cfg(test)]
mod tests;

use crate::config::Config;
use crate::editor::Editor;
use crate::errors::{AppError, AppResult};
use chrono::{Duration, Local, Months, NaiveDate};
use io::JournalIO;
use std::path::PathBuf;

/// Represents different ways to specify a date in the journal system
#[derive(Debug, Clone, PartialEq)]
pub enum DateSpecifier {
    /// Today's entry
    Today,
    
    /// Entries from the past week (excluding today)
    Retro,
    
    /// Entries from significant past intervals (1 month ago, 3 months ago, yearly anniversaries)
    Reminisce,
    
    /// A specific date's entry
    Specific(NaiveDate),
}

impl DateSpecifier {
    /// Creates a DateSpecifier from command-line arguments
    pub fn from_args(retro: bool, reminisce: bool, date_str: Option<&str>) -> AppResult<Self> {
        // If a specific date is provided, it takes precedence
        if let Some(date_str) = date_str {
            return Self::parse_date_string(date_str)
                .map(DateSpecifier::Specific)
                .map_err(|e| AppError::Journal(format!("Invalid date format: {}", e)));
        }
        
        // Otherwise, use flags
        if reminisce {
            Ok(DateSpecifier::Reminisce)
        } else if retro {
            Ok(DateSpecifier::Retro)
        } else {
            Ok(DateSpecifier::Today)
        }
    }
    
    /// Parse a date string in YYYY-MM-DD or YYYYMMDD format
    fn parse_date_string(date_str: &str) -> Result<NaiveDate, chrono::ParseError> {
        // Try parsing in YYYY-MM-DD format first
        NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .or_else(|_| NaiveDate::parse_from_str(date_str, "%Y%m%d"))
    }
    
    /// Gets the relevant dates for this date specifier
    pub fn get_dates(&self) -> Vec<NaiveDate> {
        match self {
            DateSpecifier::Today => {
                vec![Local::now().naive_local().date()]
            },
            DateSpecifier::Retro => {
                let now = Local::now().naive_local().date();
                (1..=7).map(|days| now - Duration::days(days)).collect()
            },
            DateSpecifier::Reminisce => {
                let now = Local::now();
                let today = now.naive_local().date();
                let mut dates = Vec::new();
                
                // Add specific month intervals
                if let Some(date) = today.checked_sub_months(Months::new(1)) {
                    dates.push(date);
                }
                if let Some(date) = today.checked_sub_months(Months::new(3)) {
                    dates.push(date);
                }
                if let Some(date) = today.checked_sub_months(Months::new(6)) {
                    dates.push(date);
                }
                
                // Add every year ago for the past hundred years
                for year in 1..=100 {
                    if let Some(date) = today.checked_sub_months(Months::new(12 * year)) {
                        dates.push(date);
                    }
                }
                
                // Remove duplicates and sort the dates
                dates.sort();
                dates.dedup();
                dates.reverse();
                
                dates
            },
            DateSpecifier::Specific(date) => {
                vec![*date]
            }
        }
    }
}

/// Service for journal operations that follows dependency injection pattern
pub struct JournalService {
    config: Config,
    io: Box<dyn JournalIO>,
    editor: Box<dyn Editor>,
}

impl JournalService {
    /// Creates a new JournalService with the given dependencies
    pub fn new(config: Config, io: Box<dyn JournalIO>, editor: Box<dyn Editor>) -> Self {
        JournalService {
            config,
            io,
            editor,
        }
    }
    
    /// Gets the editor command from config
    pub fn get_editor_cmd(&self) -> &str {
        &self.config.editor
    }
    
    /// Gets the journal directory from config
    pub fn get_journal_dir(&self) -> &PathBuf {
        &self.config.journal_dir
    }
    
    /// Appends a date/time header to the specified journal file
    pub fn append_date_time(&self, path: &str) -> AppResult<()> {
        let mut file = self.io.create_or_open_file(path)?;
        let now = Local::now();
        
        let content = self.io.read_file_content(path)?;
        
        let entry = if content.is_empty() {
            format!(
                "# {}\n\n## {}\n\n",
                now.format("%B %d, %Y: %A"),
                now.format("%H:%M:%S")
            )
        } else {
            format!("\n\n## {}\n\n", now.format("%H:%M:%S"))
        };
        
        self.io.append_to_file(&mut file, &entry)?;
        Ok(())
    }

    /// Gets the path for today's journal entry
    pub fn get_todays_entry_path(&self) -> AppResult<String> {
        let now = Local::now();
        self.io.generate_path_for_date(now)
    }

    /// Gets paths to entries from the past week (excluding today)
    pub fn get_retro_entries(&self) -> AppResult<Vec<String>> {
        let mut paths = Vec::new();
        
        for i in (1..=7).rev() {
            let date = Local::now() - Duration::days(i);
            let path = self.io.generate_path_for_date(date)?;
            
            if self.io.file_exists(&path) {
                paths.push(path);
            }
        }
        
        Ok(paths)
    }

    /// Gets paths to entries from significant past dates (1 month ago, 3 months ago, yearly)
    pub fn get_reminisce_entries(&self) -> AppResult<Vec<String>> {
        let mut paths = Vec::new();
        let now = Local::now();
        let today = now.naive_local().date();

        let mut dates = Vec::new();

        // Add specific month intervals
        if let Some(date) = today.checked_sub_months(Months::new(1)) {
            dates.push(date);
        }
        if let Some(date) = today.checked_sub_months(Months::new(3)) {
            dates.push(date);
        }
        if let Some(date) = today.checked_sub_months(Months::new(6)) {
            dates.push(date);
        }

        // Add every year ago for the past hundred years
        for year in 1..=100 {
            if let Some(date) = today.checked_sub_months(Months::new(12 * year)) {
                dates.push(date);
            }
        }

        // Remove duplicates and sort the dates
        dates.sort();
        dates.dedup();

        // Collect paths for existing entries
        for date in dates {
            let path = self.io.generate_path_for_naive_date(date)?;
            if self.io.file_exists(&path) {
                paths.push(path);
            }
        }

        paths.reverse();
        Ok(paths)
    }

    /// Opens journal entries based on the provided date specifier
    pub fn open_entries(&self, date_spec: &DateSpecifier) -> AppResult<()> {
        match date_spec {
            DateSpecifier::Today => {
                let path = self.get_todays_entry_path()?;
                self.append_date_time(&path)?;
                self.editor.open_files(&[path])
            },
            DateSpecifier::Retro => {
                let paths = self.get_retro_entries()?;
                if paths.is_empty() {
                    println!("No entries found for the past week");
                    return Ok(());
                }
                self.editor.open_files(&paths)
            },
            DateSpecifier::Reminisce => {
                let paths = self.get_reminisce_entries()?;
                if paths.is_empty() {
                    println!("No entries found for reminisce intervals");
                    return Ok(());
                }
                self.editor.open_files(&paths)
            },
            DateSpecifier::Specific(date) => {
                let path = self.io.generate_path_for_naive_date(*date)?;
                if !self.io.file_exists(&path) {
                    // Create a new file for the specific date
                    self.append_date_time(&path)?;
                }
                self.editor.open_files(&[path])
            }
        }
    }
    
    /// Opens today's journal entry, creating it if it doesn't exist
    pub fn open_entry(&self) -> AppResult<()> {
        self.open_entries(&DateSpecifier::Today)
    }
    
    /// Opens entries from the past week (excluding today)
    pub fn open_retro_entry(&self) -> AppResult<()> {
        self.open_entries(&DateSpecifier::Retro)
    }
    
    /// Opens entries from significant past dates (1 month ago, 3 months ago, yearly anniversaries)
    pub fn open_reminisce_entry(&self) -> AppResult<()> {
        self.open_entries(&DateSpecifier::Reminisce)
    }
    
    /// Opens a journal entry for a specific date
    pub fn open_specific_entry(&self, date: NaiveDate) -> AppResult<()> {
        self.open_entries(&DateSpecifier::Specific(date))
    }
}

// Keep the old Journal struct for backward compatibility during refactoring
pub struct Journal<T: JournalIO> {
    io: T,
}

impl<T: JournalIO> Journal<T> {
    pub fn new(io: T) -> Self {
        Journal { io }
    }

    pub fn append_date_time(&self, path: &str) -> AppResult<()> {
        let mut file = self.io.create_or_open_file(path)?;
        let now = Local::now();
        
        let content = self.io.read_file_content(path)?;
        
        let entry = if content.is_empty() {
            format!(
                "# {}\n\n## {}\n\n",
                now.format("%B %d, %Y: %A"),
                now.format("%H:%M:%S")
            )
        } else {
            format!("\n\n## {}\n\n", now.format("%H:%M:%S"))
        };
        
        self.io.append_to_file(&mut file, &entry)?;
        Ok(())
    }

    pub fn get_todays_entry_path(&self) -> AppResult<String> {
        let now = Local::now();
        self.io.generate_path_for_date(now)
    }

    pub fn get_retro_entries(&self) -> AppResult<Vec<String>> {
        let mut paths = Vec::new();
        
        for i in (1..=7).rev() {
            let date = Local::now() - Duration::days(i);
            let path = self.io.generate_path_for_date(date)?;
            
            if self.io.file_exists(&path) {
                paths.push(path);
            }
        }
        
        Ok(paths)
    }

    pub fn get_reminisce_entries(&self) -> AppResult<Vec<String>> {
        let mut paths = Vec::new();
        let now = Local::now();
        let today = now.naive_local().date();

        let mut dates = Vec::new();

        // Add specific month intervals
        if let Some(date) = today.checked_sub_months(Months::new(1)) {
            dates.push(date);
        }
        if let Some(date) = today.checked_sub_months(Months::new(3)) {
            dates.push(date);
        }
        if let Some(date) = today.checked_sub_months(Months::new(6)) {
            dates.push(date);
        }

        // Add every year ago for the past hundred years
        for year in 1..=100 {
            if let Some(date) = today.checked_sub_months(Months::new(12 * year)) {
                dates.push(date);
            }
        }

        // Remove duplicates and sort the dates
        dates.sort();
        dates.dedup();

        // Collect paths for existing entries
        for date in dates {
            let path = self.io.generate_path_for_naive_date(date)?;
            if self.io.file_exists(&path) {
                paths.push(path);
            }
        }

        paths.reverse();
        Ok(paths)
    }
}