// src/crypto/key_manager.rs — Encrypted EVM key management.
//
// Layered design:
//   * `EncryptionKey`      — Argon2id KDF (passphrase + salt -> 32-byte key) and
//                            AES-256-GCM authenticated encryption.
//   * `SecureStorage`      — trait over an encrypted-blob store; `EncryptedFileStorage`
//                            is the on-disk implementation. A keychain backend can
//                            slot in behind the same trait later.
//   * `KeyManager`         — orchestrates: encrypt-and-store / load-and-decrypt an
//                            EVM private key, returning an `ethers` `LocalWallet`.
//
// Rotation scheduling and audit logging are intentionally out of scope for this
// pass and will be added on top of this foundation.

use std::path::PathBuf;
use std::sync::Arc;

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use anyhow::{anyhow, bail, Context, Result};
use argon2::Argon2;
use async_trait::async_trait;
use ethers_signers::LocalWallet;
use rand::{rngs::OsRng, RngCore};

/// Length of the random salt (bytes) prepended to every stored key blob.
const SALT_LEN: usize = 16;
/// AES-GCM nonce length (bytes).
const NONCE_LEN: usize = 12;

/// A symmetric key derived from a user passphrase, used for AES-256-GCM.
pub struct EncryptionKey {
    key: [u8; 32],
}

impl EncryptionKey {
    /// Derive a 256-bit key from `passphrase` and `salt` using Argon2id.
    pub fn derive_from_password(passphrase: &str, salt: &[u8]) -> Result<Self> {
        let mut key = [0u8; 32];
        Argon2::default()
            .hash_password_into(passphrase.as_bytes(), salt, &mut key)
            .map_err(|e| anyhow!("Argon2 key derivation failed: {e}"))?;
        Ok(Self { key })
    }

    fn cipher(&self) -> Aes256Gcm {
        Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key))
    }

    /// Encrypt `plaintext`, returning `nonce || ciphertext`.
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let mut nonce_bytes = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher()
            .encrypt(nonce, plaintext)
            .map_err(|e| anyhow!("AES-256-GCM encryption failed: {e}"))?;

        let mut out = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        out.extend_from_slice(&nonce_bytes);
        out.extend_from_slice(&ciphertext);
        Ok(out)
    }

    /// Decrypt a `nonce || ciphertext` blob produced by [`Self::encrypt`].
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.len() <= NONCE_LEN {
            bail!("ciphertext too short: {} bytes", data.len());
        }
        let (nonce_bytes, ciphertext) = data.split_at(NONCE_LEN);
        let nonce = Nonce::from_slice(nonce_bytes);

        self.cipher()
            .decrypt(nonce, ciphertext)
            .map_err(|_| anyhow!("decryption failed — wrong passphrase or corrupted key store"))
    }
}

/// Storage for opaque encrypted blobs, keyed by a logical key id.
#[async_trait]
pub trait SecureStorage: Send + Sync {
    async fn get_encrypted(&self, key_id: &str) -> Result<Vec<u8>>;
    async fn put_encrypted(&self, key_id: &str, data: Vec<u8>) -> Result<()>;
    async fn exists(&self, key_id: &str) -> bool;
}

/// Encrypted-blob storage backed by files under a directory (`<key_id>.enc`).
pub struct EncryptedFileStorage {
    dir: PathBuf,
}

impl EncryptedFileStorage {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    fn path(&self, key_id: &str) -> PathBuf {
        self.dir.join(format!("{key_id}.enc"))
    }
}

#[async_trait]
impl SecureStorage for EncryptedFileStorage {
    async fn get_encrypted(&self, key_id: &str) -> Result<Vec<u8>> {
        let path = self.path(key_id);
        std::fs::read(&path)
            .with_context(|| format!("failed to read encrypted key from {}", path.display()))
    }

    async fn put_encrypted(&self, key_id: &str, data: Vec<u8>) -> Result<()> {
        std::fs::create_dir_all(&self.dir)
            .with_context(|| format!("failed to create key store dir {}", self.dir.display()))?;
        let path = self.path(key_id);
        std::fs::write(&path, data)
            .with_context(|| format!("failed to write encrypted key to {}", path.display()))
    }

    async fn exists(&self, key_id: &str) -> bool {
        self.path(key_id).exists()
    }
}

/// Orchestrates encrypted storage and retrieval of a single EVM private key.
pub struct KeyManager {
    storage: Arc<dyn SecureStorage>,
    key_id: String,
}

impl KeyManager {
    pub fn new(storage: Arc<dyn SecureStorage>, key_id: impl Into<String>) -> Self {
        Self {
            storage,
            key_id: key_id.into(),
        }
    }

    /// Convenience constructor using on-disk encrypted-file storage.
    pub fn file_backed(dir: impl Into<PathBuf>, key_id: impl Into<String>) -> Self {
        Self::new(Arc::new(EncryptedFileStorage::new(dir)), key_id.into())
    }

    /// Whether an encrypted key already exists in the store.
    pub async fn exists(&self) -> bool {
        self.storage.exists(&self.key_id).await
    }

    /// Encrypt `private_key_hex` (a 32-byte hex key, `0x`-optional) under
    /// `passphrase` and persist it. A fresh random salt is generated and stored
    /// alongside the ciphertext (`salt || nonce || ciphertext`).
    pub async fn store_private_key(&self, private_key_hex: &str, passphrase: &str) -> Result<()> {
        let key_bytes = decode_private_key(private_key_hex)?;

        let mut salt = [0u8; SALT_LEN];
        OsRng.fill_bytes(&mut salt);

        let enc_key = EncryptionKey::derive_from_password(passphrase, &salt)?;
        let ciphertext = enc_key.encrypt(&key_bytes)?;

        let mut blob = Vec::with_capacity(SALT_LEN + ciphertext.len());
        blob.extend_from_slice(&salt);
        blob.extend_from_slice(&ciphertext);

        self.storage.put_encrypted(&self.key_id, blob).await
    }

    /// Decrypt the stored key with `passphrase` and return it as `0x`-prefixed hex.
    pub async fn load_private_key_hex(&self, passphrase: &str) -> Result<String> {
        let blob = self.storage.get_encrypted(&self.key_id).await?;
        if blob.len() <= SALT_LEN {
            bail!("stored key blob is truncated or corrupted");
        }
        let (salt, ciphertext) = blob.split_at(SALT_LEN);

        let enc_key = EncryptionKey::derive_from_password(passphrase, salt)?;
        let key_bytes = enc_key.decrypt(ciphertext)?;

        // Validate the decrypted material is a usable EVM key before returning it.
        LocalWallet::from_bytes(&key_bytes)
            .context("decrypted key is not a valid secp256k1 private key")?;

        Ok(format!("0x{}", hex::encode(&key_bytes)))
    }
}

/// Decode a `0x`-optional hex private key into exactly 32 bytes.
fn decode_private_key(private_key_hex: &str) -> Result<Vec<u8>> {
    let bytes = hex::decode(private_key_hex.trim_start_matches("0x"))
        .context("private key is not valid hex")?;
    if bytes.len() != 32 {
        bail!("private key must be 32 bytes, got {}", bytes.len());
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // Deterministic non-zero test key (secp256k1 valid).
    const TEST_KEY: &str = "0x0000000000000000000000000000000000000000000000000000000000000001";

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let salt = b"sixteen.byte.salt";
        let key = EncryptionKey::derive_from_password("correct horse", salt).unwrap();
        let plaintext = b"battery staple";

        let blob = key.encrypt(plaintext).unwrap();
        assert_ne!(&blob[..], &plaintext[..]);
        assert_eq!(key.decrypt(&blob).unwrap(), plaintext);
    }

    #[test]
    fn wrong_passphrase_fails_to_decrypt() {
        let salt = b"sixteen.byte.salt";
        let good = EncryptionKey::derive_from_password("right", salt).unwrap();
        let bad = EncryptionKey::derive_from_password("wrong", salt).unwrap();

        let blob = good.encrypt(b"secret").unwrap();
        assert!(bad.decrypt(&blob).is_err());
    }

    #[tokio::test]
    async fn store_then_load_roundtrips_the_key() {
        let dir = TempDir::new().unwrap();
        let km = KeyManager::file_backed(dir.path(), "hammurabi");

        assert!(!km.exists().await);
        km.store_private_key(TEST_KEY, "hunter2").await.unwrap();
        assert!(km.exists().await);

        let loaded = km.load_private_key_hex("hunter2").await.unwrap();
        assert_eq!(loaded, TEST_KEY);
    }

    #[tokio::test]
    async fn load_with_wrong_passphrase_errors() {
        let dir = TempDir::new().unwrap();
        let km = KeyManager::file_backed(dir.path(), "hammurabi");
        km.store_private_key(TEST_KEY, "hunter2").await.unwrap();

        assert!(km.load_private_key_hex("nope").await.is_err());
    }

    #[test]
    fn rejects_malformed_private_key() {
        assert!(decode_private_key("0x1234").is_err()); // too short
        assert!(decode_private_key("zzzz").is_err()); // not hex
    }
}
