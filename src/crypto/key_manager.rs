// src/crypto/key_manager.rs — Enterprise-grade key management

use anyhow::{Context, Result, anyhow};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{Utc, Duration};
use serde::{Deserialize, Serialize};

/// Secure storage trait for OS-level key management
#[cfg_attr(test, mockall::automock)]
pub trait SecureStorage: Send + Sync {
    async fn get_encrypted(&self, key_id: &str) -> Result<Vec<u8>>;
    async fn put_encrypted(&self, key_id: &str, data: Vec<u8>) -> Result<()>;
    fn derive_public(&self, key_id: &str) -> Result<ethers_core::types::Address>;
}

/// Fallback encrypted file storage (development only)
pub struct EncryptedFileStorage {
    storage_path: String,
    encryption_key: Arc<EncryptionKey>,
}

impl SecureStorage for EncryptedFileStorage {
    async fn get_encrypted(&self, key_id: &str) -> Result<Vec<u8>> {
        let file_path = format!("{}/{}.enc", self.storage_path, key_id);
        let encrypted = std::fs::read(&file_path)
            .with_context(|| format!("Failed to read encrypted key from {file_path}"))?;
        
        self.encryption_key.decrypt(&encrypted)
    }
    
    async fn put_encrypted(&self, key_id: &str, data: Vec<u8>) -> Result<()> {
        let encrypted = self.encryption_key.encrypt(&data)?;
        
        let file_path = format!("{}/{}.enc", self.storage_path, key_id);
        std::fs::write(&file_path, &encrypted)
            .with_context(|| format!("Failed to write encrypted key to {file_path}"))?;
        
        Ok(())
    }
    
    fn derive_public(&self, _key_id: &str) -> Result<ethers_core::types::Address> {
        Ok(ethers_core::types::Address::zero())
    }
}

/// Encryption key derived from user master password
pub struct EncryptionKey {
    key: aes_gcm::Key<aes_gcm::Aes256>,
}

impl EncryptionKey {
    /// Derive encryption key from password using Argon2id
    pub fn derive_from_password(password: &str, salt: &[u8]) -> Result<Self> {
        let hash = argon2::hash_raw(password.as_bytes(), salt, &argon2::Config::default())?;
        
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&hash[..32]);
        
        let key = aes_gcm::Key::<aes_gcm::Aes256>::from_slice(&key_bytes);
        
        Ok(Self { key })
    }
    
    /// Encrypt data using AES-256-GCM
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
        
        let cipher = Aes256Gcm::new(&self.key);
        let nonce = aes_gcm::Aes256Gcm::generate_nonce(&mut rand::rng());
        
        let ciphertext = cipher.encrypt(&nonce, plaintext)
            .map_err(|e| anyhow!("Encryption failed: {e}"))?;
        
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }
    
    /// Decrypt data using AES-256-GCM
    pub fn decrypt(&self, ciphertext_with_nonce: &[u8]) -> Result<Vec<u8>> {
        use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
        
        let nonce_slice = &ciphertext_with_nonce[..12];
        let ciphertext = &ciphertext_with_nonce[12..];
        
        let nonce = Nonce::from_slice(nonce_slice);
        let cipher = Aes256Gcm::new(&self.key);
        
        cipher.decrypt(nonce, ciphertext)
            .map_err(|e| anyhow!("Decryption failed: {e}"))
    }
}

/// Key rotation schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationSchedule {
    rotation_interval_days: u64,
    retirement_days_after_rotation: u64,
}

impl Default for KeyRotationSchedule {
    fn default() -> Self {
        Self {
            rotation_interval_days: 90,
            retirement_days_after_rotation: 30,
        }
    }
}

impl KeyRotationSchedule {
    pub fn next_rotation_date(&self, last_rotation: chrono::DateTime<Utc>) -> chrono::DateTime<Utc> {
        last_rotation + Duration::days(self.rotation_interval_days as i64)
    }
    
    pub fn should_rotate(&self, last_rotation: chrono::DateTime<Utc>) -> bool {
        Utc::now() >= self.next_rotation_date(last_rotation)
    }
}

/// Audit logging for key operations
#[derive(Debug, Clone)]
pub enum KeyOperation {
    Load { key_id: String },
    Rotate { key_id: String, new_id: String },
    Revoke { key_id: String },
    Validate { key_id: String, result: bool },
}

pub struct AuditLogger {
    log_path: String,
}

impl AuditLogger {
    pub fn new(log_path: String) -> Self {
        Self { log_path }
    }
    
    pub fn log(&self, operation: KeyOperation) {
        let timestamp = Utc::now();
        let log_entry = serde_json::json!({
            "timestamp": timestamp.to_rfc3339(),
            "operation": match operation {
                KeyOperation::Load { .. } => "load",
                KeyOperation::Rotate { .. } => "rotate",
                KeyOperation::Revoke { .. } => "revoke",
                KeyOperation::Validate { .. } => "validate",
            },
            "key_id": match &operation {
                KeyOperation::Load { key_id } => key_id,
                KeyOperation::Rotate { key_id, .. } => key_id,
                KeyOperation::Revoke { key_id } => key_id,
                KeyOperation::Validate { key_id, .. } => key_id,
            },
        });
        
        eprintln!("[AUDIT] {:?}", log_entry);
    }
}

/// Main Key Manager
pub struct KeyManager {
    secure_store: Arc<dyn SecureStorage>,
    encryption_key: Arc<EncryptionKey>,
    rotation_schedule: Arc<RwLock<KeyRotationSchedule>>,
    audit_logger: AuditLogger,
}

impl KeyManager {
    pub fn new_development(storage_path: String, password: &str) -> Result<Self> {
        let salt = b"dev_salt_for_testing_only";
        let encryption_key = Arc::new(EncryptionKey::derive_from_password(password, salt)?);
        let storage = Arc::new(EncryptedFileStorage {
            storage_path: storage_path.clone(),
            encryption_key: encryption_key.clone(),
        });
        
        Ok(Self {
            secure_store: storage,
            encryption_key,
            rotation_schedule: Arc::new(RwLock::new(KeyRotationSchedule::default())),
            audit_logger: AuditLogger::new(format!("{}/audit.log", storage_path)),
        })
    }
    
    pub async fn load_key(&self, key_id: &str) -> Result<ethers_signers::LocalWallet> {
        let encrypted_key = self.secure_store.get_encrypted(key_id).await
            .with_context(|| format!("Failed to load encrypted key for {key_id}"))?;
        
        let key_bytes = self.encryption_key.decrypt(&encrypted_key)?;
        
        let private_key: ethers_core::types::U256 = key_bytes.as_slice().try_into()
            .map_err(|_| anyhow!("Invalid private key length"))?;
        
        let wallet = ethers_signers::LocalWallet::from_private_key(private_key)
            .context("Failed to create wallet from private key")?;
        
        self.audit_logger.log(KeyOperation::Load { key_id: key_id.to_string() });
        
        Ok(wallet)
    }
    
    pub async fn rotate_key(&self, key_id: &str) -> Result<KeyRotationResult> {
        let new_wallet = ethers_signers::LocalWallet::new(&mut rand::thread_rng());
        let new_private_key = new_wallet.secret();
        let new_key_bytes: [u8; 32] = new_private_key.to_bytes();
        
        let encrypted = self.encryption_key.encrypt(&new_key_bytes)?;
        self.secure_store.put_encrypted(key_id, encrypted).await
            .context("Failed to store new encrypted key")?;
        
        let mut schedule = self.rotation_schedule.write().await;
        schedule.rotate_key_retired(key_id).await?;
        
        self.audit_logger.log(KeyOperation::Rotate {
            key_id: key_id.to_string(),
            new_id: key_id.to_string(),
        });
        
        Ok(KeyRotationResult::Success)
    }
    
    pub fn validate_key(&self, key_id: &str, expected_public: ethers_core::types::Address) -> Result<bool> {
        let wallet = futures::executor::block_on(self.load_key(key_id))
            .context("Failed to load key for validation")?;
        
        let actual_public = wallet.address();
        let valid = actual_public == expected_public;
        
        self.audit_logger.log(KeyOperation::Validate {
            key_id: key_id.to_string(),
            result: valid,
        });
        
        Ok(valid)
    }
}

#[derive(Debug, Clone)]
pub enum KeyRotationResult {
    Success,
    Failure(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    #[test]
    fn test_encryption_key_derivation() {
        let password = "test_password";
        let salt = b"test_salt";
        
        let key1 = EncryptionKey::derive_from_password(password, salt).unwrap();
        let key2 = EncryptionKey::derive_from_password(password, salt).unwrap();
        
        assert_eq!(key1.key.as_ref(), key2.key.as_ref());
    }
    
    #[test]
    fn test_encrypt_decrypt() {
        let key = EncryptionKey::derive_from_password("test", b"salt").unwrap();
        let plaintext = b"Hello, World!";
        
        let encrypted = key.encrypt(plaintext).unwrap();
        let decrypted = key.decrypt(&encrypted).unwrap();
        
        assert_eq!(decrypted, plaintext);
    }
}
