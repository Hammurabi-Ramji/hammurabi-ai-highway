//! Millennium Falcon — the MCP (Model Context Protocol) layer.
//! Assembles the runtime context manifest and the tool surface the downstream
//! agents are allowed to call.

use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;

use crate::pipeline::{Payload, PipelineContext, Stage};
use crate::sovereign;

pub struct FalconMcp;

#[async_trait]
impl Stage for FalconMcp {
    fn name(&self) -> &'static str {
        "Millennium Falcon"
    }
    fn role(&self) -> &'static str {
        "MCP context & protocol"
    }

    async fn process(&self, ctx: &PipelineContext, mut payload: Payload) -> Result<Payload> {
        // Pull the live agent hierarchy from the gateway when reachable.
        let hierarchy = if ctx.sovereign_online {
            match sovereign::agent_hierarchy(&ctx.http, &ctx.sovereign_url).await {
                Ok(h) => {
                    let kings = h.get("kings").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
                    let serfs = h.get("serfs").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
                    payload.note(self.name(), format!("live agent hierarchy: {kings} king(s), {serfs} serf(s)"));
                    h
                }
                Err(e) => {
                    payload.note(self.name(), format!("hierarchy fetch failed ({e})"));
                    json!(null)
                }
            }
        } else {
            json!(null)
        };

        let manifest = json!({
            "protocol": "MCP/1.0",
            "gateway": ctx.sovereign_url,
            "gateway_online": ctx.sovereign_online,
            "tools": [
                "ramgenie.generate",
                "agents.hierarchy",
                "millennium_falcon.audit",
                "digital_hands.execute",
            ],
            "agent_hierarchy": hierarchy,
            "credentials_loaded": ctx.config.is_some(),
            "requirements": payload.requirements,
        });

        let tool_count = manifest["tools"].as_array().map(|a| a.len()).unwrap_or(0);
        payload.note(
            self.name(),
            format!("context manifest assembled ({tool_count} tools exposed)"),
        );
        payload.context_manifest = manifest;
        Ok(payload)
    }
}
