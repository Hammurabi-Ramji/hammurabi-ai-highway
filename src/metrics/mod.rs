// src/metrics/mod.rs — Performance monitoring module

use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Performance metrics for pipeline stages
#[derive(Debug, Clone)]
pub struct StageMetrics {
    pub stage_name: String,
    pub count: u64,
    pub total_time_ms: u64,
    pub min_time_ms: u64,
    pub max_time_ms: u64,
    pub last_10_times: Vec<u64>,
}

impl StageMetrics {
    pub fn new(stage_name: String) -> Self {
        Self {
            stage_name,
            count: 0,
            total_time_ms: 0,
            min_time_ms: u64::MAX,
            max_time_ms: 0,
            last_10_times: Vec::new(),
        }
    }
    
    pub fn record(&mut self, duration_ms: u64) {
        self.count += 1;
        self.total_time_ms += duration_ms;
        self.min_time_ms = self.min_time_ms.min(duration_ms);
        self.max_time_ms = self.max_time_ms.max(duration_ms);
        
        self.last_10_times.push(duration_ms);
        if self.last_10_times.len() > 10 {
            self.last_10_times.remove(0);
        }
    }
    
    pub fn p99_latency(&self) -> u64 {
        if self.last_10_times.is_empty() {
            return 0;
        }
        
        let mut sorted = self.last_10_times.clone();
        sorted.sort();
        
        let idx = ((sorted.len() as f64) * 0.99) as usize;
        sorted[idx.min(sorted.len() - 1)]
    }
    
    pub fn avg_latency(&self) -> f64 {
        if self.count == 0 {
            return 0.0;
        }
        self.total_time_ms as f64 / self.count as f64
    }
}

/// Overall pipeline metrics
#[derive(Debug, Clone)]
pub struct PipelineMetrics {
    pub pipeline_name: String,
    pub total_runs: u64,
    pub success_runs: u64,
    pub failure_runs: u64,
    pub avg_total_time_ms: f64,
    pub start_time: DateTime<Utc>,
    pub stage_metrics: HashMap<String, StageMetrics>,
}

impl PipelineMetrics {
    pub fn new(pipeline_name: String) -> Self {
        Self {
            pipeline_name,
            total_runs: 0,
            success_runs: 0,
            failure_runs: 0,
            avg_total_time_ms: 0.0,
            start_time: Utc::now(),
            stage_metrics: HashMap::new(),
        }
    }
    
    pub fn record_stage(&mut self, stage_name: &str, duration_ms: u64) {
        self.stage_metrics
            .entry(stage_name.to_string())
            .or_insert_with(|| StageMetrics::new(stage_name.to_string()))
            .record(duration_ms);
    }
    
    pub fn record_success(&mut self, total_time_ms: u64) {
        self.total_runs += 1;
        self.success_runs += 1;
        
        // Update average
        self.avg_total_time_ms = (self.avg_total_time_ms * (self.total_runs - 1) as f64 + total_time_ms as f64) / self.total_runs as f64;
    }
    
    pub fn record_failure(&mut self) {
        self.total_runs += 1;
        self.failure_runs += 1;
    }
    
    pub fn success_rate(&self) -> f64 {
        if self.total_runs == 0 {
            return 0.0;
        }
        self.success_runs as f64 / self.total_runs as f64 * 100.0
    }
}

/// Performance monitor singleton
pub struct PerformanceMonitor {
    metrics: Arc<RwLock<PipelineMetrics>>,
}

impl PerformanceMonitor {
    pub fn new(pipeline_name: &str) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(PipelineMetrics::new(pipeline_name.to_string()))),
        }
    }
    
    pub async fn record_stage(&self, stage_name: &str, duration_ms: u64) {
        let mut metrics = self.metrics.write().await;
        metrics.record_stage(stage_name, duration_ms);
    }
    
    pub async fn record_success(&self, total_time_ms: u64) {
        let mut metrics = self.metrics.write().await;
        metrics.record_success(total_time_ms);
    }
    
    pub async fn record_failure(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.record_failure();
    }
    
    pub async fn get_metrics(&self) -> PipelineMetrics {
        self.metrics.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_stage_metrics_record() {
        let mut metrics = StageMetrics::new("test".to_string());
        
        metrics.record(10);
        metrics.record(20);
        metrics.record(30);
        
        assert_eq!(metrics.count, 3);
        assert_eq!(metrics.total_time_ms, 60);
        assert_eq!(metrics.min_time_ms, 10);
        assert_eq!(metrics.max_time_ms, 30);
        assert!((metrics.avg_latency() - 20.0).abs() < 0.01);
    }
    
    #[tokio::test]
    async fn test_pipeline_metrics_record() {
        let monitor = PerformanceMonitor::new("test_pipeline");
        
        monitor.record_stage("stage1", 10).await;
        monitor.record_stage("stage2", 20).await;
        monitor.record_success(30).await;
        
        let metrics = monitor.get_metrics().await;
        
        assert_eq!(metrics.total_runs, 1);
        assert_eq!(metrics.success_runs, 1);
        assert!((metrics.avg_total_time_ms - 30.0).abs() < 0.01);
    }
    
    #[tokio::test]
    async fn test_pipeline_metrics_failure() {
        let monitor = PerformanceMonitor::new("test_pipeline");
        
        monitor.record_failure().await;
        
        let metrics = monitor.get_metrics().await;
        
        assert_eq!(metrics.failure_runs, 1);
        assert_eq!(metrics.success_rate(), 0.0);
    }
}
