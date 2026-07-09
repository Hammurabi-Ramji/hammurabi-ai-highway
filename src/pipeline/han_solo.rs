// src/pipeline/han_solo.rs — Updated error handling

use anyhow::Result;
use async_trait::async_trait;

use crate::pipeline::{Artifact, Payload, PipelineContext, Stage};
use crate::sovereign;

pub struct HanSolo;

#[async_trait]
impl Stage for HanSolo {
    fn name(&self) -> &'static str {
        "Han Solo"
    }
    fn role(&self) -> &'static str {
        "decision / autonomous build agent"
    }

    async fn process(&self, ctx: &PipelineContext, mut payload: Payload) -> Result<Payload> {
        if payload.cache_hit {
            payload.note(self.name(), "build skipped — Eduba supplied a cached artifact");
            return Ok(payload);
        }

        if ctx.sovereign_online {
            payload.note(self.name(), "dispatching to RamGenie codegen (/api/v1/ramgenie/generate/code)...");
            match sovereign::ramgenie_generate(&ctx.http, &ctx.sovereign_url, &payload.intent).await {
                Ok(resp) => {
                    let sections = [
                        ("backend", "src/backend"),
                        ("frontend", "ui/frontend"),
                        ("database", "db/schema"),
                        ("tests", "tests"),
                        ("documentation", "docs/README.md"),
                        ("docker", "Dockerfile"),
                    ];
                    let mut artifacts = Vec::new();
                    for (field, path) in sections {
                        if resp.get(field).and_then(|v| v.as_str()).is_some_and(|s| !s.is_empty()) {
                            artifacts.push(Artifact {
                                path: path.to_string(),
                                summary: format!("RamGenie-generated {field}"),
                                polished: false,
                            });
                        }
                    }
                    let tokens = resp.get("tokens_used").and_then(|v| v.as_i64()).unwrap_or(0);
                    payload.note(
                        self.name(),
                        format!("RamGenie returned {} section(s), {tokens} tokens", artifacts.len()),
                    );
                    payload.build_plan = vec!["RamGenie full-stack generation".to_string()];
                    payload.artifacts = artifacts;
                    return Ok(payload);
                }
                Err(e) => {
                    payload.note(self.name(), format!("RamGenie call failed ({e}) — falling back to local plan"));
                }
            }
        } else {
            payload.note(self.name(), "gateway offline — using local build plan");
        }

        let mut plan = Vec::new();
        let mut artifacts = Vec::new();

        for req in &payload.requirements {
            let (step, artifact) = match req.as_str() {
                "on-chain execution required" => (
                    "wire Venice -> 1Shot execution path",
                    Artifact {
                        path: "src/execution/onchain.rs".into(),
                        summary: "calldata build + gasless relay".into(),
                        polished: false,
                    },
                ),
                "frontend surface required" => (
                    "scaffold UI surface",
                    Artifact {
                        path: "ui/app.tsx".into(),
                        summary: "generated frontend surface".into(),
                        polished: false,
                    },
                ),
                "authentication required" => (
                    "add auth module",
                    Artifact {
                        path: "src/auth.rs".into(),
                        summary: "JWT authentication module".into(),
                        polished: false,
                    },
                ),
                _ => (
                    "scaffold generic module",
                    Artifact {
                        path: "src/module.rs".into(),
                        summary: "generic module".into(),
                        polished: false,
                    },
                ),
            };
            plan.push(step.to_string());
            artifacts.push(artifact);
        }

        payload.note(
            self.name(),
            format!(
                "computed {}-step build plan, generated {} artifact(s)",
                plan.len(),
                artifacts.len(),
            ),
        );
        payload.build_plan = plan;
        payload.artifacts = artifacts;
        Ok(payload)
    }
}
