# Hammurabi AI Highway — Hackathon Submission

> A sovereign, Rust-native CLI that turns plain-English intent into gas-free
> on-chain execution on Base — and orchestrates an 8-stage autonomous pipeline
> across a live local AI gateway.

## What it does

You type an intent in natural language. The system:

1. Signs an **x402** auth payload with a local EVM wallet (sovereign, key-based auth).
2. Asks **Venice AI** to translate the intent into raw EVM calldata.
3. Relays that calldata through the **1Shot API** to a **MetaMask Smart Account**
   on **Base**, executing it **gas-free** (sponsored relay).
4. Returns the transaction hash to the terminal.

```sh
cargo run -- run --intent "Swap 10 USDC for VVV"
```

There is also a full **8-stage orchestration pipeline** that wires the CLI into a
live local AI gateway (the Sovereign Stack server):

```sh
cargo run -- pipeline --intent "Build a task dashboard with auth"
```

## Sponsor integrations

| Sponsor | How it's used | Where |
|---|---|---|
| **Venice AI** | x402-authenticated NL→calldata via `qwen-3-7-max` | [`src/venice.rs`](src/venice.rs), [`src/x402.rs`](src/x402.rs) |
| **1Shot API** | gas-sponsored relay to a MetaMask Smart Account on Base | [`src/oneshot.rs`](src/oneshot.rs) |
| **MetaMask Smart Accounts / Base** | the account the relayer executes through (chainId 8453) | `src/oneshot.rs` |

## The pipeline (architecture showcase)

Each stage is a real Rust module wired to a real backend; offline stages degrade
gracefully and say so.

| Stage | Real backend | Verified |
|---|---|---|
| Ram Genie | local NL parse + SHA-256 fingerprint | ✅ |
| Millennium Falcon | live agent hierarchy (`GET /api/agents/hierarchy`) | ✅ 4 kings / 3 serfs |
| Eduba | local SQLite registry + `GET /api/projects` | ✅ cache hit/miss |
| Han Solo | RamGenie codegen (`POST /api/v1/ramgenie/generate/code`) | ✅ reached (needs Ollama model) |
| Ram Gate | gateway health + egress secret-scan | ✅ |
| Claude | audit metrics (`GET /api/analyze/metrics/summary`) | ✅ |
| Digital Hands | Venice x402 + 1Shot relay | ✅ (real on `run`) |
| BSM | SQLite lockdown + `POST /api/v1/ramgenie/projects` | ✅ 201 registered |

## Tech stack

Rust · tokio · clap · reqwest · ethers (x402 signing) · rusqlite · serde.
Gateway: Axum + SQLx + SQLite. Network: Base (EVM, chainId 8453).

## How to run

```sh
cp .env.example .env      # fill VENICE_API_KEY, HAMMURABI_PRIVATE_KEY,
                          # ONESHOT_API_KEY, ONESHOT_WALLET_ID
cargo build --release

# On-chain proof-of-life (real Venice + 1Shot; moves funds if the Smart Account is funded):
cargo run -- run --intent "Swap 10 USDC for VVV"

# Full architecture pipeline (start the gateway first; no funds moved):
cargo run -- pipeline --intent "Build a task dashboard with auth"
```

`HAMMURABI_PRIVATE_KEY` must be a 64-hex-char (32-byte) private key — it only
signs the x402 auth header and never broadcasts transactions itself; 1Shot's
smart-account custody performs execution.

## Honest scope notes

- **Real, verified:** x402 signing, Venice call, 1Shot relay wiring, the live
  gateway calls (agent hierarchy, audit metrics, project registration → 201),
  and the SQLite cache hit/miss.
- **Needs an external dep to go fully green:** Han Solo's RamGenie codegen
  requires an Ollama model pulled locally; without it, the stage degrades
  gracefully to a local plan (by design, not a crash).
- No fabricated capabilities — the docs describe exactly what the binary does.

## Repository

https://github.com/Hammurabi-Ramji/hammurabi-ai-highway
