# Dashboard API Guide

This guide covers the 14 comprehensive dashboard API endpoints that provide real-time monitoring, management, and observability for the Inferno AI/ML inference server.

## Overview

The Dashboard API provides a complete web-based interface for monitoring and managing Inferno deployments. All endpoints return JSON data and support real-time updates through WebSocket connections.

**Base URL**: `http://localhost:8080/dashboard`

## Authentication

All dashboard endpoints support the same authentication methods as the main API:

```bash
# API Key authentication
curl -H "Authorization: Bearer YOUR_API_KEY" http://localhost:8080/dashboard/stats

# JWT authentication
curl -H "Authorization: Bearer JWT_TOKEN" http://localhost:8080/dashboard/stats
```

## Core Endpoints

### 1. System Statistics

**Endpoint**: `GET /dashboard/stats`

Provides comprehensive system performance statistics.

```bash
curl http://localhost:8080/dashboard/stats
```

**Response Example**:
```json
{
  "timestamp": "2024-03-15T10:30:00Z",
  "uptime_seconds": 86400,
  "system": {
    "cpu_usage_percent": 45.2,
    "memory_usage_percent": 67.8,
    "memory_used_gb": 10.8,
    "memory_total_gb": 16.0,
    "disk_usage_percent": 23.4,
    "disk_used_gb": 234.5,
    "disk_total_gb": 1000.0,
    "load_average": [1.2, 1.1, 0.9],
    "network_io": {
      "bytes_sent": 1048576000,
      "bytes_received": 2097152000,
      "packets_sent": 500000,
      "packets_received": 750000
    }
  },
  "gpu": {
    "available": true,
    "count": 2,
    "devices": [
      {
        "id": 0,
        "name": "NVIDIA RTX 4090",
        "memory_used_mb": 8192,
        "memory_total_mb": 24576,
        "utilization_percent": 78.5,
        "temperature_celsius": 72
      }
    ]
  },
  "inference": {
    "requests_per_second": 15.3,
    "average_latency_ms": 150,
    "active_requests": 5,
    "total_requests": 125000,
    "total_errors": 23
  }
}
```

### 2. Model Status and Metrics

**Endpoint**: `GET /dashboard/models`

Lists all available models with their current status and performance metrics.

```bash
curl http://localhost:8080/dashboard/models
```

**Response Example**:
```json
{
  "models": [
    {
      "id": "llama-7b-q4",
      "name": "Llama 7B Q4",
      "format": "gguf",
      "file_path": "/models/llama-7b-q4.gguf",
      "size_bytes": 3800000000,
      "status": "loaded",
      "loaded_at": "2024-03-15T08:00:00Z",
      "backend_type": "gguf",
      "metadata": {
        "architecture": "llama",
        "context_length": 4096,
        "vocab_size": 32000,
        "parameters": "7B",
        "quantization": "Q4_0"
      },
      "performance": {
        "total_requests": 1500,
        "average_tokens_per_second": 25.3,
        "average_latency_ms": 145,
        "error_rate": 0.001,
        "cache_hit_rate": 0.78
      },
      "resource_usage": {
        "memory_usage_mb": 4200,
        "gpu_memory_usage_mb": 3800,
        "cpu_utilization_percent": 12.5
      }
    }
  ],
  "summary": {
    "total_models": 5,
    "loaded_models": 2,
    "total_size_gb": 45.2,
    "memory_usage_gb": 8.4
  }
}
```

### 3. Health Check with Details

**Endpoint**: `GET /dashboard/health`

Comprehensive health check with detailed component status.

```bash
curl http://localhost:8080/dashboard/health
```

**Response Example**:
```json
{
  "status": "healthy",
  "timestamp": "2024-03-15T10:30:00Z",
  "components": {
    "api_server": {
      "status": "healthy",
      "response_time_ms": 5,
      "last_check": "2024-03-15T10:30:00Z"
    },
    "model_backends": {
      "status": "healthy",
      "loaded_models": 2,
      "active_inference": 3,
      "last_check": "2024-03-15T10:29:55Z"
    },
    "cache_system": {
      "status": "healthy",
      "hit_rate": 0.89,
      "size_usage": 0.67,
      "last_check": "2024-03-15T10:29:58Z"
    },
    "database": {
      "status": "healthy",
      "connection_pool": 8,
      "query_time_ms": 2,
      "last_check": "2024-03-15T10:30:00Z"
    },
    "audit_system": {
      "status": "healthy",
      "pending_logs": 23,
      "last_flush": "2024-03-15T10:25:00Z"
    }
  },
  "alerts": [
    {
      "level": "warning",
      "component": "disk_space",
      "message": "Disk usage above 80%",
      "timestamp": "2024-03-15T10:15:00Z"
    }
  ]
}
```

### 4. Current Configuration

**Endpoint**: `GET /dashboard/config`

Displays current system configuration and settings.

```bash
curl http://localhost:8080/dashboard/config
```

**Response Example**:
```json
{
  "server": {
    "bind_address": "0.0.0.0:8080",
    "max_concurrent_requests": 100,
    "request_timeout_seconds": 300,
    "tls_enabled": true
  },
  "models": {
    "directory": "/models",
    "auto_discovery": true,
    "max_loaded_models": 5,
    "unload_timeout_minutes": 30
  },
  "cache": {
    "enabled": true,
    "type": "persistent",
    "max_size_gb": 10,
    "compression": "zstd",
    "ttl_hours": 24
  },
  "security": {
    "auth_enabled": true,
    "rate_limiting": true,
    "max_requests_per_minute": 1000,
    "audit_enabled": true
  },
  "backends": {
    "gguf": {
      "gpu_enabled": true,
      "context_size": 4096,
      "batch_size": 32
    },
    "onnx": {
      "execution_providers": ["CUDAExecutionProvider", "CPUExecutionProvider"],
      "optimization_level": "all"
    }
  }
}
```

### 5. Recent Log Entries

**Endpoint**: `GET /dashboard/logs`

Retrieves recent log entries with filtering options.

```bash
# Get recent logs
curl http://localhost:8080/dashboard/logs

# Filter by level and time
curl "http://localhost:8080/dashboard/logs?level=error&since=1h&limit=50"
```

**Query Parameters**:
- `level`: Log level filter (trace, debug, info, warn, error)
- `since`: Time range (1h, 30m, 1d)
- `limit`: Maximum number of entries (default: 100)
- `component`: Filter by component name

**Response Example**:
```json
{
  "logs": [
    {
      "timestamp": "2024-03-15T10:29:30Z",
      "level": "info",
      "component": "inference",
      "message": "Model llama-7b-q4 inference completed",
      "metadata": {
        "request_id": "req_123456",
        "latency_ms": 150,
        "tokens": 45
      }
    },
    {
      "timestamp": "2024-03-15T10:29:25Z",
      "level": "warn",
      "component": "cache",
      "message": "Cache hit rate below threshold",
      "metadata": {
        "hit_rate": 0.75,
        "threshold": 0.80
      }
    }
  ],
  "total_count": 1500,
  "filtered_count": 25
}
```

### 6. Performance Metrics

**Endpoint**: `GET /dashboard/metrics`

Detailed performance metrics and time-series data.

```bash
curl http://localhost:8080/dashboard/metrics
```

**Response Example**:
```json
{
  "timestamp": "2024-03-15T10:30:00Z",
  "metrics": {
    "requests": {
      "total": 125000,
      "rate_per_second": 15.3,
      "rate_per_minute": 918,
      "success_rate": 0.998
    },
    "latency": {
      "p50_ms": 120,
      "p90_ms": 250,
      "p95_ms": 350,
      "p99_ms": 500,
      "average_ms": 145
    },
    "throughput": {
      "tokens_per_second": 456.7,
      "requests_per_second": 15.3,
      "bytes_per_second": 1048576
    },
    "errors": {
      "total": 250,
      "rate_per_hour": 10.4,
      "types": {
        "timeout": 150,
        "model_error": 50,
        "validation_error": 30,
        "system_error": 20
      }
    },
    "resources": {
      "cpu_usage": 45.2,
      "memory_usage": 67.8,
      "gpu_usage": 78.5,
      "disk_io_rate": 1024000
    }
  },
  "time_series": {
    "requests_per_minute": [
      {"timestamp": "2024-03-15T10:25:00Z", "value": 920},
      {"timestamp": "2024-03-15T10:26:00Z", "value": 950},
      {"timestamp": "2024-03-15T10:27:00Z", "value": 890}
    ]
  }
}
```

### 7. Cache Statistics

**Endpoint**: `GET /dashboard/cache`

Comprehensive cache system statistics and performance data.

```bash
curl http://localhost:8080/dashboard/cache
```

**Response Example**:
```json
{
  "cache_types": {
    "model": {
      "enabled": true,
      "hit_rate": 0.92,
      "entries": 5,
      "size_mb": 4200,
      "max_size_mb": 8192
    },
    "response": {
      "enabled": true,
      "hit_rate": 0.78,
      "entries": 15000,
      "size_mb": 2048,
      "max_size_mb": 4096,
      "deduplication_ratio": 0.67
    },
    "metadata": {
      "enabled": true,
      "hit_rate": 0.95,
      "entries": 500,
      "size_mb": 12,
      "last_refresh": "2024-03-15T10:00:00Z"
    }
  },
  "compression": {
    "algorithm": "zstd",
    "compression_ratio": 3.4,
    "compressed_size_mb": 1800,
    "uncompressed_size_mb": 6120
  },
  "performance": {
    "average_read_time_ms": 5,
    "average_write_time_ms": 15,
    "cache_evictions": 250,
    "cache_misses": 3500
  }
}
```

### 8. Batch Job Status

**Endpoint**: `GET /dashboard/jobs`

Status and management of batch processing jobs.

```bash
curl http://localhost:8080/dashboard/jobs
```

**Response Example**:
```json
{
  "active_jobs": [
    {
      "id": "job_123456",
      "name": "model_conversion_batch",
      "type": "conversion",
      "status": "running",
      "progress": 0.65,
      "started_at": "2024-03-15T10:00:00Z",
      "estimated_completion": "2024-03-15T10:45:00Z",
      "resource_usage": {
        "cpu_percent": 80,
        "memory_mb": 2048
      }
    }
  ],
  "completed_jobs": [
    {
      "id": "job_123455",
      "name": "nightly_inference_batch",
      "status": "completed",
      "started_at": "2024-03-15T02:00:00Z",
      "completed_at": "2024-03-15T03:30:00Z",
      "results": {
        "processed_items": 1000,
        "success_rate": 0.99,
        "errors": 10
      }
    }
  ],
  "scheduled_jobs": [
    {
      "id": "job_123457",
      "name": "weekly_model_update",
      "schedule": "0 2 * * 0",
      "next_run": "2024-03-17T02:00:00Z",
      "enabled": true
    }
  ],
  "queue_stats": {
    "pending_jobs": 5,
    "max_concurrent": 3,
    "average_wait_time_minutes": 12
  }
}
```

### 9. Audit Log Entries

**Endpoint**: `GET /dashboard/audit`

Access to audit log entries with filtering and search capabilities.

```bash
# Get recent audit entries
curl http://localhost:8080/dashboard/audit

# Filter by user and action
curl "http://localhost:8080/dashboard/audit?user=admin&action=model_load&since=24h"
```

**Query Parameters**:
- `user`: Filter by username
- `action`: Filter by action type
- `resource`: Filter by resource
- `since`: Time range
- `level`: Severity level
- `limit`: Maximum entries

**Response Example**:
```json
{
  "entries": [
    {
      "id": "audit_789012",
      "timestamp": "2024-03-15T10:29:30Z",
      "user": "admin",
      "action": "model_load",
      "resource": "llama-7b-q4.gguf",
      "level": "info",
      "source_ip": "192.168.1.100",
      "user_agent": "InfernoClient/1.0",
      "details": {
        "model_size_mb": 3800,
        "load_time_ms": 2500,
        "gpu_enabled": true
      },
      "result": "success"
    },
    {
      "id": "audit_789011",
      "timestamp": "2024-03-15T10:25:00Z",
      "user": "user123",
      "action": "inference_request",
      "resource": "llama-7b-q4",
      "level": "info",
      "source_ip": "192.168.1.101",
      "details": {
        "prompt_length": 150,
        "max_tokens": 100,
        "latency_ms": 145
      },
      "result": "success"
    }
  ],
  "total_count": 50000,
  "filtered_count": 125
}
```

### 10. Distributed Worker Status

**Endpoint**: `GET /dashboard/workers`

Status of distributed workers in the cluster.

```bash
curl http://localhost:8080/dashboard/workers
```

**Response Example**:
```json
{
  "coordinator": {
    "status": "active",
    "address": "coordinator:9090",
    "uptime_seconds": 86400,
    "connected_workers": 3
  },
  "workers": [
    {
      "id": "worker_001",
      "address": "worker1:9091",
      "status": "active",
      "last_heartbeat": "2024-03-15T10:29:55Z",
      "capabilities": ["gguf", "onnx"],
      "resource_usage": {
        "cpu_percent": 45,
        "memory_percent": 60,
        "gpu_available": true,
        "gpu_usage_percent": 75
      },
      "current_load": {
        "active_requests": 3,
        "queue_length": 1,
        "average_latency_ms": 150
      },
      "models_loaded": ["llama-7b-q4", "mistral-7b"]
    }
  ],
  "cluster_stats": {
    "total_capacity": 100,
    "current_usage": 67,
    "pending_requests": 8,
    "load_balance_efficiency": 0.85
  }
}
```

### 11. System Resource Usage

**Endpoint**: `GET /dashboard/resources`

Detailed system resource monitoring and usage statistics.

```bash
curl http://localhost:8080/dashboard/resources
```

**Response Example**:
```json
{
  "cpu": {
    "usage_percent": 45.2,
    "load_average": [1.2, 1.1, 0.9],
    "cores": 16,
    "frequency_mhz": 3200,
    "per_core_usage": [40, 50, 35, 45, 30, 55, 40, 48, 42, 38, 45, 52, 35, 40, 44, 46]
  },
  "memory": {
    "total_gb": 32.0,
    "used_gb": 18.5,
    "available_gb": 13.5,
    "usage_percent": 57.8,
    "swap_total_gb": 8.0,
    "swap_used_gb": 0.5,
    "buffers_gb": 1.2,
    "cached_gb": 3.8
  },
  "disk": {
    "filesystems": [
      {
        "mount": "/",
        "total_gb": 500,
        "used_gb": 150,
        "available_gb": 350,
        "usage_percent": 30
      },
      {
        "mount": "/models",
        "total_gb": 1000,
        "used_gb": 450,
        "available_gb": 550,
        "usage_percent": 45
      }
    ],
    "io_stats": {
      "reads_per_second": 120,
      "writes_per_second": 80,
      "read_mb_per_second": 15.2,
      "write_mb_per_second": 8.7
    }
  },
  "network": {
    "interfaces": [
      {
        "name": "eth0",
        "bytes_sent": 1048576000,
        "bytes_received": 2097152000,
        "packets_sent": 500000,
        "packets_received": 750000,
        "errors": 0
      }
    ],
    "connections": {
      "active": 125,
      "time_wait": 45,
      "listening": 8
    }
  },
  "gpu": {
    "devices": [
      {
        "id": 0,
        "name": "NVIDIA RTX 4090",
        "memory_total_mb": 24576,
        "memory_used_mb": 18432,
        "memory_free_mb": 6144,
        "utilization_percent": 85,
        "temperature_celsius": 75,
        "power_usage_watts": 380,
        "fan_speed_percent": 70
      }
    ]
  }
}
```

### 12. Recent Error Reports

**Endpoint**: `GET /dashboard/errors`

Recent error reports and analysis.

```bash
curl http://localhost:8080/dashboard/errors
```

**Response Example**:
```json
{
  "recent_errors": [
    {
      "id": "error_456789",
      "timestamp": "2024-03-15T10:25:00Z",
      "type": "model_load_error",
      "severity": "high",
      "component": "gguf_backend",
      "message": "Failed to load model: insufficient GPU memory",
      "details": {
        "model": "llama-13b.gguf",
        "required_memory_mb": 12000,
        "available_memory_mb": 8000,
        "stack_trace": "..."
      },
      "resolution_status": "investigating"
    }
  ],
  "error_summary": {
    "last_24_hours": {
      "total_errors": 45,
      "critical": 2,
      "high": 8,
      "medium": 20,
      "low": 15
    },
    "top_error_types": [
      {"type": "timeout", "count": 15},
      {"type": "model_error", "count": 12},
      {"type": "validation_error", "count": 8}
    ],
    "resolution_stats": {
      "auto_resolved": 35,
      "manual_intervention": 8,
      "investigating": 2
    }
  }
}
```

### 13. Security Status

**Endpoint**: `GET /dashboard/security`

Security status and authentication information.

```bash
curl http://localhost:8080/dashboard/security
```

**Response Example**:
```json
{
  "authentication": {
    "enabled": true,
    "methods": ["jwt", "api_key"],
    "active_sessions": 12,
    "failed_attempts_last_hour": 3
  },
  "authorization": {
    "rbac_enabled": true,
    "total_users": 25,
    "total_roles": 5,
    "active_permissions": 150
  },
  "rate_limiting": {
    "enabled": true,
    "current_limits": {
      "requests_per_minute": 1000,
      "burst_size": 100
    },
    "violations_last_hour": 5
  },
  "security_events": [
    {
      "timestamp": "2024-03-15T10:20:00Z",
      "type": "failed_login",
      "source_ip": "192.168.1.200",
      "details": "Invalid API key",
      "severity": "medium"
    }
  ],
  "ssl_tls": {
    "enabled": true,
    "certificate_expiry": "2024-12-31T23:59:59Z",
    "cipher_suite": "TLS_AES_256_GCM_SHA384"
  }
}
```

### 14. System Alerts

**Endpoint**: `GET /dashboard/alerts`

Current system alerts and notifications.

```bash
curl http://localhost:8080/dashboard/alerts
```

**Response Example**:
```json
{
  "active_alerts": [
    {
      "id": "alert_123",
      "timestamp": "2024-03-15T10:15:00Z",
      "level": "warning",
      "category": "resource",
      "title": "High disk usage",
      "message": "Disk usage on /models partition is 85%",
      "details": {
        "partition": "/models",
        "usage_percent": 85,
        "threshold": 80,
        "available_gb": 150
      },
      "actions": ["cleanup_old_models", "add_storage"],
      "acknowledged": false
    }
  ],
  "resolved_alerts": [
    {
      "id": "alert_122",
      "timestamp": "2024-03-15T09:30:00Z",
      "resolved_at": "2024-03-15T10:00:00Z",
      "level": "critical",
      "title": "Model backend unresponsive",
      "resolution": "Backend restarted automatically"
    }
  ],
  "alert_rules": [
    {
      "name": "high_error_rate",
      "condition": "error_rate > 0.05",
      "severity": "high",
      "enabled": true
    },
    {
      "name": "low_cache_hit_rate",
      "condition": "cache_hit_rate < 0.80",
      "severity": "medium",
      "enabled": true
    }
  ]
}
```

## WebSocket Support

Real-time updates are available through WebSocket connections:

```javascript
// Connect to real-time dashboard updates
const ws = new WebSocket('ws://localhost:8080/dashboard/ws');

ws.onmessage = function(event) {
    const data = JSON.parse(event.data);
    console.log('Dashboard update:', data);
};

// Subscribe to specific metrics
ws.send(JSON.stringify({
    action: 'subscribe',
    topics: ['stats', 'metrics', 'alerts']
}));
```

## Rate Limiting

Dashboard API endpoints are subject to rate limiting:

- **Default limit**: 100 requests per minute per IP
- **Burst limit**: 20 requests per 10 seconds
- **WebSocket**: 1 connection per authenticated user

## Error Responses

Standard error response format:

```json
{
  "error": {
    "code": "INSUFFICIENT_PERMISSIONS",
    "message": "Access denied to dashboard endpoint",
    "details": {
      "required_permission": "dashboard:read",
      "user_permissions": ["inference:read"]
    }
  }
}
```

## Integration Examples

### Python Dashboard Client

```python
import requests
import websocket
import json

class InfernoDashboard:
    def __init__(self, base_url, api_key):
        self.base_url = base_url
        self.headers = {'Authorization': f'Bearer {api_key}'}

    def get_stats(self):
        response = requests.get(f'{self.base_url}/dashboard/stats',
                              headers=self.headers)
        return response.json()

    def get_models(self):
        response = requests.get(f'{self.base_url}/dashboard/models',
                              headers=self.headers)
        return response.json()

    def monitor_realtime(self, callback):
        ws_url = self.base_url.replace('http', 'ws') + '/dashboard/ws'
        ws = websocket.WebSocketApp(ws_url,
                                   on_message=lambda ws, msg: callback(json.loads(msg)))
        ws.run_forever()

# Usage
dashboard = InfernoDashboard('http://localhost:8080', 'your_api_key')
stats = dashboard.get_stats()
print(f"CPU Usage: {stats['system']['cpu_usage_percent']}%")
```

### React Dashboard Component

```jsx
import React, { useState, useEffect } from 'react';

function DashboardStats() {
    const [stats, setStats] = useState(null);

    useEffect(() => {
        const fetchStats = async () => {
            const response = await fetch('/dashboard/stats', {
                headers: { 'Authorization': `Bearer ${apiKey}` }
            });
            const data = await response.json();
            setStats(data);
        };

        fetchStats();
        const interval = setInterval(fetchStats, 5000); // Update every 5s

        return () => clearInterval(interval);
    }, []);

    if (!stats) return <div>Loading...</div>;

    return (
        <div className="dashboard-stats">
            <div className="metric">
                <h3>CPU Usage</h3>
                <span>{stats.system.cpu_usage_percent}%</span>
            </div>
            <div className="metric">
                <h3>Memory Usage</h3>
                <span>{stats.system.memory_usage_percent}%</span>
            </div>
            <div className="metric">
                <h3>Requests/sec</h3>
                <span>{stats.inference.requests_per_second}</span>
            </div>
        </div>
    );
}
```