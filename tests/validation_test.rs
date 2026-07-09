// Unit tests for validation module

use hammurabi_ai_highway::validation::{Calldata, CalldataVerifier};

#[tokio::test]
async fn test_calldata_verifier_creation() {
    let verifier = CalldataVerifier::new().unwrap();
    assert!(verifier.supports_chain(8453).await);
}

#[tokio::test]
async fn test_verifier_whitelisted_contract() {
    let verifier = CalldataVerifier::new().unwrap();
    let calldata = Calldata {
        to: "0x4200000000000000000000000000000000000006".to_string(),
        data: "0x12345678".to_string(),
        value: "0x0".to_string(),
    };

    let result = verifier.verify(&calldata).await.unwrap();
    assert!(result.approved);
}

#[tokio::test]
async fn test_verifier_non_whitelisted_contract() {
    let verifier = CalldataVerifier::new().unwrap();
    let calldata = Calldata {
        to: "0x0000000000000000000000000000000000000000".to_string(),
        data: "0x12345678".to_string(),
        value: "0x0".to_string(),
    };

    let result = verifier.verify(&calldata).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_verifier_high_risk() {
    let verifier = CalldataVerifier::new().unwrap();
    let calldata = Calldata {
        to: "0x4200000000000000000000000000000000000006".to_string(),
        data: "0x".repeat(2000),                // Very long data = high risk
        value: "0xde0b6b3a7640000".to_string(), // Large value
    };

    let result = verifier.verify(&calldata).await.unwrap();
    assert!(result.risk_score > 50); // Should have elevated risk score
}
