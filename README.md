# Hammurabi AI Highway CLI

A sovereign, decentralized automation tool built in Rust that abstracts away
Web3 complexity entirely. Users execute complex on-chain actions — swaps,
buys, staking — using plain natural language, with no private key exposure
and no gas fees.

## Architecture

- **Language:** Rust (`tokio`, `clap`, `reqwest`, `ethers-core`/`ethers-signers`, `serde_json`, `dotenvy`)
- **Network:** Base (EVM, chainId 8453)
- **The Brain — Venice AI:** translates natural-language intent into raw
  smart-contract calldata using the `qwen-3-7-max` model, authenticated
  via **x402** (a timestamped payload signed with our local EVM wallet,
  passed as `X-402-*` headers).
- **The Hands — 1Shot API:** takes the calldata from Venice and executes it
  through our MetaMask Smart Account on Base as a **gas-sponsored**
  transaction — the end user never pays gas or signs anything.

## Business logic

When a customer pays to use the Hammurabi platform in USDC, this CLI:

1. Swaps the inbound USDC into the platform's native token (**VVV**) via
   Venice + 1Shot.
2. Stakes that VVV to secure daily compute bandwidth (**DIEM**).

All of this happens via simple CLI commands, with the smart account and
1Shot relayer handling custody and gas.

## Execution flow

```
cargo run -- run --intent "Swap 10 USDC for VVV"
```

1. The CLI signs an x402 auth payload with `HAMMURABI_PRIVATE_KEY`.
2. It calls Venice AI, requesting only raw `{to, data, value}` calldata for
   the described Base-network action.
3. It forwards that calldata to the 1Shot API execute endpoint.
4. 1Shot routes the transaction through the smart account, pays the gas, and
   returns the transaction hash, which is printed to the terminal along with
   a BaseScan link.

## Autonomous Pipeline (the AI Highway)

Beyond the single-shot `run` command, `hammurabi pipeline` drives an intent
through a full sequence of agentic stages — the "AI Highway" orchestration.
The runtime that drives them is the **Yuduva System**. Two naming vocabularies
map onto the same eight stages:

Every stage is wired to a **real backend** — the [Sovereign Stack server](../Sovereign%20Stack/output/sovereign-stack)
(the RAMGate gateway, default `http://127.0.0.1:3000`) which hosts Millennium
Falcon, the Han Solo / AI-Highway agents, and RamGenie — plus a local SQLite
registry and the Venice+1Shot on-chain relayer. When the gateway is offline,
each stage degrades gracefully and says so, so the pipeline always completes.

| Stage              | Real backend call                                              | What it does |
|--------------------|----------------------------------------------------------------|--------------|
| `ram_genie`        | local NL parse + SHA-256 fingerprint                           | ingress / intent → requirements |
| `falcon_mcp`       | `GET /api/agents/hierarchy`                                    | live agent hierarchy into MCP context |
| `eduba`            | local SQLite registry + `GET /api/projects`                   | cache HIT/MISS + remote project state |
| `han_solo`         | `POST /api/v1/ramgenie/generate/code`                         | **real RamGenie codegen** → artifacts |
| `ram_gate`         | gateway health + egress secret-scan                           | security boundary |
| `claude_polish`    | `GET /api/analyze/metrics/summary`                            | Millennium Falcon audit metrics |
| `digital_hands`    | Venice AI `x402` + 1Shot relay                                | gasless on-chain execution |
| `bsm`              | local SQLite lockdown + `POST /api/v1/ramgenie/projects`     | finalize + register project |

Flow: **Ram Genie** parses intent → **Millennium Falcon** pulls the live agent
hierarchy → **Eduba** checks its SQLite registry + the gateway's project list →
**Han Solo** calls RamGenie codegen → **Ram Gate** authorizes → **Claude**
attaches audit metrics → **Digital Hands** relays on-chain → **BSM** locks the
artifact in and registers the project.

```sh
# 1. start the real gateway (separate terminal), then:
cargo run -- pipeline --intent "Build a task dashboard with auth"
# Set SOVEREIGN_API_URL to point at a non-default gateway address.
```

> The gateway codegen (RamGenie) requires Ollama running at `localhost:11434`.
> Without the gateway, the pipeline still runs end-to-end using local fallbacks
> (Han Solo's deterministic plan, the SQLite registry, the simulated relay) and
> labels every degraded stage explicitly.

## Technical Integration Details

The on-chain layer uses a hardened integration for autonomous execution. The
fields below document **exactly what the binary sends** (see
[x402.rs](src/x402.rs) and [oneshot.rs](src/oneshot.rs)).

### 1. Authentication Layer (x402 protocol)

Venice AI requests are authenticated with a timestamped, wallet-signed payload.
The signature is bound to the request body and the current timestamp (the local
EVM wallet signs `"{timestamp}.{body}"`), and is sent as **three** headers:

- `X-402-Address` — checksummed address of the local sovereign wallet
- `X-402-Timestamp` — unix timestamp the signature is bound to
- `X-402-Signature` — `0x`-prefixed ECDSA (EIP-191 personal_sign) signature

`POST https://api.venice.ai/api/v1/chat/completions` · model `qwen-3-7-max`.

### 2. 1Shot relayer payload

`digital_hands.rs` calls `POST https://api.1shotapi.com/v1/execute` (Bearer auth)
with this exact JSON body:

- `to` — target smart-contract address (from Venice calldata)
- `data` — hexadecimal calldata (dynamically generated via Venice AI)
- `value` — transaction value (default `0x0` for relay operations)
- `chainId` — `8453` (Base Mainnet)
- `walletId` — identifier for the delegated MetaMask Smart Account

The response's `transactionHash` / `txHash` / `hash` is surfaced to the terminal.

## Setup

```sh
cp .env.example .env
# fill in VENICE_API_KEY, HAMMURABI_PRIVATE_KEY, ONESHOT_API_KEY, ONESHOT_WALLET_ID
cargo build
cargo run -- run --intent "Swap 10 USDC for VVV"
```

## Security & Integrity Layer

The Sovereign Stack employs a tiered security posture:

- **Artifact egress scanning** — the `Ram Gate` stage scans generated artifacts
  at stage 5 for secret-like material before the pipeline proceeds to execution.
- **Execution integrity** — all Venice AI compute requests carry the
  cryptographic `X-402-*` headers above, ensuring authenticated execution.
- **Transactional trust** — 1Shot's gasless relay routes calldata through a
  verified MetaMask Smart Account, keeping on-chain boundaries intact.
- **Secret hygiene** — all secrets are read from `.env` (gitignored) via
  `dotenvy`, never hardcoded, and masked to their last 4 characters in logs.
  `HAMMURABI_PRIVATE_KEY` only signs x402 auth payloads — it never signs or
  broadcasts on-chain transactions; 1Shot's smart-account custody does that.
