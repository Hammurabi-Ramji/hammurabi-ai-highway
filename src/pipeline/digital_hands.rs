// src/pipeline/digital_hands.rs — Updated error handling

use anyhow::Result;
use async_trait::async_trait;

use crate::oneshot;
use crate::pipeline::{Payload, PipelineContext, Stage};
use crate::venice;

pub struct DigitalHands;

#[async_trait]
impl Stage for DigitalHands {
    fn name(&self) -> &'static str {
        "Digital Hands"
    }
    fn role(&self) -> &'static str {
        "execution / gasless relayer"
    }

    async fn process(&self, ctx: &PipelineContext, mut payload: Payload) -> Result<Payload> {
        let needs_onchain = payload
            .requirements
            .iter()
            .any(|r| r == "on-chain execution required");

        if !needs_onchain {
            payload.note(
                self.name(),
                "no on-chain action in intent — relayer skipped",
            );
            return Ok(payload);
        }

        match &ctx.config {
            Some(config) => {
                payload.note(
                    self.name(),
                    "signing x402 + requesting calldata from Venice AI...",
                );
                let calldata = venice::get_calldata(&ctx.http, config, &payload.intent).await?;
                payload.note(self.name(), format!("calldata target {}", calldata.to));
                let tx = oneshot::execute(&ctx.http, config, &calldata).await?;
                payload.note(self.name(), format!("1Shot relayed tx {tx}"));
                payload.onchain_tx = Some(tx);
            }
            None => {
                payload.note(
                    self.name(),
                    "SIMULATED relay (no credentials) — populate .env for live execution",
                );
                payload.onchain_tx = Some("0xSIMULATED".to_string());
            }
        }
        Ok(payload)
    }
}
