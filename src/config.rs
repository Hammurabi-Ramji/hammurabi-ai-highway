// src/config.rs — Updated config with error handling

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub venice_api_key: String,
    pub venice_base_url: String,
    pub venice_model: String,
    pub hammurabi_private_key: String,
    pub oneshot_api_key: String,
    pub oneshot_base_url: String,
    pub oneshot_wallet_id: String,
    pub base_chain_id: u64,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            venice_api_key: env_var("VENICE_API_KEY")?,
            venice_base_url: env_var("VENICE_BASE_URL").unwrap_or_else(|_| "https://api.venice.ai/api/v1".to_string()),
            venice_model: env_var("VENICE_MODEL").unwrap_or_else(|_| "qwen-3-7-max".to_string()),
            // Resolved at runtime: from the encrypted key store (preferred) or
            // this plaintext env var as a fallback. Empty when neither is set.
            hammurabi_private_key: env_var("HAMMURABI_PRIVATE_KEY").unwrap_or_default(),
            oneshot_api_key: env_var("ONESHOT_API_KEY")?,
            oneshot_base_url: env_var("ONESHOT_BASE_URL").unwrap_or_else(|_| "https://api.1shotapi.com/v1".to_string()),
            oneshot_wallet_id: env_var("ONESHOT_WALLET_ID")?,
            base_chain_id: env_var("BASE_CHAIN_ID")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(8453), // Base mainnet
        })
    }
}

fn env_var(name: &str) -> Result<String> {
    std::env::var(name)
        .with_context(|| format!("Environment variable {name} not set"))
}

pub fn mask(secret: &str) -> String {
    if secret.len() <= 4 {
        format!("****{}", &secret[..])
    } else {
        format!("****{}", &secret[secret.len() - 4..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mask_short_secret() {
        assert_eq!(mask("abc"), "****abc");
    }
    
    #[test]
    fn test_mask_long_secret() {
        assert_eq!(mask("verylongsecret"), "****cret");
    }
    
    #[test]
    fn test_mask_medium_secret() {
        // mask() reveals the last four characters for secrets longer than four.
        assert_eq!(mask("secret"), "****cret");
    }
}
