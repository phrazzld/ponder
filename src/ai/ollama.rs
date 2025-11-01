//! Ollama HTTP client for embeddings and chat completion.
//!
//! This module provides a simple client for interacting with the Ollama API
//! for generating embeddings and chat completions.

use crate::constants::DEFAULT_CHAT_MODEL;
use crate::errors::{AIError, AppResult};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use tracing::debug;

/// A message in a chat conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// The role of the message sender (system, user, assistant)
    pub role: String,
    /// The content of the message
    pub content: String,
}

impl Message {
    /// Creates a new system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }

    /// Creates a new user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    /// Creates a new assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }
}

/// Request body for embedding generation.
#[derive(Debug, Serialize)]
struct EmbedRequest {
    model: String,
    prompt: String,
}

/// Response from embedding generation.
#[derive(Debug, Deserialize)]
struct EmbedResponse {
    embedding: Vec<f32>,
}

/// Creates JSON schema for reflection decision (search vs respond).
///
/// This schema enforces the structure expected by ReflectionDecision enum.
fn reflection_decision_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "action": {
                "type": "string",
                "enum": ["search", "respond"]
            },
            "temporal_constraint": {
                "type": "object",
                "properties": {
                    "type": {"type": "string", "enum": ["none", "relative", "absolute"]},
                    "days_ago": {"type": "number"},
                    "start_date": {"type": "string"},
                    "end_date": {"type": "string"}
                },
                "required": ["type"]
            },
            "reasoning": {"type": "string"}
        },
        "required": ["action", "reasoning"]
    })
}

/// Creates JSON schema for query analysis (topic + temporal extraction).
///
/// This schema enforces the structure expected by QueryAnalysis struct.
fn query_analysis_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "topic": {"type": "string"},
            "temporal_constraint": {
                "type": "object",
                "properties": {
                    "type": {"type": "string", "enum": ["none", "relative", "absolute"]},
                    "days_ago": {"type": "number"},
                    "start_date": {"type": "string"},
                    "end_date": {"type": "string"}
                },
                "required": ["type"]
            },
            "confidence": {"type": "number"}
        },
        "required": ["topic", "temporal_constraint", "confidence"]
    })
}

/// Request body for chat completion.
#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<serde_json::Value>, // Can be string "json" or schema object
}

/// Response from chat completion.
#[derive(Debug, Deserialize)]
struct ChatResponse {
    message: Message,
}

/// Response chunk from streaming chat completion.
#[derive(Debug, Deserialize)]
struct ChatStreamChunk {
    message: Message,
    done: bool,
}

/// Type-safe wrapper for embedding model name.
///
/// Prevents accidental swapping of model name and prompt text in embed() calls.
#[derive(Debug, Clone)]
pub struct EmbedModel(String);

impl EmbedModel {
    /// Creates a new embedding model identifier.
    pub fn new(model: impl Into<String>) -> Self {
        Self(model.into())
    }

    /// Gets the model name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for EmbedModel {
    fn from(s: &str) -> Self {
        EmbedModel(s.to_string())
    }
}

/// Type-safe wrapper for prompt text.
///
/// Prevents accidental swapping of model name and prompt text in embed() calls.
#[derive(Debug, Clone)]
pub struct PromptText(String);

impl PromptText {
    /// Creates a new prompt text.
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }

    /// Gets the prompt text as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for PromptText {
    fn from(s: &str) -> Self {
        PromptText(s.to_string())
    }
}

impl From<String> for PromptText {
    fn from(s: String) -> Self {
        PromptText(s)
    }
}

/// Client for interacting with Ollama API.
pub struct OllamaClient {
    base_url: String,
    client: Client,
}

impl OllamaClient {
    /// Creates a new Ollama client.
    ///
    /// # Arguments
    ///
    /// * `base_url` - Base URL of the Ollama API (e.g., "http://127.0.0.1:11434")
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: Client::new(),
        }
    }

    /// Generates an embedding with retry logic for transient failures.
    ///
    /// Retries with exponential backoff on HTTP 500 errors (Ollama worker crashes).
    /// Other errors (404, 400, network issues) fail immediately without retry.
    ///
    /// # Arguments
    ///
    /// * `model` - Name of the embedding model
    /// * `text` - Text to generate embedding for
    /// * `max_retries` - Maximum number of retry attempts (default: 3)
    ///
    /// # Errors
    ///
    /// Returns an error if all retries are exhausted or for non-retryable errors.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ponder::ai::OllamaClient;
    /// let client = OllamaClient::new("http://127.0.0.1:11434");
    /// let embedding = client.embed_with_retry("nomic-embed-text", "sample text", 3)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn embed_with_retry(
        &self,
        model: &str,
        text: &str,
        max_retries: u32,
    ) -> AppResult<Vec<f32>> {
        let mut attempt = 0;

        loop {
            match self.embed(model, text) {
                Ok(embedding) => return Ok(embedding),
                Err(e) => {
                    // Check if error is retryable (HTTP 500)
                    let error_msg = format!("{}", e);
                    let is_http_500 = error_msg.contains("HTTP 500");

                    if !is_http_500 || attempt >= max_retries {
                        // Non-retryable error or exhausted retries
                        return Err(e);
                    }

                    attempt += 1;
                    let backoff_ms = 100 * 2_u64.pow(attempt);

                    debug!(
                        "Ollama HTTP 500 error on attempt {}/{}, retrying after {}ms",
                        attempt,
                        max_retries + 1,
                        backoff_ms
                    );

                    std::thread::sleep(std::time::Duration::from_millis(backoff_ms));
                }
            }
        }
    }

    /// Generates an embedding for the given text.
    ///
    /// # Arguments
    ///
    /// * `model` - Name of the embedding model (e.g., "nomic-embed-text")
    /// * `text` - Text to generate embedding for
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Ollama API is not reachable
    /// - Model is not found
    /// - API returns an error response
    pub fn embed(&self, model: &str, text: &str) -> AppResult<Vec<f32>> {
        debug!("Generating embedding with model: {}", model);

        let url = format!("{}/api/embeddings", self.base_url);
        let request = EmbedRequest {
            model: model.to_string(),
            prompt: text.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .map_err(AIError::OllamaOffline)?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().unwrap_or_default();

            if status.as_u16() == 404 {
                return Err(AIError::ModelNotFound(model.to_string()).into());
            }

            // Check if this is a "model doesn't support operation" error
            if error_text.contains("does not support embedding")
                || error_text.contains("doesn't support embedding")
                || error_text.contains("not support embeddings")
            {
                return Err(AIError::ModelNotSupported {
                    model: model.to_string(),
                    operation: "embeddings".to_string(),
                    suggestion: crate::constants::DEFAULT_EMBED_MODEL.to_string(),
                }
                .into());
            }

            return Err(
                AIError::InvalidResponse(format!("HTTP {}: {}", status, error_text)).into(),
            );
        }

        let embed_response: EmbedResponse = response.json().map_err(|e| {
            AIError::InvalidResponse(format!("Failed to parse embedding response: {}", e))
        })?;

        debug!(
            "Generated embedding with {} dimensions",
            embed_response.embedding.len()
        );
        Ok(embed_response.embedding)
    }

    /// Sends a chat completion request.
    ///
    /// # Arguments
    ///
    /// * `model` - Name of the chat model (e.g., "llama3.2:3b")
    /// * `messages` - Conversation messages
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Ollama API is not reachable
    /// - Model is not found
    /// - API returns an error response
    pub fn chat(&self, model: &str, messages: &[Message]) -> AppResult<String> {
        debug!("Sending chat request with model: {}", model);

        let url = format!("{}/api/chat", self.base_url);
        let request = ChatRequest {
            model: model.to_string(),
            messages: messages.to_vec(),
            stream: false,
            format: None,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .map_err(AIError::OllamaOffline)?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().unwrap_or_default();

            if status.as_u16() == 404 {
                return Err(AIError::ModelNotFound(model.to_string()).into());
            }

            return Err(
                AIError::InvalidResponse(format!("HTTP {}: {}", status, error_text)).into(),
            );
        }

        let chat_response: ChatResponse = response.json().map_err(|e| {
            AIError::InvalidResponse(format!("Failed to parse chat response: {}", e))
        })?;

        debug!("Received chat response");
        Ok(chat_response.message.content)
    }

    /// Sends a chat completion request using an optional model (defaults to config or DEFAULT_CHAT_MODEL).
    ///
    /// This is a convenience wrapper around `chat()` that allows callers to optionally
    /// specify a model, falling back to DEFAULT_CHAT_MODEL if none is provided.
    ///
    /// # Arguments
    ///
    /// * `model` - Optional model name. If None, uses DEFAULT_CHAT_MODEL
    /// * `messages` - The conversation messages to send
    ///
    /// # Returns
    ///
    /// The assistant's response text
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Ollama API is not reachable
    /// - Model is not found
    /// - API returns an error response
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ponder::ai::{OllamaClient, Message};
    /// let client = OllamaClient::new("http://127.0.0.1:11434");
    /// let messages = vec![Message::user("Hello!")];
    ///
    /// // Use default model
    /// let response = client.chat_with_model(None, &messages).unwrap();
    ///
    /// // Use specific model
    /// let response = client.chat_with_model(Some("llama3.1:8b"), &messages).unwrap();
    /// ```
    pub fn chat_with_model(&self, model: Option<&str>, messages: &[Message]) -> AppResult<String> {
        let model_name = model.unwrap_or(DEFAULT_CHAT_MODEL);
        self.chat(model_name, messages)
    }

    /// Analyzes the sentiment of text using an LLM.
    ///
    /// Returns a sentiment score between -1.0 (very negative) and 1.0 (very positive).
    /// Uses the chat model to analyze emotional tone and parse the numeric response.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to analyze for sentiment
    ///
    /// # Returns
    ///
    /// A sentiment score between -1.0 and 1.0, where:
    /// - -1.0 = Very negative
    /// - -0.5 = Moderately negative
    /// - 0.0 = Neutral
    /// - 0.5 = Moderately positive
    /// - 1.0 = Very positive
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Ollama API is not reachable
    /// - Model returns invalid response format
    /// - Response cannot be parsed as a number
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ponder::ai::OllamaClient;
    /// let client = OllamaClient::new("http://127.0.0.1:11434");
    /// let score = client.analyze_sentiment("I had a wonderful day!").unwrap();
    /// assert!(score > 0.0); // Positive sentiment
    /// ```
    pub fn analyze_sentiment(&self, text: &str) -> AppResult<f32> {
        use crate::ai::prompts::sentiment_prompt;

        let messages = sentiment_prompt(text);
        let response = self.chat(DEFAULT_CHAT_MODEL, &messages)?;

        // Parse the response, which should be a single number
        let trimmed = response.trim();
        let score: f32 = trimmed.parse().map_err(|e| {
            AIError::InvalidResponse(format!(
                "Failed to parse sentiment score '{}': {}",
                trimmed, e
            ))
        })?;

        // Clamp to valid range
        let clamped = score.clamp(-1.0, 1.0);
        if (score - clamped).abs() > 0.01 {
            debug!(
                "Sentiment score {} out of range, clamped to {}",
                score, clamped
            );
        }

        Ok(clamped)
    }

    /// Sends a chat completion request with JSON format enforcement.
    ///
    /// This method is specifically for structured output use cases where you need
    /// the LLM to return valid JSON. Ollama will enforce JSON formatting in the response.
    ///
    /// # Arguments
    ///
    /// * `messages` - Conversation messages (should include JSON schema in system prompt)
    ///
    /// # Returns
    ///
    /// The assistant's response as a JSON string
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Ollama API is not reachable
    /// - Model is not found
    /// - API returns an error response
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ponder::ai::{OllamaClient, Message};
    /// let client = OllamaClient::new("http://127.0.0.1:11434");
    /// let messages = vec![
    ///     Message::system("Return JSON with name and age"),
    ///     Message::user("Tell me about Alice, who is 30")
    /// ];
    /// let schema = serde_json::json!({
    ///     "type": "object",
    ///     "properties": {
    ///         "name": {"type": "string"},
    ///         "age": {"type": "number"}
    ///     }
    /// });
    /// let json_response = client.chat_with_json_schema(&messages, schema).unwrap();
    /// ```
    /// Sends a chat request with JSON schema enforcement using Ollama's grammar-based constraints.
    ///
    /// Uses Ollama's structured outputs feature (requires Ollama v0.5+) to guarantee
    /// the response conforms to the provided JSON schema. This is more reliable than
    /// prompting-based JSON mode as it uses grammar constraints at token generation.
    ///
    /// # Arguments
    ///
    /// * `messages` - The conversation messages
    /// * `schema` - JSON schema object defining required structure
    ///
    /// # Returns
    ///
    /// Returns the model's response as a JSON string guaranteed to match the schema.
    pub fn chat_with_json_schema(
        &self,
        messages: &[Message],
        schema: serde_json::Value,
    ) -> AppResult<String> {
        debug!("Sending chat request with JSON schema enforcement");

        let url = format!("{}/api/chat", self.base_url);
        let request = ChatRequest {
            model: DEFAULT_CHAT_MODEL.to_string(),
            messages: messages.to_vec(),
            stream: false,
            format: Some(schema), // Use grammar-based constraints via schema
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .map_err(AIError::OllamaOffline)?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().unwrap_or_default();

            if status.as_u16() == 404 {
                return Err(AIError::ModelNotFound(DEFAULT_CHAT_MODEL.to_string()).into());
            }

            return Err(
                AIError::InvalidResponse(format!("HTTP {}: {}", status, error_text)).into(),
            );
        }

        let chat_response: ChatResponse = response.json().map_err(|e| {
            AIError::InvalidResponse(format!("Failed to parse chat response: {}", e))
        })?;

        debug!("Received JSON-schema-formatted chat response");
        Ok(chat_response.message.content)
    }

    /// Sends a chat request for reflection decision with enforced schema.
    ///
    /// Uses grammar-based JSON schema constraints to guarantee the response
    /// matches the ReflectionDecision structure (search vs respond).
    ///
    /// # Arguments
    ///
    /// * `messages` - The conversation messages
    ///
    /// # Returns
    ///
    /// Returns JSON string conforming to reflection decision schema.
    pub fn chat_reflection_decision(&self, messages: &[Message]) -> AppResult<String> {
        self.chat_with_json_schema(messages, reflection_decision_schema())
    }

    /// Sends a chat request for query analysis with enforced schema.
    ///
    /// Uses grammar-based JSON schema constraints to guarantee the response
    /// matches the QueryAnalysis structure (topic + temporal + confidence).
    ///
    /// # Arguments
    ///
    /// * `messages` - The conversation messages
    ///
    /// # Returns
    ///
    /// Returns JSON string conforming to query analysis schema.
    pub fn chat_query_analysis(&self, messages: &[Message]) -> AppResult<String> {
        self.chat_with_json_schema(messages, query_analysis_schema())
    }

    /// Extracts key topics from text using an LLM.
    ///
    /// Returns a list of 3-5 main topics or themes identified in the text.
    /// Uses the chat model to analyze content and parse the JSON response.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to analyze for topics
    ///
    /// # Returns
    ///
    /// A vector of topic strings (typically 3-5 topics). Returns empty vector
    /// if no topics could be extracted.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Ollama API is not reachable
    /// - Model returns invalid JSON format
    /// - Response cannot be parsed as string array
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ponder::ai::OllamaClient;
    /// let client = OllamaClient::new("http://127.0.0.1:11434");
    /// let topics = client.extract_topics("Today I worked on my career plans.").unwrap();
    /// assert!(!topics.is_empty());
    /// ```
    pub fn extract_topics(&self, text: &str) -> AppResult<Vec<String>> {
        use crate::ai::prompts::topic_extraction_prompt;

        let messages = topic_extraction_prompt(text);
        let response = self.chat(DEFAULT_CHAT_MODEL, &messages)?;

        // Parse the response, which should be a JSON array of strings
        let trimmed = response.trim();
        let topics: Vec<String> = serde_json::from_str(trimmed).map_err(|e| {
            AIError::InvalidResponse(format!("Failed to parse topics JSON '{}': {}", trimmed, e))
        })?;

        debug!("Extracted {} topics", topics.len());
        Ok(topics)
    }

    /// Sends a chat completion request with streaming response.
    ///
    /// Returns chunks of the response as they arrive, enabling real-time display.
    /// This method is async and requires a tokio runtime.
    ///
    /// # Arguments
    ///
    /// * `model` - Name of the chat model
    /// * `messages` - Conversation messages
    ///
    /// # Returns
    ///
    /// A vector of response content strings (chunks)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Ollama API is not reachable
    /// - Model is not found
    /// - API returns an error response
    /// - Streaming response cannot be parsed
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ponder::ai::{OllamaClient, Message};
    /// # use tokio;
    /// # tokio::runtime::Runtime::new().unwrap().block_on(async {
    /// let client = OllamaClient::new("http://127.0.0.1:11434");
    /// let messages = vec![Message::user("Hello!")];
    /// let chunks = client.chat_stream("gemma3:4b", &messages).await.unwrap();
    /// for chunk in chunks {
    ///     print!("{}", chunk);
    /// }
    /// # });
    /// ```
    pub async fn chat_stream(&self, model: &str, messages: &[Message]) -> AppResult<Vec<String>> {
        use futures::StreamExt;

        debug!("Sending streaming chat request with model: {}", model);

        let url = format!("{}/api/chat", self.base_url);
        let request = ChatRequest {
            model: model.to_string(),
            messages: messages.to_vec(),
            stream: true,
            format: None,
        };

        // Create async client (note: this is separate from the blocking client)
        let async_client = reqwest::Client::new();

        let response = async_client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(AIError::OllamaOffline)?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            if status.as_u16() == 404 {
                return Err(AIError::ModelNotFound(model.to_string()).into());
            }

            return Err(
                AIError::InvalidResponse(format!("HTTP {}: {}", status, error_text)).into(),
            );
        }

        // Process streaming response chunk by chunk
        let mut chunks = Vec::new();
        let mut bytes_stream = response.bytes_stream();
        let mut buffer = String::new();

        while let Some(result) = bytes_stream.next().await {
            let bytes = result
                .map_err(|e| AIError::InvalidResponse(format!("Failed to read stream: {}", e)))?;

            // Convert bytes to string and add to buffer
            let text = std::str::from_utf8(&bytes)
                .map_err(|e| AIError::InvalidResponse(format!("Invalid UTF-8 in stream: {}", e)))?;
            buffer.push_str(text);

            // Process complete lines from buffer
            while let Some(newline_pos) = buffer.find('\n') {
                let line = buffer[..newline_pos].trim().to_string();
                buffer.drain(..=newline_pos);

                if line.is_empty() {
                    continue;
                }

                // Parse JSON response chunk
                let chunk: ChatStreamChunk = serde_json::from_str(&line).map_err(|e| {
                    AIError::InvalidResponse(format!(
                        "Failed to parse stream chunk '{}': {}",
                        line, e
                    ))
                })?;

                if !chunk.message.content.is_empty() {
                    chunks.push(chunk.message.content);
                }

                if chunk.done {
                    debug!("Received {} chunks from stream", chunks.len());
                    return Ok(chunks);
                }
            }
        }

        debug!("Received {} chunks from stream", chunks.len());
        Ok(chunks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_constructors() {
        let system = Message::system("You are a helpful assistant");
        assert_eq!(system.role, "system");
        assert_eq!(system.content, "You are a helpful assistant");

        let user = Message::user("Hello");
        assert_eq!(user.role, "user");
        assert_eq!(user.content, "Hello");

        let assistant = Message::assistant("Hi there!");
        assert_eq!(assistant.role, "assistant");
        assert_eq!(assistant.content, "Hi there!");
    }

    #[test]
    fn test_ollama_client_creation() {
        let client = OllamaClient::new("http://localhost:11434");
        assert_eq!(client.base_url, "http://localhost:11434");
    }

    #[test]
    fn test_chat_with_model_uses_default() {
        // This is a unit test that verifies the method signature and default behavior
        // Integration tests with actual Ollama instance are in tests/ops_integration_tests.rs
        let client = OllamaClient::new("http://localhost:11434");
        let messages = vec![Message::user("test")];

        // Verify that calling with None would use DEFAULT_CHAT_MODEL
        // We can't actually call it without Ollama running, but we can verify
        // the method exists and has the right signature
        let _result: Result<String, _> = client.chat_with_model(None, &messages);
        let _result: Result<String, _> = client.chat_with_model(Some("custom-model"), &messages);
    }

    #[test]
    fn test_analyze_sentiment_method_exists() {
        // Unit test verifying method signature
        // Integration tests with actual Ollama in tests/ops_integration_tests.rs
        let client = OllamaClient::new("http://localhost:11434");
        let _result: Result<f32, _> = client.analyze_sentiment("test text");
    }

    #[test]
    fn test_extract_topics_method_exists() {
        // Unit test verifying method signature
        // Integration tests with actual Ollama in tests/ops_integration_tests.rs
        let client = OllamaClient::new("http://localhost:11434");
        let _result: Result<Vec<String>, _> = client.extract_topics("test text");
    }
}
