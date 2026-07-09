// src/error.rs — Comprehensive error taxonomy
//
// Public error-taxonomy scaffolding. The live pipeline currently threads
// `anyhow::Error`; these typed errors are retained as the intended API surface
// and exercised by unit tests, so unused variants are expected for now.
#![allow(dead_code)]

use thiserror::Error;

/// Main error type for the pipeline
#[derive(Error, Debug, Clone)]
pub enum PipelineError {
    #[error("Stage failed: {stage} - {reason}")]
    StageFailed { stage: String, reason: String },

    #[error("Venice AI service unavailable")]
    VeniceAIUnavailable,

    #[error("1Shot relay failed: {reason}")]
    RelayFailed { reason: String },

    #[error("Database error: {reason}")]
    DatabaseError { reason: String },

    #[error("Calldata validation failed: {reason}")]
    CalldataValidationFailed { reason: String },

    #[error("Private key error: {reason}")]
    PrivateKeyError { reason: String },

    #[error("Cache error: {reason}")]
    CacheError { reason: String },

    #[error("Security gate blocked: {reason}")]
    SecurityGateBlocked { reason: String },

    #[error("Pipeline configuration error: {reason}")]
    ConfigurationError { reason: String },

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Replay attack detected")]
    ReplayAttackDetected,

    #[error("Invalid private key format")]
    InvalidPrivateKey,

    #[error("Key rotation required")]
    KeyRotationRequired,

    #[error("Network error: {reason}")]
    NetworkError { reason: String },

    #[error("JSON parsing error: {reason}")]
    JsonParseError { reason: String },
}

impl PipelineError {
    /// Get error code for machine-readable handling
    pub fn code(&self) -> &'static str {
        match self {
            PipelineError::StageFailed { .. } => "PIPELINE001",
            PipelineError::VeniceAIUnavailable => "VENICE001",
            PipelineError::RelayFailed { .. } => "RELAY001",
            PipelineError::DatabaseError { .. } => "DB001",
            PipelineError::CalldataValidationFailed { .. } => "CALDATA001",
            PipelineError::PrivateKeyError { .. } => "KEY001",
            PipelineError::CacheError { .. } => "CACHE001",
            PipelineError::SecurityGateBlocked { .. } => "SEC001",
            PipelineError::ConfigurationError { .. } => "CONFIG001",
            PipelineError::RateLimitExceeded => "RATE001",
            PipelineError::ReplayAttackDetected => "REPLAY001",
            PipelineError::InvalidPrivateKey => "KEY002",
            PipelineError::KeyRotationRequired => "KEY003",
            PipelineError::NetworkError { .. } => "NET001",
            PipelineError::JsonParseError { .. } => "JSON001",
        }
    }

    /// Determine if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            PipelineError::VeniceAIUnavailable
                | PipelineError::RelayFailed { .. }
                | PipelineError::NetworkError { .. }
                | PipelineError::RateLimitExceeded
        )
    }

    /// Get recommended recovery action
    pub fn recovery_action(&self) -> RecoveryAction {
        match self {
            PipelineError::StageFailed { stage, .. } => {
                RecoveryAction::RetryWithFallback(stage.clone())
            }
            PipelineError::VeniceAIUnavailable => RecoveryAction::UseLocalFallback,
            PipelineError::RelayFailed { .. } => RecoveryAction::QueueForRetry,
            PipelineError::DatabaseError { .. } => RecoveryAction::UseCachedData,
            _ => RecoveryAction::Abort,
        }
    }
}

/// Recovery action recommendations
#[derive(Debug, Clone, PartialEq, Default)]
pub enum RecoveryAction {
    Retry {
        max_attempts: u32,
        delay_ms: u64,
    },
    RetryWithFallback(String),
    UseLocalFallback,
    QueueForRetry,
    UseCachedData,
    #[default]
    Abort,
}

/// Stage-specific errors
#[derive(Error, Debug)]
pub enum StageError {
    #[error("Precondition not met: {0}")]
    PreconditionNotMet(String),

    #[error("Guarantee failed: {0}")]
    GuaranteeFailed(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),
}

/// Validation errors
#[derive(Error, Debug)]
pub enum ValidationErrors {
    #[error("Invalid contract address: {0}")]
    InvalidContractAddress(String),

    #[error("Invalid ABI signature: {0}")]
    InvalidAbiSignature(String),

    #[error("Transaction value too large: {0}")]
    TransactionValueTooLarge(String),

    #[error("Replay attack detected")]
    ReplayAttack,
}

/// Security errors
#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Secret detected in artifact")]
    SecretDetected,

    #[error("High risk score: {0}")]
    HighRiskScore(u32),

    #[error("Unauthorized access attempt")]
    UnauthorizedAccess,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Key validation failed: {0}")]
    KeyValidationFailed(String),
}

/// Integration test errors
#[derive(Error, Debug)]
pub enum IntegrationTestError {
    #[error("Test environment not ready")]
    EnvironmentNotReady,

    #[error("Mock service unavailable: {0}")]
    MockServiceUnavailable(String),

    #[error("Test data not found: {0}")]
    TestDataNotFound(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        let err = PipelineError::VeniceAIUnavailable;
        assert_eq!(err.code(), "VENICE001");

        let err = PipelineError::RelayFailed {
            reason: "test".to_string(),
        };
        assert_eq!(err.code(), "RELAY001");
    }

    #[test]
    fn test_retryable_errors() {
        assert!(PipelineError::VeniceAIUnavailable.is_retryable());
        assert!(PipelineError::NetworkError {
            reason: "test".to_string()
        }
        .is_retryable());
        assert!(!PipelineError::InvalidPrivateKey.is_retryable());
    }

    #[test]
    fn test_recovery_actions() {
        let err = PipelineError::VeniceAIUnavailable;
        assert_eq!(err.recovery_action(), RecoveryAction::UseLocalFallback);

        let err = PipelineError::RelayFailed {
            reason: "test".to_string(),
        };
        assert_eq!(err.recovery_action(), RecoveryAction::QueueForRetry);
    }
}
