// Integration tests for the full pipeline

use hammurabi_ai_highway::pipeline::{stages, Payload, PipelineContext, Stage};
use tempfile::TempDir;

/// Build a pipeline context backed by an isolated, empty temp database file.
fn temp_ctx() -> (TempDir, PipelineContext) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir
        .path()
        .join("registry.db")
        .to_str()
        .unwrap()
        .to_string();
    let ctx = PipelineContext {
        db_path,
        ..PipelineContext::default()
    };
    (temp_dir, ctx)
}

async fn run_pipeline(ctx: &PipelineContext, intent: &str) -> Payload {
    let mut payload = Payload::new(intent);
    for stage in &stages() {
        payload = stage.process(ctx, payload).await.unwrap();
    }
    payload
}

#[tokio::test]
async fn test_full_pipeline_end_to_end() {
    let (_tmp, ctx) = temp_ctx();

    let payload = run_pipeline(&ctx, "Swap 10 USDC for VVV").await;

    assert!(payload.finalized);
}

#[tokio::test]
async fn test_pipeline_with_cache_hit() {
    let (_tmp, ctx) = temp_ctx();
    let intent = "Test intent for cache hit";

    // First run: cache miss — BSM persists the generated artifact.
    let first = run_pipeline(&ctx, intent).await;
    assert!(first.finalized);
    assert!(!first.cache_hit);

    // Second run with the same intent: Eduba should serve it from the registry.
    let second = run_pipeline(&ctx, intent).await;
    assert!(second.cache_hit);
    assert!(second.finalized);
}

#[tokio::test]
async fn test_ram_genie_stage() {
    use hammurabi_ai_highway::pipeline::ram_genie::RamGenie;

    let (_tmp, ctx) = temp_ctx();
    let stage = RamGenie;

    let payload = Payload::new("Swap 10 USDC for VVV");
    let payload = stage.process(&ctx, payload).await.unwrap();

    assert!(!payload.intent_hash.is_empty());
    assert!(!payload.requirements.is_empty());
    assert!(payload
        .requirements
        .contains(&"on-chain execution required".to_string()));
}

#[tokio::test]
async fn test_eduba_stage_cache_hit() {
    use hammurabi_ai_highway::pipeline::eduba::Eduba;

    let (_tmp, ctx) = temp_ctx();

    // Seed the registry with a cached artifact.
    let conn = rusqlite::Connection::open(&ctx.db_path).unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS code_registry (
            intent_hash TEXT PRIMARY KEY,
            intent TEXT NOT NULL,
            artifact TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO code_registry (intent_hash, intent, artifact)
         VALUES (?1, ?2, ?3)",
        rusqlite::params!["0123456789abcdef", "Test intent", "cached artifact"],
    )
    .unwrap();

    let stage = Eduba;
    let mut payload = Payload::new("Test intent");
    payload.intent_hash = "0123456789abcdef".to_string();

    let payload = stage.process(&ctx, payload).await.unwrap();

    assert!(payload.cache_hit);
    assert_eq!(payload.retrieved, Some("cached artifact".to_string()));
}
