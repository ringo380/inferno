# Audit System and Batch Processing Guide

This guide covers Inferno's enterprise-grade audit logging system and comprehensive batch processing capabilities with cron scheduling, retry logic, and monitoring.

## Audit System Overview

The audit system provides comprehensive logging of all operations, security events, and system changes with compression and configurable retention.

### Key Features

- **Comprehensive Logging**: All operations, access attempts, and configuration changes
- **Compression**: Efficient storage with configurable compression
- **Reports**: Generate summary, security, and performance reports
- **Integrity Validation**: Detect gaps and verify timestamps
- **Retention Policies**: Configurable data retention and archival

## Audit System Configuration

Audit settings are applied with `inferno audit configure`. Run `inferno audit configure --show` to print the current configuration.

```bash
# Enable audit logging with compression and a retention policy
inferno audit configure \
  --enable true \
  --compression true \
  --retention-days 365 \
  --max-file-size 50 \
  --max-files 100 \
  --log-level medium-and-above \
  --storage-path /var/log/inferno/audit

# Show the current configuration
inferno audit configure --show
```

Available `configure` flags: `--enable`, `--log-level` (`all`, `critical-only`, `high-and-above`, `medium-and-above`, `low-and-above`, `info-only`), `--storage-path`, `--max-file-size` (MB), `--max-files`, `--retention-days`, `--compression`, `--show`.

## Audit Operations

### Query and View Audit Events

```bash
# Query events with filters
inferno audit query --severities critical --actors admin --limit 50

# Filter by event type and time window
inferno audit query --event-types model-management,api-call --start-time 2024-03-15T00:00:00Z

# Full-text search across logs
inferno audit search "failed login" --limit 50

# Tail recent events (optionally follow)
inferno audit tail --lines 20 --follow

# Export events to a file (OUTPUT is positional)
inferno audit export audit_report.json --format json
```

### Audit Log Management

```bash
# Show audit statistics
inferno audit stats --range-hours 24 --group-by severity

# Validate log integrity
inferno audit validate --check-gaps --verify-timestamps

# Archive old logs to a destination
inferno audit archive --destination /archive/inferno --older-than-days 90 --compression gzip

# Clean up old logs
inferno audit cleanup --older-than-days 30

# Generate a report (types: summary, security, performance, user-activity, system-events, detailed)
inferno audit report security --period-days 30 --format pdf
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

### Create a Queue

Jobs run inside a named queue. Create one, giving it a queue ID and optional limits:

```bash
# Create a queue with concurrency and size limits
inferno queue create \
  --name "Batch Processing" \
  --max-concurrent 5 \
  --max-size 1000 \
  my-queue
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

### Submit and Schedule Jobs

```bash
# Submit a one-off job into the queue
inferno queue submit \
  --name "model_validation" \
  --input-file prompts.txt \
  --model llama-7b-q4 \
  my-queue

# Schedule a recurring job with a cron expression (nightly at 2 AM)
inferno queue schedule \
  --name "nightly_batch" \
  --schedule-type cron \
  --expression "0 2 * * *" \
  --input-file prompts.txt \
  --model llama-7b-q4 \
  my-queue
```

### Job Management

```bash
# List all queues
inferno queue list-queues

# List jobs in a queue
inferno queue list-jobs my-queue

# View job status
inferno queue job-status my-queue job-123

# Cancel a job
inferno queue cancel my-queue job-123

# Retry a failed job
inferno queue retry my-queue job-123

# Pause/resume a queue
inferno queue pause my-queue
inferno queue resume my-queue
```

### Job Monitoring

```bash
# Monitor a queue in real time
inferno queue monitor my-queue

# Get queue metrics
inferno queue metrics my-queue

# Export queue data to a file
inferno queue export --output jobs.json my-queue
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

Schedule a recurring job by passing the cron string to `inferno queue schedule
--schedule-type cron --expression "<cron>"`. A full invocation looks like:

```bash
# Daily at 2 AM
inferno queue schedule --schedule-type cron --expression "0 2 * * *" \
  --name daily-batch --input-file prompts.txt --model llama-7b-q4 my-queue
```

Common `--expression` values and what they mean:

```bash
# Every minute
--expression "* * * * *"

# Every hour at minute 0
--expression "0 * * * *"

# Daily at 2 AM
--expression "0 2 * * *"

# Weekly on Sunday at 3 AM
--expression "0 3 * * 0"

# Monthly on the 1st at midnight
--expression "0 0 1 * *"

# Every 15 minutes
--expression "*/15 * * * *"

# Weekdays at 9 AM
--expression "0 9 * * 1-5"

# Custom: Every 2 hours between 9 AM and 5 PM on weekdays
--expression "0 9-17/2 * * 1-5"
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
            - queue
            - submit
            - --name
            - k8s-model-sync
            - --input-file
            - /data/prompts.txt
            - --model
            - llama-7b-q4
            - my-queue
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
- Configure appropriate retention policies
- Regularly validate audit log integrity (`inferno audit validate`)
- Archive old logs to long-term storage (`inferno audit archive`)
- Enable compression to reduce storage footprint

### Batch Processing
- Use appropriate resource limits for jobs
- Implement proper error handling and logging
- Test cron expressions before deployment
- Monitor job performance and adjust as needed
- Use job dependencies for complex workflows

### Security
- Restrict access to the audit log storage directory
- Implement proper access controls for job management
- Monitor for unusual job patterns or failures

### Performance
- Optimize job resource allocation
- Use compression for large audit logs
- Schedule resource-intensive jobs during off-peak hours
- Monitor queue performance and adjust concurrency limits
- Archive completed job data periodically