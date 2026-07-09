// src/pipeline/mod.rs — Updated with default PipelineContext

//! Hammurabi AI Highway — autonomous orchestration pipeline.

use anyhow::{Context, Result};
use async_trait::async_trait;

pub mod bsm;
pub mod claude_polish;
pub mod digital_hands;
pub mod eduba;
pub mod falcon_mcp;
pub mod han_solo;
pub mod ram_gate;
pub mod ram_genie;

#[cfg(test)]
pub mod bsm_test;
#[cfg(test)]
pub mod digital_hands_test;
#[cfg(test)]
pub mod eduba_test;
#[cfg(test)]
pub mod han_solo_test;
#[cfg(test)]
pub mod ram_genie_test;

/// A unit of generated work passing through the pipeline.
#[derive(Debug, Clone)]
pub struct Artifact {
    pub path: String,
    pub summary: String,
    pub polished: bool,
}

/// The payload threaded through every stage.
#[derive(Debug, Default, Clone)]
pub struct Payload {
    pub intent: String,
    pub intent_hash: String,
    pub requirements: Vec<String>,
    pub context_manifest: serde_json::Value,
    pub cache_hit: bool,
    pub retrieved: Option<String>,
    pub build_plan: Vec<String>,
    pub artifacts: Vec<Artifact>,
    pub security_verdict: Option<String>,
    pub onchain_tx: Option<String>,
    pub finalized: bool,
    pub result: String,
    pub log: Vec<String>,
}

impl Payload {
    pub fn new(intent: &str) -> Self {
        Self {
            intent: intent.to_string(),
            ..Default::default()
        }
    }

    pub fn note(&mut self, stage: &str, msg: impl Into<String>) {
        let line = format!("[{stage}] {}", msg.into());
        println!("  {line}");
        self.log.push(line);
    }
}

/// Shared runtime context.
pub struct PipelineContext {
    pub config: Option<crate::config::Config>,
    pub db_path: String,
    pub http: reqwest::Client,
    pub sovereign_url: String,
    pub sovereign_online: bool,
}

impl Default for PipelineContext {
    fn default() -> Self {
        Self {
            config: None,
            db_path: "eduba_registry.db".to_string(),
            http: reqwest::Client::new(),
            sovereign_url: "http://127.0.0.1:3000".to_string(),
            sovereign_online: false,
        }
    }
}

#[async_trait]
pub trait Stage: Send + Sync {
    fn name(&self) -> &'static str;
    fn role(&self) -> &'static str;
    async fn process(&self, ctx: &PipelineContext, payload: Payload) -> Result<Payload>;
}

/// The ordered pipeline of stages.
pub fn stages() -> Vec<Box<dyn Stage>> {
    vec![
        Box::new(ram_genie::RamGenie),
        Box::new(falcon_mcp::FalconMcp),
        Box::new(eduba::Eduba),
        Box::new(han_solo::HanSolo),
        Box::new(ram_gate::RamGate),
        Box::new(claude_polish::ClaudePolish),
        Box::new(digital_hands::DigitalHands),
        Box::new(bsm::Bsm),
    ]
}

/// Run a user intent through the full pipeline.
pub async fn run(intent: &str) -> Result<Payload> {
    println!("=== Hammurabi AI Highway — Autonomous Pipeline ===");
    println!("Eduba System runtime online.");
    println!("Intent: \"{intent}\n");

    let config = crate::config::Config::from_env().ok();
    if config.is_none() {
        println!("(no .env credentials — Digital Hands will simulate on-chain execution)");
    }

    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .context("failed to build HTTP client")?;

    let sovereign_url = crate::sovereign::base_url();
    let sovereign_online = crate::sovereign::health(&http, &sovereign_url).await;
    println!(
        "Sovereign Stack gateway @ {sovereign_url}: {}\n",
        if sovereign_online {
            "ONLINE"
        } else {
            "offline (stages degrade gracefully)"
        }
    );

    let ctx = PipelineContext {
        config,
        db_path: std::env::var("EDUBA_DB_PATH").unwrap_or_else(|_| "eduba_registry.db".to_string()),
        http,
        sovereign_url,
        sovereign_online,
    };

    let pipeline = stages();
    let total = pipeline.len();
    let mut payload = Payload::new(intent);

    for (i, stage) in pipeline.iter().enumerate() {
        println!(
            "-- Lane {}/{}: {} ({}) --",
            i + 1,
            total,
            stage.name(),
            stage.role()
        );

        payload = stage.process(&ctx, payload).await.map_err(|e| {
            eprintln!("[ERROR] Stage {} failed: {}", stage.name(), e);
            e
        })?;

        println!();
    }

    println!("=== Pipeline complete ===");
    println!("{}", payload.result);
    Ok(payload)
}
