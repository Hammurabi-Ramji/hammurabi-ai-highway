//! Ram Genie — the Language Processor (LP). Pipeline ingress.
//! Interprets the user's natural-language intent and translates it into coarse
//! systemic requirements for the rest of the pipeline.

use anyhow::Result;
use async_trait::async_trait;
use sha2::{Digest, Sha256};

use crate::pipeline::{Payload, PipelineContext, Stage};

pub struct RamGenie;

#[async_trait]
impl Stage for RamGenie {
    fn name(&self) -> &'static str {
        "Ram Genie"
    }
    fn role(&self) -> &'static str {
        "ingress / language processor"
    }

    async fn process(&self, _ctx: &PipelineContext, mut payload: Payload) -> Result<Payload> {
        let intent = payload.intent.clone();

        // Stable fingerprint of the intent — used by Eduba for cache lookups.
        let digest = hex::encode(Sha256::digest(intent.as_bytes()));
        payload.intent_hash = digest[..16].to_string();
        payload.note(self.name(), format!("intent fingerprint → {}", payload.intent_hash));

        // Translate natural language into systemic requirements (simulated LP).
        let lower = intent.to_lowercase();
        let mut reqs = Vec::new();
        if lower.contains("swap")
            || lower.contains("buy")
            || lower.contains("stake")
            || lower.contains("token")
        {
            reqs.push("on-chain execution required".to_string());
        }
        if lower.contains("dashboard") || lower.contains("ui") || lower.contains("app") {
            reqs.push("frontend surface required".to_string());
        }
        if lower.contains("auth") || lower.contains("login") {
            reqs.push("authentication required".to_string());
        }
        if reqs.is_empty() {
            reqs.push("generic build request".to_string());
        }

        payload.note(
            self.name(),
            format!("derived {} requirement(s): {}", reqs.len(), reqs.join(", ")),
        );
        payload.requirements = reqs;
        Ok(payload)
    }
}
