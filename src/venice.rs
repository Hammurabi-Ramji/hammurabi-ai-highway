// Hammurabi AI Highway — Venice AI integration ("The Brain").
// Translates a natural-language intent into raw Base-network calldata using
// Venice's qwen-3-7-max model, authenticated via x402.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::config::Config;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Calldata {
    pub to: String,
    pub data: String,
    #[serde(default = "default_value")]
    pub value: String,
}

fn default_value() -> String {
    "0x0".to_string()
}

const SYSTEM_PROMPT: &str = r#"You are a calldata generator for the Base network (EVM, chainId 8453).
Given a natural-language description of a token swap, buy, or staking action, respond with ONLY a
single-line JSON object containing the raw transaction fields needed to execute it:
{"to": "0x<target contract address>", "data": "0x<hex-encoded calldata>", "value": "0x<hex wei amount, 0x0 if none>"}
Do not include any explanation, markdown formatting, or additional fields. Return raw JSON only."#;

/// Ask Venice AI to translate `intent` into `{to, data, value}` calldata.
pub async fn get_calldata(client: &reqwest::Client, config: &Config, intent: &str) -> Result<Calldata> {
    let body = json!({
        "model": config.venice_model,
        "messages": [
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": intent},
        ],
        "temperature": 0,
    });
    let body_str = body.to_string();

    // Use x402 v2 for enhanced security with rate limiting
    let auth = crate::security::X402AuthV2::new(&config.hammurabi_private_key).await?;
    let signed = auth.sign(&body_str).await?;

    let mut request = client
        .post(format!("{}/chat/completions", config.venice_base_url))
        .bearer_auth(&config.venice_api_key)
        .header("Content-Type", "application/json");

    for (name, value) in signed.headers() {
        request = request.header(name, value);
    }

    let response = request
        .body(body_str)
        .send()
        .await
        .context("Venice AI request failed (network error)")?;

    let status = response.status();
    if !status.is_success() {
        let text = response.text().await.unwrap_or_default();
        bail!("Venice AI returned HTTP {status}: {text}");
    }

    let payload: Value = response
        .json()
        .await
        .context("Failed to parse Venice AI response as JSON")?;

    let content = payload["choices"][0]["message"]["content"]
        .as_str()
        .context("Venice AI response missing choices[0].message.content")?;

    parse_calldata(content)
}

/// Extract the `{to, data, value}` JSON object from a model response,
/// tolerating markdown code fences around the JSON.
fn parse_calldata(content: &str) -> Result<Calldata> {
    let cleaned = content
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    serde_json::from_str(cleaned)
        .with_context(|| format!("Failed to parse calldata JSON from Venice AI response:\n{cleaned}"))
}
