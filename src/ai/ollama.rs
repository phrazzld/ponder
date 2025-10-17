//! Ollama HTTP client for embeddings and chat completion.
//!
//! This module provides a simple client for interacting with the Ollama API
//! for generating embeddings and chat completions.

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

/// Request body for chat completion.
#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
}

/// Response from chat completion.
#[derive(Debug, Deserialize)]
struct ChatResponse {
    message: Message,
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
}
