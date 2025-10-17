# Inferno Helm Chart

Production-grade Helm chart for Inferno v0.8.0+ - Enterprise-grade AI/ML model inference engine with streaming, queuing, and monitoring.

## Quick Start

### Prerequisites
- Kubernetes 1.20+
- Helm 3.0+
- Persistent Volume provisioner (for storage)

### Installation

#### Development
```bash
helm install inferno ./helm/inferno -f helm/inferno/values-dev.yaml
```

#### Staging
```bash
helm install inferno ./helm/inferno -f helm/inferno/values-staging.yaml -n inferno-staging --create-namespace
```

#### Production
```bash
helm install inferno ./helm/inferno -f helm/inferno/values-prod.yaml -n inferno-prod --create-namespace
```

### Upgrade
```bash
helm upgrade inferno ./helm/inferno -f helm/inferno/values-prod.yaml -n inferno-prod
```

### Uninstall
```bash
helm uninstall inferno -n inferno-prod
```

## Chart Structure

```
helm/inferno/
├── Chart.yaml                 # Chart metadata
├── values.yaml               # Default values
├── values-dev.yaml           # Development overrides
├── values-staging.yaml       # Staging overrides
├── values-prod.yaml          # Production overrides
├── .helmignore               # Files to exclude from package
├── README.md                 # This file
└── templates/
    ├── NOTES.txt             # Post-installation notes
    ├── _helpers.tpl          # Template helpers
    ├── deployment.yaml       # Deployment manifest
    ├── service.yaml          # Service manifests
    ├── configmap.yaml        # ConfigMap and Secrets
    ├── pvc.yaml              # PersistentVolumeClaims
    ├── rbac.yaml             # ServiceAccount, Role, RoleBinding, NetworkPolicy
    ├── hpa.yaml              # HPA and VPA
    └── pdb.yaml              # Pod Disruption Budget
```

## Configuration

### Global Settings

| Parameter | Default | Description |
|-----------|---------|-------------|
| `namespace` | `default` | Kubernetes namespace |
| `environment` | `development` | Environment (development, staging, production) |
| `instanceName` | `inferno` | Pod identification name |

### Image Configuration

| Parameter | Default | Description |
|-----------|---------|-------------|
| `image.repository` | `inferno` | Container image repository |
| `image.tag` | `0.8.0` | Container image tag |
| `image.pullPolicy` | `IfNotPresent` | Image pull policy |

### Deployment Configuration

| Parameter | Default | Description |
|-----------|---------|-------------|
| `replicaCount` | `3` | Number of pod replicas |
| `strategy.type` | `RollingUpdate` | Deployment strategy |
| `strategy.rollingUpdate.maxSurge` | `1` | Max pods above desired during rollout |
| `strategy.rollingUpdate.maxUnavailable` | `0` | Max pods below desired during rollout |

### Resource Management

| Parameter | Default | Description |
|-----------|---------|-------------|
| `resources.requests.cpu` | `1000m` | CPU request per pod |
| `resources.requests.memory` | `2Gi` | Memory request per pod |
| `resources.limits.cpu` | `2000m` | CPU limit per pod |
| `resources.limits.memory` | `4Gi` | Memory limit per pod |

### Autoscaling

| Parameter | Default | Description |
|-----------|---------|-------------|
| `autoscaling.enabled` | `true` | Enable HPA |
| `autoscaling.minReplicas` | `2` | Minimum replicas |
| `autoscaling.maxReplicas` | `10` | Maximum replicas |
| `autoscaling.metrics.cpu.utilization` | `70` | CPU threshold (%) |
| `autoscaling.metrics.memory.utilization` | `80` | Memory threshold (%) |

### Storage Configuration

| Parameter | Default | Description |
|-----------|---------|-------------|
| `persistence.enabled` | `true` | Enable persistent storage |
| `persistence.storageClass` | `standard` | Storage class name |
| `persistence.models.size` | `100Gi` | Models volume size |
| `persistence.cache.size` | `50Gi` | Cache volume size |
| `persistence.queue.size` | `10Gi` | Queue volume size |

### Service Configuration

| Parameter | Default | Description |
|-----------|---------|-------------|
| `service.internal.type` | `ClusterIP` | Internal service type |
| `service.internal.port` | `8000` | Internal service port |
| `service.external.enabled` | `true` | Enable external service |
| `service.external.type` | `LoadBalancer` | External service type |
| `service.external.port` | `80` | External service port |

### Health Checks

| Parameter | Default | Description |
|-----------|---------|-------------|
| `healthChecks.startup.enabled` | `true` | Enable startup probe |
| `healthChecks.readiness.enabled` | `true` | Enable readiness probe |
| `healthChecks.liveness.enabled` | `true` | Enable liveness probe |

### Security

| Parameter | Default | Description |
|-----------|---------|-------------|
| `rbac.create` | `true` | Create RBAC resources |
| `networkPolicy.enabled` | `true` | Enable NetworkPolicy |
| `podSecurityContext.runAsNonRoot` | `true` | Run container as non-root |
| `podSecurityContext.runAsUser` | `1000` | Container user ID |
| `securityContext.allowPrivilegeEscalation` | `false` | Prevent privilege escalation |

### Monitoring

| Parameter | Default | Description |
|-----------|---------|-------------|
| `monitoring.prometheus.enabled` | `true` | Enable Prometheus metrics |
| `monitoring.prometheus.port` | `9090` | Metrics port |
| `monitoring.serviceMonitor.enabled` | `false` | Create ServiceMonitor for Prometheus Operator |

## Environment-Specific Values

### Development (values-dev.yaml)
- **Replicas**: 1 (single pod)
- **Resources**: 100m CPU request, 512Mi memory request
- **Logging**: Debug level, pretty format
- **Storage**: 20Gi models, 10Gi cache
- **Autoscaling**: Disabled
- **Monitoring**: Disabled
- **NetworkPolicy**: Disabled

### Staging (values-staging.yaml)
- **Replicas**: 2 (HA testing)
- **Resources**: 500m CPU request, 1Gi memory request
- **Logging**: Info level, JSON format
- **Storage**: 200Gi models, 100Gi cache
- **Autoscaling**: Enabled (min 2, max 5)
- **Monitoring**: Enabled
- **NetworkPolicy**: Enabled

### Production (values-prod.yaml)
- **Replicas**: 3+ (HA with autoscaling)
- **Resources**: 1 CPU request, 2Gi memory request
- **Logging**: Warning level, JSON format
- **Storage**: 500Gi models, 200Gi cache
- **Autoscaling**: Enabled (min 3, max 20)
- **Monitoring**: Fully enabled
- **NetworkPolicy**: Strict rules
- **Pod Disruption Budget**: Minimum 2 available

## Advanced Usage

### Custom Values
```bash
helm install inferno ./helm/inferno \
  --set replicaCount=5 \
  --set resources.limits.cpu=4000m \
  --set persistence.models.size=1Ti
```

### Storage Class Override
```bash
helm install inferno ./helm/inferno \
  -f helm/inferno/values-prod.yaml \
  --set persistence.storageClass=fast-ssd
```

### External IPs
```bash
helm install inferno ./helm/inferno \
  --set service.external.externalIPs[0]="10.0.0.1"
```

### OAuth2 Configuration
```bash
helm install inferno ./helm/inferno \
  --set secrets.OAUTH2_CLIENT_ID="your-client-id" \
  --set secrets.OAUTH2_CLIENT_SECRET="your-secret" \
  --set secrets.OAUTH2_PROVIDER_URL="https://provider.example.com"
```

### Sentry Error Tracking
```bash
helm install inferno ./helm/inferno \
  --set secrets.INFERNO_SENTRY_DSN="https://your-sentry-dsn@sentry.io/12345"
```

## Operations

### Check Deployment Status
```bash
kubectl get deployment -n inferno-prod inferno
kubectl get pods -n inferno-prod -l app.kubernetes.io/name=inferno
kubectl logs -n inferno-prod -l app.kubernetes.io/name=inferno --tail 100 -f
```

### Port Forwarding
```bash
# API access
kubectl port-forward -n inferno-prod svc/inferno 8000:8000

# Metrics access
kubectl port-forward -n inferno-prod svc/inferno 9090:9090
```

### Test Health Endpoint
```bash
curl http://localhost:8000/health
```

### Scale Manually
```bash
kubectl scale deployment inferno --replicas=5 -n inferno-prod
```

### View HPA Status
```bash
kubectl get hpa -n inferno-prod inferno -w
kubectl describe hpa -n inferno-prod inferno
```

### Update Image
```bash
helm upgrade inferno ./helm/inferno \
  --set image.tag="0.9.0" \
  -n inferno-prod
```

### Rollback
```bash
helm rollback inferno 1 -n inferno-prod
```

## Monitoring Integration

### Prometheus Annotations
The deployment automatically includes Prometheus scrape annotations:
```yaml
prometheus.io/scrape: "true"
prometheus.io/port: "9090"
prometheus.io/path: "/metrics"
```

### ServiceMonitor (Prometheus Operator)
Enable optional ServiceMonitor:
```bash
helm install inferno ./helm/inferno \
  --set monitoring.serviceMonitor.enabled=true \
  -n inferno-prod
```

## Security Best Practices

### Pod Security Policy
The chart includes:
- Non-root user (UID 1000)
- No privilege escalation
- Read-only root filesystem (optional)
- NetworkPolicy enforcement (production)
- RBAC with minimal permissions

### Network Security
- Ingress restricted to ingress-nginx namespace
- Egress to DNS (UDP:53) and HTTPS (TCP:443)
- Pod-to-pod communication within namespace

### Secrets Management
- Sensitive data in Kubernetes Secrets
- Support for external secret providers
- OAuth2 integration ready

## Troubleshooting

### Pods not starting
```bash
kubectl describe pod <pod-name> -n inferno-prod
kubectl logs <pod-name> -n inferno-prod
```

### Storage issues
```bash
kubectl get pvc -n inferno-prod
kubectl describe pvc <pvc-name> -n inferno-prod
```

### Scaling not working
```bash
kubectl describe hpa -n inferno-prod
kubectl get hpa -n inferno-prod -o yaml
```

### Network connectivity
```bash
kubectl exec -it <pod-name> -n inferno-prod -- curl localhost:8000/health
kubectl get networkpolicy -n inferno-prod
```

## Uninstall

```bash
helm uninstall inferno -n inferno-prod
```

This will remove all Helm-managed resources except PersistentVolumes (they must be deleted separately if desired).

## Support

- **Documentation**: [Inferno Deployment Guide](../../docs/DEPLOYMENT_GUIDE.md)
- **GitHub**: [ringo380/inferno](https://github.com/ringo380/inferno)
- **Issues**: [GitHub Issues](https://github.com/ringo380/inferno/issues)

## License

MIT License - See LICENSE file in repository

---

**Chart Version**: 0.8.0
**Inferno Version**: 0.8.0
**Kubernetes**: 1.20+
**Helm**: 3.0+
