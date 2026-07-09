// src/crypto/mod.rs — Encrypted key management.

pub mod key_manager;

// Full public surface of the key-management layer. The binary currently uses
// only `KeyManager`; the rest are re-exported for library consumers and tests.
#[allow(unused_imports)]
pub use key_manager::{EncryptedFileStorage, EncryptionKey, KeyManager, SecureStorage};
