//! Ram Gate — secure network routing & gateway.
//! The boundary every payload must clear before execution/egress. Implements a
//! Highway-Police-style check: no secret-like material may cross the gate.

use anyhow::{bail, Result};
use async_trait::async_trait;

use crate::pipeline::{Payload, PipelineContext, Stage};

pub struct RamGate;

#[async_trait]
impl Stage for RamGate {
    fn name(&self) -> &'static str {
        "Ram Gate"
    }
    fn role(&self) -> &'static str {
        "security / gateway"
    }

    async fn process(&self, ctx: &PipelineContext, mut payload: Payload) -> Result<Payload> {
        // Refuse to let anything resembling a hardcoded secret leave the boundary.
        let leak = payload.artifacts.iter().any(|a| {
            a.summary.contains("sk-") || (a.summary.contains("0x") && a.summary.len() > 60)
        });

        if leak {
            payload.security_verdict = Some("BLOCKED: secret-like material in artifact".into());
            payload.note(self.name(), "egress BLOCKED — secret-like material detected");
            bail!("Ram Gate blocked egress: secret-like material in artifacts");
        }

        let gateway = if ctx.sovereign_online { "RAMGate online" } else { "RAMGate offline" };
        payload.security_verdict = Some("authorized".into());
        payload.note(self.name(), format!("boundary authorized ({gateway}) — payload cleared for execution"));
        Ok(payload)
    }
}
