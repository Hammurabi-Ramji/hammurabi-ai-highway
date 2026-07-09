// src/pipeline/bsm_test.rs — Unit tests for BSM finalization stage

#[cfg(test)]
mod tests {

    use crate::pipeline::{bsm, Artifact, Payload, PipelineContext, Stage};

    use tempfile::TempDir;

    #[tokio::test]
    async fn test_bsm_persists_artifacts() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir
            .path()
            .join("registry.db")
            .to_str()
            .unwrap()
            .to_string();

        let stage = bsm::Bsm;
        let mut payload = Payload::new("Test build");
        payload.intent_hash = "0123456789abcdef".to_string();
        payload.artifacts = vec![Artifact {
            path: "src/main.rs".to_string(),
            summary: "Main application".to_string(),
            polished: false,
        }];

        let ctx = PipelineContext {
            db_path: db_path.clone(),
            ..PipelineContext::default()
        };

        let result = stage.process(&ctx, payload).await.unwrap();

        assert!(result.finalized);

        // Verify artifact persisted to database
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        let mut stmt = conn
            .prepare("SELECT artifact FROM code_registry WHERE intent_hash = ?1")
            .unwrap();
        let artifact: String = stmt
            .query_row(["0123456789abcdef"], |row| row.get(0))
            .unwrap();

        assert!(artifact.contains("src/main.rs"));
    }

    #[tokio::test]
    async fn test_bsm_skips_cache_hit() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir
            .path()
            .join("registry.db")
            .to_str()
            .unwrap()
            .to_string();

        let stage = bsm::Bsm;
        let mut payload = Payload::new("Test build");
        payload.cache_hit = true; // This is a cache hit, should skip persistence
        payload.artifacts = vec![Artifact {
            path: "src/main.rs".to_string(),
            summary: "Should not persist".to_string(),
            polished: false,
        }];

        let ctx = PipelineContext {
            db_path: db_path.clone(),
            ..PipelineContext::default()
        };

        let result = stage.process(&ctx, payload).await.unwrap();

        assert!(result.finalized);

        // Nothing should have been persisted on a cache hit. BSM skips all DB
        // work in that case, so the table may not even exist — either way, zero
        // rows were written.
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM code_registry", [], |row| row.get(0))
            .unwrap_or(0);
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_bsm_no_artifacts() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir
            .path()
            .join("registry.db")
            .to_str()
            .unwrap()
            .to_string();

        let stage = bsm::Bsm;
        let mut payload = Payload::new("Test build");
        payload.artifacts = vec![]; // No artifacts

        let ctx = PipelineContext {
            db_path: db_path.clone(),
            ..PipelineContext::default()
        };

        let result = stage.process(&ctx, payload).await.unwrap();

        assert!(result.finalized);
    }

    #[tokio::test]
    async fn test_bsm_finalized_flag() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir
            .path()
            .join("registry.db")
            .to_str()
            .unwrap()
            .to_string();

        let stage = bsm::Bsm;
        let mut payload = Payload::new("Test build");
        payload.artifacts = vec![Artifact {
            path: "test.rs".to_string(),
            summary: "test".to_string(),
            polished: false,
        }];

        let ctx = PipelineContext {
            db_path: db_path.clone(),
            ..PipelineContext::default()
        };

        let result = stage.process(&ctx, payload).await.unwrap();

        assert!(result.finalized);
    }

    #[tokio::test]
    async fn test_bsm_intent_hash_preservation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir
            .path()
            .join("registry.db")
            .to_str()
            .unwrap()
            .to_string();

        let stage = bsm::Bsm;
        let original_hash = "abcdef1234567890";
        let mut payload = Payload::new("Test build");
        payload.intent_hash = original_hash.to_string();
        payload.artifacts = vec![Artifact {
            path: "test.rs".to_string(),
            summary: "test".to_string(),
            polished: false,
        }];

        let ctx = PipelineContext {
            db_path: db_path.clone(),
            ..PipelineContext::default()
        };

        let result = stage.process(&ctx, payload).await.unwrap();

        assert_eq!(result.intent_hash, original_hash);
    }

    #[tokio::test]
    async fn test_bsm_multiple_artifacts() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir
            .path()
            .join("registry.db")
            .to_str()
            .unwrap()
            .to_string();

        let stage = bsm::Bsm;
        let mut payload = Payload::new("Test build");
        payload.intent_hash = "0123456789abcdef".to_string();
        payload.artifacts = vec![
            Artifact {
                path: "src/main.rs".to_string(),
                summary: "Main".to_string(),
                polished: false,
            },
            Artifact {
                path: "src/lib.rs".to_string(),
                summary: "Lib".to_string(),
                polished: false,
            },
        ];

        let ctx = PipelineContext {
            db_path: db_path.clone(),
            ..PipelineContext::default()
        };

        let result = stage.process(&ctx, payload).await.unwrap();

        assert!(result.finalized);

        // Verify artifacts persisted
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        let mut stmt = conn
            .prepare("SELECT artifact FROM code_registry WHERE intent_hash = ?1")
            .unwrap();
        let artifact: String = stmt
            .query_row(["0123456789abcdef"], |row| row.get(0))
            .unwrap();

        assert!(artifact.contains("src/main.rs"));
        assert!(artifact.contains("src/lib.rs"));
    }
}
