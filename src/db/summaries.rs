//! Summary CRUD operations.
//!
//! This module provides functions for creating, reading, and querying
//! AI-generated summaries (daily, weekly, monthly) in the database.

use crate::errors::{AppResult, DatabaseError};
use rusqlite::{params, Connection, OptionalExtension};
use std::str::FromStr;
use tracing::debug;

/// Summary granularity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SummaryLevel {
    Daily,
    Weekly,
    Monthly,
}

impl SummaryLevel {
    /// Convert to database string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            SummaryLevel::Daily => "daily",
            SummaryLevel::Weekly => "weekly",
            SummaryLevel::Monthly => "monthly",
        }
    }

    /// Convert to display-friendly emoji icon.
    pub fn icon(&self) -> &'static str {
        match self {
            SummaryLevel::Daily => "📄",
            SummaryLevel::Weekly => "📆",
            SummaryLevel::Monthly => "📅",
        }
    }

    /// Convert to display-friendly label (plural).
    pub fn label_plural(&self) -> &'static str {
        match self {
            SummaryLevel::Daily => "Daily Summaries",
            SummaryLevel::Weekly => "Weekly Summaries",
            SummaryLevel::Monthly => "Monthly Summaries",
        }
    }
}

impl FromStr for SummaryLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "daily" => Ok(SummaryLevel::Daily),
            "weekly" => Ok(SummaryLevel::Weekly),
            "monthly" => Ok(SummaryLevel::Monthly),
            _ => Err(format!("Invalid summary level: {}", s)),
        }
    }
}

/// Represents a summary in the database.
#[derive(Debug, Clone)]
pub struct Summary {
    pub id: i64,
    pub date: String,
    pub level: SummaryLevel,
    pub summary_encrypted: Vec<u8>,
    pub topics: Option<String>,
    pub sentiment: Option<f64>,
    pub word_count: Option<i64>,
    pub created_at: String,
}

/// Inserts or updates a summary.
///
/// If a summary with the same date and level already exists, it will be updated.
/// Returns the summary ID.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `date` - Date string (YYYY-MM-DD format)
/// * `level` - Summary granularity level
/// * `summary_encrypted` - Encrypted summary content
/// * `topics` - Optional JSON array of topics
/// * `sentiment` - Optional sentiment score (-1.0 to 1.0)
/// * `word_count` - Optional word count
///
/// # Errors
///
/// Returns an error if the database operation fails.
pub fn upsert_summary(
    conn: &Connection,
    date: &str,
    level: SummaryLevel,
    summary_encrypted: &[u8],
    topics: Option<&str>,
    sentiment: Option<f64>,
    word_count: Option<i64>,
) -> AppResult<i64> {
    debug!("Upserting {} summary for date {}", level.as_str(), date);

    conn.execute(
        r#"
        INSERT INTO summaries (date, level, summary_encrypted, topics, sentiment, word_count)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        ON CONFLICT(date, level) DO UPDATE SET
            summary_encrypted = excluded.summary_encrypted,
            topics = excluded.topics,
            sentiment = excluded.sentiment,
            word_count = excluded.word_count,
            created_at = CURRENT_TIMESTAMP
        "#,
        params![
            date,
            level.as_str(),
            summary_encrypted,
            topics,
            sentiment,
            word_count
        ],
    )
    .map_err(DatabaseError::Sqlite)?;

    // Get the summary ID
    let summary_id: i64 = conn
        .query_row(
            "SELECT id FROM summaries WHERE date = ?1 AND level = ?2",
            params![date, level.as_str()],
            |row| row.get(0),
        )
        .map_err(DatabaseError::Sqlite)?;

    debug!("Summary upserted with id {}", summary_id);
    Ok(summary_id)
}

/// Retrieves a summary by date and level.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `date` - Date string (YYYY-MM-DD format)
/// * `level` - Summary granularity level
///
/// # Errors
///
/// Returns an error if the database operation fails.
/// Returns `Ok(None)` if no summary exists for the given date and level.
pub fn get_summary(
    conn: &Connection,
    date: &str,
    level: SummaryLevel,
) -> AppResult<Option<Summary>> {
    debug!("Getting {} summary for date {}", level.as_str(), date);

    let result = conn
        .query_row(
            r#"
            SELECT id, date, level, summary_encrypted, topics, sentiment, word_count, created_at
            FROM summaries
            WHERE date = ?1 AND level = ?2
            "#,
            params![date, level.as_str()],
            |row| {
                let level_str: String = row.get(2)?;
                Ok(Summary {
                    id: row.get(0)?,
                    date: row.get(1)?,
                    level: level_str.parse().unwrap(),
                    summary_encrypted: row.get(3)?,
                    topics: row.get(4)?,
                    sentiment: row.get(5)?,
                    word_count: row.get(6)?,
                    created_at: row.get(7)?,
                })
            },
        )
        .optional()
        .map_err(DatabaseError::Sqlite)?;

    Ok(result)
}

/// Lists summaries for a specific level, ordered by date descending.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `level` - Summary granularity level to filter by
/// * `limit` - Maximum number of summaries to return
///
/// # Errors
///
/// Returns an error if the database operation fails.
pub fn list_summaries(
    conn: &Connection,
    level: SummaryLevel,
    limit: usize,
) -> AppResult<Vec<Summary>> {
    debug!("Listing {} summaries (limit: {})", level.as_str(), limit);

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, date, level, summary_encrypted, topics, sentiment, word_count, created_at
            FROM summaries
            WHERE level = ?1
            ORDER BY date DESC
            LIMIT ?2
            "#,
        )
        .map_err(DatabaseError::Sqlite)?;

    let summaries = stmt
        .query_map(params![level.as_str(), limit as i64], |row| {
            let level_str: String = row.get(2)?;
            Ok(Summary {
                id: row.get(0)?,
                date: row.get(1)?,
                level: level_str.parse().unwrap(),
                summary_encrypted: row.get(3)?,
                topics: row.get(4)?,
                sentiment: row.get(5)?,
                word_count: row.get(6)?,
                created_at: row.get(7)?,
            })
        })
        .map_err(DatabaseError::Sqlite)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(DatabaseError::Sqlite)?;

    debug!("Found {} summaries", summaries.len());
    Ok(summaries)
}

/// Lists all summaries ordered by date descending, regardless of level.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `limit` - Maximum number of summaries to return
///
/// # Errors
///
/// Returns an error if the database operation fails.
pub fn list_all_summaries(conn: &Connection, limit: usize) -> AppResult<Vec<Summary>> {
    debug!("Listing all summaries (limit: {})", limit);

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, date, level, summary_encrypted, topics, sentiment, word_count, created_at
            FROM summaries
            ORDER BY date DESC, level
            LIMIT ?1
            "#,
        )
        .map_err(DatabaseError::Sqlite)?;

    let summaries = stmt
        .query_map(params![limit as i64], |row| {
            let level_str: String = row.get(2)?;
            Ok(Summary {
                id: row.get(0)?,
                date: row.get(1)?,
                level: level_str.parse().unwrap(),
                summary_encrypted: row.get(3)?,
                topics: row.get(4)?,
                sentiment: row.get(5)?,
                word_count: row.get(6)?,
                created_at: row.get(7)?,
            })
        })
        .map_err(DatabaseError::Sqlite)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(DatabaseError::Sqlite)?;

    debug!("Found {} summaries", summaries.len());
    Ok(summaries)
}

/// Lists summaries within a date range for a specific level.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `level` - Summary granularity level to filter by
/// * `start_date` - Start date (inclusive, YYYY-MM-DD format)
/// * `end_date` - End date (inclusive, YYYY-MM-DD format)
///
/// # Errors
///
/// Returns an error if the database operation fails.
pub fn list_summaries_by_date_range(
    conn: &Connection,
    level: SummaryLevel,
    start_date: &str,
    end_date: &str,
) -> AppResult<Vec<Summary>> {
    debug!(
        "Listing {} summaries from {} to {}",
        level.as_str(),
        start_date,
        end_date
    );

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, date, level, summary_encrypted, topics, sentiment, word_count, created_at
            FROM summaries
            WHERE level = ?1 AND date >= ?2 AND date <= ?3
            ORDER BY date ASC
            "#,
        )
        .map_err(DatabaseError::Sqlite)?;

    let summaries = stmt
        .query_map(params![level.as_str(), start_date, end_date], |row| {
            let level_str: String = row.get(2)?;
            Ok(Summary {
                id: row.get(0)?,
                date: row.get(1)?,
                level: level_str.parse().unwrap(),
                summary_encrypted: row.get(3)?,
                topics: row.get(4)?,
                sentiment: row.get(5)?,
                word_count: row.get(6)?,
                created_at: row.get(7)?,
            })
        })
        .map_err(DatabaseError::Sqlite)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(DatabaseError::Sqlite)?;

    debug!("Found {} summaries in range", summaries.len());
    Ok(summaries)
}

/// Gets recent summaries with hierarchical priority.
///
/// Ensures monthly summaries are always included by querying each level separately.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `monthly_limit` - Max monthly summaries to return
/// * `weekly_limit` - Max weekly summaries to return
/// * `daily_limit` - Max daily summaries to return
///
/// # Returns
///
/// Combined list ordered: Monthly → Weekly → Daily, each internally ordered by date DESC
///
/// # Errors
///
/// Returns an error if database operations fail.
pub fn get_recent_summaries_hierarchical(
    conn: &Connection,
    monthly_limit: usize,
    weekly_limit: usize,
    daily_limit: usize,
) -> AppResult<Vec<Summary>> {
    let mut results = Vec::new();

    // Get monthly summaries
    let monthly = list_summaries(conn, SummaryLevel::Monthly, monthly_limit)?;
    results.extend(monthly);

    // Get weekly summaries
    let weekly = list_summaries(conn, SummaryLevel::Weekly, weekly_limit)?;
    results.extend(weekly);

    // Get daily summaries
    let daily = list_summaries(conn, SummaryLevel::Daily, daily_limit)?;
    results.extend(daily);

    debug!(
        "Hierarchical query returned {} total summaries",
        results.len()
    );
    Ok(results)
}

/// Format a summary date for display based on its level.
///
/// # Arguments
///
/// * `date` - Date string in YYYY-MM-DD format
/// * `level` - Summary granularity level
///
/// # Returns
///
/// User-friendly formatted date string:
/// - Monthly: "October 2025" or "2025-10"
/// - Weekly: "Week ending Oct 31, 2025"
/// - Daily: "Oct 31, 2025"
pub fn format_summary_date(date: &str, level: SummaryLevel) -> String {
    use chrono::NaiveDate;

    let parsed_date = match NaiveDate::parse_from_str(date, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => return date.to_string(), // Fallback to original if parse fails
    };

    match level {
        SummaryLevel::Monthly => {
            // Format as "October 2025"
            parsed_date.format("%B %Y").to_string()
        }
        SummaryLevel::Weekly => {
            // Format as "Week ending Oct 31, 2025"
            format!("Week ending {}", parsed_date.format("%b %d, %Y"))
        }
        SummaryLevel::Daily => {
            // Format as "Oct 31, 2025"
            parsed_date.format("%b %d, %Y").to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::create_tables;
    use rusqlite::Connection;

    #[test]
    fn test_upsert_summary() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();

        let encrypted = b"encrypted_summary_content";
        let id = upsert_summary(
            &conn,
            "2024-01-15",
            SummaryLevel::Daily,
            encrypted,
            Some(r#"["work", "productivity"]"#),
            Some(0.75),
            Some(150),
        )
        .unwrap();

        assert!(id > 0);

        // Verify it was inserted
        let summary = get_summary(&conn, "2024-01-15", SummaryLevel::Daily)
            .unwrap()
            .unwrap();
        assert_eq!(summary.date, "2024-01-15");
        assert_eq!(summary.level, SummaryLevel::Daily);
        assert_eq!(summary.summary_encrypted, encrypted);
        assert_eq!(summary.topics.unwrap(), r#"["work", "productivity"]"#);
        assert_eq!(summary.sentiment.unwrap(), 0.75);
        assert_eq!(summary.word_count.unwrap(), 150);
    }

    #[test]
    fn test_upsert_summary_update() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();

        // Insert initial summary
        let encrypted1 = b"first_version";
        let id1 = upsert_summary(
            &conn,
            "2024-01-15",
            SummaryLevel::Daily,
            encrypted1,
            None,
            Some(0.5),
            Some(100),
        )
        .unwrap();

        // Update same date+level
        let encrypted2 = b"second_version";
        let id2 = upsert_summary(
            &conn,
            "2024-01-15",
            SummaryLevel::Daily,
            encrypted2,
            Some(r#"["updated"]"#),
            Some(0.8),
            Some(200),
        )
        .unwrap();

        // Should be same ID (update, not insert)
        assert_eq!(id1, id2);

        // Verify updated content
        let summary = get_summary(&conn, "2024-01-15", SummaryLevel::Daily)
            .unwrap()
            .unwrap();
        assert_eq!(summary.summary_encrypted, encrypted2);
        assert_eq!(summary.topics.unwrap(), r#"["updated"]"#);
        assert_eq!(summary.sentiment.unwrap(), 0.8);
        assert_eq!(summary.word_count.unwrap(), 200);
    }

    #[test]
    fn test_get_summary_not_found() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();

        let result = get_summary(&conn, "2024-01-15", SummaryLevel::Daily).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_list_summaries_by_level() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();

        // Insert multiple summaries
        upsert_summary(
            &conn,
            "2024-01-15",
            SummaryLevel::Daily,
            b"day1",
            None,
            None,
            None,
        )
        .unwrap();
        upsert_summary(
            &conn,
            "2024-01-16",
            SummaryLevel::Daily,
            b"day2",
            None,
            None,
            None,
        )
        .unwrap();
        upsert_summary(
            &conn,
            "2024-01-15",
            SummaryLevel::Weekly,
            b"week1",
            None,
            None,
            None,
        )
        .unwrap();

        // List daily summaries
        let dailies = list_summaries(&conn, SummaryLevel::Daily, 10).unwrap();
        assert_eq!(dailies.len(), 2);
        assert!(dailies.iter().all(|s| s.level == SummaryLevel::Daily));

        // List weekly summaries
        let weeklies = list_summaries(&conn, SummaryLevel::Weekly, 10).unwrap();
        assert_eq!(weeklies.len(), 1);
        assert_eq!(weeklies[0].level, SummaryLevel::Weekly);
    }

    #[test]
    fn test_list_summaries_limit() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();

        // Insert 5 summaries
        for i in 1..=5 {
            upsert_summary(
                &conn,
                &format!("2024-01-{:02}", i),
                SummaryLevel::Daily,
                b"content",
                None,
                None,
                None,
            )
            .unwrap();
        }

        // List with limit of 3
        let summaries = list_summaries(&conn, SummaryLevel::Daily, 3).unwrap();
        assert_eq!(summaries.len(), 3);

        // Should be ordered by date DESC (most recent first)
        assert_eq!(summaries[0].date, "2024-01-05");
        assert_eq!(summaries[1].date, "2024-01-04");
        assert_eq!(summaries[2].date, "2024-01-03");
    }

    #[test]
    fn test_list_all_summaries() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();

        // Insert summaries at different levels
        upsert_summary(
            &conn,
            "2024-01-15",
            SummaryLevel::Daily,
            b"day",
            None,
            None,
            None,
        )
        .unwrap();
        upsert_summary(
            &conn,
            "2024-01-15",
            SummaryLevel::Weekly,
            b"week",
            None,
            None,
            None,
        )
        .unwrap();
        upsert_summary(
            &conn,
            "2024-01-15",
            SummaryLevel::Monthly,
            b"month",
            None,
            None,
            None,
        )
        .unwrap();

        let all_summaries = list_all_summaries(&conn, 10).unwrap();
        assert_eq!(all_summaries.len(), 3);
    }

    #[test]
    fn test_summary_level_conversion() {
        assert_eq!(SummaryLevel::Daily.as_str(), "daily");
        assert_eq!(SummaryLevel::Weekly.as_str(), "weekly");
        assert_eq!(SummaryLevel::Monthly.as_str(), "monthly");

        assert_eq!("daily".parse::<SummaryLevel>(), Ok(SummaryLevel::Daily));
        assert_eq!("weekly".parse::<SummaryLevel>(), Ok(SummaryLevel::Weekly));
        assert_eq!("monthly".parse::<SummaryLevel>(), Ok(SummaryLevel::Monthly));
        assert!("invalid".parse::<SummaryLevel>().is_err());
    }
}
