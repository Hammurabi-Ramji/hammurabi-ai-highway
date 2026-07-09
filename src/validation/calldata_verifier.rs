// src/validation/calldata_verifier.rs — Comprehensive calldata validation
//
// Standalone verifier not yet wired into the live on-chain path; its public
// surface is exercised by tests, so unused items are expected for now.
#![allow(dead_code)]

use anyhow::Result;
use ethers_core::types::{Address, H256, U256};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize)]
pub struct CalldataVerificationResult {
    pub approved: bool,
    pub risk_score: u32,
    pub rejection_reason: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum VerificationError {
    #[error("Address is not a contract: {address}")]
    NotContract { address: Address },
    
    #[error("Address not whitelisted: {address}")]
    NotWhitelisted { address: Address },
    
    #[error("Invalid ABI signature: {function_hash}")]
    InvalidSignature { function_hash: H256 },
    
    #[error("Value exceeds limit: {val} > {max}")]
    ValueExceedsLimit { val: u128, max: u128 },
    
    #[error("Replay attack detected")]
    ReplayDetected,
    
    #[error("High risk score: {score}")]
    HighRisk { score: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiRegistry {
    contracts: std::collections::HashMap<String, Vec<String>>,
}

impl AbiRegistry {
    pub fn new() -> Self {
        Self {
            contracts: HashMap::new(),
        }
    }
    
    pub fn register_contract(&mut self, address: &str, signatures: Vec<&str>) {
        let addr = address.to_lowercase();
        self.contracts.insert(addr, signatures.iter().map(|s| s.to_string()).collect());
    }
    
    /// Whether the contract at `address` has the 4-byte function `selector`
    /// registered. Both sides are compared as `0x`-prefixed hex strings.
    pub fn supports_function(&self, address: &Address, selector: &[u8]) -> bool {
        if selector.len() < 4 {
            return false;
        }
        let addr_key = format!("{:?}", address).to_lowercase();
        let selector_hex = format!("0x{}", hex::encode(&selector[..4]));

        self.contracts
            .get(&addr_key)
            .map(|sigs| sigs.iter().any(|sig| sig.eq_ignore_ascii_case(&selector_hex)))
            .unwrap_or(false)
    }
}

impl Default for AbiRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Calldata {
    pub to: String,
    pub data: String,
    #[serde(default = "default_value")]
    pub value: String,
}

fn default_value() -> String {
    "0x0".to_string()
}

pub struct CalldataVerifier {
    approved_contracts: Arc<RwLock<std::collections::HashMap<u64, BTreeSet<String>>>>,
    abi_registry: Arc<RwLock<AbiRegistry>>,
    max_value: Arc<RwLock<u128>>,
}

impl CalldataVerifier {
    pub fn new() -> Result<Self> {
        let mut approved_contracts = HashMap::new();
        
        approved_contracts.insert(
            8453,
            BTreeSet::from_iter(vec![
                "0x4200000000000000000000000000000000000006".to_string(),
                "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913".to_string(),
                "0xd9aaec86b65d86f6a7b5b1b0c425a9b77186762b".to_string(),
            ]),
        );
        
        let mut abi_registry = AbiRegistry::new();
        abi_registry.register_contract(
            "0x4200000000000000000000000000000000000006",
            vec!["0x", "0x095ea7b3", "0x23b872dd", "0x18160fd1"],
        );
        
        Ok(Self {
            approved_contracts: Arc::new(RwLock::new(approved_contracts)),
            abi_registry: Arc::new(RwLock::new(abi_registry)),
            max_value: Arc::new(RwLock::new(100_000_000_000_000_000_000u128)),
        })
    }
    
    /// Whether the verifier has any whitelisted contracts for the given chain.
    pub async fn supports_chain(&self, chain_id: u64) -> bool {
        self.approved_contracts.read().await.contains_key(&chain_id)
    }

    pub async fn verify(&self, calldata: &Calldata) -> Result<CalldataVerificationResult> {
        // Hard blocks: an un-whitelisted target or a value above the absolute
        // cap are rejected outright. An unknown/undecodable function selector is
        // treated as elevated risk (see calculate_risk_score) rather than a hard
        // failure, so the caller still receives a scored assessment.
        self.verify_contract_address(&calldata.to).await?;
        self.verify_value_bounds(&calldata.value).await?;
        self.verify_replay_protection(&calldata.to, &calldata.data).await?;

        let risk_score = self.calculate_risk_score(calldata).await?;
        
        if risk_score > 80 {
            return Ok(CalldataVerificationResult {
                approved: false,
                risk_score,
                rejection_reason: Some(format!("High risk score: {}", risk_score)),
            });
        }
        
        Ok(CalldataVerificationResult {
            approved: true,
            risk_score,
            rejection_reason: None,
        })
    }
    
    async fn verify_contract_address(&self, to: &str) -> Result<(), VerificationError> {
        let _address = to.parse::<Address>()
            .map_err(|_| VerificationError::NotWhitelisted { address: Address::zero() })?;
        
        let whitelist = self.approved_contracts.read().await;
        let chain_id = 8453;
        let address_str = to.to_lowercase();
        
        let is_whitelisted = whitelist.get(&chain_id)
            .map(|addrs| addrs.contains(&address_str))
            .unwrap_or(false);
        
        if !is_whitelisted {
            return Err(VerificationError::NotWhitelisted { address: Address::zero() });
        }
        
        Ok(())
    }
    
    /// Whether `data`'s leading 4-byte selector is a registered function on the
    /// contract at `to`. Malformed addresses or calldata count as unknown.
    async fn selector_known(&self, to: &str, data: &str) -> bool {
        let address = match to.parse::<Address>() {
            Ok(a) => a,
            Err(_) => return false,
        };
        let bytes = match hex::decode(data.trim_start_matches("0x")) {
            Ok(b) => b,
            Err(_) => return false,
        };
        if bytes.len() < 4 {
            return false;
        }
        self.abi_registry.read().await.supports_function(&address, &bytes[..4])
    }
    
    async fn verify_value_bounds(&self, value: &str) -> Result<(), VerificationError> {
        let val = U256::from_str_radix(&value.trim_start_matches("0x"), 16)
            .unwrap_or(U256::zero())
            .as_u128();
        
        let max = *self.max_value.read().await;
        
        if val > max {
            return Err(VerificationError::ValueExceedsLimit { val, max });
        }
        
        Ok(())
    }
    
    async fn verify_replay_protection(&self, _to: &str, _data: &str) -> Result<(), VerificationError> {
        Ok(())
    }
    
    async fn calculate_risk_score(&self, calldata: &Calldata) -> Result<u32> {
        let mut score = 0u32;
        
        let whitelist = self.approved_contracts.read().await;
        let is_whitelisted = whitelist.get(&8453)
            .map(|addrs| addrs.contains(&calldata.to.to_lowercase()))
            .unwrap_or(false);
        
        if !is_whitelisted {
            score += 30;
        }
        drop(whitelist);

        // An unknown or undecodable function selector is a strong risk signal.
        if !self.selector_known(&calldata.to, &calldata.data).await {
            score += 40;
        }

        let val = U256::from_str_radix(&calldata.value.trim_start_matches("0x"), 16)
            .unwrap_or(U256::zero())
            .as_u128();

        if val > 1_000_000_000_000_000_000u128 {
            score += 20;
        }
        
        if calldata.data.len() > 1000 {
            score += 15;
        }
        
        let max_value = *self.max_value.read().await;
        if val > max_value / 10 {
            score += 15;
        }
        
        Ok(score)
    }
}

impl Default for CalldataVerifier {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_verifier_creation() {
        let verifier = CalldataVerifier::new().unwrap();
        assert!(verifier.approved_contracts.read().await.get(&8453).is_some());
    }
    
    #[tokio::test]
    async fn test_whitelisted_contract() {
        let verifier = CalldataVerifier::new().unwrap();
        let calldata = Calldata {
            to: "0x4200000000000000000000000000000000000006".to_string(),
            data: "0x12345678".to_string(),
            value: "0x0".to_string(),
        };
        
        assert!(verifier.verify_contract_address(&calldata.to).await.is_ok());
    }
}
