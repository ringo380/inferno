# Inferno Audit System - Advanced Features

The Inferno audit system provides enterprise-grade logging, monitoring, and alerting capabilities with comprehensive security features.

## 🚀 Features Implemented

### ✅ Compression Support
- **Gzip compression** with configurable compression levels (1-9)
- **Zstd compression** with configurable compression levels (1-22)
- **Automatic compression** during log file creation
- **Decompression utilities** for reading compressed audit logs
- **Significant storage savings** (typically 70-90% reduction)

### ✅ Encryption Support
- **AES-256-GCM encryption** for sensitive audit data
- **Secure key management** via environment variables
- **Nonce-based encryption** for maximum security
- **Selective encryption** (sensitive fields only or full content)
- **Key generation utilities** for easy setup

### ✅ Advanced Alerting
- **Multi-channel alerting**: Webhook, Email, Slack
- **Rate limiting** to prevent alert spam
- **Custom alert templates** with variable substitution
- **Configurable alert conditions** with severity thresholds
- **Retry mechanisms** with exponential backoff
- **Rich alert formatting** with context information

## 📋 Configuration

### Basic Configuration
```toml
[audit]
enabled = true
compression_enabled = true
compression_method = "Gzip"  # "None", "Gzip", "Zstd"
compression_level = 6
encryption_enabled = true
encryption_key_env = "INFERNO_AUDIT_ENCRYPTION_KEY"
```

### Alerting Configuration
```toml
[audit.alerting]
enabled = true
rate_limit_per_hour = 60

[audit.alerting.webhook]
enabled = true
url = "https://your-webhook.com/alerts"
timeout_seconds = 30
retry_count = 3

[audit.alerting.email]
enabled = true
smtp_host = "smtp.company.com"
smtp_port = 587
from_address = "audit@company.com"
to_addresses = ["security@company.com"]

[audit.alerting.slack]
enabled = true
webhook_url = "https://hooks.slack.com/services/..."
channel = "#security-alerts"
```

## 🔧 Environment Variables

### Required for Encryption
```bash
# Generate a 256-bit encryption key
export INFERNO_AUDIT_ENCRYPTION_KEY=$(openssl rand -base64 32)
```

### Required for Email Alerts
```bash
export INFERNO_SMTP_PASSWORD="your-smtp-password"
```

## 💻 Usage Examples

### Programmatic Usage
```rust
use inferno::audit::*;

// Create configuration with compression and encryption
let config = AuditConfiguration {
    compression_enabled: true,
    compression_method: CompressionMethod::Gzip,
    encryption_enabled: true,
    alerting: AlertConfiguration {
        enabled: true,
        webhook: WebhookConfig {
            enabled: true,
            url: "https://alerts.company.com/webhook".to_string(),
            // ...
        },
        // ...
    },
    // ...
};

// Initialize audit logger
let logger = AuditLogger::new(config).await?;

// Log events (automatically compressed and encrypted)
logger.log_event(audit_event).await?;

// Query with filtering
let events = logger.query_events(AuditQuery {
    severities: Some(vec![Severity::Critical]),
    limit: Some(100),
    // ...
}).await?;
```

### CLI Usage
```bash
# Enable audit logging with compression
inferno audit configure --enable true --compression true

# Show current configuration
inferno audit configure --show

# Query events
inferno audit query --severities critical --limit 50

# Export logs (OUTPUT is positional)
inferno audit export audit_export.json --format json
```

Encryption and alerting are configured via the config file (see the sections above), not through CLI subcommands.

## 🔐 Security Features

### Encryption
- **AES-256-GCM**: Industry-standard authenticated encryption
- **Unique nonces**: Every encryption operation uses a unique nonce
- **Key rotation**: Support for key rotation via environment variables
- **Selective encryption**: Choose between full content or sensitive fields only

### Secure Key Management
- **Environment variable storage**: Keys never stored in code or config files
- **Base64 encoding**: Safe key transmission and storage
- **Key validation**: Automatic validation of key format and length
- **Development warnings**: Clear warnings when using generated keys

### Access Control
- **File permissions**: Audit logs use restrictive file permissions
- **Network security**: TLS/SSL for all external communications
- **Input validation**: Comprehensive validation of all inputs
- **Error handling**: Secure error handling that doesn't leak sensitive data

## 📊 Performance

### Compression Benchmarks
- **Gzip**: 70-85% size reduction, fast compression/decompression
- **Zstd**: 75-90% size reduction, excellent performance balance
- **Memory usage**: Streaming compression minimizes memory overhead
- **CPU impact**: Configurable compression levels balance speed vs. size

### Encryption Performance
- **AES-256-GCM**: Hardware-accelerated on modern CPUs
- **Minimal overhead**: ~16 bytes overhead per encrypted block
- **Streaming encryption**: Large files encrypted efficiently
- **Key caching**: Encryption keys cached for performance

## 🚨 Alert Examples

### Webhook Alert Payload
```json
{
  "alert_type": "audit_event",
  "severity": "Critical",
  "event_type": "SecurityEvent",
  "timestamp": "2024-01-15T10:30:00Z",
  "hostname": "inferno-prod-01",
  "environment": "production",
  "event": {
    "id": "evt-12345",
    "action": "unauthorized_access",
    "actor": "unknown@suspicious.com",
    "resource": "admin_api",
    "description": "Failed authentication attempt",
    "success": false
  }
}
```

### Email Alert Format
```
Subject: [Critical] Audit Alert: unauthorized_access on inferno-prod-01

Event ID: evt-12345
Severity: Critical
Event Type: SecurityEvent
Actor: unknown@suspicious.com
Resource: admin_api (Api)
Action: unauthorized_access
Success: ❌
Error: Authentication failed

Description: Failed authentication attempt

Context:
- Application: inferno
- Version: 1.0.0
- Environment: production
- Duration: 50ms
```

### Slack Alert Format
Rich Slack message with colored attachments, fields for key information, and actionable buttons.

## 🛠️ Troubleshooting

### Common Issues

#### Compression Not Working
```bash
# Check configuration
inferno audit configure --show

# Verify permissions
ls -la ./logs/audit/
```

#### Encryption Key Issues
```bash
# Verify key format (should be 44 characters base64)
echo $INFERNO_AUDIT_ENCRYPTION_KEY | wc -c

# Generate a new key
export INFERNO_AUDIT_ENCRYPTION_KEY=$(openssl rand -base64 32)
```

#### Alert Delivery Problems
```bash
# Test webhook connectivity
curl -X POST "https://your-webhook.com/test"
```

### Debugging

Enable debug logging:
```bash
export INFERNO_LOG_LEVEL=debug
export INFERNO_LOG_FORMAT=json
```

Check audit statistics:
```bash
inferno audit stats
```

## 📈 Monitoring

### Key Metrics
- **Compression ratio**: Monitor storage savings
- **Encryption overhead**: Track performance impact
- **Alert delivery rate**: Ensure alerts reach destinations
- **Event processing latency**: Monitor audit system performance
- **Error rates**: Track failed audit operations

### Integrity Checks
```bash
# Validate log integrity (check for gaps and verify timestamps)
inferno audit validate --check-gaps --verify-timestamps
```

## 🔄 Migration

### From Previous Versions
1. **Backup existing logs**: `cp -r logs/audit logs/audit.backup`
2. **Update configuration**: Add new compression/encryption settings to the config file
3. **Set environment variables**: Configure encryption keys
4. **Validate integrity**: `inferno audit validate`
5. **Gradual rollout**: Enable features incrementally

### Configuration Validation
```bash
# Validate the config file
inferno config validate
```

## 📚 Best Practices

1. **Key Rotation**: Rotate encryption keys regularly
2. **Compression Tuning**: Balance compression level with performance
3. **Alert Filtering**: Configure appropriate alert thresholds
4. **Log Retention**: Set appropriate retention periods
5. **Monitoring**: Monitor audit system health continuously
6. **Testing**: Regularly test alert delivery mechanisms
7. **Backup**: Backup audit logs and configuration
8. **Documentation**: Document custom alert conditions and templates

## 🔗 Related Documentation

- [Security Guide](SECURITY.md)
- [Configuration Reference](CONFIG.md)
- [API Documentation](API.md)
- [Deployment Guide](DEPLOYMENT.md)
- [Troubleshooting Guide](TROUBLESHOOTING.md)