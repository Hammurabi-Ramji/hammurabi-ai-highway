// Hammurabi AI Highway CLI
// Sovereign, decentralized automation: natural language -> Venice AI calldata
// -> 1Shot gas-sponsored execution on Base. No private keys ever leave this
// process; no gas is ever paid by the user.

mod config;
mod crypto;
mod error;
mod oneshot;
mod pipeline;
mod security;
mod sovereign;
mod validation;
mod venice;
mod x402;


use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use config::Config;
use std::io::Write;
use std::path::PathBuf;

/// Logical id of the single signing key in the encrypted store.
const KEY_ID: &str = "signing";

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
    /// Manage the encrypted EVM signing key
    Keys {
        #[command(subcommand)]
        action: KeyAction,
    },
}

#[derive(Subcommand)]
enum KeyAction {
    /// Encrypt an EVM private key into the local key store under a passphrase
    Init,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { intent } => run(&intent).await,
        Commands::Pipeline { intent } => {
            // Make the signing key available to the pipeline's own config load
            // (Digital Hands reads it for the on-chain step).
            if let Some(key) = resolve_signing_key().await? {
                std::env::set_var("HAMMURABI_PRIVATE_KEY", key);
            }
            pipeline::run(&intent).await.map(|_| ())
        }
        Commands::Keys { action } => match action {
            KeyAction::Init => keys_init().await,
        },
    }
}

/// Directory holding the encrypted key store (override with `HAMMURABI_KEY_STORE`).
fn key_store_dir() -> PathBuf {
    std::env::var("HAMMURABI_KEY_STORE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("hammurabi_keys"))
}

fn key_manager() -> crypto::KeyManager {
    crypto::KeyManager::file_backed(key_store_dir(), KEY_ID)
}

/// Read a secret from the terminal without echoing it.
fn prompt_hidden(prompt: &str) -> Result<String> {
    rpassword::prompt_password(prompt).context("failed to read input from terminal")
}

/// Resolve the EVM signing key. Prefers the encrypted store (prompting for a
/// passphrase); falls back to the plaintext `HAMMURABI_PRIVATE_KEY` env var;
/// returns `None` if neither is available.
async fn resolve_signing_key() -> Result<Option<String>> {
    let km = key_manager();
    if km.exists().await {
        let passphrase = prompt_hidden("Passphrase to unlock signing key: ")?;
        let key = km.load_private_key_hex(&passphrase).await?;
        return Ok(Some(key));
    }
    match std::env::var("HAMMURABI_PRIVATE_KEY") {
        Ok(k) if !k.trim().is_empty() => {
            eprintln!(
                "⚠ Using plaintext HAMMURABI_PRIVATE_KEY from the environment. \
                 Run `hammurabi keys init` to encrypt it at rest."
            );
            Ok(Some(k))
        }
        _ => Ok(None),
    }
}

/// `keys init`: encrypt a private key (from the env var, or a hidden prompt)
/// under a passphrase and persist it to the local key store.
async fn keys_init() -> Result<()> {
    let km = key_manager();
    if km.exists().await {
        bail!(
            "an encrypted key already exists at {}. Remove it to re-initialize.",
            key_store_dir().join(format!("{KEY_ID}.enc")).display()
        );
    }

    let private_key = match std::env::var("HAMMURABI_PRIVATE_KEY") {
        Ok(k) if !k.trim().is_empty() => {
            println!("Encrypting the private key from HAMMURABI_PRIVATE_KEY.");
            k
        }
        _ => prompt_hidden("EVM private key (hex): ")?,
    };

    let passphrase = prompt_hidden("New passphrase: ")?;
    if passphrase.len() < 8 {
        bail!("passphrase must be at least 8 characters");
    }
    let confirm = prompt_hidden("Confirm passphrase: ")?;
    if passphrase != confirm {
        bail!("passphrases do not match");
    }

    km.store_private_key(private_key.trim(), &passphrase).await?;

    println!(
        "✓ Encrypted signing key written to {}",
        key_store_dir().join(format!("{KEY_ID}.enc")).display()
    );
    println!("  You can now remove HAMMURABI_PRIVATE_KEY from your .env.");
    Ok(())
}

async fn run(intent: &str) -> Result<()> {
    println!("=== Hammurabi AI Highway ===");
    println!("Intent: \"{intent}\"\n");

    let mut config = Config::from_env()?;
    match resolve_signing_key().await? {
        Some(key) => config.hammurabi_private_key = key,
        None => bail!(
            "no signing key available — run `hammurabi keys init` to create an \
             encrypted key, or set HAMMURABI_PRIVATE_KEY"
        ),
    }
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
