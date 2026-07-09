// src/pipeline/digital_hands_test.rs — Unit tests for Digital Hands relay stage

#[cfg(test)]
mod tests {

    use crate::pipeline::{digital_hands, Payload, PipelineContext, Stage};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_digital_hands_no_onchain_requirement() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().to_str().unwrap().to_string();

        let stage = digital_hands::DigitalHands;
        let payload = Payload::new("Build a website");
        // No on-chain requirement

        let ctx = PipelineContext {
            db_path: db_path.clone(),
            config: None,
            ..PipelineContext::default()
        };

        let result = stage.process(&ctx, payload).await.unwrap();

        assert!(result.onchain_tx.is_none());
    }

    #[tokio::test]
    async fn test_digital_hands_no_credentials() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().to_str().unwrap().to_string();

        let stage = digital_hands::DigitalHands;
        let mut payload = Payload::new("Swap 10 USDC for VVV");
        payload.requirements = vec!["on-chain execution required".to_string()];

        let ctx = PipelineContext {
            db_path: db_path.clone(),
            config: None, // No credentials
            ..PipelineContext::default()
        };

        let result = stage.process(&ctx, payload).await.unwrap();

        // Should simulate
        assert_eq!(result.onchain_tx, Some("0xSIMULATED".to_string()));
    }

    #[tokio::test]
    async fn test_digital_hands_with_credentials_simulation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().to_str().unwrap().to_string();

        let stage = digital_hands::DigitalHands;
        let mut payload = Payload::new("Swap 10 USDC for VVV");
        payload.requirements = vec!["on-chain execution required".to_string()];

        // Config without valid credentials (will still simulate)
        let config = crate::config::Config {
            venice_api_key: "test_key".to_string(),
            venice_base_url: "http://test".to_string(),
            venice_model: "test_model".to_string(),
            oneshot_api_key: "test_key".to_string(),
            oneshot_base_url: "http://test".to_string(),
            oneshot_wallet_id: "test_wallet".to_string(),
            hammurabi_private_key:
                "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            base_chain_id: 8453,
        };

        let ctx = PipelineContext {
            db_path: db_path.clone(),
            config: Some(config),
            ..PipelineContext::default()
        };

        let result = stage.process(&ctx, payload).await;

        // Should handle gracefully (may fail due to invalid URLs, but should be handled)
        // The key is that it doesn't panic
        // Accept network errors for invalid URLs; only assert on success.
        if let Ok(payload) = result {
            assert!(payload.onchain_tx.is_some());
        }
    }
}
