// Hammurabi AI Highway — 1Shot API integration ("The Hands").
// Routes Venice-generated calldata to 1Shot's gas-sponsored execution relayer,
// which fires it through our MetaMask Smart Account on Base.

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};

use crate::config::Config;
use crate::venice::Calldata;

/// Submit `calldata` to 1Shot for gas-free execution on Base. Returns the
/// resulting transaction hash.
pub async fn execute(client: &reqwest::Client, config: &Config, calldata: &Calldata) -> Result<String> {
    let body = json!({
        "chainId": config.base_chain_id,
        "walletId": config.oneshot_wallet_id,
        "to": calldata.to,
        "data": calldata.data,
        "value": calldata.value,
    });

    let response = client
        .post(format!("{}/execute", config.oneshot_base_url))
        .bearer_auth(&config.oneshot_api_key)
        .json(&body)
        .send()
        .await
        .context("1Shot API request failed (network error)")?;

    let status = response.status();
    if !status.is_success() {
        let text = response.text().await.unwrap_or_default();
        bail!("1Shot API returned HTTP {status}: {text}");
    }

    let payload: Value = response
        .json()
        .await
        .context("Failed to parse 1Shot API response as JSON")?;

    payload
        .get("transactionHash")
        .or_else(|| payload.get("txHash"))
        .or_else(|| payload.get("hash"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .with_context(|| format!("1Shot API response missing a transaction hash field:\n{payload}"))
}
