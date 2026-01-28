# Secure Deployment Guide

## Overview

This guide covers secure deployment practices for Inferno in production environments.

## Pre-Deployment Checklist

### Environment Setup

1. **Create a dedicated service user**:
   ```bash
   useradd -r -s /bin/false inferno
   ```

2. **Set up secure directories**:
   ```bash
   mkdir -p /opt/inferno/{bin,config,data,models,logs}
   chown -R inferno:inferno /opt/inferno
   chmod 750 /opt/inferno
   chmod 700 /opt/inferno/config
   ```

3. **Configure environment variables**:
   ```bash
   # /etc/inferno/env (chmod 600)
   INFERNO_JWT_SECRET=<generated-secret>
   INFERNO_ADMIN_PASSWORD=<generated-password>
   INFERNO_LOG_LEVEL=info
   INFERNO_MODELS_DIR=/opt/inferno/models
   ```

## Docker Deployment

### Secure Dockerfile

```dockerfile
FROM rust:1.75-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN useradd -r -s /bin/false inferno
COPY --from=builder /app/target/release/inferno /usr/local/bin/
USER inferno
EXPOSE 8080
CMD ["inferno", "serve"]
```

### Docker Compose with Security Options

```yaml
version: '3.8'
services:
  inferno:
    image: inferno:latest
    user: "1000:1000"
    read_only: true
    security_opt:
      - no-new-privileges:true
    cap_drop:
      - ALL
    environment:
      - INFERNO_JWT_SECRET_FILE=/run/secrets/jwt_secret
      - INFERNO_ADMIN_PASSWORD_FILE=/run/secrets/admin_password
    secrets:
      - jwt_secret
      - admin_password
    volumes:
      - models:/opt/inferno/models:ro
      - data:/opt/inferno/data
    tmpfs:
      - /tmp:size=100M,mode=1777
    networks:
      - inferno-net
    deploy:
      resources:
        limits:
          memory: 16G
          cpus: '4'

secrets:
  jwt_secret:
    external: true
  admin_password:
    external: true

networks:
  inferno-net:
    driver: bridge
```

## Kubernetes Deployment

### Secure Pod Configuration

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: inferno
spec:
  securityContext:
    runAsNonRoot: true
    runAsUser: 1000
    fsGroup: 1000
  containers:
  - name: inferno
    image: inferno:latest
    securityContext:
      allowPrivilegeEscalation: false
      readOnlyRootFilesystem: true
      capabilities:
        drop:
          - ALL
    resources:
      limits:
        memory: "16Gi"
        cpu: "4"
      requests:
        memory: "4Gi"
        cpu: "1"
    env:
    - name: INFERNO_JWT_SECRET
      valueFrom:
        secretKeyRef:
          name: inferno-secrets
          key: jwt-secret
    - name: INFERNO_ADMIN_PASSWORD
      valueFrom:
        secretKeyRef:
          name: inferno-secrets
          key: admin-password
    volumeMounts:
    - name: models
      mountPath: /opt/inferno/models
      readOnly: true
    - name: data
      mountPath: /opt/inferno/data
    - name: tmp
      mountPath: /tmp
  volumes:
  - name: models
    persistentVolumeClaim:
      claimName: inferno-models
  - name: data
    persistentVolumeClaim:
      claimName: inferno-data
  - name: tmp
    emptyDir:
      sizeLimit: 100Mi
```

## Reverse Proxy Configuration

### Nginx with TLS

```nginx
server {
    listen 443 ssl http2;
    server_name inferno.example.com;

    ssl_certificate /etc/ssl/certs/inferno.crt;
    ssl_certificate_key /etc/ssl/private/inferno.key;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256;
    ssl_prefer_server_ciphers off;

    # Security headers
    add_header Strict-Transport-Security "max-age=63072000" always;
    add_header X-Content-Type-Options nosniff;
    add_header X-Frame-Options DENY;
    add_header X-XSS-Protection "1; mode=block";

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Rate limiting
        limit_req zone=inferno burst=20 nodelay;
    }
}
```

## Monitoring and Alerting

### Log Aggregation

Configure log forwarding for security monitoring:

```toml
[logging]
format = "json"
level = "info"
file = "/var/log/inferno/inferno.log"
```

### Security Alerts

Monitor for:
- Failed authentication attempts (>10 in 5 minutes)
- Rate limit violations
- Unusual API patterns
- Error rate spikes

## Backup and Recovery

### Data Backup

```bash
# Backup script
#!/bin/bash
BACKUP_DIR=/opt/backups/inferno
DATE=$(date +%Y%m%d_%H%M%S)

# Backup data (encrypt sensitive files)
tar -czf - /opt/inferno/data | \
  openssl enc -aes-256-cbc -salt -pbkdf2 -out $BACKUP_DIR/data_$DATE.tar.gz.enc

# Backup config (excluding secrets)
cp /opt/inferno/config/config.toml $BACKUP_DIR/config_$DATE.toml
```

## Security Updates

### Update Process

1. Review release notes for security fixes
2. Test update in staging environment
3. Backup current installation
4. Apply update during maintenance window
5. Verify functionality and security settings
6. Monitor for issues

```bash
# Update command
cargo install inferno --version X.Y.Z
```
