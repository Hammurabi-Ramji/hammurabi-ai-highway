# Architecture Guide

## Overview

This document describes the complete architecture of the Hammurabi AI Highway CLI.

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     Hammurabi AI Highway CLI                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐ │
│  │   Ram Genie  │───▶│ Millennium    │───▶│    Eduba         │ │
│  │   (LP)       │    │ Falcon (MCP)  │    │   (Cache)        │ │
│  └──────────────┘    └──────────────┘    └──────────────────┘ │
│                                             │                   │
│                                             ▼                   │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐ │
│  │     BSM      │◀───│   Claude     │◀───│    Ram Gate      │ │
│  │   (Lockdown) │    │   (Polish)   │    │   (Security)     │ │
│  └──────────────┘    └──────────────┘    └──────────────────┘ │
│                                       ▲                        │
│  ┌──────────────┐    ┌──────────────┐ │                        │
│  │  Digital     │◀───│    Han Solo  │─┘                        │
│  │  Hands       │    │   (Build)    │                          │
│  │   (Relay)    │    └──────────────┘                          │
│  └──────────────┘                                              │
│                                                                  │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                    ┌───────┴───────┐
                    │   Venice AI   │   ← Natural Language → Calldata
                    │ (qwen-3-7-max)│
                    └───────┬───────┘
                            │
                    ┌───────┴───────┐
                    │    1Shot      │   ← Gasless Relay (Smart Account)
                    │      API      │
                    └───────┬───────┘
                            │
                    ┌───────┴───────┐
                    │  Base Chain   │   ← EVM Execution
                    │  (8453)       │
                    └───────────────┘
```

## Module Structure

### Core Modules

- `main.rs` - CLI entry point
- `config.rs` - Environment configuration
- `venice.rs` - Venice AI integration
- `oneshot.rs` - 1Shot API integration
- `x402.rs` - x402 authentication protocol
- `sovereign.rs` - Sovereign Stack gateway client

### Pipeline Modules

- `pipeline/mod.rs` - Pipeline orchestration
- `pipeline/ram_genie.rs` - Language processor
- `pipeline/falcon_mcp.rs` - MCP context assembly
- `pipeline/eduba.rs` - Cache layer
- `pipeline/han_solo.rs` - Build agent
- `pipeline/ram_gate.rs` - Security gateway
- `pipeline/claude_polish.rs` - Polish stage
- `pipeline/digital_hands.rs` - Execution relay
- `pipeline/bsm.rs` - Final lockdown

### Security Modules

- `crypto/key_manager.rs` - Key management
- `validation/calldata_verifier.rs` - Calldata validation
- `security/x402_v2.rs` - Enhanced x402 auth
- `error.rs` - Error taxonomy

### Performance Modules

- `metrics/performance.rs` - Performance monitoring (TODO)

## Data Flow

1. User provides natural language intent via CLI
2. Ram Genie parses intent → requirements + fingerprint
3. Falcon MCP assembles context + tool manifest
4. Eduba checks cache (HIT → return artifact, MISS → proceed)
5. Han Solo generates artifacts (or retrieves from cache)
6. Ram Gate scans for secrets
7. Claude polishes artifacts
8. Digital Hands executes on-chain (Venice AI → 1Shot)
9. BSM persists to Eduba + registers project

## Error Handling

All errors use the `PipelineError` enum with:
- Machine-readable error codes
- Retryability flags
- Recovery actions

Example:
```rust
match stage.process(&ctx, payload).await {
    Ok(payload) => payload,
    Err(PipelineError::VeniceAIUnavailable) => {
        // Use fallback
        handle_fallback()
    }
    Err(e) => return Err(e),
}
```

## Security Model

### Private Key Management
- Keys encrypted at rest using AES-256-GCM
- Derived from master password using Argon2id
- Stored in OS secure storage (Windows DPAPI, macOS Keychain, or Linux Secret Service)

### x402 Authentication
- Timestamped ECDSA signatures
- Rate limiting per wallet address
- Replay protection

### Calldata Validation
- Contract address whitelist
- ABI signature verification
- Value bounds checking
- Risk scoring (0-100)

## Performance Characteristics

### Target Latencies

| Stage | Target p99 |
|-------|------------|
| Ram Genie | <2ms |
| Falcon MCP | <50ms |
| Eduba | <1ms |
| Han Solo | 500-2000ms |
| Ram Gate | <2ms |
| Claude Polish | <2ms |
| Digital Hands | 1000-2000ms |
| BSM | <2ms |

### Memory Usage
- Target: <10MB heap
- No memory leaks (<1% growth per hour)

## Deployment

### Docker
```bash
docker build -t hammurabi-cli .
docker run -v $(pwd)/.env:/app/.env hammurabi-cli
```

### K8s
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: hammurabi-config
data:
  VENICE_API_KEY: "..."
  ONESHOT_API_KEY: "..."
```

### Environment Variables
- `VENICE_API_KEY` - Venice AI API key
- `VENICE_MODEL` - Model to use (default: qwen-3-7-max)
- `HAMMURABI_PRIVATE_KEY` - Private key for x402
- `ONESHOT_API_KEY` - 1Shot API key
- `ONESHOT_WALLET_ID` - 1Shot wallet ID
- `SOVEREIGN_API_URL` - Sovereign Stack gateway URL
- `EDUBA_DB_PATH` - SQLite database path

## Monitoring

### Metrics to Collect
- Pipeline stage timing
- Cache hit rate
- Validation pass rate
- Error rates by type
- Memory usage

### Logging
- Structured JSON logs
- Audit trail for all security events
- Request tracing with correlation IDs

## Extending the Pipeline

### Adding a New Stage

1. Implement `Stage` trait in `pipeline/mod.rs`
2. Add stage to `stages()` vector
3. Implement preconditions and guarantees

```rust
pub struct MyNewStage;

#[async_trait]
impl Stage for MyNewStage {
    fn name(&self) -> &'static str { "MyNewStage" }
    fn role(&self) -> &'static str { "my stage role" }
    
    async fn process(&self, ctx: &PipelineContext, payload: Payload) -> Result<Payload> {
        // Implementation
        Ok(payload)
    }
}
```

### Adding a New External Service

1. Implement client trait in `interfaces/` module
2. Inject client via dependency injection
3. Add client to `PipelineContext`

## Future Roadmap

- [ ] Multi-chain support (Arbitrum, Optimism, Polygon)
- [ ] Mobile app (iOS/Android)
- [ ] Web UI
- [ ] Advanced caching with TTL
- [ ] Multi-key signing
- [ ] Real Redis integration
- [ ] Prometheus metrics export
