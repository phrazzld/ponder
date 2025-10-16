//! Cryptographic operations for journal encryption and security.
//!
//! This module provides encryption, session management, and secure temporary file handling
//! for the Ponder journaling system. It uses the age encryption format with passphrase-based
//! encryption for simplicity and security.
//!
//! # Module Structure
//!
//! - `age`: Core encryption/decryption functions using the age crate
//! - `session`: Session key management with auto-lock timeout
//! - `temp`: Secure temporary file handling with tmpfs preference
//!
//! # Example
//!
//! ```no_run
//! use ponder::crypto::{SessionManager, encrypt_with_passphrase, decrypt_with_passphrase};
//! use age::secrecy::SecretString;
//!
//! let mut session = SessionManager::new(30); // 30-minute timeout
//! let passphrase = session.unlock()?;
//!
//! let plaintext = b"Secret journal entry";
//! let encrypted = encrypt_with_passphrase(plaintext, passphrase)?;
//! let decrypted = decrypt_with_passphrase(&encrypted, passphrase)?;
//! assert_eq!(plaintext, decrypted.as_slice());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod age;
pub mod session;
pub mod temp;

// Re-export commonly used types
pub use self::age::{
    decrypt_file_streaming, decrypt_with_passphrase, encrypt_file_streaming,
    encrypt_with_passphrase,
};
pub use self::session::SessionManager;
pub use self::temp::{decrypt_to_temp, encrypt_from_temp, get_secure_temp_dir};
