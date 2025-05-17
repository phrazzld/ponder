#[cfg(test)]
mod journal_tests {
    // These tests now use the DateSpecifier from journal_logic module
    use crate::journal_logic::DateSpecifier;
    use chrono::{Duration, Local, NaiveDate};

    #[test]
    fn test_date_specifier_from_args() {
        // This test is retained for compatibility during migration
        // to ensure DateSpecifier in journal_logic works the same way
        let result = DateSpecifier::from_args(false, false, None);
        assert!(result.is_ok());
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
}
