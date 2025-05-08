use clap::{Parser, ArgGroup};
use chrono::NaiveDate;
use std::str::FromStr;

/// A simple journaling tool for daily reflections
#[derive(Parser, Debug)]
#[clap(name = "ponder", about = "A simple journaling tool for daily reflections")]
#[clap(author, version, long_about = None)]
#[clap(group(ArgGroup::new("entry_type").args(&["retro", "reminisce", "date"])))]
pub struct CliArgs {
    /// Opens entries from the past week excluding today
    #[clap(short = 'r', long, conflicts_with_all = &["reminisce", "date"])]
    pub retro: bool,
    
    /// Opens entries from significant past intervals (1 month ago, 3 months ago, yearly anniversaries)
    #[clap(short = 'm', long, conflicts_with_all = &["retro", "date"])]
    pub reminisce: bool,
    
    /// Opens an entry for a specific date (format: YYYY-MM-DD or YYYYMMDD)
    #[clap(short = 'd', long, conflicts_with_all = &["retro", "reminisce"])]
    pub date: Option<String>,
    
    /// Print verbose output
    #[clap(short = 'v', long)]
    pub verbose: bool,
}

impl CliArgs {
    /// Parse command-line arguments
    pub fn parse() -> Self {
        CliArgs::parse_from(std::env::args())
    }
    
    /// Get the date if specified, parsing it into a NaiveDate
    pub fn parse_date(&self) -> Option<Result<NaiveDate, chrono::ParseError>> {
        self.date.as_ref().map(|date_str| {
            // Try parsing in YYYY-MM-DD format first
            NaiveDate::from_str(date_str)
                .or_else(|_| {
                    // Try parsing in YYYYMMDD format if the first format failed
                    NaiveDate::parse_from_str(date_str, "%Y%m%d")
                })
        })
    }
}

/// Parse command-line arguments - for backward compatibility
pub fn parse_args() -> CliArgs {
    CliArgs::parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;
    
    #[test]
    fn test_default_args() {
        let args = CliArgs::parse_from(vec!["ponder"]);
        assert!(!args.retro);
        assert!(!args.reminisce);
        assert!(args.date.is_none());
        assert!(!args.verbose);
    }
    
    #[test]
    fn test_retro_flag() {
        let args = CliArgs::parse_from(vec!["ponder", "--retro"]);
        assert!(args.retro);
        assert!(!args.reminisce);
        assert!(args.date.is_none());
        
        // Test short form
        let args = CliArgs::parse_from(vec!["ponder", "-r"]);
        assert!(args.retro);
        assert!(!args.reminisce);
        assert!(args.date.is_none());
    }
    
    #[test]
    fn test_reminisce_flag() {
        let args = CliArgs::parse_from(vec!["ponder", "--reminisce"]);
        assert!(!args.retro);
        assert!(args.reminisce);
        assert!(args.date.is_none());
        
        // Test short form
        let args = CliArgs::parse_from(vec!["ponder", "-m"]);
        assert!(!args.retro);
        assert!(args.reminisce);
        assert!(args.date.is_none());
    }
    
    #[test]
    fn test_date_option() {
        let args = CliArgs::parse_from(vec!["ponder", "--date", "2023-01-15"]);
        assert!(!args.retro);
        assert!(!args.reminisce);
        assert_eq!(args.date, Some("2023-01-15".to_string()));
        
        // Test short form
        let args = CliArgs::parse_from(vec!["ponder", "-d", "20230115"]);
        assert!(!args.retro);
        assert!(!args.reminisce);
        assert_eq!(args.date, Some("20230115".to_string()));
    }
    
    #[test]
    fn test_verbose_flag() {
        let args = CliArgs::parse_from(vec!["ponder", "--verbose"]);
        assert!(args.verbose);
        
        // Test short form
        let args = CliArgs::parse_from(vec!["ponder", "-v"]);
        assert!(args.verbose);
        
        // Test with other flags
        let args = CliArgs::parse_from(vec!["ponder", "--retro", "--verbose"]);
        assert!(args.retro);
        assert!(args.verbose);
    }
    
    #[test]
    fn test_parse_date() {
        // Test ISO format
        let args = CliArgs {
            retro: false,
            reminisce: false,
            date: Some("2023-01-15".to_string()),
            verbose: false,
        };
        
        let parsed_date = args.parse_date().unwrap().unwrap();
        assert_eq!(parsed_date.year(), 2023);
        assert_eq!(parsed_date.month(), 1);
        assert_eq!(parsed_date.day(), 15);
        
        // Test compact format
        let args = CliArgs {
            retro: false,
            reminisce: false,
            date: Some("20230115".to_string()),
            verbose: false,
        };
        
        let parsed_date = args.parse_date().unwrap().unwrap();
        assert_eq!(parsed_date.year(), 2023);
        assert_eq!(parsed_date.month(), 1);
        assert_eq!(parsed_date.day(), 15);
        
        // Test None case
        let args = CliArgs {
            retro: false,
            reminisce: false,
            date: None,
            verbose: false,
        };
        
        assert!(args.parse_date().is_none());
        
        // Test invalid date
        let args = CliArgs {
            retro: false,
            reminisce: false,
            date: Some("invalid-date".to_string()),
            verbose: false,
        };
        
        assert!(args.parse_date().unwrap().is_err());
    }
}