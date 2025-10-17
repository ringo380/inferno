# Inferno Kubernetes Deployment

Production-grade Kubernetes manifests for Inferno v0.8.0 using Kustomize.

## Directory Structure

```
k8s/
├── base/                    # Base manifests (shared across environments)
│   ├── deployment.yaml      # Deployment with health probes
│   ├── service.yaml         # Service definitions
│   ├── configmap.yaml       # ConfigMap and Secrets
│   ├── pvc.yaml             # PersistentVolumeClaims
│   ├── rbac.yaml            # ServiceAccount and RBAC
│   ├── hpa.yaml             # HorizontalPodAutoscaler
│   └── kustomization.yaml   # Kustomization base config
└── overlays/                # Environment-specific overlays
    ├── dev/                 # Development: 1 replica, debug
    │   └── kustomization.yaml
    ├── staging/             # Staging: 2 replicas, info
    │   └── kustomization.yaml
    └── prod/                # Production: 3+ replicas, monitoring
        ├── kustomization.yaml
        └── pdb.yaml         # Pod Disruption Budget
```

## Quick Start

### Prerequisites

```bash
# Install kubectl
kubectl version --client

# Install kustomize
kustomize version

# Or use kubectl built-in kustomize
kubectl kustomize --help
```

### Deploy to Development

```bash
# Build manifests
kubectl kustomize k8s/overlays/dev/

# Apply to cluster
kubectl apply -k k8s/overlays/dev/

# Verify deployment
kubectl get pods -n default
kubectl get svc -n default
```

### Deploy to Staging

```bash
# Create namespace
kubectl create namespace inferno-staging

# Apply manifests
kubectl apply -k k8s/overlays/staging/

# Check status
kubectl get deployment -n inferno-staging
kubectl get hpa -n inferno-staging
```

### Deploy to Production

```bash
# Create namespace
kubectl create namespace inferno-prod

# Apply manifests
kubectl apply -k k8s/overlays/prod/

# Verify high availability
kubectl get pdb -n inferno-prod
kubectl get hpa -n inferno-prod
kubectl get pods -n inferno-prod -o wide
```

## Manifests Overview

### Deployment (deployment.yaml)

- **Replicas**: Base 3 (overridden by environment)
- **Strategy**: RollingUpdate (max 1 surge, 0 unavailable)
- **Health Probes**:
  - Startup: 30s to initialize
  - Readiness: Every 10s, ready to accept traffic
  - Liveness: Every 30s, restart if unhealthy
- **Resource Limits**:
  - Dev: 100m CPU / 512Mi memory request, 1 CPU / 2Gi limit
  - Staging: 500m / 1Gi request, 1.5 CPU / 3Gi limit
  - Prod: 1 CPU / 2Gi request, 2 CPU / 4Gi limit
- **Pod Anti-Affinity**: Spread pods across nodes

### Service (service.yaml)

- **Internal Service**: `inferno:8000` (ClusterIP)
- **Headless Service**: `inferno-headless:8000` (for direct pod access)
- **External Service**: `inferno-external` (LoadBalancer/NodePort)
- **Ports**: 8000 (API), 9090 (metrics)

### Configuration (configmap.yaml)

- **ConfigMap**: Non-sensitive environment variables
  - Logging (level, format)
  - API configuration
  - Model and inference settings
  - Queue configuration
  - Cache strategy
  - Streaming options
- **Secrets**: Sensitive data (auth keys, OAuth credentials)

### Storage (pvc.yaml)

- **Models**: 100Gi (dev), 500Gi (prod)
- **Cache**: 50Gi (dev), 200Gi (prod)
- **Queue**: 10Gi (all environments)

### RBAC (rbac.yaml)

- **ServiceAccount**: `inferno` for pod identity
- **Role**: Read access to ConfigMaps, Secrets, Pods
- **NetworkPolicy**: Ingress from ingress controller, egress to DNS/HTTPS

### Autoscaling (hpa.yaml)

- **HPA**: Scale 2-10 pods (dev), 2-20 (prod)
- **Metrics**: CPU 70% threshold, Memory 80%
- **Scale Up**: Fast (0s stabilization)
- **Scale Down**: Slow (5m stabilization)
- **VPA**: Optional vertical pod autoscaler

## Environment-Specific Configuration

### Development

```bash
kubectl apply -k k8s/overlays/dev/
```

- Single replica for local testing
- Debug logging enabled
- Minimal resources
- Fast startup for iterations

### Staging

```bash
kubectl apply -k k8s/overlays/staging/
```

- 2 replicas for high availability testing
- Info-level logging
- Moderate resources
- Similar to production but smaller scale

### Production

```bash
kubectl apply -k k8s/overlays/prod/
```

- 3 replicas, scales to 20 with HPA
- Warning-level logging (JSON format)
- Full resources
- Pod Disruption Budget for availability
- Monitoring enabled

## Operations

### View Status

```bash
# Development
kubectl get pods -n default
kubectl get svc -n default

# Staging
kubectl get pods -n inferno-staging
kubectl logs deployment/staging-inferno -n inferno-staging

# Production
kubectl get pods -n inferno-prod --show-labels
kubectl describe pod <pod-name> -n inferno-prod
```

### Check Health

```bash
# Port forward to test
kubectl port-forward svc/inferno 8000:8000 -n inferno-prod

# Test health endpoint
curl http://localhost:8000/health

# Check metrics
curl http://localhost:9090/metrics
```

### Scaling

```bash
# Manual scale
kubectl scale deployment inferno --replicas=5 -n inferno-prod

# HPA status
kubectl get hpa -n inferno-prod
kubectl describe hpa inferno -n inferno-prod

# Watch scaling
kubectl get hpa -n inferno-prod -w
```

### Updates

```bash
# Update image
kubectl set image deployment/inferno inferno=inferno:0.9.0 -n inferno-prod

# Rollout status
kubectl rollout status deployment/inferno -n inferno-prod

# Rollback if needed
kubectl rollout undo deployment/inferno -n inferno-prod
```

### Troubleshooting

```bash
# Check events
kubectl get events -n inferno-prod --sort-by='.lastTimestamp'

# View logs
kubectl logs deployment/inferno -n inferno-prod
kubectl logs -f pod/<pod-name> -n inferno-prod

# Check resource usage
kubectl top pods -n inferno-prod
kubectl top nodes

# Describe pod for issues
kubectl describe pod <pod-name> -n inferno-prod
```

## Storage

### PersistentVolumes

By default, PVCs use the "standard" storage class. Customize:

```yaml
# In base/pvc.yaml or overlay
storageClassName: fast-ssd  # or your storage class
```

### Backup Strategy

```bash
# Backup ConfigMap
kubectl get cm inferno-config -n inferno-prod -o yaml > backup-config.yaml

# Backup PVC data (if supported by storage class)
kubectl get pvc -n inferno-prod

# Volume snapshots (if CSI driver available)
kubectl get volumesnapshot -n inferno-prod
```

## Monitoring Integration

Prometheus scrape configuration (annotations on deployment):

```yaml
prometheus.io/scrape: "true"
prometheus.io/port: "9090"
prometheus.io/path: "/metrics"
```

## Security

### Network Policies

- Ingress from ingress controller or same namespace
- Egress to DNS (53), HTTPS (443), and local traffic

### RBAC

- ServiceAccount `inferno` with minimal permissions
- Read-only access to ConfigMaps and Secrets
- Can create events for alerting

### Non-Root User

- Runs as UID 1000 (inferno user)
- No privilege escalation

### Pod Security Policy

Optional: Add PSP for additional security

## Advanced Customization

### Custom Resource Class

Override storage class:

```bash
kustomize build k8s/overlays/prod/ | \
  sed 's/storageClassName: standard/storageClassName: fast-ssd/' | \
  kubectl apply -f -
```

### Resource Quotas

Limit namespace resources:

```yaml
apiVersion: v1
kind: ResourceQuota
metadata:
  name: inferno-quota
  namespace: inferno-prod
spec:
  hard:
    pods: "20"
    requests.cpu: "20"
    requests.memory: "40Gi"
    limits.cpu: "40"
    limits.memory: "80Gi"
```

### Network Policy Customization

Edit `rbac.yaml` to restrict traffic further.

## Support

- **Documentation**: See `docs/DEPLOYMENT_GUIDE.md`
- **Issues**: https://github.com/ringo380/inferno/issues
- **Kubernetes Docs**: https://kubernetes.io/docs/

---

**Inferno Version**: v0.8.0
**Kubernetes**: 1.20+
**Last Updated**: 2024-Q4
