# Security Guide

## Overview

This document describes the security architecture and mitigations for the Hammurabi AI Highway CLI.

## Threat Model

### Assets to Protect
- Private key for x402 authentication
- Venice AI API key
- 1Shot API key and wallet ID
- User transaction data

### Threats
1. Private key theft
2. Malicious Venice AI calldata
3. Replay attacks
4. Secret leaks in artifacts
5. Rate limiting abuse
6. Injection attacks

## Security Controls

### 1. Private Key Management

**Threat:** Private key stolen from `.env` file

**Mitigations:**
- Keys encrypted at rest using AES-256-GCM
- Key rotation every 90 days
- OS-level secure storage (DPAPI/Keychain)
- Audit logging for all key operations

**Implementation:** `src/crypto/key_manager.rs`

```rust
// Load encrypted key from secure storage
let wallet = key_manager.load_key("main").await?;

// Validate key before use
assert!(key_manager.validate_key("main", expected_public)?);
```

### 2. Calldata Validation

**Threat:** Malicious calldata from Venice AI

**Mitigations:**
- Contract address whitelist
- ABI signature verification
- Value bounds checking
- Risk scoring with auto-rejection
- Replay protection

**Implementation:** `src/validation/calldata_verifier.rs`

```rust
let verifier = CalldataVerifier::new()?;
let result = verifier.verify(&calldata).await?;

if !result.approved {
    return Err(format!("Rejected: {}", result.rejection_reason));
}
```

### 3. x402 Authentication

**Threat:** Unauthorized Venice AI requests

**Mitigations:**
- Timestamped ECDSA signatures
- Rate limiting (100 requests/hour)
- Replay protection
- Multi-key signing support

**Implementation:** `src/security/x402_v2.rs`

```rust
let auth = X402AuthV2::new(private_key).await?;
let signed = auth.sign(&body).await?;
// Add signed headers to request
```

### 4. Secret Detection

**Threat:** Secrets leaked in generated artifacts

**Mitigations:**
- Ram Gate security scanning
- Integration with gitleaks/trufflehog (TODO)
- Artifact encryption (TODO)

**Implementation:** `src/pipeline/ram_gate.rs`

```rust
// Current: Simple pattern matching
if artifact.summary.contains("sk-") {
    return Err("Secret detected".into());
}

// TODO: Use gitleaks/trufflehog for comprehensive scanning
```

### 5. Rate Limiting

**Threat:** Abuse of Venice AI / 1Shot APIs

**Mitigations:**
- Per-wallet rate limiting
- Token bucket algorithm
- Graceful degradation on limit

**Implementation:** `src/security/x402_v2.rs`

```rust
if !rate_limiter.allow(&wallet_address, 1).await {
    return Err("Rate limit exceeded".into());
}
```

### 6. Error Handling

**Threat:** Information leakage through errors

**Mitigations:**
- Typed error taxonomy
- Generic error messages
- No stack traces in production
- Audit logging for all security events

**Implementation:** `src/error.rs`

```rust
// All errors follow consistent pattern
#[derive(Error, Debug)]
pub enum PipelineError {
    #[error("Stage failed: {stage} - {reason}")]
    StageFailed { stage: String, reason: String },
    // ... other variants
}

// Machine-readable codes for monitoring
err.code() // -> "PIPELINE001", etc.

// Retryability flags
err.is_retryable() // -> true/false

// Recovery actions
err.recovery_action() // -> RecoveryAction enum
```

## Security Checklist

Before deployment:

- [ ] All secrets in secure storage (not plaintext `.env`)
- [ ] Private key rotation scheduled
- [ ] Calldata verifier whitelist reviewed
- [ ] Rate limits configured appropriately
- [ ] Security audit logs enabled
- [ ] Error messages don't leak sensitive info
- [ ] All dependencies scanned for vulnerabilities
- [ ] RAM Gate scanning upgraded to gitleaks/trufflehog

## Security Monitoring

### Metrics to Track
- Failed x402 authentication attempts
- Rate limit hits
- Calldata rejections
- Secret detections
- Key rotation events

### Alerting Thresholds
- >5 failed auth attempts in 1 minute → Alert
- >10 rate limit hits → Alert
- Any secret detection → **Critical** Alert
- Any calldata rejection → Log and review

## Incident Response

### Private Key Compromise
1. Rotate key immediately
2. Audit all x402 requests since last rotation
3. Revoke affected wallet access
4. Review security logs
5. Update key management procedures

### Malicious Calldata Detected
1. Block the transaction
2. Log full calldata details
3. Investigate Venice AI response
4. Update whitelist if needed
5. Report to Venice AI team

### Secret Leak Prevention
1. Block the pipeline at Ram Gate
2. Scan all artifacts with gitleaks/trufflehog
3. Rotate any exposed credentials
4. Update artifact templates
5. Add new patterns to Ram Gate

## Compliance

### OWASP Top 10 Coverage

| Category | Status | Notes |
|----------|--------|-------|
| A01 Broken Access Control | ✅ | Rate limiting, authentication |
| A02 Cryptographic Failures | ✅ | AES-256-GCM, Argon2id |
| A03 Injection | ✅ | Parameterized queries, input validation |
| A04 Insecure Design | ⚠️ | Ongoing review |
| A05 Security Misconfiguration | ⚠️ | TODO: Harden configs |
| A06 Vulnerable Components | ✅ | Regular dependency scans |
| A07 Auth Failures | ✅ | x402, key validation |
| A08 Data Integrity | ✅ | Calldata validation, replay protection |
| A09 Logging Failures | ✅ | Structured logging, audit trails |
| A10 SSRF | ✅ | URL validation, whitelist |

## Future Security Enhancements

- [ ] Hardware security module (HSM) integration
- [ ] Multi-key threshold signing
- [ ] Real Redis-based replay protection
- [ ] gitleaks/trufflehog integration for Ram Gate
- [ ] Automated secret scanning on commit
- [ ] Security-focused fuzz testing
- [ ] Third-party security audit

## References

- [OWASP Top 10 2021](https://owasp.org/www-project-top-ten/)
- [NIST Cryptographic Standards](https://csrc.nist.gov/)
- [Argon2 Specification](https://argon2.online/)
- [AES-GCM Documentation](https://datatracker.ietf.org/doc/html/rfc5116)
