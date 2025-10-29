//! High-level operations for journaling with encryption and AI.
//!
//! This module provides user-facing operations that orchestrate the core
//! functionality of Ponder v2.0: editing encrypted journal entries, querying
//! with RAG, generating reflections, and semantic search.

pub mod ask;
pub mod backup;
pub mod detection;
pub mod edit;
pub mod migration;
pub mod patterns;
pub mod reflect;
pub mod reindex;
pub mod search;
pub mod summarize;

// Re-export commonly used functions
pub use ask::ask_question;
pub use backup::{create_backup, restore_backup, verify_backup, RestoreReport};
pub use detection::{
    detect_migration_state, is_migrated, scan_v1_entries, MigrationDetectionResult,
};
pub use edit::edit_entry;
pub use migration::{migrate_all_entries, migrate_entry, verify_migration, MigrationResult};
pub use patterns::{
    detect_temporal_patterns, detect_topic_patterns, TemporalPatterns, TopicCluster, TopicPatterns,
};
pub use reflect::reflect_on_entry;
pub use reindex::{reindex_entries, ReindexReport};
pub use search::search_entries;
pub use summarize::{generate_daily_summary, generate_monthly_summary, generate_weekly_summary};
