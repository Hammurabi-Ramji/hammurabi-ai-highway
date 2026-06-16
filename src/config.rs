// Hammurabi AI Highway — environment configuration.
// All secrets are loaded from `.env` (gitignored) — never hardcoded.

use anyhow::{Context, Result};

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
        // Load .env if present; ignore if missing (env vars may be set another way).
        dotenvy::dotenv().ok();

        Ok(Self {
            venice_api_key: env_var("VENICE_API_KEY")?,
            venice_base_url: env_var_or(
                "VENICE_BASE_URL",
                "https://api.venice.ai/api/v1",
            ),
            venice_model: env_var_or("VENICE_MODEL", "qwen-3-7-max"),
            hammurabi_private_key: env_var("HAMMURABI_PRIVATE_KEY")?,
            oneshot_api_key: env_var("ONESHOT_API_KEY")?,
            oneshot_base_url: env_var_or("ONESHOT_BASE_URL", "https://api.1shotapi.com/v1"),
            oneshot_wallet_id: env_var("ONESHOT_WALLET_ID")?,
            base_chain_id: 8453,
        })
    }
}

fn env_var(key: &str) -> Result<String> {
    std::env::var(key)
        .with_context(|| format!("Missing required env var `{key}` — see .env.example"))
}

fn env_var_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Mask a secret for safe printing — keep only the last 4 characters.
pub fn mask(secret: &str) -> String {
    if secret.len() <= 4 {
        "****".to_string()
    } else {
        format!("****{}", &secret[secret.len() - 4..])
    }
}
