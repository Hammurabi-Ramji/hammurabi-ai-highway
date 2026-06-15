//! Real HTTP client for the Sovereign Stack server — the RAMGate gateway that
//! hosts Millennium Falcon (code auditor), the Han Solo / AI-Highway agent
//! orchestrator, and RamGenie (NLP -> code).
//!
//! Base URL via `SOVEREIGN_API_URL` (default `http://127.0.0.1:3000`). The
//! server's real routes (see sovereign-stack/crates/server/src/routes):
//!   - GET  /health
//!   - POST /api/v1/ramgenie/generate/code   (RamGenie codegen)
//!   - POST /api/v1/ramgenie/projects        (register project)
//!   - GET  /api/agents                       (Han Solo agent roster)
//!   - GET  /api/agents/hierarchy             (ruler/king/serf hierarchy)
//!   - GET  /api/projects                     (Millennium Falcon projects)
//!   - GET  /api/analyze/metrics/summary      (audit metrics)

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};

/// Resolve the Sovereign Stack base URL from the environment.
pub fn base_url() -> String {
    std::env::var("SOVEREIGN_API_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".to_string())
}

/// Is the gateway reachable and healthy?
pub async fn health(client: &reqwest::Client, base: &str) -> bool {
    matches!(
        client.get(format!("{base}/health")).send().await,
        Ok(r) if r.status().is_success()
    )
}

async fn get_json(client: &reqwest::Client, url: String) -> Result<Value> {
    let resp = client
        .get(&url)
        .send()
        .await
        .with_context(|| format!("GET {url} failed"))?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        bail!("GET {url} -> HTTP {status}: {text}");
    }
    serde_json::from_str(&text).with_context(|| format!("GET {url}: response was not JSON"))
}

/// RamGenie: translate a prompt into a full-stack code bundle.
pub async fn ramgenie_generate(
    client: &reqwest::Client,
    base: &str,
    prompt: &str,
) -> Result<Value> {
    let body = json!({ "prompt": prompt });
    let url = format!("{base}/api/v1/ramgenie/generate/code");
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .context("RamGenie generate request failed")?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        bail!("RamGenie generate -> HTTP {status}: {text}");
    }
    serde_json::from_str(&text).context("RamGenie generate: response was not JSON")
}

/// Han Solo: the agent orchestrator hierarchy (ruler / kings / serfs).
pub async fn agent_hierarchy(client: &reqwest::Client, base: &str) -> Result<Value> {
    get_json(client, format!("{base}/api/agents/hierarchy")).await
}

/// Millennium Falcon: existing indexed projects (used as a remote state check).
pub async fn list_projects(client: &reqwest::Client, base: &str) -> Result<Value> {
    get_json(client, format!("{base}/api/projects")).await
}

/// Millennium Falcon: audit/analysis metrics summary.
pub async fn analysis_metrics(client: &reqwest::Client, base: &str) -> Result<Value> {
    get_json(client, format!("{base}/api/analyze/metrics/summary")).await
}

/// Register a project in RamGenie (BSM lockdown). Returns true on 2xx.
pub async fn create_ramgenie_project(
    client: &reqwest::Client,
    base: &str,
    name: &str,
) -> Result<bool> {
    let body = json!({ "name": name, "status": "completed", "intent": "fullstack" });
    let url = format!("{base}/api/v1/ramgenie/projects");
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .context("RamGenie create-project request failed")?;
    Ok(resp.status().is_success())
}
