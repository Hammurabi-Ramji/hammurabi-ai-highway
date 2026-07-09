// src/security/x402_v2.rs — Enhanced x402 authentication
//
// The rate-limiter internals here are placeholders (see `allow` below): `sign`
// and the header emitter are used by the Venice path, but the limiter fields
// and helpers are reserved for the future Redis-backed implementation.
#![allow(dead_code)]

use anyhow::{Context, Result};
use ethers_core::utils::to_checksum;
use ethers_signers::{LocalWallet, Signer};
use std::collections::HashMap;
use std::time::Duration;
use chrono::Utc;

/// Rate limiter structure (simplified)
pub struct RateLimiter {
    requests: HashMap<String, Vec<u64>>,
    window_size: Duration,
    max_requests: u64,
}

impl RateLimiter {
    pub fn new(max_requests_per_hour: u64) -> Self {
        Self {
            requests: HashMap::new(),
            window_size: Duration::from_secs(3600),
            max_requests: max_requests_per_hour,
        }
    }
    
    pub async fn allow(&self, _address: &str, _count: u64) -> bool {
        // Simplified: in production use Redis
        true
    }
}

/// x402 v2 authentication with rate limiting and multi-key support
pub struct X402AuthV2 {
    wallet: LocalWallet,
    rate_limiter: RateLimiter,
}

impl X402AuthV2 {
    pub async fn new(private_key: &str) -> Result<Self> {
        let wallet: LocalWallet = private_key
            .parse()
            .context("Invalid private key")?;
        
        Ok(Self {
            wallet,
            rate_limiter: RateLimiter::new(100),
        })
    }
    
    /// Sign request with rate limiting
    pub async fn sign(&self, body: &str) -> Result<X402Auth> {
        // Check rate limit
        let address = self.wallet.address();
        if !self.rate_limiter.allow(&format!("{:?}", address), 1).await {
            anyhow::bail!("Rate limit exceeded");
        }
        
        let timestamp = Utc::now().timestamp();
        let message = format!("{timestamp}.{body}");
        
        let signature = self.wallet
            .sign_message(message.as_bytes())
            .await
            .context("Failed to sign x402 auth payload")?;
        
        Ok(X402Auth {
            address: to_checksum(&address, None),
            timestamp: timestamp.to_string(),
            signature: format!("0x{}", hex::encode(signature.to_vec())),
        })
    }
    
    /// Get wallet address
    pub fn address(&self) -> ethers_core::types::Address {
        self.wallet.address()
    }
}

/// x402 authentication headers
#[derive(Debug, Clone)]
pub struct X402Auth {
    pub address: String,
    pub timestamp: String,
    pub signature: String,
}

impl X402Auth {
    pub fn headers(&self) -> HashMap<&'static str, String> {
        let mut headers = HashMap::new();
        headers.insert("X-402-Address", self.address.clone());
        headers.insert("X-402-Timestamp", self.timestamp.clone());
        headers.insert("X-402-Signature", self.signature.clone());
        headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_x402_v2_creation() {
        let auth = X402AuthV2::new("0x0000000000000000000000000000000000000000000000000000000000000001")
            .await
            .unwrap();
        
        assert_eq!(
            auth.address(),
            "0x7E5F4552091A69125d5DfCb7b8C2659029395Bdf"
                .parse::<ethers_core::types::Address>()
                .unwrap()
        );
    }
    
    #[tokio::test]
    async fn test_sign() {
        let auth = X402AuthV2::new("0x0000000000000000000000000000000000000000000000000000000000000001")
            .await
            .unwrap();
        
        let signed = auth.sign("test body").await.unwrap();
        
        assert!(!signed.address.is_empty());
        assert!(!signed.timestamp.is_empty());
        assert!(!signed.signature.is_empty());
    }
}
