//! Hammurabi AI Highway — autonomous orchestration pipeline.
//!
//! This module is the "Eduba System" runtime: the core platform that drives a
//! user intent through a fixed sequence of agentic stages. Each stage is a
//! small, self-contained module implementing the [`Stage`] trait. The two
//! design vocabularies the project uses map onto these stages as follows:
//!
//! | Stage module       | Star-Wars name      | Architecture-doc name   | Role                          |
//! |--------------------|---------------------|-------------------------|-------------------------------|
//! | `ram_genie`        | Ram Genie           | Ram Genie (LP)          | ingress / NL intent parsing   |
//! | `falcon_mcp`       | Millennium Falcon   | (MCP layer)             | context & protocol assembly   |
//! | `eduba`            | (—)                 | Eduba                   | state check / SQLite registry |
//! | `han_solo`         | Han Solo            | Greta                   | decision / build agent        |
//! | `ram_gate`         | Ram Gate            | (—)                     | security gateway              |
//! | `claude_polish`    | (—)                 | Claude                  | UI/UX polish                  |
//! | `digital_hands`    | Digital Hands       | (—)                     | execution relayer (real)      |
//! | `bsm`              | (—)                 | BSM                     | finalize / lockdown           |
//!
//! Stages that have a real backend (Eduba's SQLite registry, Digital Hands'
//! Venice+1Shot relay) do real work; the decision/codegen stages run honest
//! deterministic simulations so the pipeline executes end-to-end without
//! pretending to be a finished backend.

use anyhow::Result;
use async_trait::async_trait;

use crate::config::Config;

pub mod bsm;
pub mod claude_polish;
pub mod digital_hands;
pub mod eduba;
pub mod falcon_mcp;
pub mod han_solo;
pub mod ram_gate;
pub mod ram_genie;

/// A unit of generated work passing through the pipeline.
#[derive(Debug, Clone)]
pub struct Artifact {
    pub path: String,
    pub summary: String,
    pub polished: bool,
}

/// The payload threaded through every stage. Each stage reads what it needs
/// and appends its own contribution.
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

    /// Record (and print) a line of progress attributed to a stage.
    pub fn note(&mut self, stage: &str, msg: impl Into<String>) {
        let line = format!("[{stage}] {}", msg.into());
        println!("  {line}");
        self.log.push(line);
    }
}

/// Shared runtime context ("Eduba System" environment) handed to every stage.
pub struct PipelineContext {
    /// Loaded credentials, or `None` when running in simulation mode.
    pub config: Option<Config>,
    /// Path to Eduba's local SQLite registry.
    pub db_path: String,
    /// Shared HTTP client for all outbound calls.
    pub http: reqwest::Client,
    /// Base URL of the Sovereign Stack gateway (RAMGate).
    pub sovereign_url: String,
    /// Whether the Sovereign Stack gateway answered its health check at startup.
    pub sovereign_online: bool,
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
    println!("Intent: \"{intent}\"\n");

    let config = Config::from_env().ok();
    if config.is_none() {
        println!("(no .env credentials — Digital Hands will simulate on-chain execution)");
    }

    // A bounded timeout so a slow/hanging gateway (e.g. Ollama loading a model)
    // degrades a stage gracefully instead of blocking the whole pipeline.
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .unwrap_or_default();
    let sovereign_url = crate::sovereign::base_url();
    let sovereign_online = crate::sovereign::health(&http, &sovereign_url).await;
    println!(
        "Sovereign Stack gateway @ {sovereign_url}: {}\n",
        if sovereign_online { "ONLINE" } else { "offline (stages degrade gracefully)" }
    );

    let ctx = PipelineContext {
        config,
        db_path: std::env::var("EDUBA_DB_PATH")
            .unwrap_or_else(|_| "eduba_registry.db".to_string()),
        http,
        sovereign_url,
        sovereign_online,
    };

    let pipeline = stages();
    let total = pipeline.len();
    let mut payload = Payload::new(intent);

    for (i, stage) in pipeline.iter().enumerate() {
        println!("-- Lane {}/{}: {} ({}) --", i + 1, total, stage.name(), stage.role());
        payload = stage.process(&ctx, payload).await?;
        println!();
    }

    println!("=== Pipeline complete ===");
    println!("{}", payload.result);
    Ok(payload)
}
