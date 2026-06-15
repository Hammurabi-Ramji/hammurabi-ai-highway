// Hammurabi AI Highway — x402 cryptographic auth.
// Signs a timestamped request body with the local EVM wallet (EIP-191 personal_sign)
// so Venice AI can verify the request originated from our sovereign identity.

use anyhow::{Context, Result};
use ethers_core::utils::to_checksum;
use ethers_signers::{LocalWallet, Signer};
use std::collections::HashMap;

pub struct X402Auth {
    pub address: String,
    pub timestamp: String,
    pub signature: String,
}

impl X402Auth {
    /// Sign `body` with `private_key`, binding the signature to the current
    /// unix timestamp so replayed requests can be rejected server-side.
    pub async fn sign(private_key: &str, body: &str) -> Result<Self> {
        let wallet: LocalWallet = private_key
            .parse()
            .context("Invalid HAMMURABI_PRIVATE_KEY — expected a 0x-prefixed secp256k1 hex key")?;

        let timestamp = chrono::Utc::now().timestamp().to_string();
        let message = format!("{timestamp}.{body}");

        let signature = wallet
            .sign_message(message.as_bytes())
            .await
            .context("Failed to sign x402 auth payload")?;

        Ok(Self {
            address: to_checksum(&wallet.address(), None),
            timestamp,
            signature: format!("0x{}", hex::encode(signature.to_vec())),
        })
    }

    /// HTTP headers carrying the x402 auth payload.
    pub fn headers(&self) -> HashMap<&'static str, String> {
        let mut headers = HashMap::new();
        headers.insert("X-402-Address", self.address.clone());
        headers.insert("X-402-Timestamp", self.timestamp.clone());
        headers.insert("X-402-Signature", self.signature.clone());
        headers
    }
}
