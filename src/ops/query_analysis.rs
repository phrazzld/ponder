//! Query analysis for temporal-aware context retrieval.
//!
//! This module provides query understanding capabilities to extract semantic topics
//! and temporal constraints before performing vector search. This enables filtering
//! journal entries by date ranges when users ask time-sensitive questions.

use crate::ai::{Message, OllamaClient};
use crate::errors::{AIError, AppResult};
use chrono::{Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use tracing::debug;

/// Structured analysis of a user's query, extracting topic and temporal constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryAnalysis {
    /// Core topic or subject of the query (e.g., "work", "relationships", "health")
    pub topic: String,

    /// Temporal constraint extracted from the query (if any)
    pub temporal_constraint: TemporalConstraint,

    /// Confidence in the query analysis (0.0-1.0)
    #[serde(default = "default_confidence")]
    pub confidence: f32,
}

fn default_confidence() -> f32 {
    0.8
}

/// Temporal constraint extracted from user query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TemporalConstraint {
    /// No temporal constraint in query (search all dates)
    None,

    /// Absolute date range (e.g., "between Jan 1 and Jan 31")
    Absolute {
        start_date: String, // YYYY-MM-DD format
        end_date: String,   // YYYY-MM-DD format
    },

    /// Relative to current date (e.g., "past 2 weeks", "last month")
    Relative { days_ago: i64 },
}

impl Default for TemporalConstraint {
    fn default() -> Self {
        TemporalConstraint::None
    }
}

impl TemporalConstraint {
    /// Converts temporal constraint to an absolute date range.
    ///
    /// # Arguments
    ///
    /// * `reference_date` - The reference date (typically today) for relative constraints
    ///
    /// # Returns
    ///
    /// Optional tuple of (start_date, end_date). Returns None if constraint is None.
    pub fn to_date_range(&self, reference_date: NaiveDate) -> Option<(NaiveDate, NaiveDate)> {
        match self {
            TemporalConstraint::None => None,
            TemporalConstraint::Absolute {
                start_date,
                end_date,
            } => {
                // Parse YYYY-MM-DD format
                let start = NaiveDate::parse_from_str(start_date, "%Y-%m-%d").ok()?;
                let end = NaiveDate::parse_from_str(end_date, "%Y-%m-%d").ok()?;
                Some((start, end))
            }
            TemporalConstraint::Relative { days_ago } => {
                // Calculate date range from reference date
                let start = reference_date - Duration::days(*days_ago);
                let end = reference_date;
                Some((start, end))
            }
        }
    }
}

/// System prompt for query analysis with structured JSON output.
const QUERY_ANALYSIS_PROMPT: &str = r#"You are a query analysis assistant. Analyze the user's journal query and extract:
1. The core topic/subject (one or two words)
2. Any temporal constraint (time-related filter)

Respond with ONLY valid JSON matching this schema:
{
  "topic": "string (core subject, e.g., 'work', 'relationships', 'travel')",
  "temporal_constraint": {
    "type": "none" | "absolute" | "relative",
    // If type is "absolute":
    "start_date": "YYYY-MM-DD",
    "end_date": "YYYY-MM-DD",
    // If type is "relative":
    "days_ago": <number of days>
  },
  "confidence": 0.0-1.0
}

Temporal parsing examples:
- "past couple weeks" → {"type": "relative", "days_ago": 14}
- "last week" → {"type": "relative", "days_ago": 7}
- "past month" → {"type": "relative", "days_ago": 30}
- "last 3 months" → {"type": "relative", "days_ago": 90}
- "this year" → {"type": "relative", "days_ago": <days since Jan 1>}
- "yesterday" → {"type": "relative", "days_ago": 1}
- "recently" → {"type": "relative", "days_ago": 14}
- "lately" → {"type": "relative", "days_ago": 30}
- no time mention → {"type": "none"}

Only include fields relevant to the constraint type. Return ONLY the JSON object, no other text."#;

/// Analyzes a user query to extract topic and temporal constraints.
///
/// Uses LLM with structured JSON output to decompose the query into:
/// - Core topic/subject
/// - Temporal window (if mentioned)
///
/// This enables temporal-aware context retrieval before RAG generation.
///
/// # Arguments
///
/// * `ai_client` - Ollama client for LLM inference
/// * `query` - User's natural language query
///
/// # Returns
///
/// Structured query analysis with topic and temporal constraint
///
/// # Errors
///
/// Returns error if:
/// - Ollama is unreachable
/// - LLM returns invalid JSON
/// - JSON doesn't match expected schema
///
/// # Example
///
/// ```no_run
/// # use ponder::ai::OllamaClient;
/// # use ponder::ops::query_analysis::analyze_query;
/// let client = OllamaClient::new("http://127.0.0.1:11434");
/// let analysis = analyze_query(&client, "what have I been doing the past 2 weeks?").unwrap();
/// println!("Topic: {}", analysis.topic);
/// ```
pub fn analyze_query(ai_client: &OllamaClient, query: &str) -> AppResult<QueryAnalysis> {
    debug!("Analyzing query for temporal constraints: {}", query);

    let messages = vec![Message::system(QUERY_ANALYSIS_PROMPT), Message::user(query)];

    // Call LLM with JSON format (Ollama supports format parameter)
    let response = ai_client.chat_with_json_format(&messages)?;

    // Parse JSON response into QueryAnalysis
    let analysis: QueryAnalysis = serde_json::from_str(&response).map_err(|e| {
        AIError::InvalidResponse(format!(
            "Failed to parse query analysis JSON: {}. Response: {}",
            e, response
        ))
    })?;

    debug!(
        "Query analysis complete - Topic: '{}', Constraint: {:?}",
        analysis.topic, analysis.temporal_constraint
    );

    Ok(analysis)
}

/// Helper function to get today's date in the local timezone.
pub fn today() -> NaiveDate {
    Utc::now().date_naive()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_temporal_constraint_none() {
        let constraint = TemporalConstraint::None;
        let result = constraint.to_date_range(NaiveDate::from_ymd_opt(2025, 10, 29).unwrap());
        assert!(result.is_none());
    }

    #[test]
    fn test_temporal_constraint_relative() {
        let constraint = TemporalConstraint::Relative { days_ago: 14 };
        let reference = NaiveDate::from_ymd_opt(2025, 10, 29).unwrap();
        let result = constraint.to_date_range(reference).unwrap();

        assert_eq!(result.0, NaiveDate::from_ymd_opt(2025, 10, 15).unwrap());
        assert_eq!(result.1, NaiveDate::from_ymd_opt(2025, 10, 29).unwrap());
    }

    #[test]
    fn test_temporal_constraint_absolute() {
        let constraint = TemporalConstraint::Absolute {
            start_date: "2025-01-01".to_string(),
            end_date: "2025-01-31".to_string(),
        };
        let reference = NaiveDate::from_ymd_opt(2025, 10, 29).unwrap();
        let result = constraint.to_date_range(reference).unwrap();

        assert_eq!(result.0, NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
        assert_eq!(result.1, NaiveDate::from_ymd_opt(2025, 1, 31).unwrap());
    }

    #[test]
    fn test_query_analysis_json_deserialization() {
        let json = r#"{
            "topic": "work",
            "temporal_constraint": {
                "type": "relative",
                "days_ago": 7
            },
            "confidence": 0.95
        }"#;

        let analysis: QueryAnalysis = serde_json::from_str(json).unwrap();
        assert_eq!(analysis.topic, "work");
        assert_eq!(analysis.confidence, 0.95);

        match analysis.temporal_constraint {
            TemporalConstraint::Relative { days_ago } => assert_eq!(days_ago, 7),
            _ => panic!("Expected Relative constraint"),
        }
    }

    #[test]
    fn test_query_analysis_json_none_constraint() {
        let json = r#"{
            "topic": "relationships",
            "temporal_constraint": {
                "type": "none"
            }
        }"#;

        let analysis: QueryAnalysis = serde_json::from_str(json).unwrap();
        assert_eq!(analysis.topic, "relationships");
        assert_eq!(analysis.confidence, 0.8); // Default value

        match analysis.temporal_constraint {
            TemporalConstraint::None => {}
            _ => panic!("Expected None constraint"),
        }
    }

    #[test]
    fn test_today_returns_valid_date() {
        let date = today();
        // Just verify it returns a valid date without panicking
        assert!(date.year() >= 2025);
    }
}
