#[cfg(test)]
mod tests {
    use crate::journal::DateSpecifier;
    use crate::errors::AppError;
    use chrono::{Duration, Local, NaiveDate};
    
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
}