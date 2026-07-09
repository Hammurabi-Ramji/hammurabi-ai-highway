// Unit tests for error module

use hammurabi_ai_highway::error::{PipelineError, RecoveryAction};

#[test]
fn test_error_codes() {
    let err = PipelineError::VeniceAIUnavailable;
    assert_eq!(err.code(), "VENICE001");

    let err = PipelineError::RelayFailed {
        reason: "test".to_string(),
    };
    assert_eq!(err.code(), "RELAY001");

    let err = PipelineError::DatabaseError {
        reason: "test".to_string(),
    };
    assert_eq!(err.code(), "DB001");
}

#[test]
fn test_retryable_errors() {
    assert!(PipelineError::VeniceAIUnavailable.is_retryable());
    assert!(PipelineError::NetworkError {
        reason: "test".to_string()
    }
    .is_retryable());
    assert!(PipelineError::RateLimitExceeded.is_retryable());

    assert!(!PipelineError::PrivateKeyError {
        reason: "test".to_string()
    }
    .is_retryable());
    assert!(!PipelineError::SecurityGateBlocked {
        reason: "test".to_string()
    }
    .is_retryable());
}

#[test]
fn test_recovery_actions() {
    let err = PipelineError::VeniceAIUnavailable;
    assert_eq!(err.recovery_action(), RecoveryAction::UseLocalFallback);

    let err = PipelineError::RelayFailed {
        reason: "test".to_string(),
    };
    assert_eq!(err.recovery_action(), RecoveryAction::QueueForRetry);

    let err = PipelineError::StageFailed {
        stage: "test".to_string(),
        reason: "test".to_string(),
    };
    assert!(matches!(
        err.recovery_action(),
        RecoveryAction::RetryWithFallback(_)
    ));

    let err = PipelineError::SecurityGateBlocked {
        reason: "test".to_string(),
    };
    assert_eq!(err.recovery_action(), RecoveryAction::Abort);
}

#[test]
fn test_all_errors_have_codes() {
    // Ensure all error variants have codes defined
    let _ = PipelineError::StageFailed {
        stage: "test".to_string(),
        reason: "test".to_string(),
    }
    .code();
    let _ = PipelineError::VeniceAIUnavailable.code();
    let _ = PipelineError::RelayFailed {
        reason: "test".to_string(),
    }
    .code();
    let _ = PipelineError::DatabaseError {
        reason: "test".to_string(),
    }
    .code();
    let _ = PipelineError::CalldataValidationFailed {
        reason: "test".to_string(),
    }
    .code();
    let _ = PipelineError::PrivateKeyError {
        reason: "test".to_string(),
    }
    .code();
    let _ = PipelineError::CacheError {
        reason: "test".to_string(),
    }
    .code();
    let _ = PipelineError::SecurityGateBlocked {
        reason: "test".to_string(),
    }
    .code();
    let _ = PipelineError::ConfigurationError {
        reason: "test".to_string(),
    }
    .code();
    let _ = PipelineError::RateLimitExceeded.code();
    let _ = PipelineError::ReplayAttackDetected.code();
    let _ = PipelineError::InvalidPrivateKey.code();
    let _ = PipelineError::KeyRotationRequired.code();
    let _ = PipelineError::NetworkError {
        reason: "test".to_string(),
    }
    .code();
    let _ = PipelineError::JsonParseError {
        reason: "test".to_string(),
    }
    .code();
}
