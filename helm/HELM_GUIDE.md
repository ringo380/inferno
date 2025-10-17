# Helm Chart Guide for Inferno

This guide explains the Helm chart structure and how to use it for deploying Inferno in different environments.

## Overview

The Inferno Helm chart provides a complete, production-ready deployment configuration for Kubernetes. It supports three environment profiles:
- **Development**: Single replica, debug logging, minimal resources
- **Staging**: Two replicas, info logging, moderate resources, HPA enabled
- **Production**: Three+ replicas, warning logging, full resources, strict security

## File Structure

```
helm/inferno/
├── Chart.yaml                    # Helm chart metadata and version
├── values.yaml                   # Default configuration values
├── values-dev.yaml              # Development environment overrides
├── values-staging.yaml          # Staging environment overrides
├── values-prod.yaml             # Production environment overrides
├── .helmignore                  # Files excluded from package
├── README.md                    # Chart documentation
│
└── templates/
    ├── NOTES.txt                # Post-installation instructions
    ├── _helpers.tpl             # Reusable template functions
    ├── deployment.yaml          # Kubernetes Deployment with health probes
    ├── service.yaml             # ClusterIP, Headless, and LoadBalancer services
    ├── configmap.yaml           # ConfigMap and Secret resources
    ├── pvc.yaml                 # PersistentVolumeClaims (models, cache, queue)
    ├── rbac.yaml                # ServiceAccount, Role, RoleBinding, NetworkPolicy
    ├── hpa.yaml                 # HorizontalPodAutoscaler and VerticalPodAutoscaler
    └── pdb.yaml                 # Pod Disruption Budget
```

## Quick Start Commands

### Development
```bash
helm install inferno ./helm/inferno -f helm/inferno/values-dev.yaml
kubectl get pods
kubectl logs -f deployment/inferno
```

### Staging
```bash
helm install inferno ./helm/inferno \
  -f helm/inferno/values-staging.yaml \
  -n inferno-staging \
  --create-namespace
```

### Production
```bash
helm install inferno ./helm/inferno \
  -f helm/inferno/values-prod.yaml \
  -n inferno-prod \
  --create-namespace
```

### Upgrade
```bash
helm upgrade inferno ./helm/inferno \
  -f helm/inferno/values-prod.yaml \
  -n inferno-prod
```

### Uninstall
```bash
helm uninstall inferno -n inferno-prod
```

## Template Structure

### _helpers.tpl
Provides 9 reusable Helm functions:
- `inferno.name`: Chart name sanitized
- `inferno.fullname`: Release-qualified name (e.g., "release-inferno")
- `inferno.chart`: Chart identifier (e.g., "inferno-0.8.0")
- `inferno.labels`: Standard Kubernetes labels
- `inferno.selectorLabels`: Pod selector labels
- `inferno.serviceAccountName`: ServiceAccount name
- `inferno.image`: Full image reference with registry/tag
- `inferno.port`: API port number
- `inferno.namespace`: Namespace name

### deployment.yaml
Renders the Kubernetes Deployment with:
- Configurable replicas (3 default, 1 for dev, 2 for staging)
- RollingUpdate strategy (maxSurge: 1, maxUnavailable: 0)
- Health probes:
  - **Startup**: 5s initial delay, 15 failures to restart (30s max)
  - **Readiness**: 10s initial delay, every 10s (detects unhealthy pods)
  - **Liveness**: 30s initial delay, every 30s (restarts unresponsive pods)
- Resource requests and limits (configurable per environment)
- Pod anti-affinity for distribution across nodes
- Volume mounts for models, cache, queue, config
- Security context (non-root user 1000)
- Service account reference for RBAC

### service.yaml
Creates three services:
1. **ClusterIP** (internal): Standard in-cluster service on port 8000
2. **Headless**: Direct pod DNS resolution for distributed systems
3. **LoadBalancer/NodePort** (external): Optional external access on port 80 (dev: disabled, staging: NodePort, prod: LoadBalancer)

### configmap.yaml
Renders ConfigMap and Secret from values:
- **ConfigMap**: Non-sensitive environment variables (80+ variables)
  - Logging configuration
  - API configuration
  - Model and inference settings
  - Queue configuration
  - Cache strategy and TTL
  - Streaming configuration
  - Deployment environment flag
  - Metrics settings
- **Secret**: Sensitive data (authentication, OAuth2, Sentry DSN)

### pvc.yaml
Creates three PersistentVolumeClaims (when enabled):
- **models**: 100Gi (dev: 20Gi, staging: 200Gi, prod: 500Gi)
- **cache**: 50Gi (dev: 10Gi, staging: 100Gi, prod: 200Gi)
- **queue**: 10Gi (consistent across environments)

Storage class configurable per environment (default: "standard").

### rbac.yaml
Deploys RBAC security model (when enabled):
- **ServiceAccount**: Provides pod identity
- **Role**: Grants minimal permissions
  - Read ConfigMaps and Secrets
  - Read Pod information
  - Create/patch Events
- **RoleBinding**: Connects role to service account
- **NetworkPolicy**: Restricts traffic
  - Ingress: From ingress-nginx namespace or same namespace
  - Egress: DNS (UDP:53), HTTPS (TCP:443), pod-to-pod

### hpa.yaml
Configures autoscaling (when enabled):
- **HorizontalPodAutoscaler**:
  - CPU threshold: 70%
  - Memory threshold: 80%
  - Scale up: Fast (0s stabilization)
  - Scale down: Slow (5min stabilization)
- **VerticalPodAutoscaler** (optional, disabled by default):
  - Right-sizing recommendations
  - Min: 500m CPU, 1Gi memory
  - Max: 4 CPU, 8Gi memory

### pdb.yaml
Pod Disruption Budget (production):
- Minimum 2 pods available during disruptions
- Prevents accidental node drain from losing all pods

## Configuration Hierarchy

Values are applied in this order (highest to lowest priority):
1. `--set` CLI flags
2. Environment-specific values file (values-prod.yaml)
3. Base values.yaml defaults

Example:
```bash
helm install inferno ./helm/inferno \
  -f helm/inferno/values-prod.yaml \
  --set replicaCount=5 \
  --set resources.limits.cpu=4000m
```

## Environment Comparison

| Aspect | Development | Staging | Production |
|--------|-------------|---------|-----------|
| Namespace | default | inferno-staging | inferno-prod |
| Replicas | 1 | 2 | 3 (autoscales to 20) |
| CPU Request | 100m | 500m | 1000m |
| Memory Request | 512Mi | 1Gi | 2Gi |
| Logging | debug/pretty | info/json | warn/json |
| HPA | Disabled | Enabled (2-5) | Enabled (3-20) |
| Models Storage | 20Gi | 200Gi | 500Gi |
| NetworkPolicy | Disabled | Enabled | Strict |
| PDB | Disabled | Enabled (1) | Enabled (2) |
| Profiling | Enabled | Enabled | Disabled |
| Metrics | Disabled | Enabled | Enabled |

## Advanced Customization

### Using Multiple Values Files
```bash
helm install inferno ./helm/inferno \
  -f helm/inferno/values-prod.yaml \
  -f helm/inferno/values-prod-custom.yaml
```

### Dry-Run and Preview
```bash
# See what will be deployed
helm install inferno ./helm/inferno \
  -f helm/inferno/values-prod.yaml \
  --dry-run --debug

# Actually apply
helm install inferno ./helm/inferno \
  -f helm/inferno/values-prod.yaml
```

### Update Single Value
```bash
helm upgrade inferno ./helm/inferno \
  -f helm/inferno/values-prod.yaml \
  --set replicaCount=5
```

### Custom Storage Class
```bash
helm install inferno ./helm/inferno \
  -f helm/inferno/values-prod.yaml \
  --set persistence.storageClass=fast-ssd
```

### External IPs for LoadBalancer
```bash
helm install inferno ./helm/inferno \
  -f helm/inferno/values-prod.yaml \
  --set 'service.external.externalIPs[0]=10.0.0.1' \
  --set 'service.external.externalIPs[1]=10.0.0.2'
```

### OAuth2 Configuration
```bash
helm install inferno ./helm/inferno \
  -f helm/inferno/values-prod.yaml \
  --set secrets.OAUTH2_CLIENT_ID="your-id" \
  --set secrets.OAUTH2_CLIENT_SECRET="your-secret" \
  --set secrets.OAUTH2_PROVIDER_URL="https://provider.example.com"
```

## Monitoring

### Port Forward to Metrics
```bash
kubectl port-forward -n inferno-prod svc/inferno 9090:9090
curl http://localhost:9090/metrics
```

### Prometheus Scrape Configuration
The deployment includes annotations for Prometheus:
```yaml
prometheus.io/scrape: "true"
prometheus.io/port: "9090"
prometheus.io/path: "/metrics"
```

### ServiceMonitor (Prometheus Operator)
Enable if using Prometheus Operator:
```bash
helm install inferno ./helm/inferno \
  -f helm/inferno/values-prod.yaml \
  --set monitoring.serviceMonitor.enabled=true
```

## Troubleshooting

### View Generated Manifests
```bash
helm template inferno ./helm/inferno \
  -f helm/inferno/values-prod.yaml
```

### Check Installation History
```bash
helm history inferno -n inferno-prod
helm status inferno -n inferno-prod
```

### Rollback to Previous Release
```bash
helm rollback inferno 1 -n inferno-prod
```

### Debug Deployment Issues
```bash
kubectl describe pod <pod-name> -n inferno-prod
kubectl logs <pod-name> -n inferno-prod
kubectl events -n inferno-prod --sort-by='.lastTimestamp'
```

## Security Considerations

### Pod Security
- Runs as non-root user (UID 1000)
- No privilege escalation allowed
- Dropped all Linux capabilities

### Network Security
- NetworkPolicy restricts ingress and egress
- Ingress only from ingress-nginx namespace or same namespace
- Egress to DNS, HTTPS, and pod-to-pod only

### Secrets Management
- Kubernetes Secrets for sensitive data
- Support for external secret providers (Sealed Secrets, External Secrets Operator)
- Never commit sensitive values to git

## Version Management

- **Chart Version**: 0.8.0 (matches Inferno version)
- **Kubernetes**: 1.20+
- **Helm**: 3.0+
- **Image Tag**: Configurable (default: 0.8.0)

## Backup and Recovery

### Backup ConfigMap
```bash
kubectl get cm -n inferno-prod inferno-config -o yaml > backup-config.yaml
```

### Backup PVC
```bash
# List PVCs
kubectl get pvc -n inferno-prod

# Create snapshot (if CSI driver available)
kubectl get volumesnapshot -n inferno-prod
```

### Restore from Release
```bash
helm rollback inferno <revision> -n inferno-prod
```

## Support and Documentation

- **Chart README**: `helm/inferno/README.md`
- **Deployment Guide**: `docs/DEPLOYMENT_GUIDE.md`
- **GitHub**: https://github.com/ringo380/inferno
- **Issues**: https://github.com/ringo380/inferno/issues

---

**Last Updated**: 2024-Q4
**Helm Chart Version**: 0.8.0
**Inferno Version**: 0.8.0
