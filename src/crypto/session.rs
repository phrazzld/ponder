//! Session key management with auto-lock timeout.
//!
//! This module manages encryption passphrases with automatic locking after a timeout period,
//! and ensures secure memory zeroization when passphrases are no longer needed.

use crate::errors::{AppResult, CryptoError};
use age::secrecy::SecretString;
use std::time::{Duration, Instant};
use tracing::{debug, info};

/// Manages encryption session state with auto-lock timeout.
///
/// The session manager caches the user's passphrase for a configurable timeout period,
/// automatically locking after inactivity. SecretString handles memory zeroization internally.
///
/// # Example
///
/// ```no_run
/// use ponder::crypto::SessionManager;
/// use age::secrecy::SecretString;
///
/// let mut session = SessionManager::new(30); // 30-minute timeout
///
/// // Unlock with passphrase
/// let passphrase = SecretString::new("my-secret".to_string());
/// session.unlock(passphrase);
///
/// // Get cached passphrase within timeout
/// let cached = session.get_passphrase()?;
///
/// // Explicit lock clears cached passphrase
/// session.lock();
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct SessionManager {
    passphrase: Option<SecretString>,
    last_access: Option<Instant>,
    timeout: Duration,
}

impl SessionManager {
    /// Create a new session manager with the specified timeout in minutes.
    ///
    /// # Example
    ///
    /// ```
    /// use ponder::crypto::SessionManager;
    ///
    /// let session = SessionManager::new(30); // 30-minute timeout
    /// assert!(session.is_locked());
    /// ```
    pub fn new(timeout_minutes: u64) -> Self {
        Self {
            passphrase: None,
            last_access: None,
            timeout: Duration::from_secs(timeout_minutes * 60),
        }
    }

    /// Store a passphrase and start the timeout timer.
    ///
    /// This unlocks the vault and caches the passphrase for the configured timeout period.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ponder::crypto::SessionManager;
    /// use age::secrecy::SecretString;
    ///
    /// let mut session = SessionManager::new(30);
    /// let passphrase = SecretString::new("my-secret".to_string());
    /// session.unlock(passphrase);
    /// assert!(!session.is_locked());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn unlock(&mut self, passphrase: SecretString) {
        self.passphrase = Some(passphrase);
        self.last_access = Some(Instant::now());
    }

    /// Get the cached passphrase if available and not timed out.
    ///
    /// Returns a reference to the cached passphrase, or an error if the vault is locked.
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::VaultLocked` if no passphrase is cached or if the timeout has elapsed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ponder::crypto::SessionManager;
    /// use age::secrecy::SecretString;
    ///
    /// let mut session = SessionManager::new(30);
    /// let passphrase = SecretString::new("my-secret".to_string());
    /// session.unlock(passphrase);
    ///
    /// // Get the passphrase within timeout
    /// let cached = session.get_passphrase()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn get_passphrase(&self) -> AppResult<&SecretString> {
        if self.is_locked() {
            return Err(crate::errors::CryptoError::VaultLocked.into());
        }

        // Update check: if we have a passphrase and haven't timed out, return it
        self.passphrase
            .as_ref()
            .ok_or_else(|| crate::errors::CryptoError::VaultLocked.into())
    }

    /// Check if the session has timed out.
    ///
    /// Returns `true` if no passphrase is cached or if the timeout has elapsed.
    ///
    /// # Example
    ///
    /// ```
    /// use ponder::crypto::SessionManager;
    ///
    /// let session = SessionManager::new(30);
    /// assert!(session.is_locked()); // Initially locked
    /// ```
    pub fn is_locked(&self) -> bool {
        match (self.passphrase.as_ref(), self.last_access) {
            (None, _) => true,
            (Some(_), None) => true, // Should never happen, but treat as locked
            (Some(_), Some(last_access)) => {
                let elapsed = Instant::now().duration_since(last_access);
                elapsed >= self.timeout
            }
        }
    }

    /// Explicitly lock the vault and zeroize the cached passphrase.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ponder::crypto::SessionManager;
    /// use age::secrecy::SecretString;
    ///
    /// let mut session = SessionManager::new(30);
    /// let passphrase = SecretString::new("my-secret".to_string());
    /// session.unlock(passphrase);
    /// session.lock(); // Explicitly clear passphrase
    /// assert!(session.is_locked());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn lock(&mut self) {
        // Drop the passphrase - SecretString will zeroize on drop
        self.passphrase = None;
        self.last_access = None;
    }

    /// Prompts user for a new passphrase with confirmation.
    ///
    /// Used during first-run when creating an encrypted journal.
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::PassphraseMismatch` if confirmations don't match.
    fn prompt_for_new_passphrase() -> AppResult<SecretString> {
        debug!("Prompting for new passphrase (first-run)");

        println!("\n🔐 Creating new encrypted journal");
        println!("Choose a strong passphrase to protect your journal entries.\n");

        let passphrase = rpassword::prompt_password("Enter passphrase: ")
            .map_err(|e| CryptoError::PassphrasePrompt(e.to_string()))?;

        let confirmation = rpassword::prompt_password("Confirm passphrase: ")
            .map_err(|e| CryptoError::PassphrasePrompt(e.to_string()))?;

        if passphrase != confirmation {
            return Err(CryptoError::PassphraseMismatch.into());
        }

        if passphrase.is_empty() {
            return Err(CryptoError::EmptyPassphrase.into());
        }

        info!("New passphrase set successfully");
        Ok(SecretString::new(passphrase))
    }

    /// Prompts user for existing passphrase.
    ///
    /// Used when unlocking an existing encrypted journal.
    ///
    /// # Errors
    ///
    /// Returns error if stdin reading fails.
    fn prompt_for_existing_passphrase() -> AppResult<SecretString> {
        debug!("Prompting for existing passphrase");

        println!("\n🔓 Unlocking encrypted journal");

        let passphrase = rpassword::prompt_password("Enter passphrase: ")
            .map_err(|e| CryptoError::PassphrasePrompt(e.to_string()))?;

        if passphrase.is_empty() {
            return Err(CryptoError::EmptyPassphrase.into());
        }

        Ok(SecretString::new(passphrase))
    }

    /// Gets cached passphrase or prompts user if vault is locked.
    ///
    /// # Arguments
    ///
    /// * `db_exists` - Whether the database file already exists (first-run detection)
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - User declines passphrase prompt
    /// - Passphrase confirmation fails (new passphrase)
    /// - stdin reading fails
    ///
    /// # Testing
    ///
    /// For non-interactive testing, set `PONDER_TEST_PASSPHRASE` environment variable.
    /// This bypasses interactive prompting and uses the provided value.
    pub fn get_passphrase_or_prompt(&mut self, db_exists: bool) -> AppResult<&SecretString> {
        if self.is_locked() {
            debug!(
                "Vault locked, prompting for passphrase (db_exists={})",
                db_exists
            );

            // Check for test passphrase env var (for non-interactive testing)
            let passphrase = if let Ok(test_passphrase) = std::env::var("PONDER_TEST_PASSPHRASE") {
                debug!("Using PONDER_TEST_PASSPHRASE for non-interactive testing");
                SecretString::new(test_passphrase)
            } else if db_exists {
                Self::prompt_for_existing_passphrase()?
            } else {
                Self::prompt_for_new_passphrase()?
            };

            self.unlock(passphrase);
        } else {
            debug!("Using cached passphrase");
        }

        self.last_access = Some(Instant::now());
        self.passphrase
            .as_ref()
            .ok_or_else(|| CryptoError::VaultLocked.into())
    }
}

impl Drop for SessionManager {
    fn drop(&mut self) {
        self.lock();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use age::secrecy::{ExposeSecret, SecretString};
    use std::thread;

    #[test]
    fn test_session_initially_locked() {
        let session = SessionManager::new(30);
        assert!(session.is_locked());

        // Trying to get passphrase should fail
        let result = session.get_passphrase();
        assert!(result.is_err());
    }

    #[test]
    fn test_session_unlock_and_cache() {
        let mut session = SessionManager::new(30);
        let passphrase = SecretString::new("test-passphrase".to_string());

        // Unlock with passphrase
        session.unlock(passphrase);

        // Should now be unlocked
        assert!(!session.is_locked());

        // Should be able to get passphrase
        let cached = session.get_passphrase();
        assert!(cached.is_ok());
    }

    #[test]
    fn test_session_timeout() {
        // Use a very short timeout (1 second) for testing
        let mut session = SessionManager::new(0); // 0 minutes = immediate timeout
        let passphrase = SecretString::new("test-passphrase".to_string());

        // Unlock
        session.unlock(passphrase);

        // Wait for timeout (plus a small buffer)
        thread::sleep(Duration::from_millis(100));

        // Should be locked after timeout
        assert!(session.is_locked());

        // Getting passphrase should fail
        let result = session.get_passphrase();
        assert!(result.is_err());
    }

    #[test]
    fn test_explicit_lock() {
        let mut session = SessionManager::new(30);
        let passphrase = SecretString::new("test-passphrase".to_string());

        // Unlock
        session.unlock(passphrase);
        assert!(!session.is_locked());

        // Explicitly lock
        session.lock();
        assert!(session.is_locked());

        // Getting passphrase should fail
        let result = session.get_passphrase();
        assert!(result.is_err());
    }

    #[test]
    fn test_drop_zeroizes() {
        // Create a session in a scope
        {
            let mut session = SessionManager::new(30);
            let passphrase = SecretString::new("test-passphrase".to_string());
            session.unlock(passphrase);
            assert!(!session.is_locked());
            // Session dropped here
        }

        // After scope, session is dropped and passphrase should be zeroized
        // We can't directly verify memory is zeroed, but the Drop impl calls lock()
        // which sets passphrase to None, triggering SecretString's zeroization
    }

    #[test]
    fn test_passphrase_caching_within_timeout() {
        let mut session = SessionManager::new(30);
        let passphrase = SecretString::new("test-passphrase".to_string());

        // Unlock
        session.unlock(passphrase);

        // Multiple accesses within timeout should succeed
        for _ in 0..5 {
            let result = session.get_passphrase();
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_session_uses_test_passphrase_env_var() {
        std::env::set_var("PONDER_TEST_PASSPHRASE", "unit-test-passphrase");
        let mut session = SessionManager::new(30);

        let secret = session
            .get_passphrase_or_prompt(true)
            .expect("env-provided passphrase should unlock session");
        assert_eq!(secret.expose_secret(), "unit-test-passphrase");

        std::env::remove_var("PONDER_TEST_PASSPHRASE");
    }
}
