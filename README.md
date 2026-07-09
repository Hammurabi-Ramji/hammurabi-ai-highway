# Hammurabi AI Highway — v0.1.0

A sovereign, Rust-native CLI that turns plain-English intent into gas-free
on-chain execution on Base, and orchestrates an 8-stage autonomous pipeline
across a local AI gateway.

> **Status: early-stage / hackathon submission.** The crate builds cleanly and
> the full test suite passes, but this is not production-hardened software.
> Some modules (see [Project status](#-project-status)) are scaffolding that is
> not yet wired into the live path. See [SUBMISSION.md](SUBMISSION.md) for the
> hackathon write-up.

---

## 🚀 What it does

You type an intent in natural language. The system:

1. Signs an **x402** auth payload with a local EVM wallet (sovereign, key-based auth).
2. Asks **Venice AI** to translate the intent into raw EVM calldata.
3. Relays that calldata through the **1Shot API** to a **MetaMask Smart Account**
   on **Base**, executing it **gas-free** (sponsored relay).
4. Returns the transaction hash to the terminal.

There is also an 8-stage orchestration pipeline that wires the CLI into a local
AI gateway (the Sovereign Stack server). Offline stages degrade gracefully.

---

## 🛠 Installation & build

```bash
git clone <your-repo-url>
cd HammurabiAIHighway

cp .env.example .env        # fill in credentials (see below)

cargo build                 # development
cargo build --release       # production build
```

### Required environment (`.env`)

| Variable | Purpose |
|----------|---------|
| `VENICE_API_KEY` | Venice AI API key |
| `HAMMURABI_PRIVATE_KEY` | local EVM wallet key used for x402 signing |
| `ONESHOT_API_KEY`, `ONESHOT_WALLET_ID` | 1Shot relay credentials |

Without credentials the CLI still runs the pipeline; Digital Hands simulates the
on-chain step and says so.

---

## ▶️ Run

```bash
# Single on-chain intent
cargo run -- run --intent "Swap 10 USDC for VVV"

# Full 8-stage orchestration pipeline
cargo run -- pipeline --intent "Build a task dashboard with auth"
```

---

## 🧩 The pipeline

Each stage is a real Rust module (`src/pipeline/`) wired to a real backend;
offline stages degrade gracefully.

| Stage | Role |
|-------|------|
| Ram Genie | NL parse + SHA-256 intent fingerprint |
| Millennium Falcon | live agent hierarchy from the gateway |
| Eduba | SQLite registry cache (hit/miss) |
| Han Solo | codegen (gateway RamGenie, or local build plan offline) |
| Ram Gate | gateway health + egress secret-scan |
| Claude | audit metrics |
| Digital Hands | Venice x402 + 1Shot relay (real on `run`) |
| BSM | SQLite lockdown + gateway project registration |

---

## 🧪 Testing

```bash
cargo test                              # everything
cargo test --lib                        # unit tests
cargo test --test integration_test      # full-pipeline integration
cargo test --test validation_test       # calldata verifier
cargo test --test error_test            # error taxonomy
```

**Current state:** 52 tests across unit, integration, and validation suites —
all passing. Coverage is meaningful for the pipeline stages, the SQLite cache,
the calldata verifier, and the error taxonomy; it is **not** yet comprehensive.

---

## 📚 Documentation

~1,400 lines of guides live in [`docs/`](docs/):

- [Architecture](docs/01_architecture.md)
- [Security guide](docs/02_security_guide.md)
- [Deployment](docs/03_deployment.md)
- [Troubleshooting](docs/04_troubleshooting.md)
- [Performance](docs/05_performance.md)

> Note: the performance guide and the `benches/` benchmarks are **illustrative**.
> The current benchmark harness simulates stage timings with fixed sleeps rather
> than measuring the real implementations — treat the numbers as targets, not
> measurements.

---

## 📊 Project status

Honest snapshot of what is wired in versus aspirational:

| Area | Status |
|------|--------|
| CLI + 8-stage pipeline | ✅ working, tested |
| Venice AI → 1Shot relay on Base | ✅ real on `run` (needs credentials) |
| x402 request signing | ✅ used on the live Venice path (`security::x402_v2`) |
| SQLite intent cache (Eduba/BSM) | ✅ working, tested |
| Calldata verifier (`src/validation`) | ⚠️ standalone + tested, **not yet wired** into the on-chain path |
| Performance metrics (`src/metrics`) | ⚠️ implemented + tested, **not yet wired** |
| Typed error taxonomy (`src/error`) | ⚠️ present; live pipeline currently uses `anyhow` |
| Encrypted key management (`src/crypto`) | ⛔ **deferred** — does not build against pinned crates; commented out |
| Benchmarks (`benches/`) | ⚠️ simulated timings, not real measurements |

---

## 🗺 Roadmap

- Wire the calldata verifier into the Digital Hands relay path.
- Rewrite and re-enable the `crypto` key-management module against its actual
  crate APIs, then integrate encrypted key storage.
- Consolidate the two x402 signers (`x402` and `security::x402_v2`).
- Replace simulated benchmarks with real measurements.
- Redis-based replay protection; multi-chain support.

---

## 📄 License / attribution

See [SUBMISSION.md](SUBMISSION.md) for the hackathon submission details.

*Champions Have No Master.*
