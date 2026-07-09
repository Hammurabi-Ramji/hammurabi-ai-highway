// src/pipeline/eduba_test.rs — Unit tests for Eduba cache stage

#[cfg(test)]
mod tests {
    
    use crate::pipeline::{Payload, PipelineContext, eduba, Stage};
    
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_eduba_cache_miss() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().to_str().unwrap().to_string();
        
        let stage = eduba::Eduba;
        let intent = "New test intent never seen before";
        let payload = Payload::new(intent);
        
        let ctx = PipelineContext {
            db_path: db_path.clone(),
            ..PipelineContext::default()
        };
        
        let result = stage.process(&ctx, payload).await.unwrap();
        
        assert!(!result.cache_hit);
        assert_eq!(result.retrieved, None);
    }
    
    #[tokio::test]
    async fn test_eduba_cache_hit() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("registry.db").to_str().unwrap().to_string();

        // Create database and insert cached artifact
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS code_registry (
                intent_hash TEXT PRIMARY KEY,
                intent TEXT NOT NULL,
                artifact TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
            [],
        ).unwrap();
        
        let intent = "Test cached intent";
        let hash = "0123456789abcdef";
        let artifact = "cached artifact summary";
        
        conn.execute(
            "INSERT INTO code_registry (intent_hash, intent, artifact) 
             VALUES (?1, ?2, ?3)",
            rusqlite::params![hash, intent, artifact],
        ).unwrap();
        
        // Test cache hit
        let stage = eduba::Eduba;
        let mut payload = Payload::new(intent);
        payload.intent_hash = hash.to_string();
        
        let ctx = PipelineContext {
            db_path: db_path.clone(),
            ..PipelineContext::default()
        };
        
        let result = stage.process(&ctx, payload).await.unwrap();
        
        assert!(result.cache_hit);
        assert_eq!(result.retrieved, Some(artifact.to_string()));
    }
    
    #[tokio::test]
    async fn test_eduba_cache_miss_different_hash() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().to_str().unwrap().to_string();
        
        let stage = eduba::Eduba;
        let intent = "Test intent";
        let mut payload = Payload::new(intent);
        payload.intent_hash = "0123456789abcdef".to_string();
        
        let ctx = PipelineContext {
            db_path: db_path.clone(),
            ..PipelineContext::default()
        };
        
        let result = stage.process(&ctx, payload).await.unwrap();
        
        assert!(!result.cache_hit);
    }
    
    #[tokio::test]
    async fn test_eduba_database_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().to_str().unwrap().to_string();
        
        let stage = eduba::Eduba;
        let intent = "Test intent";
        let payload = Payload::new(intent);
        
        let ctx = PipelineContext {
            db_path: db_path.clone(),
            ..PipelineContext::default()
        };
        
        let result = stage.process(&ctx, payload).await.unwrap();
        
        assert!(!result.cache_hit);
        assert!(std::path::Path::new(&db_path).exists());
    }
    
    #[tokio::test]
    async fn test_eduba_intent_hash_preservation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().to_str().unwrap().to_string();
        
        let stage = eduba::Eduba;
        let original_hash = "0123456789abcdef1234567890abcdef";
        let mut payload = Payload::new("Test intent");
        payload.intent_hash = original_hash.to_string();
        
        let ctx = PipelineContext {
            db_path: db_path.clone(),
            ..PipelineContext::default()
        };
        
        let result = stage.process(&ctx, payload).await.unwrap();
        
        assert_eq!(result.intent_hash, original_hash);
    }
}
