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

/// System prompt for sentiment analysis.
///
/// This prompt instructs the AI to analyze emotional tone and return a numeric score.
pub const SENTIMENT_PROMPT: &str = r#"You are a sentiment analysis assistant. Your role is to analyze text and determine its emotional tone.

Respond with ONLY a single number between -1.0 and 1.0:
- -1.0 = Very negative (despair, anger, severe distress)
- -0.5 = Moderately negative (frustration, sadness, worry)
- 0.0 = Neutral (factual, balanced, neither positive nor negative)
- 0.5 = Moderately positive (contentment, hope, mild joy)
- 1.0 = Very positive (elation, gratitude, strong happiness)

Be nuanced in your assessment. Most real entries fall between -0.7 and 0.7.
Respond with just the number, nothing else."#;

/// System prompt for topic extraction.
///
/// This prompt instructs the AI to extract key topics/themes and return them as JSON.
pub const TOPIC_EXTRACTION_PROMPT: &str = r#"You are a topic extraction assistant. Your role is to analyze text and identify the main topics or themes.

Extract 3-5 key topics from the text. Topics should be:
- Concrete and specific (e.g., "career planning", "family dinner", "anxiety about deadlines")
- Single phrases or short descriptions (2-5 words)
- Representative of the main themes in the text

Respond with ONLY a JSON array of strings. Example format:
["topic one", "topic two", "topic three"]

Do not include any other text, explanation, or formatting - just the JSON array."#;

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

/// Builds messages for sentiment analysis of text.
///
/// Creates a conversation that asks the AI to analyze the emotional tone
/// of the provided text and return a sentiment score.
///
/// # Arguments
///
/// * `text` - The text to analyze for sentiment
///
/// # Returns
///
/// A vector of messages suitable for chat completion that will return
/// a sentiment score between -1.0 and 1.0.
pub fn sentiment_prompt(text: &str) -> Vec<Message> {
    vec![
        Message::system(SENTIMENT_PROMPT),
        Message::user(format!(
            r#"Analyze the sentiment of this text:

---
{}
---

Respond with only a number between -1.0 and 1.0."#,
            text
        )),
    ]
}

/// Builds messages for topic extraction from text.
///
/// Creates a conversation that asks the AI to identify and extract the main
/// topics or themes from the provided text as a JSON array.
///
/// # Arguments
///
/// * `text` - The text to analyze for topics
///
/// # Returns
///
/// A vector of messages suitable for chat completion that will return
/// a JSON array of topic strings.
pub fn topic_extraction_prompt(text: &str) -> Vec<Message> {
    vec![
        Message::system(TOPIC_EXTRACTION_PROMPT),
        Message::user(format!(
            r#"Extract the main topics from this text:

---
{}
---

Respond with only a JSON array of 3-5 topic strings."#,
            text
        )),
    ]
}

/// System prompt for daily summaries.
///
/// This prompt instructs the AI to create a concise daily summary.
pub const SUMMARY_DAILY_PROMPT: &str = r#"You are a journal summarization assistant. Your role is to create concise, meaningful summaries of journal entries.

For daily summaries:
- Capture the main events, thoughts, and emotions
- Highlight key moments or decisions
- Note any patterns or themes
- Keep it brief but meaningful (2-3 sentences)
- Maintain the author's voice and perspective

Be objective yet empathetic. Focus on what matters most to the author."#;

/// System prompt for weekly summaries.
///
/// This prompt instructs the AI to create a weekly summary from daily summaries.
pub const SUMMARY_WEEKLY_PROMPT: &str = r#"You are a journal summarization assistant. Your role is to create weekly summaries from daily summaries.

For weekly summaries:
- Synthesize patterns and themes across the week
- Highlight major events, accomplishments, or challenges
- Note emotional trajectory or significant shifts
- Identify growth areas or recurring topics
- Keep it focused (1 paragraph)

Connect the dots between days. Show the bigger picture."#;

/// System prompt for monthly summaries.
///
/// This prompt instructs the AI to create a monthly summary from weekly summaries.
pub const SUMMARY_MONTHLY_PROMPT: &str = r#"You are a journal summarization assistant. Your role is to create monthly summaries from weekly summaries.

For monthly summaries:
- Synthesize the month's major themes and patterns
- Highlight significant accomplishments, challenges, or changes
- Note overall emotional tone and trajectory
- Identify key insights or personal growth
- Keep it comprehensive yet concise (2 paragraphs)

Provide perspective on the month as a whole. What defined this period?"#;

/// Builds messages for daily summary generation.
///
/// # Arguments
///
/// * `entry_content` - The full content of the daily journal entry
///
/// # Returns
///
/// A vector of messages suitable for chat completion.
pub fn daily_summary_prompt(entry_content: &str) -> Vec<Message> {
    vec![
        Message::system(SUMMARY_DAILY_PROMPT),
        Message::user(format!(
            r#"Please create a concise daily summary of this journal entry:

---
{}
---

Provide a 2-3 sentence summary capturing the main themes, events, and emotions."#,
            entry_content
        )),
    ]
}

/// Builds messages for weekly summary generation.
///
/// # Arguments
///
/// * `daily_summaries` - A slice of daily summary texts for the week
///
/// # Returns
///
/// A vector of messages suitable for chat completion.
pub fn weekly_summary_prompt(daily_summaries: &[String]) -> Vec<Message> {
    let combined = daily_summaries
        .iter()
        .enumerate()
        .map(|(i, summary)| format!("Day {}: {}", i + 1, summary))
        .collect::<Vec<_>>()
        .join("\n\n");

    vec![
        Message::system(SUMMARY_WEEKLY_PROMPT),
        Message::user(format!(
            r#"Please create a weekly summary from these daily summaries:

---
{}
---

Provide a focused paragraph synthesizing the week's themes, patterns, and key moments."#,
            combined
        )),
    ]
}

/// Builds messages for monthly summary generation.
///
/// # Arguments
///
/// * `weekly_summaries` - A slice of weekly summary texts for the month
///
/// # Returns
///
/// A vector of messages suitable for chat completion.
pub fn monthly_summary_prompt(weekly_summaries: &[String]) -> Vec<Message> {
    let combined = weekly_summaries
        .iter()
        .enumerate()
        .map(|(i, summary)| format!("Week {}: {}", i + 1, summary))
        .collect::<Vec<_>>()
        .join("\n\n");

    vec![
        Message::system(SUMMARY_MONTHLY_PROMPT),
        Message::user(format!(
            r#"Please create a monthly summary from these weekly summaries:

---
{}
---

Provide 2 paragraphs synthesizing the month's major themes, accomplishments, and insights."#,
            combined
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

    #[test]
    fn test_sentiment_prompt_structure() {
        let text = "I'm feeling great today!";
        let messages = sentiment_prompt(text);

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "system");
        assert_eq!(messages[0].content, SENTIMENT_PROMPT);
        assert_eq!(messages[1].role, "user");
        assert!(messages[1].content.contains(text));
        assert!(messages[1].content.contains("-1.0"));
        assert!(messages[1].content.contains("1.0"));
    }

    #[test]
    fn test_sentiment_prompt_contains_text() {
        let text = "This is a test entry with specific content.";
        let messages = sentiment_prompt(text);
        assert!(messages[1].content.contains(text));
    }

    #[test]
    fn test_topic_extraction_prompt_structure() {
        let text = "Today I worked on my career plans and had dinner with family.";
        let messages = topic_extraction_prompt(text);

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "system");
        assert_eq!(messages[0].content, TOPIC_EXTRACTION_PROMPT);
        assert_eq!(messages[1].role, "user");
        assert!(messages[1].content.contains(text));
        assert!(messages[1].content.contains("JSON"));
    }

    #[test]
    fn test_topic_extraction_prompt_contains_text() {
        let text = "Specific test content about multiple topics.";
        let messages = topic_extraction_prompt(text);
        assert!(messages[1].content.contains(text));
    }

    #[test]
    fn test_daily_summary_prompt_structure() {
        let entry = "Today was productive. I finished the project and felt accomplished.";
        let messages = daily_summary_prompt(entry);

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "system");
        assert_eq!(messages[0].content, SUMMARY_DAILY_PROMPT);
        assert_eq!(messages[1].role, "user");
        assert!(messages[1].content.contains(entry));
        assert!(messages[1].content.contains("summary"));
    }

    #[test]
    fn test_weekly_summary_prompt_structure() {
        let dailies = vec![
            "Monday summary".to_string(),
            "Tuesday summary".to_string(),
            "Wednesday summary".to_string(),
        ];
        let messages = weekly_summary_prompt(&dailies);

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "system");
        assert_eq!(messages[0].content, SUMMARY_WEEKLY_PROMPT);
        assert_eq!(messages[1].role, "user");
        assert!(messages[1].content.contains("Monday summary"));
        assert!(messages[1].content.contains("Day 1:"));
        assert!(messages[1].content.contains("Day 3:"));
    }

    #[test]
    fn test_monthly_summary_prompt_structure() {
        let weeklies = vec![
            "Week 1 summary".to_string(),
            "Week 2 summary".to_string(),
            "Week 3 summary".to_string(),
            "Week 4 summary".to_string(),
        ];
        let messages = monthly_summary_prompt(&weeklies);

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "system");
        assert_eq!(messages[0].content, SUMMARY_MONTHLY_PROMPT);
        assert_eq!(messages[1].role, "user");
        assert!(messages[1].content.contains("Week 1 summary"));
        assert!(messages[1].content.contains("Week 1:"));
        assert!(messages[1].content.contains("Week 4:"));
    }

    #[test]
    fn test_summary_prompts_include_content() {
        let daily_entry = "Test entry content";
        let daily_messages = daily_summary_prompt(daily_entry);
        assert!(daily_messages[1].content.contains(daily_entry));

        let weekly_summaries = vec!["Test summary".to_string()];
        let weekly_messages = weekly_summary_prompt(&weekly_summaries);
        assert!(weekly_messages[1].content.contains("Test summary"));

        let monthly_summaries = vec!["Test weekly".to_string()];
        let monthly_messages = monthly_summary_prompt(&monthly_summaries);
        assert!(monthly_messages[1].content.contains("Test weekly"));
    }
}
