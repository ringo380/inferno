# Audit System and Batch Processing Guide

This guide covers Inferno's enterprise-grade audit logging system and comprehensive batch processing capabilities with cron scheduling, retry logic, and monitoring.

## Audit System Overview

The audit system provides comprehensive logging of all operations, security events, and system changes with encryption, compression, and multi-channel alerting capabilities.

### Key Features

- **Comprehensive Logging**: All operations, access attempts, and configuration changes
- **Encryption**: AES-256 encryption for sensitive audit data
- **Compression**: Efficient storage with configurable algorithms
- **Multi-channel Alerting**: Real-time notifications via email, Slack, webhooks
- **Compliance Reports**: Automated generation of audit reports
- **Tamper Detection**: Cryptographic integrity validation
- **Retention Policies**: Configurable data retention and archival

## Audit System Configuration

### Enable Audit Logging

```bash
# Enable audit system with encryption
inferno audit enable --encryption

# Enable with specific channels
inferno audit enable --channels email,slack,webhook

# Configure audit settings
inferno audit configure \
  --encryption-key-file /secure/audit.key \
  --compression gzip \
  --retention-days 365 \
  --max-size-gb 50
```

### Configuration File

```toml
# .inferno.toml
[audit]
enabled = true
log_level = "info"  # trace, debug, info, warn, error
storage_path = "/var/log/inferno/audit"

[audit.encryption]
enabled = true
algorithm = "aes256"
key_file = "/secure/audit.key"
rotate_keys_days = 90

[audit.compression]
enabled = true
algorithm = "gzip"  # gzip, zstd, lz4
level = 6
threshold_kb = 64

[audit.retention]
max_age_days = 365
max_size_gb = 100
archive_to_s3 = true
cleanup_interval_hours = 24

[audit.alerts]
enabled = true
channels = ["email", "slack", "webhook"]
severity_threshold = "warn"

[audit.alerts.email]
smtp_server = "smtp.company.com"
recipients = ["security@company.com"]
template = "security_alert"

[audit.alerts.slack]
webhook_url = "https://hooks.slack.com/services/..."
channel = "#security-alerts"

[audit.alerts.webhook]
url = "https://api.company.com/security/alerts"
headers = { "Authorization" = "Bearer token" }
```

## Audit Operations

### View Audit Logs

```bash
# View recent audit entries
inferno audit logs

# Filter by user and time
inferno audit logs --user admin --since 24h

# Filter by action type
inferno audit logs --action model_load,inference_request

# Export to file
inferno audit export --format json --output audit_report.json

# Search logs
inferno audit search --query "failed login" --since 7d
```

### Audit Log Management

```bash
# Check audit system status
inferno audit status

# Rotate audit logs
inferno audit rotate

# Verify log integrity
inferno audit verify --check-encryption

# Generate compliance report
inferno audit report --period monthly --format pdf
```

### Alert Configuration

```bash
# Test alert channels
inferno audit test-alerts

# Configure alert rules
inferno audit alert-rule create \
  --name "multiple_failed_logins" \
  --condition "failed_logins > 5 in 10m" \
  --severity high \
  --channels email,slack

# List alert rules
inferno audit alert-rules list

# Disable specific alerts
inferno audit alert-rule disable multiple_failed_logins
```

## Audit Event Types

### Authentication Events

```json
{
  "timestamp": "2024-03-15T10:30:00Z",
  "event_type": "authentication",
  "action": "login_success",
  "user": "admin",
  "source_ip": "192.168.1.100",
  "user_agent": "InfernoClient/1.0",
  "session_id": "sess_123456",
  "details": {
    "auth_method": "api_key",
    "permissions": ["admin:read", "admin:write"]
  }
}
```

### Model Operations

```json
{
  "timestamp": "2024-03-15T10:30:00Z",
  "event_type": "model_operation",
  "action": "model_load",
  "user": "service_account",
  "resource": "llama-7b-q4.gguf",
  "details": {
    "model_size_mb": 3800,
    "backend_type": "gguf",
    "load_time_ms": 2500,
    "gpu_enabled": true
  },
  "result": "success"
}
```

### Security Events

```json
{
  "timestamp": "2024-03-15T10:30:00Z",
  "event_type": "security",
  "action": "access_denied",
  "user": "guest",
  "source_ip": "192.168.1.200",
  "resource": "/admin/config",
  "details": {
    "required_permission": "admin:read",
    "user_permissions": ["inference:read"],
    "reason": "insufficient_permissions"
  },
  "severity": "medium"
}
```

### System Events

```json
{
  "timestamp": "2024-03-15T10:30:00Z",
  "event_type": "system",
  "action": "config_change",
  "user": "admin",
  "resource": "server_config",
  "details": {
    "changes": {
      "max_concurrent_requests": {"old": 50, "new": 100},
      "cache_enabled": {"old": false, "new": true}
    }
  },
  "result": "success"
}
```

## Batch Processing System

The batch processing system provides production-ready job management with cron scheduling, retry logic, and comprehensive monitoring.

### Key Features

- **Cron Scheduling**: Full cron expression support for complex schedules
- **Job Queue**: Persistent job queue with priority handling
- **Retry Logic**: Configurable retry policies with exponential backoff
- **Resource Management**: CPU and memory limits per job
- **Monitoring**: Real-time job status and progress tracking
- **Dependencies**: Support for job chains and dependencies

## Batch Queue Configuration

### Enable Batch Processing

```bash
# Enable batch queue system
inferno batch-queue enable

# Configure queue settings
inferno batch-queue configure \
  --max-concurrent 5 \
  --retry-attempts 3 \
  --default-timeout 3600
```

### Configuration File

```toml
[batch_queue]
enabled = true
storage_path = "/var/lib/inferno/queue"
max_concurrent_jobs = 5
default_timeout_seconds = 3600

[batch_queue.retry]
max_attempts = 3
base_delay_seconds = 30
max_delay_seconds = 3600
backoff_multiplier = 2.0

[batch_queue.resources]
default_cpu_limit = 2.0
default_memory_limit_gb = 4.0
monitor_interval_seconds = 30

[batch_queue.scheduling]
enable_cron = true
timezone = "UTC"
max_scheduled_jobs = 100

[batch_queue.notifications]
enabled = true
channels = ["email", "webhook"]
notify_on = ["job_failed", "job_completed"]
```

## Batch Job Operations

### Create and Manage Jobs

```bash
# Create a simple batch job
inferno batch-queue create \
  --name "model_validation" \
  --command "validate /models/*.gguf" \
  --timeout 1800

# Create scheduled job with cron expression
inferno batch-queue create \
  --name "nightly_conversion" \
  --command "convert model /staging/model.pt /production/model.gguf" \
  --schedule "0 2 * * *" \
  --enabled

# Create job with dependencies
inferno batch-queue create \
  --name "post_processing" \
  --command "optimize /production/model.gguf" \
  --depends-on "nightly_conversion"

# Create job with resource limits
inferno batch-queue create \
  --name "heavy_processing" \
  --command "benchmark --model large-model.gguf" \
  --cpu-limit 4.0 \
  --memory-limit 8GB \
  --priority high
```

### Job Management

```bash
# List all jobs
inferno batch-queue list

# List running jobs
inferno batch-queue list --status running

# View job details
inferno batch-queue show job_123456

# Cancel a job
inferno batch-queue cancel job_123456

# Retry a failed job
inferno batch-queue retry job_123456

# Pause/resume queue
inferno batch-queue pause
inferno batch-queue resume
```

### Job Monitoring

```bash
# Monitor job progress
inferno batch-queue monitor job_123456

# View job logs
inferno batch-queue logs job_123456

# Get queue statistics
inferno batch-queue stats

# Export job history
inferno batch-queue export --format csv --output jobs.csv
```

## Cron Scheduling

### Cron Expression Format

```
┌───────────── minute (0 - 59)
│ ┌─────────── hour (0 - 23)
│ │ ┌───────── day of month (1 - 31)
│ │ │ ┌─────── month (1 - 12)
│ │ │ │ ┌───── day of week (0 - 6) (Sunday to Saturday)
│ │ │ │ │
* * * * *
```

### Common Cron Examples

```bash
# Every minute
inferno batch-queue create --schedule "* * * * *"

# Every hour at minute 0
inferno batch-queue create --schedule "0 * * * *"

# Daily at 2 AM
inferno batch-queue create --schedule "0 2 * * *"

# Weekly on Sunday at 3 AM
inferno batch-queue create --schedule "0 3 * * 0"

# Monthly on the 1st at midnight
inferno batch-queue create --schedule "0 0 1 * *"

# Every 15 minutes
inferno batch-queue create --schedule "*/15 * * * *"

# Weekdays at 9 AM
inferno batch-queue create --schedule "0 9 * * 1-5"

# Custom: Every 2 hours between 9 AM and 5 PM on weekdays
inferno batch-queue create --schedule "0 9-17/2 * * 1-5"
```

### Advanced Scheduling

```bash
# Multiple schedules for the same job
inferno batch-queue create \
  --name "multi_schedule_job" \
  --command "maintenance" \
  --schedule "0 2 * * *" \
  --additional-schedule "0 14 * * 6,0"

# Conditional scheduling
inferno batch-queue create \
  --name "conditional_job" \
  --command "cleanup if disk_usage > 80%" \
  --schedule "0 */4 * * *" \
  --condition "disk_usage_percent > 80"
```

## Job Types and Templates

### Model Processing Jobs

```bash
# Model conversion job
inferno batch-queue create \
  --name "convert_models" \
  --template model_conversion \
  --input-dir /staging/models \
  --output-dir /production/models \
  --format gguf \
  --schedule "0 1 * * *"

# Model validation job
inferno batch-queue create \
  --name "validate_models" \
  --template model_validation \
  --model-dir /production/models \
  --schedule "0 3 * * *"
```

### Inference Batch Jobs

```bash
# Batch inference job
inferno batch-queue create \
  --name "batch_inference" \
  --template batch_inference \
  --model "llama-7b-q4" \
  --input-file prompts.txt \
  --output-file results.json \
  --batch-size 10
```

### Maintenance Jobs

```bash
# Cache cleanup job
inferno batch-queue create \
  --name "cache_cleanup" \
  --template cache_maintenance \
  --max-age 7d \
  --max-size 10GB \
  --schedule "0 4 * * *"

# Log rotation job
inferno batch-queue create \
  --name "log_rotation" \
  --template log_maintenance \
  --max-files 10 \
  --compress true \
  --schedule "0 5 * * 0"
```

## Job Configuration Templates

### Model Conversion Template

```yaml
# templates/model_conversion.yaml
name: "model_conversion"
description: "Convert models between formats"
parameters:
  - name: "input_dir"
    type: "string"
    required: true
  - name: "output_dir"
    type: "string"
    required: true
  - name: "format"
    type: "enum"
    values: ["gguf", "onnx", "pytorch"]
    default: "gguf"
command: |
  for model in ${input_dir}/*; do
    inferno convert model "$model" "${output_dir}/$(basename "$model")" --format ${format}
  done
resources:
  cpu_limit: 2.0
  memory_limit_gb: 4.0
timeout_seconds: 7200
retry:
  max_attempts: 2
  backoff_multiplier: 1.5
```

### Batch Inference Template

```yaml
# templates/batch_inference.yaml
name: "batch_inference"
description: "Process multiple inference requests"
parameters:
  - name: "model"
    type: "string"
    required: true
  - name: "input_file"
    type: "string"
    required: true
  - name: "output_file"
    type: "string"
    required: true
  - name: "batch_size"
    type: "integer"
    default: 10
command: |
  inferno batch \
    --model ${model} \
    --input ${input_file} \
    --output ${output_file} \
    --batch-size ${batch_size}
resources:
  cpu_limit: 4.0
  memory_limit_gb: 8.0
  gpu_required: true
timeout_seconds: 3600
```

## Monitoring and Alerting

### Job Monitoring

```bash
# Real-time job monitoring
inferno batch-queue monitor --real-time

# Performance dashboard
inferno batch-queue dashboard

# Resource usage monitoring
inferno batch-queue resources --job job_123456
```

### Alert Configuration

```bash
# Configure job alerts
inferno batch-queue alert-rule create \
  --name "job_failure_rate" \
  --condition "failure_rate > 0.10 in 1h" \
  --action "notify_admin"

# Configure resource alerts
inferno batch-queue alert-rule create \
  --name "high_resource_usage" \
  --condition "cpu_usage > 90% for 5m" \
  --action "scale_down"
```

## Integration Examples

### Python Batch Client

```python
from inferno_client import BatchQueue
from datetime import datetime, timedelta

# Initialize batch queue client
queue = BatchQueue("http://localhost:8080")

# Create a scheduled job
job = queue.create_job(
    name="daily_model_sync",
    command="rsync -av /staging/models/ /production/models/",
    schedule="0 1 * * *",
    timeout=3600,
    retry_attempts=3
)

# Monitor job progress
def monitor_job(job_id):
    while True:
        status = queue.get_job_status(job_id)
        print(f"Job {job_id}: {status['status']} - {status['progress']}%")

        if status['status'] in ['completed', 'failed', 'cancelled']:
            break

        time.sleep(10)

# Schedule a one-time job
future_time = datetime.now() + timedelta(hours=2)
queue.schedule_job(
    name="future_processing",
    command="process_data --large-dataset",
    run_at=future_time,
    resources={
        "cpu_limit": 8.0,
        "memory_limit_gb": 16.0
    }
)
```

### Kubernetes CronJob Integration

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: inferno-model-sync
spec:
  schedule: "0 2 * * *"
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: model-sync
            image: inferno:latest
            command:
            - inferno
            - batch-queue
            - create
            - --name
            - k8s-model-sync
            - --command
            - sync_models --source s3://models --dest /models
            resources:
              requests:
                cpu: 1
                memory: 2Gi
              limits:
                cpu: 2
                memory: 4Gi
          restartPolicy: OnFailure
```

## Best Practices

### Audit System
- Enable encryption for sensitive audit data
- Configure appropriate retention policies
- Set up multiple alert channels for redundancy
- Regularly verify audit log integrity
- Archive old logs to long-term storage

### Batch Processing
- Use appropriate resource limits for jobs
- Implement proper error handling and logging
- Test cron expressions before deployment
- Monitor job performance and adjust as needed
- Use job dependencies for complex workflows

### Security
- Encrypt audit logs containing sensitive data
- Use secure channels for alert notifications
- Implement proper access controls for job management
- Regularly rotate encryption keys
- Monitor for unusual job patterns or failures

### Performance
- Optimize job resource allocation
- Use compression for large audit logs
- Schedule resource-intensive jobs during off-peak hours
- Monitor queue performance and adjust concurrency limits
- Archive completed job data periodically