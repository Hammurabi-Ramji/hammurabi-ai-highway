// src/pipeline/eduba.rs — Eduba cache stage with updated error handling

use anyhow::Result;
use async_trait::async_trait;

use crate::pipeline::{Payload, PipelineContext, Stage};

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
        payload.note(self.name(), format!("intent hash: {}", payload.intent_hash));

        // Check cache in Eduba registry. The registry is a binary SQLite file,
        // so probe for existence with a filesystem check (a UTF-8 read would
        // always fail on binary content) and treat any open/query error — such
        // as a missing table on a brand-new database — as a cache miss.
        let result = if std::path::Path::new(&ctx.db_path).exists() {
            match rusqlite::Connection::open(&ctx.db_path) {
                Ok(conn) => conn
                    .query_row(
                        "SELECT artifact FROM code_registry WHERE intent_hash = ?1",
                        [&payload.intent_hash],
                        |row| row.get::<_, String>(0),
                    )
                    .ok(),
                Err(_) => None,
            }
        } else {
            None
        };

        match result {
            Some(artifact) => {
                payload.cache_hit = true;
                payload.retrieved = Some(artifact.clone());
                payload.note(self.name(), format!("cache HIT — artifact: {}", artifact));
            }
            None => {
                payload.cache_hit = false;
                payload.note(self.name(), "cache MISS — proceeding to build");
            }
        }

        Ok(payload)
    }
}
