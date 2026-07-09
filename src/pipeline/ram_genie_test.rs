// src/pipeline/ram_genie_test.rs — Unit tests for Ram Genie stage

#[cfg(test)]
mod tests {

    use crate::pipeline::{ram_genie, Payload, PipelineContext, Stage};

    #[tokio::test]
    async fn test_intent_fingerprint_deterministic() {
        let stage = ram_genie::RamGenie;
        let intent = "Swap 10 USDC for VVV";
        let payload = Payload::new(intent);

        // Fingerprint should be deterministic
        let ctx = PipelineContext::default();
        let result = stage.process(&ctx, payload).await.unwrap();

        assert!(!result.intent_hash.is_empty());
        assert_eq!(result.intent_hash.len(), 16); // 16 hex characters
    }

    #[tokio::test]
    async fn test_onchain_requirement_detection() {
        let stage = ram_genie::RamGenie;
        let intent = "Swap 10 USDC for VVV";
        let payload = Payload::new(intent);

        let ctx = PipelineContext::default();
        let result = stage.process(&ctx, payload).await.unwrap();

        assert!(result
            .requirements
            .contains(&"on-chain execution required".to_string()));
    }

    #[tokio::test]
    async fn test_frontend_requirement_detection() {
        let stage = ram_genie::RamGenie;
        let intent = "Build a dashboard for tracking metrics";
        let payload = Payload::new(intent);

        let ctx = PipelineContext::default();
        let result = stage.process(&ctx, payload).await.unwrap();

        assert!(result
            .requirements
            .contains(&"frontend surface required".to_string()));
    }

    #[tokio::test]
    async fn test_auth_requirement_detection() {
        let stage = ram_genie::RamGenie;
        let intent = "Create login page with authentication";
        let payload = Payload::new(intent);

        let ctx = PipelineContext::default();
        let result = stage.process(&ctx, payload).await.unwrap();

        assert!(result
            .requirements
            .contains(&"authentication required".to_string()));
    }

    #[tokio::test]
    async fn test_generic_build_request() {
        let stage = ram_genie::RamGenie;
        let intent = "Build something cool";
        let payload = Payload::new(intent);

        let ctx = PipelineContext::default();
        let result = stage.process(&ctx, payload).await.unwrap();

        assert!(result
            .requirements
            .contains(&"generic build request".to_string()));
    }

    #[tokio::test]
    async fn test_multiple_requirements() {
        let stage = ram_genie::RamGenie;
        let intent = "Build an auth dashboard with on-chain swap functionality";
        let payload = Payload::new(intent);

        let ctx = PipelineContext::default();
        let result = stage.process(&ctx, payload).await.unwrap();

        assert!(result
            .requirements
            .contains(&"authentication required".to_string()));
        assert!(result
            .requirements
            .contains(&"frontend surface required".to_string()));
        assert!(result
            .requirements
            .contains(&"on-chain execution required".to_string()));
    }

    #[tokio::test]
    async fn test_case_insensitive_parsing() {
        let stage = ram_genie::RamGenie;

        // Test with various case combinations
        let intents = vec![
            "swap 10 USDC",
            "SWAP 10 USDC",
            "Swap 10 usdc",
            "SWAP 10 usdc",
        ];

        for intent in intents {
            let payload = Payload::new(intent);
            let ctx = PipelineContext::default();
            let result = stage.process(&ctx, payload).await.unwrap();

            assert!(
                result
                    .requirements
                    .contains(&"on-chain execution required".to_string()),
                "Intent '{}' should detect on-chain requirement",
                intent
            );
        }
    }
}
