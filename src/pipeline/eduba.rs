//! Eduba — system core & code repository manager.
//! Performs the state check against a real local SQLite registry before any
//! compute is spent on generation: cache HIT retrieves existing modules, cache
//! MISS delegates to the Han Solo build agent.

use anyhow::{Context, Result};
use async_trait::async_trait;
use rusqlite::Connection;

use crate::pipeline::{Payload, PipelineContext, Stage};
use crate::sovereign;

pub struct Eduba;

#[async_trait]
impl Stage for Eduba {
    fn name(&self) -> &'static str {
        "Eduba"
    }
    fn role(&self) -> &'static str {
        "state check / SQLite registry"
    }

    async fn process(&self, ctx: &PipelineContext, mut payload: Payload) -> Result<Payload> {
        let conn = Connection::open(&ctx.db_path)
            .with_context(|| format!("Eduba: failed to open registry at {}", ctx.db_path))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS code_registry (
                intent_hash TEXT PRIMARY KEY,
                intent      TEXT NOT NULL,
                artifact    TEXT NOT NULL,
                created_at  TEXT NOT NULL DEFAULT (datetime('now'))
            )",
            [],
        )
        .context("Eduba: failed to ensure code_registry table")?;

        let hit: Option<String> = conn
            .query_row(
                "SELECT artifact FROM code_registry WHERE intent_hash = ?1",
                [&payload.intent_hash],
                |row| row.get(0),
            )
            .ok();

        match hit {
            Some(artifact) => {
                payload.cache_hit = true;
                payload.note(
                    self.name(),
                    format!("cache HIT for {} — retrieving existing modules", payload.intent_hash),
                );
                payload.retrieved = Some(artifact);
            }
            None => {
                payload.note(
                    self.name(),
                    format!(
                        "cache MISS for {} — delegating to Han Solo build agent",
                        payload.intent_hash
                    ),
                );
            }
        }

        // Cross-reference the gateway's Millennium Falcon project registry.
        if ctx.sovereign_online {
            if let Ok(projects) = sovereign::list_projects(&ctx.http, &ctx.sovereign_url).await {
                let count = projects.as_array().map(|a| a.len()).unwrap_or(0);
                payload.note(self.name(), format!("gateway reports {count} indexed project(s)"));
            }
        }
        Ok(payload)
    }
}
