//! AI operations for journal insights and semantic search.
//!
//! This module provides integration with Ollama for local LLM inference,
//! including embedding generation for semantic search and chat completion
//! for journal insights.
//!
//! # Module Structure
//!
//! - `ollama`: HTTP client for Ollama API
//! - `chunking`: Text chunking for embedding generation
//! - `prompts`: System prompts and message builders
//!
//! # Example
//!
//! ```no_run
//! use ponder::ai::OllamaClient;
//!
//! let client = OllamaClient::new("http://127.0.0.1:11434");
//! let embedding = client.embed("nomic-embed-text", "Hello world")?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod chunking;
pub mod ollama;
pub mod prompts;

// Re-export commonly used types
pub use ollama::{Message, OllamaClient};
pub use prompts::{ask_prompt, reflect_prompt, SYSTEM_PROMPT};
