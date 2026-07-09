// src/pipeline/han_solo_test.rs — Unit tests for Han Solo build stage

#[cfg(test)]
mod tests {

    use crate::pipeline::{han_solo, Payload, PipelineContext, Stage};

    use tempfile::TempDir;

    #[tokio::test]
    async fn test_han_solo_cache_hit() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().to_str().unwrap().to_string();

        let stage = han_solo::HanSolo;
        let mut payload = Payload::new("Test intent");
        payload.cache_hit = true;

        let ctx = PipelineContext {
            db_path: db_path.clone(),
            ..PipelineContext::default()
        };

        let result = stage.process(&ctx, payload).await.unwrap();

        assert!(result.cache_hit);
        // Should return early without generating artifacts
        assert!(result.artifacts.is_empty());
    }

    #[tokio::test]
    async fn test_han_solo_gateway_offline() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().to_str().unwrap().to_string();

        let stage = han_solo::HanSolo;
        let mut payload = Payload::new("on-chain execution required");
        payload.requirements = vec!["on-chain execution required".to_string()];

        let ctx = PipelineContext {
            db_path: db_path.clone(),
            sovereign_online: false, // Gateway offline
            ..PipelineContext::default()
        };

        let result = stage.process(&ctx, payload).await.unwrap();

        // Should fall back to local plan
        assert!(!result.artifacts.is_empty());
        assert_eq!(result.artifacts[0].path, "src/execution/onchain.rs");
    }

    #[tokio::test]
    async fn test_han_solo_frontend_generation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().to_str().unwrap().to_string();

        let stage = han_solo::HanSolo;
        let mut payload = Payload::new("frontend surface required");
        payload.requirements = vec!["frontend surface required".to_string()];

        let ctx = PipelineContext {
            db_path: db_path.clone(),
            ..PipelineContext::default()
        };

        let result = stage.process(&ctx, payload).await.unwrap();

        assert!(!result.artifacts.is_empty());
        assert_eq!(result.artifacts[0].path, "ui/app.tsx");
        assert_eq!(result.artifacts[0].summary, "generated frontend surface");
    }

    #[tokio::test]
    async fn test_han_solo_auth_generation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().to_str().unwrap().to_string();

        let stage = han_solo::HanSolo;
        let mut payload = Payload::new("authentication required");
        payload.requirements = vec!["authentication required".to_string()];

        let ctx = PipelineContext {
            db_path: db_path.clone(),
            ..PipelineContext::default()
        };

        let result = stage.process(&ctx, payload).await.unwrap();

        assert!(!result.artifacts.is_empty());
        assert_eq!(result.artifacts[0].path, "src/auth.rs");
        assert_eq!(result.artifacts[0].summary, "JWT authentication module");
    }

    #[tokio::test]
    async fn test_han_solo_generic_module() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().to_str().unwrap().to_string();

        let stage = han_solo::HanSolo;
        let mut payload = Payload::new("generic requirement");
        payload.requirements = vec!["generic requirement".to_string()];

        let ctx = PipelineContext {
            db_path: db_path.clone(),
            ..PipelineContext::default()
        };

        let result = stage.process(&ctx, payload).await.unwrap();

        assert!(!result.artifacts.is_empty());
        assert_eq!(result.artifacts[0].path, "src/module.rs");
        assert_eq!(result.artifacts[0].summary, "generic module");
    }
}
