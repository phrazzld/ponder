//! High-level operations for journaling with encryption and AI.
//!
//! This module provides user-facing operations that orchestrate the core
//! functionality of Ponder v2.0: editing encrypted journal entries, querying
//! with RAG, generating reflections, and semantic search.

pub mod ask;
pub mod backup;
pub mod edit;
pub mod reflect;
pub mod search;

// Re-export commonly used functions
pub use ask::ask_question;
pub use backup::{create_backup, restore_backup, verify_backup, RestoreReport};
pub use edit::edit_entry;
pub use reflect::reflect_on_entry;
pub use search::search_entries;
