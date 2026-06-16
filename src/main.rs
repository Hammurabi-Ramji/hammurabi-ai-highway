// Hammurabi AI Highway CLI
// Sovereign, decentralized automation: natural language -> Venice AI calldata
// -> 1Shot gas-sponsored execution on Base. No private keys ever leave this
// process; no gas is ever paid by the user.

mod config;
mod oneshot;
mod pipeline;
mod sovereign;
mod venice;
mod x402;

use anyhow::Result;
use clap::{Parser, Subcommand};
use config::Config;
use std::io::Write;

#[derive(Parser)]
#[command(
    name = "hammurabi",
    version,
    about = "Hammurabi AI Highway CLI — natural language to gas-free on-chain execution"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Execute an on-chain intent via Venice AI + 1Shot API on Base
    Run {
        /// Natural language description of the desired on-chain action,
        /// e.g. "Swap 10 USDC for VVV"
        #[arg(long)]
        intent: String,
    },
    /// Run the full autonomous pipeline:
    /// Ram Genie -> Millennium Falcon -> Eduba -> Han Solo -> Ram Gate
    /// -> Claude -> Digital Hands -> BSM
    Pipeline {
        /// Natural language description of what to build/execute
        #[arg(long)]
        intent: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { intent } => run(&intent).await,
        Commands::Pipeline { intent } => pipeline::run(&intent).await.map(|_| ()),
    }
}

async fn run(intent: &str) -> Result<()> {
    println!("=== Hammurabi AI Highway ===");
    println!("Intent: \"{intent}\"\n");

    let config = Config::from_env()?;
    println!("Venice AI key:  {}", config::mask(&config.venice_api_key));
    println!("1Shot API key:  {}", config::mask(&config.oneshot_api_key));
    println!("1Shot wallet:   {}\n", config.oneshot_wallet_id);

    let client = reqwest::Client::new();

    step(&format!("Signing x402 auth and querying Venice AI ({})...", config.venice_model));
    let calldata = venice::get_calldata(&client, &config, intent).await?;
    done();
    println!("  to:    {}", calldata.to);
    println!("  data:  {}", truncate(&calldata.data, 24));
    println!("  value: {}\n", calldata.value);

    step("Routing calldata to 1Shot API (gas-sponsored execution on Base)...");
    let tx_hash = oneshot::execute(&client, &config, &calldata).await?;
    done();

    println!("\n=== Execution complete ===");
    println!("Transaction Hash: {tx_hash}");
    println!("BaseScan:         https://basescan.org/tx/{tx_hash}");

    Ok(())
}

fn step(msg: &str) {
    print!("{msg}");
    let _ = std::io::stdout().flush();
}

fn done() {
    println!(" done.");
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}
