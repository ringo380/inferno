# Inferno Monitoring & Observability

Complete monitoring infrastructure for Inferno v0.8.0 using Prometheus, Grafana, and Alertmanager.

## Quick Start

### 1. Deploy Monitoring Stack

```bash
# Using Prometheus Operator (Kubernetes)
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm repo update
helm install prometheus prometheus-community/kube-prometheus-stack \
  -n monitoring --create-namespace

# Deploy Inferno ServiceMonitor
kubectl apply -f monitoring/servicemonitor.yaml

# Deploy Inferno PrometheusRule (alerts)
kubectl apply -f monitoring/prometheusrule.yaml
```

### 2. Configure Prometheus (Standalone)

```bash
# Copy config file
cp monitoring/prometheus-config.yaml /etc/prometheus/prometheus.yaml

# Copy alert rules
cp monitoring/prometheus-rules.yaml /etc/prometheus/rules/

# Restart Prometheus
systemctl restart prometheus
```

### 3. Set Up Grafana Dashboard

```bash
# Port forward to Grafana
kubectl port-forward -n monitoring svc/prometheus-grafana 3000:80

# Open http://localhost:3000 (default: admin/prom-operator)

# Import dashboard:
# - Click "Dashboards" → "Import"
# - Upload monitoring/grafana-dashboard.json
# - Select Prometheus datasource
```

## Files

### Prometheus Configuration
- **prometheus-config.yaml** - Global Prometheus scrape config with Kubernetes SD
- **prometheus-rules.yaml** - Alert rules and recording rules
- **servicemonitor.yaml** - Prometheus Operator ServiceMonitor for Kubernetes

### Alert Rules
- **prometheusrule.yaml** - PrometheusRule CRD for Kubernetes deployment
- **prometheus-rules.yaml** - Raw alert rules (standalone)

### Grafana
- **grafana-dashboard.json** - Overview dashboard (8 panels)
- **grafana-datasource.yaml** - Prometheus datasource config

### Documentation
- **MONITORING_GUIDE.md** - Comprehensive monitoring guide
- **README.md** - This file

## Features

### 20+ Alert Rules
- Pod health monitoring
- API performance (latency, error rate)
- Queue depth and utilization
- Inference performance
- Cache efficiency
- Resource utilization (CPU, memory, disk)
- Model management
- Security (auth failures)

### Recording Rules
Pre-computed metrics for dashboard performance:
- Request rates
- Error rates
- Latency percentiles (P95, P99)
- Queue utilization
- Cache hit rate

### Grafana Dashboard
8-panel overview dashboard showing:
1. Pod status gauge
2. Request rate (requests/sec)
3. API latency percentiles
4. 5xx error rate
5. Request queue depth
6. Inference latency (P95)
7. Cache hit rate
8. Pod memory usage

## Metrics Collected

### HTTP API Metrics
- `inferno_http_requests_total` - Total requests by endpoint/status
- `inferno_http_request_duration_seconds` - Request latency histogram
- `inferno_http_requests_in_progress` - Current in-flight requests

### Inference Metrics
- `inferno_inference_requests_total` - Total inferences by model
- `inferno_inference_duration_seconds` - Inference latency by model
- `inferno_inference_errors_total` - Inference errors by model
- `inferno_tokens_generated_total` - Total tokens generated

### Queue Metrics
- `inferno_queue_pending_requests` - Pending request count
- `inferno_queue_max_capacity` - Queue capacity
- `inferno_queue_processed_total` - Processed requests
- `inferno_queue_dropped_total` - Dropped requests

### Cache Metrics
- `inferno_cache_hits_total` - Cache hits
- `inferno_cache_misses_total` - Cache misses
- `inferno_cache_evictions_total` - Cache evictions
- `inferno_cache_size_bytes` - Cache size

### Resource Metrics
- `container_cpu_usage_seconds_total` - CPU usage
- `container_memory_usage_bytes` - Memory usage
- `node_filesystem_avail_bytes` - Disk space

## Alert Thresholds

### Critical (Page On-Call)
- Pod down for 2 minutes
- Queue >500 pending requests
- Memory >3900Mi (threshold: 4Gi)
- Disk space <5% available
- Persistence write failures

### Warning (Email Ops)
- API latency P95 >1s
- 5xx error rate >5%
- Queue >100 pending requests
- CPU >1800m (threshold: 2 CPU)
- Memory >3500Mi
- Disk space <15% available

### Info (Dashboard Only)
- Cache hit rate <60%
- Rate limiting active
- Model load failures

## Helm Integration

Enable monitoring in Helm chart:

```bash
# Enable ServiceMonitor
helm install inferno ./helm/inferno \
  -f helm/inferno/values-prod.yaml \
  --set monitoring.serviceMonitor.enabled=true

# Enable PrometheusRule (alerts)
helm install inferno ./helm/inferno \
  -f helm/inferno/values-prod.yaml \
  --set monitoring.prometheusRule.enabled=true

# Both (recommended for production)
helm install inferno ./helm/inferno \
  -f helm/inferno/values-prod.yaml \
  --set monitoring.serviceMonitor.enabled=true \
  --set monitoring.prometheusRule.enabled=true
```

## Configuration

### Scrape Interval

Adjust collection frequency:
```yaml
monitoring:
  prometheus:
    scrapeInterval: 30s  # Default: 30s
                         # Dev: 60s (less load)
                         # Prod: 15s (more detail)
```

### Environment Specific

**Development** (values-dev.yaml):
```yaml
monitoring:
  prometheus:
    enabled: false  # Disable metrics in dev
  serviceMonitor:
    enabled: false
```

**Staging** (values-staging.yaml):
```yaml
monitoring:
  prometheus:
    enabled: true
    scrapeInterval: 30s
  serviceMonitor:
    enabled: true
```

**Production** (values-prod.yaml):
```yaml
monitoring:
  prometheus:
    enabled: true
    scrapeInterval: 15s  # More frequent
  serviceMonitor:
    enabled: true
  prometheusRule:
    enabled: true  # Enable alerts
```

## Querying Metrics

### PromQL Examples

```promql
# Request rate (5-minute average)
rate(inferno_http_requests_total[5m])

# P95 API latency
histogram_quantile(0.95, rate(inferno_http_request_duration_seconds_bucket[5m]))

# Error rate percentage
100 * rate(inferno_http_requests_total{status=~"5.."}[5m]) / rate(inferno_http_requests_total[5m])

# Queue utilization
inferno_queue_pending_requests / inferno_queue_max_capacity

# Cache hit rate
rate(inferno_cache_hits_total[5m]) / (rate(inferno_cache_hits_total[5m]) + rate(inferno_cache_misses_total[5m]))

# Per-model inference rate
rate(inferno_inference_requests_total[5m]) by (model)

# Pod memory in MB
container_memory_usage_bytes{pod=~"inferno.*"} / 1024 / 1024
```

## Troubleshooting

### Prometheus not scraping metrics

```bash
# Check ServiceMonitor
kubectl get servicemonitor -n inferno-prod

# Check targets in Prometheus UI
kubectl port-forward -n monitoring svc/prometheus 9090:9090
# Visit http://localhost:9090/targets

# Verify metrics endpoint
kubectl port-forward -n inferno-prod svc/inferno 9090:9090
curl http://localhost:9090/metrics | head -20
```

### Alerts not firing

```bash
# Check PrometheusRule
kubectl get prometheusrule -n inferno-prod
kubectl describe prometheusrule inferno -n inferno-prod

# Check alert evaluation
# In Prometheus UI: http://localhost:9090/alerts

# Manually query alert condition
# E.g., for InfernoPodDown:
up{job="inferno"} == 0
```

### No dashboard data

```bash
# Verify Prometheus datasource in Grafana
# Settings → Data Sources → Prometheus → Test

# Query data directly in Prometheus
curl "http://prometheus:9090/api/v1/query?query=up"

# Check metrics exist
kubectl port-forward -n inferno-prod svc/inferno 9090:9090
curl http://localhost:9090/metrics | grep inferno_
```

## Performance Impact

| Component | CPU | Memory | Storage |
|-----------|-----|--------|---------|
| Prometheus scrape | <5% | <10Mi | 1-2MB/hr |
| Alert evaluation | <2% | <5Mi | None |
| Grafana queries | Varies | 50-100Mi | None |

## Retention Policies

Default (30 days):
```bash
prometheus --storage.tsdb.retention.time=30d
```

For longer retention:
```bash
prometheus --storage.tsdb.retention.time=90d
```

For size limits:
```bash
prometheus --storage.tsdb.retention.size=50GB
```

## Alerting Integrations

### Slack
```bash
# Set webhook URL in alertmanager-config.yaml
slack_api_url: 'https://hooks.slack.com/services/YOUR/WEBHOOK/URL'
```

### PagerDuty
```bash
# Set service key in alertmanager-config.yaml
service_key: 'YOUR_PAGERDUTY_SERVICE_KEY'
```

### Email
```bash
# Configure SMTP in alertmanager-config.yaml
smtp_smarthost: 'smtp.example.com:587'
smtp_from: 'alerts@example.com'
```

## Best Practices

1. **Alert on symptoms, not causes**
   - Alert on high latency, not CPU usage
   - Alert on queue depth, not throughput

2. **Use recording rules**
   - Pre-compute common aggregations
   - Reduces query load on Prometheus

3. **Meaningful descriptions**
   - Include runbook links
   - Suggest remediation steps

4. **Regular testing**
   - Test alert receivers weekly
   - Verify dashboard accuracy

5. **Monitor the monitor**
   - Track Prometheus disk usage
   - Monitor Grafana performance

## Documentation

- **MONITORING_GUIDE.md** - Comprehensive setup guide
- **Helm Chart** - `helm/inferno/README.md`
- **Deployment** - `docs/DEPLOYMENT_GUIDE.md`

## Support

- GitHub: https://github.com/ringo380/inferno
- Issues: https://github.com/ringo380/inferno/issues

---

**Version**: Inferno v0.8.0
**Last Updated**: 2024-Q4
**Prometheus**: 2.30+
**Grafana**: 7.0+
