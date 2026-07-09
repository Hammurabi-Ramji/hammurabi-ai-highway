# Deployment Guide

## Overview

This guide covers deploying the Hammurabi AI Highway CLI in various environments.

## Quick Start

### Development (Local)

```bash
# Clone repository
git clone <repo-url>
cd hammurabiOS

# Copy environment template
cp .env.example .env

# Edit .env with your credentials
# - VENICE_API_KEY
# - ONESHOT_API_KEY  
# - ONESHOT_WALLET_ID
# - HAMMURABI_PRIVATE_KEY

# Build
cargo build --release

# Run
./target/release/hammurabi -- pipeline --intent "Test"
```

### Docker Deployment

#### Build Docker Image

```dockerfile
# Dockerfile
FROM rust:1.75 AS builder

WORKDIR /app

# Copy dependency files first for caching
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch

# Copy source
COPY src ./src

# Build
RUN cargo build --release

# Runtime image
FROM debian:booklet-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/hammurabi .
COPY --from=builder /app/.env.example .env

ENV EDUBA_DB_PATH=/data/eduba.db

ENTRYPOINT ["./hammurabi"]
```

```bash
# Build image
docker build -t hammurabi-cli .

# Run container
docker run -d \
  --name hammurabi-cli \
  -v $(pwd)/data:/data \
  -e VENICE_API_KEY=$VENICE_API_KEY \
  -e ONESHOT_API_KEY=$ONESHOT_API_KEY \
  -e ONESHOT_WALLET_ID=$ONESHOT_WALLET_ID \
  -e HAMMURABI_PRIVATE_KEY=$HAMMURABI_PRIVATE_KEY \
  hammurabi-cli -- run --intent "Swap 10 USDC for VVV"
```

### Kubernetes Deployment

#### ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: hammurabi-config
data:
  VENICE_API_KEY: "your-venice-key"
  ONESHOT_API_KEY: "your-oneshot-key"
  ONESHOT_WALLET_ID: "your-wallet-id"
  HAMMURABI_PRIVATE_KEY: "your-private-key"
  EDUBA_DB_PATH: "/data/eduba.db"
```

#### Secret (for sensitive values)

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: hammurabi-secrets
type: Opaque
stringData:
  HAMMURABI_PRIVATE_KEY: "your-encrypted-private-key"
  VENICE_API_KEY: "your-venice-key"
  ONESHOT_API_KEY: "your-oneshot-key"
```

#### Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: hammurabi-cli
spec:
  replicas: 1
  selector:
    matchLabels:
      app: hammurabi-cli
  template:
    metadata:
      labels:
        app: hammurabi-cli
    spec:
      containers:
      - name: hammurabi
        image: hammurabi-cli:latest
        args: ["--pipeline", "--intent", "Deployment task"]
        envFrom:
        - configMapRef:
            name: hammurabi-config
        - secretRef:
            name: hammurabi-secrets
        volumeMounts:
        - name: data
          mountPath: /data
      volumes:
      - name: data
        persistentVolumeClaim:
          claimName: hammurabi-data
```

#### PersistentVolumeClaim

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: hammurabi-data
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 1Gi
```

### VPS Deployment (ZAP Hosting)

```bash
# SSH into VPS
ssh root@your-vps-ip

# Install dependencies
apt update
apt install -y curl git

# Download release
curl -LO https://github.com/hammurabi-coding/hammurabi-cli/releases/latest/download/hammurabi-linux-x86_64.tar.gz
tar -xzf hammurabi-linux-x86_64.tar.gz
chmod +x hammurabi

# Create systemd service
cat > /etc/systemd/system/hammurabi.service << EOF
[Unit]
Description=Hammurabi AI Highway CLI
After=network.target

[Service]
Type=simple
User=hammurabi
WorkingDirectory=/opt/hammurabi
ExecStart=/opt/hammurabi/hammurabi -- pipeline --interval=3600
Restart=always
RestartSec=30

Environment="VENICE_API_KEY=${VENICE_API_KEY}"
Environment="ONESHOT_API_KEY=${ONESHOT_API_KEY}"
Environment="ONESHOT_WALLET_ID=${ONESHOT_WALLET_ID}"
Environment="HAMMURABI_PRIVATE_KEY=${HAMMURABI_PRIVATE_KEY}"
Environment="EDUBA_DB_PATH=/opt/hammurabi/data/eduba.db"

[Install]
WantedBy=multi-user.target
EOF

# Start service
systemctl daemon-reload
systemctl enable hammurabi
systemctl start hammurabi

# Monitor logs
journalctl -u hammurabi -f
```

## Production Checklist

- [ ] All secrets in secure storage (not plaintext)
- [ ] Private key in encrypted storage
- [ ] Calldata validator whitelist configured
- [ ] Rate limits configured appropriately
- [ ] Logging enabled for all stages
- [ ] Backup strategy for Eduba database
- [ ] Monitoring and alerting configured
- [ ] Rollback plan documented

## Monitoring

### Logs

```bash
# Application logs
tail -f logs/app.log

# Security events
grep "SECURITY" logs/security.log

# Pipeline runs
grep "Pipeline complete" logs/pipeline.log
```

### Metrics

```bash
# Check Eduba database size
du -sh data/eduba_registry.db

# Check disk usage
df -h

# Check process memory
ps aux | grep hammurabi
```

## Upgrades

### Rolling Update (Kubernetes)

```bash
# Update image
kubectl set image deployment/hammurabi-cli hammurabi=hammurabi-cli:v1.1.0

# Check rollout
kubectl rollout status deployment/hammurabi-cli

# If issues, rollback
kubectl rollout undo deployment/hammurabi-cli
```

### Database Backup

```bash
# Backup Eduba database
cp data/eduba_registry.db data/eduba_registry.db.backup.$(date +%Y%m%d)

# Restore
cp data/eduba_registry.db.backup.20260616 data/eduba_registry.db
```

## Troubleshooting

### Common Issues

**1. x402 authentication fails**
```
Error: Failed to sign x402 auth payload
```
**Fix:** Check HAMMURABI_PRIVATE_KEY format - should be 0x-prefixed 64-char hex

**2. Calldata validation blocks transaction**
```
CalldataValidationFailed: Address not whitelisted
```
**Fix:** Add contract address to whitelist in CalldataVerifier

**3. Pipeline hangs at Venice AI stage**
```
Venice AI request failed (network error)
```
**Fix:** Check network connectivity, verify VENICE_API_KEY is valid

**4. Database errors**
```
Database error: I/O error: database is locked
```
**Fix:** SQLite WAL mode should handle this; check file permissions

## Security Best Practices

1. **Rotate private keys** every 90 days
2. **Use dedicated wallet** for platform operations
3. **Monitor x402 auth** for suspicious patterns
4. **Regular security audits** of artifacts
5. **Enable audit logging** for all security events
6. **Use HTTPS** for all API communications
7. **Implement rate limiting** to prevent abuse

## Support

- Documentation: [docs/01_architecture.md](docs/01_architecture.md)
- Security Guide: [docs/02_security_guide.md](docs/02_security_guide.md)
- Issues: GitHub Issues
- Community: GitHub Discussions
