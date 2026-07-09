// src/pipeline/bsm.rs — Updated error handling

use anyhow::{Context, Result};
use async_trait::async_trait;
use rusqlite::Connection;

use crate::pipeline::{Payload, PipelineContext, Stage};
use crate::sovereign;

pub struct Bsm;

#[async_trait]
impl Stage for Bsm {
    fn name(&self) -> &'static str {
        "BSM"
    }
    fn role(&self) -> &'static str {
        "finalize / lockdown"
    }

    async fn process(&self, ctx: &PipelineContext, mut payload: Payload) -> Result<Payload> {
        if !payload.cache_hit && !payload.artifacts.is_empty() {
            let artifact_summary = payload
                .artifacts
                .iter()
                .map(|a| format!("{} ({})", a.path, a.summary))
                .collect::<Vec<_>>()
                .join("; ");

            let conn = Connection::open(&ctx.db_path)
                .with_context(|| format!("BSM: failed to open registry at {}", ctx.db_path))?;
            // Ensure the registry table exists — BSM may be the first stage to
            // touch a fresh database (Eduba only reads), so it owns creation.
            conn.execute(
                "CREATE TABLE IF NOT EXISTS code_registry (
                    intent_hash TEXT PRIMARY KEY,
                    intent TEXT NOT NULL,
                    artifact TEXT NOT NULL,
                    created_at TEXT NOT NULL DEFAULT (datetime('now'))
                )",
                [],
            )
            .context("BSM: failed to ensure registry table exists")?;
            conn.execute(
                "INSERT OR REPLACE INTO code_registry (intent_hash, intent, artifact)
                 VALUES (?1, ?2, ?3)",
                rusqlite::params![payload.intent_hash, payload.intent, artifact_summary],
            )
            .context("BSM: failed to persist artifact to registry")?;

            payload.note(self.name(), "artifact locked into Eduba registry (future runs = cache hit)");

            if ctx.sovereign_online {
                let name = format!("hammurabi-{}", payload.intent_hash);
                match sovereign::create_ramgenie_project(&ctx.http, &ctx.sovereign_url, &name).await {
                    Ok(true) => payload.note(self.name(), format!("registered project '{name}' on gateway")),
                    Ok(false) => payload.note(self.name(), "gateway rejected project registration"),
                    Err(e) => payload.note(self.name(), format!("gateway registration failed ({e})")),
                }
            }
        }

        payload.finalized = true;
        let tx = payload.onchain_tx.clone().unwrap_or_else(|| "none".to_string());
        payload.result = format!(
            "Finalized | requirements={} artifacts={} security={} tx={}",
            payload.requirements.len(),
            payload.artifacts.len(),
            payload.security_verdict.clone().unwrap_or_else(|| "n/a".into()),
            tx,
        );
        payload.note(self.name(), "BSM lockdown complete");
        Ok(payload)
    }
}
