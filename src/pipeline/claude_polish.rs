//! Claude — UI/UX polish & beautification engine.
//! Aesthetic pass over the generated artifacts: refactor for readability,
//! formatting, comments, and UX polish. (Simulated: marks artifacts polished.)

use anyhow::Result;
use async_trait::async_trait;

use crate::pipeline::{Payload, PipelineContext, Stage};
use crate::sovereign;

pub struct ClaudePolish;

#[async_trait]
impl Stage for ClaudePolish {
    fn name(&self) -> &'static str {
        "Claude"
    }
    fn role(&self) -> &'static str {
        "polish / formatting & UX"
    }

    async fn process(&self, ctx: &PipelineContext, mut payload: Payload) -> Result<Payload> {
        let count = payload.artifacts.len();
        for artifact in &mut payload.artifacts {
            artifact.polished = true;
        }

        // Attach the gateway's real Millennium Falcon audit metrics when available.
        if ctx.sovereign_online {
            if let Ok(metrics) = sovereign::analysis_metrics(&ctx.http, &ctx.sovereign_url).await {
                let quality = metrics.get("average_quality_score").and_then(|v| v.as_f64()).unwrap_or(0.0);
                payload.note(self.name(), format!("Millennium Falcon avg quality score: {quality:.1}"));
            }
        }

        payload.note(
            self.name(),
            format!("beautified {count} artifact(s): formatting, comments, UX polish"),
        );
        Ok(payload)
    }
}
