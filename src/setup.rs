//! Setup and dependency management utilities.
//!
//! This module provides helpers for checking and installing Ollama models,
//! prompting users for consent, and managing first-run setup.

use crate::ai::OllamaClient;
use crate::errors::{AIError, AppError, AppResult};
use std::io::{self, Write};
use std::process::Command;
use tracing::{debug, info, warn};

/// Prompts the user for a yes/no answer with a default.
///
/// # Arguments
///
/// * `question` - The question to ask
/// * `default` - Default answer if user just presses Enter
///
/// # Returns
///
/// Returns `true` for yes, `false` for no.
pub fn prompt_yes_no(question: &str, default: bool) -> bool {
    let prompt = if default {
        format!("{} [Y/n] ", question)
    } else {
        format!("{} [y/N] ", question)
    };

    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    match input.trim().to_lowercase().as_str() {
        "" => default,
        "y" | "yes" => true,
        "n" | "no" => false,
        _ => {
            println!("Please answer yes or no.");
            prompt_yes_no(question, default)
        }
    }
}

/// Checks if a specific Ollama model is installed.
///
/// # Arguments
///
/// * `client` - Ollama client instance
/// * `model` - Name of the model to check
///
/// # Returns
///
/// Returns `Ok(true)` if model is installed, `Ok(false)` if not.
/// Returns `Err` if Ollama is not running or connection fails.
pub fn check_model_installed(client: &OllamaClient, model: &str) -> AppResult<bool> {
    debug!("Checking if model '{}' is installed", model);

    // Try to use the model with a minimal request to check if it exists
    // We use a tiny prompt to minimize resource usage
    let result = client.embed(model, "test");

    match result {
        Ok(_) => {
            info!("Model '{}' is installed", model);
            Ok(true)
        }
        Err(AppError::AI(AIError::ModelNotFound(_))) => {
            // Model not found - this is expected when checking
            debug!("Model '{}' not found", model);
            Ok(false)
        }
        Err(e) => {
            // Other errors (Ollama offline, etc) should propagate
            Err(e)
        }
    }
}

/// Pulls an Ollama model using the `ollama pull` command.
///
/// # Arguments
///
/// * `model` - Name of the model to pull
///
/// # Returns
///
/// Returns `Ok(())` if model was pulled successfully, `Err` otherwise.
///
/// # Errors
///
/// Returns an error if:
/// - `ollama` command is not found
/// - Model pull fails
/// - User cancels operation
pub fn pull_model(model: &str) -> AppResult<()> {
    info!("Pulling model: {}", model);
    println!("\nPulling {} (this may take a minute)...", model);

    let status = Command::new("ollama")
        .args(["pull", model])
        .status()
        .map_err(|e| {
            AppError::Config(format!(
                "Failed to run 'ollama pull' command: {}\n\
                 \n\
                 Is Ollama installed? Visit https://ollama.com/download",
                e
            ))
        })?;

    if !status.success() {
        return Err(AppError::Config(format!(
            "Failed to pull model '{}'. Check your internet connection or model name.",
            model
        )));
    }

    println!("✓ {} ready!", model);
    Ok(())
}

/// Checks if a model is installed, and offers to install it if not.
///
/// # Arguments
///
/// * `client` - Ollama client instance
/// * `model` - Name of the model to check/install
/// * `model_type` - Description of model type (e.g., "embedding", "chat")
///
/// # Returns
///
/// Returns `Ok(())` if model is available (either already installed or freshly pulled).
/// Returns `Err` if model is unavailable and user declined installation.
///
/// # Errors
///
/// Returns an error if:
/// - Ollama connection fails
/// - Model pull fails
/// - User declines installation when model is missing
pub fn ensure_model_available(
    client: &OllamaClient,
    model: &str,
    model_type: &str,
) -> AppResult<()> {
    match check_model_installed(client, model) {
        Ok(true) => {
            debug!("{} model '{}' is available", model_type, model);
            Ok(())
        }
        Ok(false) => {
            // Model not found - offer to install
            warn!("{} model '{}' not found", model_type, model);
            println!("\n{} model '{}' is not installed.", model_type, model);

            if prompt_yes_no("Would you like to pull it now?", true) {
                pull_model(model)?;
                println!();
                Ok(())
            } else {
                println!("\nTo install manually, run:");
                println!("  ollama pull {}", model);
                println!();
                Err(AIError::ModelNotFound(model.to_string()).into())
            }
        }
        Err(e) => {
            // Connection error or other issue
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_prompt_yes_no_default_yes() {
        // This test is for documentation - actual testing requires stdin simulation
        // In practice, this would be tested with integration tests
    }

    #[test]
    fn test_prompt_yes_no_default_no() {
        // This test is for documentation - actual testing requires stdin simulation
    }
}
