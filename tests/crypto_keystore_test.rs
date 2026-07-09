// Black-box verification of the encrypted key store through the public API.
//
// This exercises every part of the `keys init` / key-resolution path EXCEPT the
// interactive `rpassword` terminal read (which by design cannot run headless):
// encryption-at-rest, on-disk blob format, salt/nonce randomization, round-trip
// decryption, and wrong-passphrase rejection.

use hammurabi_ai_highway::crypto::KeyManager;
use tempfile::TempDir;

// 32-byte key with a distinctive repeating pattern so we can prove the on-disk
// blob is genuinely encrypted (the pattern must NOT appear verbatim in it).
const KEY_HEX: &str = "0xa1b2c3d4a1b2c3d4a1b2c3d4a1b2c3d4a1b2c3d4a1b2c3d4a1b2c3d4a1b2c3d4";
const PATTERN: [u8; 8] = [0xa1, 0xb2, 0xc3, 0xd4, 0xa1, 0xb2, 0xc3, 0xd4];
const PASSPHRASE: &str = "correct-horse-battery-staple";

#[tokio::test]
async fn keystore_encrypts_on_disk_and_roundtrips() {
    let dir = TempDir::new().unwrap();
    let km = KeyManager::file_backed(dir.path(), "signing");

    assert!(!km.exists().await, "store should start empty");

    km.store_private_key(KEY_HEX, PASSPHRASE).await.unwrap();
    assert!(km.exists().await, "store should exist after write");

    // Inspect the raw on-disk blob: salt(16) + nonce(12) + ciphertext(32 + 16
    // GCM tag) = 76 bytes.
    let blob = std::fs::read(dir.path().join("signing.enc")).unwrap();
    assert_eq!(blob.len(), 76, "unexpected encrypted blob length");

    // The plaintext key pattern must not appear verbatim — proving it is stored
    // encrypted, not in the clear.
    assert!(
        !blob.windows(PATTERN.len()).any(|w| w == PATTERN),
        "plaintext key material found in the on-disk blob"
    );

    // Correct passphrase round-trips to the exact key.
    let loaded = km.load_private_key_hex(PASSPHRASE).await.unwrap();
    assert_eq!(loaded, KEY_HEX);

    // Wrong passphrase is rejected by the GCM authentication tag.
    assert!(km.load_private_key_hex("wrong-passphrase").await.is_err());
}

#[tokio::test]
async fn each_store_uses_a_fresh_salt_and_nonce() {
    // Storing the same key under the same passphrase twice must produce
    // different on-disk blobs (random salt + nonce) — i.e. non-deterministic
    // ciphertext — while both still decrypt to the same key.
    let d1 = TempDir::new().unwrap();
    let d2 = TempDir::new().unwrap();
    let km1 = KeyManager::file_backed(d1.path(), "signing");
    let km2 = KeyManager::file_backed(d2.path(), "signing");

    km1.store_private_key(KEY_HEX, "same-pass").await.unwrap();
    km2.store_private_key(KEY_HEX, "same-pass").await.unwrap();

    let b1 = std::fs::read(d1.path().join("signing.enc")).unwrap();
    let b2 = std::fs::read(d2.path().join("signing.enc")).unwrap();
    assert_ne!(b1, b2, "identical blobs => salt/nonce not randomized");

    assert_eq!(
        km1.load_private_key_hex("same-pass").await.unwrap(),
        km2.load_private_key_hex("same-pass").await.unwrap(),
    );
}
