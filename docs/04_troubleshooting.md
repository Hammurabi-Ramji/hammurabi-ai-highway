# Troubleshooting Guide

## Common Issues and Solutions

### Issue 1: Private Key Loading Errors

**Symptoms:**
```
Error: Invalid HAMMURABI_PRIVATE_KEY — expected a 0x-prefixed secp256k1 hex key
```

**Causes:**
- Private key has invalid format (not 64 hex characters)
- Missing `0x` prefix
- Contains non-hexadecimal characters

**Solutions:**
1. Verify key format:
   ```bash
   # Should be exactly 66 characters (0x + 64 hex chars)
   echo $HAMMURABI_PRIVATE_KEY | wc -c
   ```

2. Use a valid private key:
   ```bash
   # Generate new test key (DO NOT USE WITH REAL FUNDS)
   openssl rand -hex 32
   
   # Or use ethers.js
   npx ethers | grep privateKey
   ```

3. Update `.env` file:
   ```env
   HAMMURABI_PRIVATE_KEY=0x0000000000000000000000000000000000000000000000000000000000000000
   ```

---

### Issue 2: Calldata Validation Blocks Transaction

**Symptoms:**
```
CalldataValidationFailed: Address not whitelisted: 0x...
```

**Causes:**
- Target contract not in whitelist
- Contract address typo
- Using non-Base network

**Solutions:**
1. Check contract address:
   ```rust
   // In src/validation/calldata_verifier.rs
   let whitelist = self.approved_contracts.read().await;
   // Check if address is in 8453 network whitelist
   ```

2. Add contract to whitelist:
   ```rust
   approved_contracts.insert(
       8453,
       BTreeSet::from_iter(vec![
           "0xnew_contract_address".to_string(),
           // ... other addresses
       ]),
   );
   ```

3. Verify network:
   - CLI uses Base (chain ID 8453) by default
   - Update `base_chain_id` in config if using different network

---

### Issue 3: Venice AI Request Timeout

**Symptoms:**
```
Venice AI request failed (network error)
```

**Causes:**
- Network connectivity issues
- Venice API rate limiting
- Invalid API key

**Solutions:**
1. Check API key:
   ```bash
   echo $VENICE_API_KEY
   # Should not be empty
   ```

2. Test Venice API manually:
   ```bash
   curl -X POST https://api.venice.ai/api/v1/health \
     -H "Authorization: Bearer $VENICE_API_KEY"
   ```

3. Check network:
   ```bash
   ping api.venice.ai
   ```

4. Increase timeout:
   ```rust
   // In src/main.rs
   let client = reqwest::Client::builder()
       .timeout(std::time::Duration::from_secs(30)) // Increase from 20
       .build()?;
   ```

---

### Issue 4: 1Shot Relay Fails

**Symptoms:**
```
1Shot API returned HTTP 401: Unauthorized
```

**Causes:**
- Invalid 1Shot API key
- Expired wallet credentials
- Network issues

**Solutions:**
1. Regenerate API key:
   - Log in to 1Shot dashboard
   - Generate new API key
   - Update `.env` file

2. Check wallet status:
   ```bash
   # Check wallet on Base network
   # Use Etherscan to verify wallet exists
   ```

3. Verify network connectivity:
   ```bash
   curl -X POST https://api.1shotapi.com/v1/execute \
     -H "Authorization: Bearer $ONESHOT_API_KEY"
   ```

---

### Issue 5: Eduba Database Errors

**Symptoms:**
```
Database error: I/O error: database is locked
```

**Causes:**
- SQLite WAL mode conflict
- File permission issues
- Multiple processes accessing database

**Solutions:**
1. Check file permissions:
   ```bash
   ls -la eduba_registry.db
   # Should be readable and writable
   ```

2. Reset database:
   ```bash
   mv eduba_registry.db eduba_registry.db.backup
   # CLI will recreate on next run
   ```

3. Use different database path:
   ```bash
   EDUBA_DB_PATH=/tmp/hammurabi_cache.db hammurabi -- run --intent "test"
   ```

---

### Issue 6: Pipeline Hangs at Stage

**Symptoms:**
```
-- Lane X/Y: StageName ...
(pipeline hangs indefinitely)
```

**Causes:**
- Stage waiting for network timeout
- Deadlock in async runtime
- Resource exhaustion

**Solutions:**
1. Check timeout settings:
   ```rust
   // In src/main.rs or src/pipeline/mod.rs
   let http = reqwest::Client::builder()
       .timeout(std::time::Duration::from_secs(20))
       .build()?;
   ```

2. Enable verbose logging:
   ```bash
   RUST_LOG=debug hammurabi -- pipeline --intent "test"
   ```

3. Check resource usage:
   ```bash
   # Monitor memory usage
   top -p $(pgrep hammurabi)
   ```

---

### Issue 7: High Risk Score on Calldata Validation

**Symptoms:**
```
Calldata validation failed: High risk score: 85
```

**Causes:**
- Transaction value too high
- Non-whitelisted contract
- Complex transaction (large data size)
- Unusual transaction pattern

**Solutions:**
1. Check risk factors:
   - Contract whitelisted?
   - Transaction value reasonable?
   - Data size < 1000 bytes?

2. Adjust validator configuration:
   ```rust
   // In CalldataVerifier
   let max_value = Arc::new(RwLock::new(1_000_000_000_000_000_000_000u128)); // Increase limit
   ```

3. Review contract security:
   - Audit contract before use
   - Start with small test transactions

---

### Issue 8: Secret Detection in Ram Gate

**Symptoms:**
```
Ram Gate blocked egress: potential secret detected
```

**Causes:**
- Generated code contains sensitive patterns
- Comments include API keys or secrets
- Configuration includes credentials

**Solutions:**
1. Review artifacts:
   ```bash
   # Check generated files
   cat src/execution/onchain.rs
   ```

2. Update artifact templates:
   - Remove sensitive patterns from templates
   - Use environment variable placeholders

3. Disable detection (not recommended):
   ```rust
   // In src/pipeline/ram_gate.rs
   // Modify risk scoring to be less aggressive
   ```

---

### Issue 9: Docker Deployment Fails

**Symptoms:**
```
docker run: Error response from daemon: driver failed programming external connectivity
```

**Causes:**
- Port conflicts
- Permission issues
- Volume mount problems

**Solutions:**
1. Check port availability:
   ```bash
   lsof -i :8080
   ```

2. Fix volume permissions:
   ```bash
   mkdir -p data
   chmod 755 data
   docker run -v $(pwd)/data:/data ...
   ```

3. Use different network:
   ```bash
   docker network create hammurabi-net
   docker run --network hammurabi-net ...
   ```

---

### Issue 10: K8s Deployment Fails

**Symptoms:**
```
Error: configmap "hammurabi-config" not found
```

**Causes:**
- ConfigMap not created
- Namespace mismatch
- Secret not applied

**Solutions:**
1. Create ConfigMap:
   ```bash
   kubectl create configmap hammurabi-config \
     --from-env-file=.env
   ```

2. Check namespace:
   ```bash
   kubectl config view --minify | grep namespace
   kubectl get configmaps -n <namespace>
   ```

3. Apply secrets:
   ```bash
   kubectl apply -f secrets.yaml
   ```

---

## Performance Troubleshooting

### Slow Pipeline Execution

**Symptoms:**
- Pipeline takes >10 seconds
- Individual stages slow

**Diagnosis:**
```bash
# Enable timing logs
RUST_LOG=trace hammurabi -- pipeline --intent "test"

# Check stage timings
grep "done" logs/pipeline.log
```

**Solutions:**
1. Enable caching:
   ```bash
   # Use existing Eduba cache
   hammurabi -- pipeline --intent "repeated-intent"
   ```

2. Reduce external dependencies:
   - Cache RamGenie results
   - Use local fallbacks

3. Optimize network:
   - Use CDN for dependencies
   - Enable connection pooling

### Memory Leaks

**Symptoms:**
- Memory usage grows over time
- Application crashes with OOM

**Diagnosis:**
```bash
# Monitor memory
watch -n 1 "ps aux | grep hammurabi"

# Check for goroutine leaks
pprof -http=:8080 http://localhost:8080/debug/pprof/goroutine
```

**Solutions:**
1. Reduce heap allocation
2. Close database connections
3. Clear caches periodically

---

## Debug Tools

### Enable Debug Logging

```bash
RUST_LOG=debug hammurabi -- pipeline --intent "test"
```

Available log levels:
- `error` - Errors only
- `warn` - Warnings and errors
- `info` - Normal operations (default)
- `debug` - Detailed debugging
- `trace` - Very detailed tracing

### Generate Crash Reports

```bash
# Enable backtraces
RUST_BACKTRACE=1 hammurabi -- run --intent "test"

# Save to file
RUST_BACKTRACE=full hammurabi -- pipeline --intent "test" > crash.log 2>&1
```

### Network Debugging

```bash
# Enable HTTP debug
RUST_LOG=reqwest=debug hammurabi -- run --intent "test"

# Capture traffic
mitmproxy &
export HTTP_PROXY=http://127.0.0.1:8080
```

---

## Contact Support

- **Documentation Issues**: Check [docs/](docs/) directory
- **Security Issues**: See [Security Guide](docs/02_security_guide.md)
- **General Support**: GitHub Issues
- **Emergency**: Email security@hammurabi-coding.company
