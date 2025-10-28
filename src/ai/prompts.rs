//! System prompts and message builders for AI interactions.
//!
//! This module provides pre-defined prompts and utilities for constructing
//! messages for different AI-powered journal operations.

use super::ollama::Message;

/// System prompt for journal-related AI interactions.
///
/// This prompt establishes the AI's role as a thoughtful journal assistant
/// that helps with reflection, insights, and knowledge retrieval.
pub const SYSTEM_PROMPT: &str = r#"You are a thoughtful and insightful journal assistant. Your role is to help users:

1. Reflect on their journal entries with depth and nuance
2. Find connections and patterns across different entries
3. Retrieve relevant information from their personal knowledge base
4. Provide gentle guidance and thought-provoking questions

Guidelines:
- Be warm, empathetic, and non-judgmental
- Respect the personal and private nature of journal entries
- Focus on helping users understand themselves better
- Cite specific entries when referencing past content
- Ask clarifying questions when needed
- Avoid generic advice - tailor responses to the user's actual experiences

Remember: You're working with someone's personal thoughts and experiences.
Treat them with care and respect."#;

/// Builds messages for reflecting on a journal entry.
///
/// Creates a conversation that asks the AI to provide thoughtful reflection
/// on a specific journal entry.
///
/// # Arguments
///
/// * `entry_content` - The full content of the journal entry
///
/// # Returns
///
/// A vector of messages suitable for chat completion.
pub fn reflect_prompt(entry_content: &str) -> Vec<Message> {
    vec![
        Message::system(SYSTEM_PROMPT),
        Message::user(format!(
            r#"Please reflect on this journal entry. Provide thoughtful insights,
identify key themes, and suggest questions for deeper exploration.

Entry:
---
{}
---

Focus on:
1. Main themes and emotions expressed
2. Patterns or connections to broader life contexts
3. Questions for deeper reflection
4. Positive developments or growth areas

Keep your reflection concise (2-3 paragraphs) and actionable."#,
            entry_content
        )),
    ]
}

/// Builds messages for answering a question using journal context.
///
/// Creates a conversation that asks the AI to answer a question based on
/// retrieved journal entries.
///
/// # Arguments
///
/// * `question` - The user's question
/// * `context_chunks` - Relevant journal entry excerpts
///
/// # Returns
///
/// A vector of messages suitable for chat completion.
pub fn ask_prompt(question: &str, context_chunks: &[String]) -> Vec<Message> {
    let context = if context_chunks.is_empty() {
        "No relevant journal entries found.".to_string()
    } else {
        context_chunks
            .iter()
            .enumerate()
            .map(|(i, chunk)| format!("[Context {}]\n{}\n", i + 1, chunk))
            .collect::<Vec<_>>()
            .join("\n")
    };

    vec![
        Message::system(SYSTEM_PROMPT),
        Message::user(format!(
            r#"Based on the following journal entries, please answer this question:

Question: {}

Relevant journal entries:
---
{}
---

Instructions:
1. Answer based primarily on the provided context
2. Cite specific entries when making references
3. If the context doesn't contain enough information, say so
4. Be specific and concrete in your response
5. If you see patterns across entries, point them out

Provide a clear, helpful answer."#,
            question, context
        )),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reflect_prompt_structure() {
        let entry = "Today was a good day. I learned something new.";
        let messages = reflect_prompt(entry);

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "system");
        assert_eq!(messages[0].content, SYSTEM_PROMPT);
        assert_eq!(messages[1].role, "user");
        assert!(messages[1].content.contains(entry));
        assert!(messages[1].content.contains("reflect"));
    }

    #[test]
    fn test_ask_prompt_with_context() {
        let question = "What did I learn last week?";
        let context = vec![
            "Monday: Learned about Rust".to_string(),
            "Tuesday: Practiced async programming".to_string(),
        ];
        let messages = ask_prompt(question, &context);

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "system");
        assert_eq!(messages[1].role, "user");
        assert!(messages[1].content.contains(question));
        assert!(messages[1].content.contains("Rust"));
        assert!(messages[1].content.contains("async programming"));
    }

    #[test]
    fn test_ask_prompt_without_context() {
        let question = "What did I do today?";
        let messages = ask_prompt(question, &[]);

        assert_eq!(messages.len(), 2);
        assert!(messages[1].content.contains(question));
        assert!(messages[1]
            .content
            .contains("No relevant journal entries found"));
    }

    #[test]
    fn test_system_prompt_not_empty() {
        // Clippy wants !is_empty() but that doesn't work on consts,
        // so we just check that it contains expected content
        assert!(SYSTEM_PROMPT.contains("journal"));
        assert!(SYSTEM_PROMPT.contains("Reflect") || SYSTEM_PROMPT.contains("reflect"));
    }

    #[test]
    fn test_reflect_prompt_includes_guidance() {
        let messages = reflect_prompt("Test entry");
        assert!(messages[1].content.contains("themes"));
        assert!(messages[1].content.contains("reflect"));
    }

    #[test]
    fn test_ask_prompt_includes_instructions() {
        let messages = ask_prompt("test question", &["context".to_string()]);
        assert!(messages[1].content.contains("based") || messages[1].content.contains("Based"));
        assert!(messages[1].content.contains("cite") || messages[1].content.contains("Cite"));
    }
}
