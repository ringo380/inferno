# 🚀 Production Deployment Guide

Complete guide for deploying Inferno in production environments with enterprise-grade reliability, security, and scalability.

## Overview

This guide covers everything you need for a robust production deployment:

- ✅ **Infrastructure Planning** - Sizing, architecture, and resource planning
- ✅ **Container Deployment** - Docker and Kubernetes production setups
- ✅ **Security Hardening** - Authentication, encryption, and access control
- ✅ **Monitoring & Observability** - Comprehensive monitoring stack
- ✅ **High Availability** - Failover, backup, and disaster recovery
- ✅ **Performance Tuning** - Optimization for maximum throughput
- ✅ **Maintenance** - Updates, scaling, and operational procedures

**Target Audience**: DevOps Engineers, Platform Engineers, System Administrators
**Prerequisites**: Basic Docker/Kubernetes knowledge, systems administration experience

## Infrastructure Planning

### System Requirements

#### Minimum Production Requirements

| Component | Specification | Purpose |
|-----------|---------------|---------|
| **CPU** | 8 cores, 3.0+ GHz | Inference processing |
| **RAM** | 32GB | Model loading + overhead |
| **Storage** | 500GB SSD | Models, cache, logs |
| **GPU** | 8GB VRAM (optional) | Acceleration |
| **Network** | 1Gbps | Model downloads, API traffic |

#### Recommended Production Requirements

| Component | Specification | Purpose |
|-----------|---------------|---------|
| **CPU** | 16+ cores, 3.5+ GHz | High-throughput inference |
| **RAM** | 64GB+ | Multiple large models |
| **Storage** | 2TB+ NVMe SSD | Extensive model library |
| **GPU** | 24GB+ VRAM | GPU acceleration |
| **Network** | 10Gbps | High-bandwidth operations |

#### Enterprise/High-Scale Requirements

| Component | Specification | Purpose |
|-----------|---------------|---------|
| **CPU** | 32+ cores, 4.0+ GHz | Extreme performance |
| **RAM** | 128GB+ | Massive model support |
| **Storage** | 10TB+ NVMe SSD RAID | Enterprise model repository |
| **GPU** | Multiple GPUs, 48GB+ each | Distributed GPU processing |
| **Network** | 25Gbps+ | Enterprise networking |

### Architecture Patterns

#### Single Node Deployment

```
┌─────────────────────────────────────────┐
│              Load Balancer              │
│          (nginx/HAProxy)                │
└─────────────────────────────────────────┘
                      │
┌─────────────────────────────────────────┐
│            Inferno Instance             │
│  ┌─────────────┐  ┌─────────────────┐   │
│  │   API       │  │    Models       │   │
│  │  Gateway    │  │   Backend       │   │
│  └─────────────┘  └─────────────────┘   │
│  ┌─────────────┐  ┌─────────────────┐   │
│  │    Cache    │  │   Monitoring    │   │
│  │   System    │  │    Stack        │   │
│  └─────────────┘  └─────────────────┘   │
└─────────────────────────────────────────┘
                      │
┌─────────────────────────────────────────┐
│           Storage Layer                 │
│  ┌─────────────┐  ┌─────────────────┐   │
│  │   Models    │  │     Logs        │   │
│  │   Storage   │  │   & Metrics     │   │
│  └─────────────┘  └─────────────────┘   │
└─────────────────────────────────────────┘
```

#### Multi-Node Deployment

```
┌─────────────────────────────────────────────────────────────────┐
│                         Load Balancer                           │
│                      (nginx/HAProxy/AWS ALB)                    │
└─────────────────────────────────────────────────────────────────┘
                                  │
           ┌──────────────────────┼──────────────────────┐
           │                      │                      │
           ▼                      ▼                      ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Inferno Node 1 │    │  Inferno Node 2 │    │  Inferno Node 3 │
│                 │    │                 │    │                 │
│  ┌───────────┐  │    │  ┌───────────┐  │    │  ┌───────────┐  │
│  │API Gateway│  │    │  │API Gateway│  │    │  │API Gateway│  │
│  └───────────┘  │    │  └───────────┘  │    │  └───────────┘  │
│  ┌───────────┐  │    │  ┌───────────┐  │    │  ┌───────────┐  │
│  │ Models A  │  │    │  │ Models B  │  │    │  │ Models C  │  │
│  └───────────┘  │    │  └───────────┘  │    │  └───────────┘  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
           │                      │                      │
           └──────────────────────┼──────────────────────┘
                                  │
┌─────────────────────────────────────────────────────────────────┐
│                      Shared Services                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
│  │  Shared     │  │ Prometheus  │  │   Redis     │              │
│  │  Storage    │  │  + Grafana  │  │   Cache     │              │
│  └─────────────┘  └─────────────┘  └─────────────┘              │
└─────────────────────────────────────────────────────────────────┘
```

## Docker Production Deployment

### Production Dockerfile

```dockerfile
# Multi-stage build for optimal production image
FROM rust:1.70-alpine AS builder

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    cmake \
    make \
    g++

WORKDIR /app
COPY . .

# Build release binary with optimization
RUN cargo build --release --locked
RUN strip target/release/inferno

# Production image
FROM alpine:3.18

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    openssl \
    libgcc \
    tini

# Create non-root user
RUN addgroup -g 1000 inferno && \
    adduser -D -s /bin/sh -u 1000 -G inferno inferno

# Create directories with proper permissions
RUN mkdir -p /data/{models,cache,logs} /etc/inferno && \
    chown -R inferno:inferno /data /etc/inferno

# Copy binary
COPY --from=builder /app/target/release/inferno /usr/local/bin/inferno
RUN chmod +x /usr/local/bin/inferno

# Switch to non-root user
USER inferno

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

EXPOSE 8080 9090
VOLUME ["/data"]

ENTRYPOINT ["/sbin/tini", "--"]
CMD ["inferno", "serve", "--config", "/etc/inferno/config.toml"]
```

### Docker Compose Production Stack

```yaml
# docker-compose.prod.yml
version: '3.8'

services:
  # Load Balancer
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx/nginx.conf:/etc/nginx/nginx.conf:ro
      - ./nginx/ssl:/etc/nginx/ssl:ro
    depends_on:
      - inferno-1
      - inferno-2
    restart: unless-stopped
    networks:
      - inferno-network

  # Inferno Instances
  inferno-1:
    image: inferno:latest
    ports:
      - "8081:8080"
      - "9091:9090"
    volumes:
      - inferno-models:/data/models
      - inferno-cache-1:/data/cache
      - ./config/inferno.toml:/etc/inferno/config.toml:ro
      - inferno-logs-1:/var/log/inferno
    environment:
      - INFERNO_LOG_LEVEL=info
      - INFERNO_GPU_ENABLED=true
      - INFERNO_INSTANCE_ID=inferno-1
    deploy:
      resources:
        limits:
          memory: 16G
          cpus: '8'
        reservations:
          memory: 8G
          cpus: '4'
    restart: unless-stopped
    networks:
      - inferno-network
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  inferno-2:
    image: inferno:latest
    ports:
      - "8082:8080"
      - "9092:9090"
    volumes:
      - inferno-models:/data/models  # Shared models
      - inferno-cache-2:/data/cache  # Separate cache
      - ./config/inferno.toml:/etc/inferno/config.toml:ro
      - inferno-logs-2:/var/log/inferno
    environment:
      - INFERNO_LOG_LEVEL=info
      - INFERNO_GPU_ENABLED=true
      - INFERNO_INSTANCE_ID=inferno-2
    deploy:
      resources:
        limits:
          memory: 16G
          cpus: '8'
        reservations:
          memory: 8G
          cpus: '4'
    restart: unless-stopped
    networks:
      - inferno-network
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  # Redis for coordination and caching
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis-data:/data
      - ./config/redis.conf:/usr/local/etc/redis/redis.conf:ro
    command: redis-server /usr/local/etc/redis/redis.conf
    deploy:
      resources:
        limits:
          memory: 2G
          cpus: '2'
    restart: unless-stopped
    networks:
      - inferno-network

  # Monitoring Stack
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--storage.tsdb.retention.time=30d'
      - '--web.console.libraries=/etc/prometheus/console_libraries'
      - '--web.console.templates=/etc/prometheus/consoles'
      - '--web.enable-lifecycle'
    restart: unless-stopped
    networks:
      - inferno-network

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    volumes:
      - grafana-data:/var/lib/grafana
      - ./monitoring/grafana/dashboards:/etc/grafana/provisioning/dashboards:ro
      - ./monitoring/grafana/datasources:/etc/grafana/provisioning/datasources:ro
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin123
      - GF_USERS_ALLOW_SIGN_UP=false
      - GF_INSTALL_PLUGINS=grafana-clock-panel,grafana-simple-json-datasource
    restart: unless-stopped
    networks:
      - inferno-network

  # Log aggregation
  loki:
    image: grafana/loki:latest
    ports:
      - "3100:3100"
    volumes:
      - ./monitoring/loki.yml:/etc/loki/local-config.yaml:ro
      - loki-data:/loki
    command: -config.file=/etc/loki/local-config.yaml
    restart: unless-stopped
    networks:
      - inferno-network

  promtail:
    image: grafana/promtail:latest
    volumes:
      - ./monitoring/promtail.yml:/etc/promtail/config.yml:ro
      - inferno-logs-1:/var/log/inferno-1:ro
      - inferno-logs-2:/var/log/inferno-2:ro
    command: -config.file=/etc/promtail/config.yml
    restart: unless-stopped
    networks:
      - inferno-network

volumes:
  inferno-models:
    driver: local
    driver_opts:
      type: none
      o: bind
      device: /data/inferno/models
  inferno-cache-1:
    driver: local
  inferno-cache-2:
    driver: local
  inferno-logs-1:
    driver: local
  inferno-logs-2:
    driver: local
  redis-data:
    driver: local
  prometheus-data:
    driver: local
  grafana-data:
    driver: local
  loki-data:
    driver: local

networks:
  inferno-network:
    driver: bridge
```

### NGINX Load Balancer Configuration

```nginx
# nginx/nginx.conf
events {
    worker_connections 1024;
}

http {
    upstream inferno_backend {
        least_conn;
        server inferno-1:8080 max_fails=3 fail_timeout=30s;
        server inferno-2:8080 max_fails=3 fail_timeout=30s;
    }

    # Rate limiting
    limit_req_zone $binary_remote_addr zone=api:10m rate=100r/s;
    limit_req_zone $binary_remote_addr zone=models:10m rate=10r/s;

    # Security headers
    add_header X-Frame-Options DENY;
    add_header X-Content-Type-Options nosniff;
    add_header X-XSS-Protection "1; mode=block";
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;

    server {
        listen 80;
        server_name your-domain.com;
        return 301 https://$server_name$request_uri;
    }

    server {
        listen 443 ssl http2;
        server_name your-domain.com;

        # SSL Configuration
        ssl_certificate /etc/nginx/ssl/cert.pem;
        ssl_certificate_key /etc/nginx/ssl/key.pem;
        ssl_protocols TLSv1.2 TLSv1.3;
        ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512;
        ssl_prefer_server_ciphers off;

        # Request size limits
        client_max_body_size 100M;

        # API endpoints
        location /v1/ {
            limit_req zone=api burst=200 nodelay;
            proxy_pass http://inferno_backend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;

            # Timeouts
            proxy_connect_timeout 60s;
            proxy_send_timeout 300s;
            proxy_read_timeout 300s;
        }

        # Model management endpoints
        location /v1/models/ {
            limit_req zone=models burst=50 nodelay;
            proxy_pass http://inferno_backend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }

        # WebSocket support
        location /ws {
            proxy_pass http://inferno_backend;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }

        # Health check
        location /health {
            proxy_pass http://inferno_backend;
            access_log off;
        }

        # Dashboard
        location /dashboard {
            proxy_pass http://inferno_backend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }

        # Metrics (restrict access)
        location /metrics {
            allow 10.0.0.0/8;
            allow 172.16.0.0/12;
            allow 192.168.0.0/16;
            deny all;
            proxy_pass http://inferno_backend;
        }
    }
}
```

## Kubernetes Deployment

### Namespace and RBAC

```yaml
# k8s/namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: inferno
  labels:
    name: inferno

---
apiVersion: v1
kind: ServiceAccount
metadata:
  name: inferno
  namespace: inferno

---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: inferno
rules:
- apiGroups: [""]
  resources: ["pods", "services", "endpoints"]
  verbs: ["get", "list", "watch"]
- apiGroups: ["apps"]
  resources: ["deployments", "replicasets"]
  verbs: ["get", "list", "watch"]

---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: inferno
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: ClusterRole
  name: inferno
subjects:
- kind: ServiceAccount
  name: inferno
  namespace: inferno
```

### ConfigMap and Secrets

```yaml
# k8s/configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: inferno-config
  namespace: inferno
data:
  inferno.toml: |
    models_dir = "/data/models"
    log_level = "info"
    log_format = "json"

    [server]
    bind_address = "0.0.0.0"
    port = 8080
    workers = 8

    [backend_config]
    gpu_enabled = true
    context_size = 4096
    batch_size = 64

    [cache]
    enabled = true
    max_size_gb = 20
    compression = "zstd"
    persist = true

    [security]
    auth_enabled = true
    rate_limit = 5000
    cors_enabled = true

    [observability]
    prometheus_enabled = true
    metrics_port = 9090
    tracing_enabled = true

---
apiVersion: v1
kind: Secret
metadata:
  name: inferno-secrets
  namespace: inferno
type: Opaque
data:
  jwt-secret: <base64-encoded-jwt-secret>
  api-key: <base64-encoded-api-key>
```

### Persistent Volumes

```yaml
# k8s/storage.yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: inferno-models-pvc
  namespace: inferno
spec:
  accessModes:
    - ReadWriteMany
  storageClassName: fast-ssd
  resources:
    requests:
      storage: 1Ti

---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: inferno-cache-pvc
  namespace: inferno
spec:
  accessModes:
    - ReadWriteOnce
  storageClassName: fast-ssd
  resources:
    requests:
      storage: 500Gi
```

### Deployment

```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: inferno
  namespace: inferno
  labels:
    app: inferno
spec:
  replicas: 3
  selector:
    matchLabels:
      app: inferno
  template:
    metadata:
      labels:
        app: inferno
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "9090"
        prometheus.io/path: "/metrics"
    spec:
      serviceAccountName: inferno
      containers:
      - name: inferno
        image: inferno:latest
        ports:
        - containerPort: 8080
          name: http
        - containerPort: 9090
          name: metrics
        env:
        - name: INFERNO_LOG_LEVEL
          value: "info"
        - name: INFERNO_INSTANCE_ID
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        - name: INFERNO_JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: inferno-secrets
              key: jwt-secret
        volumeMounts:
        - name: config
          mountPath: /etc/inferno
          readOnly: true
        - name: models
          mountPath: /data/models
        - name: cache
          mountPath: /data/cache
        resources:
          requests:
            memory: "8Gi"
            cpu: "4"
            nvidia.com/gpu: 1
          limits:
            memory: "16Gi"
            cpu: "8"
            nvidia.com/gpu: 1
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 60
          periodSeconds: 30
          timeoutSeconds: 10
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
        startupProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 30
      volumes:
      - name: config
        configMap:
          name: inferno-config
      - name: models
        persistentVolumeClaim:
          claimName: inferno-models-pvc
      - name: cache
        persistentVolumeClaim:
          claimName: inferno-cache-pvc
      nodeSelector:
        inferno.ai/node-type: "gpu"
      tolerations:
      - key: "nvidia.com/gpu"
        operator: "Exists"
        effect: "NoSchedule"
```

### Service and Ingress

```yaml
# k8s/service.yaml
apiVersion: v1
kind: Service
metadata:
  name: inferno-service
  namespace: inferno
  labels:
    app: inferno
spec:
  selector:
    app: inferno
  ports:
  - name: http
    port: 80
    targetPort: 8080
  - name: metrics
    port: 9090
    targetPort: 9090
  type: ClusterIP

---
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: inferno-ingress
  namespace: inferno
  annotations:
    kubernetes.io/ingress.class: "nginx"
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
    nginx.ingress.kubernetes.io/rate-limit: "100"
    nginx.ingress.kubernetes.io/rate-limit-window: "1m"
    nginx.ingress.kubernetes.io/proxy-body-size: "100m"
    nginx.ingress.kubernetes.io/proxy-read-timeout: "300"
    nginx.ingress.kubernetes.io/proxy-send-timeout: "300"
spec:
  tls:
  - hosts:
    - api.yourcompany.com
    secretName: inferno-tls
  rules:
  - host: api.yourcompany.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: inferno-service
            port:
              number: 80
```

### Horizontal Pod Autoscaler

```yaml
# k8s/hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: inferno-hpa
  namespace: inferno
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: inferno
  minReplicas: 3
  maxReplicas: 20
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  - type: Pods
    pods:
      metric:
        name: inferno_requests_per_second
      target:
        type: AverageValue
        averageValue: "100"
  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 10
        periodSeconds: 60
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
      - type: Percent
        value: 50
        periodSeconds: 60
      - type: Pods
        value: 2
        periodSeconds: 60
      selectPolicy: Max
```

## Security Configuration

### Authentication Setup

```bash
# Initialize security system
inferno security init --production

# Create service accounts
inferno security user add api-service --role service --scopes "inference,models"
inferno security user add monitoring --role readonly --scopes "metrics,health"
inferno security user add admin --role admin --scopes "*"

# Generate API keys
inferno security key create --name production-api --user api-service --expires 365d
inferno security key create --name monitoring-key --user monitoring --expires 30d

# Configure JWT settings
inferno config set security.jwt_expiry 24h
inferno config set security.jwt_refresh_enabled true
inferno config set security.jwt_issuer "inferno.yourcompany.com"
```

### Network Security

```bash
# Configure firewall rules
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow 22/tcp    # SSH
sudo ufw allow 80/tcp    # HTTP
sudo ufw allow 443/tcp   # HTTPS
sudo ufw allow from 10.0.0.0/8 to any port 9090  # Metrics (internal only)
sudo ufw enable

# Configure rate limiting
inferno config set security.rate_limit 5000
inferno config set security.burst_limit 1000
inferno config set security.rate_limit_window "1m"

# Enable IP filtering
inferno security config --ip-whitelist "10.0.0.0/8,172.16.0.0/12,192.168.0.0/16"
inferno security config --block-tor true
inferno security config --block-cloud-providers false
```

### TLS/SSL Configuration

```bash
# Generate SSL certificates (using Let's Encrypt)
sudo certbot certonly --nginx -d api.yourcompany.com

# Or use custom certificates
openssl req -x509 -nodes -days 365 -newkey rsa:4096 \
  -keyout /etc/ssl/private/inferno.key \
  -out /etc/ssl/certs/inferno.crt \
  -subj "/C=US/ST=State/L=City/O=Organization/CN=api.yourcompany.com"

# Configure Inferno for SSL
inferno config set server.ssl_enabled true
inferno config set server.ssl_cert "/etc/ssl/certs/inferno.crt"
inferno config set server.ssl_key "/etc/ssl/private/inferno.key"
```

### Audit Logging

```bash
# Enable comprehensive audit logging
inferno audit enable --level full
inferno audit config --retention 90d
inferno audit config --compression gzip
inferno audit config --encryption aes256

# Configure audit alerts
inferno audit alerts --email security@yourcompany.com
inferno audit alerts --webhook https://alerts.yourcompany.com/audit
inferno audit alerts --slack-webhook https://hooks.slack.com/services/xxx
```

## Monitoring and Observability

### Prometheus Configuration

```yaml
# monitoring/prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "alert_rules.yml"

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093

scrape_configs:
  - job_name: 'inferno'
    static_configs:
      - targets: ['inferno-1:9090', 'inferno-2:9090']
    scrape_interval: 10s
    metrics_path: /metrics

  - job_name: 'node-exporter'
    static_configs:
      - targets: ['node-exporter:9100']

  - job_name: 'nvidia-gpu'
    static_configs:
      - targets: ['nvidia-gpu-exporter:9445']

  - job_name: 'redis'
    static_configs:
      - targets: ['redis:6379']
```

### Alert Rules

```yaml
# monitoring/alert_rules.yml
groups:
- name: inferno.rules
  rules:
  - alert: InfernoInstanceDown
    expr: up{job="inferno"} == 0
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: "Inferno instance {{ $labels.instance }} is down"
      description: "Inferno instance has been down for more than 5 minutes"

  - alert: HighLatency
    expr: inferno_request_duration_seconds{quantile="0.95"} > 2.0
    for: 10m
    labels:
      severity: warning
    annotations:
      summary: "High latency detected"
      description: "95th percentile latency is {{ $value }}s"

  - alert: HighMemoryUsage
    expr: inferno_memory_usage_percent > 90
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: "High memory usage on {{ $labels.instance }}"
      description: "Memory usage is {{ $value }}%"

  - alert: ModelLoadFailure
    expr: increase(inferno_model_load_failures_total[5m]) > 0
    for: 0s
    labels:
      severity: warning
    annotations:
      summary: "Model load failure detected"
      description: "{{ $value }} model load failures in the last 5 minutes"

  - alert: GPUUtilizationLow
    expr: nvidia_gpu_utilization_percent < 10
    for: 30m
    labels:
      severity: info
    annotations:
      summary: "Low GPU utilization"
      description: "GPU utilization has been below 10% for 30 minutes"
```

### Grafana Dashboards

```json
{
  "dashboard": {
    "id": null,
    "title": "Inferno Production Dashboard",
    "tags": ["inferno", "ai", "production"],
    "panels": [
      {
        "title": "Request Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(inferno_requests_total[5m])",
            "legendFormat": "{{ instance }}"
          }
        ]
      },
      {
        "title": "Response Latency",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(inferno_request_duration_seconds_bucket[5m]))",
            "legendFormat": "95th percentile"
          },
          {
            "expr": "histogram_quantile(0.50, rate(inferno_request_duration_seconds_bucket[5m]))",
            "legendFormat": "50th percentile"
          }
        ]
      },
      {
        "title": "GPU Utilization",
        "type": "graph",
        "targets": [
          {
            "expr": "nvidia_gpu_utilization_percent",
            "legendFormat": "GPU {{ gpu }}"
          }
        ]
      },
      {
        "title": "Memory Usage",
        "type": "graph",
        "targets": [
          {
            "expr": "inferno_memory_usage_bytes / 1024 / 1024 / 1024",
            "legendFormat": "{{ instance }}"
          }
        ]
      }
    ]
  }
}
```

## Performance Tuning

### Hardware Optimization

```bash
# CPU optimization
echo 'performance' | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

# Disable CPU frequency scaling
sudo systemctl disable ondemand

# Configure NUMA
numactl --hardware
numactl --cpubind=0 --membind=0 inferno serve

# GPU optimization
nvidia-smi -pm 1  # Enable persistence mode
nvidia-smi -ac 5001,1590  # Set memory and GPU clocks
```

### Application Tuning

```toml
# inferno.toml - Production tuning
[server]
bind_address = "0.0.0.0"
port = 8080
workers = 16  # 2x CPU cores
max_connections = 1000
keep_alive_timeout = 30

[backend_config]
gpu_enabled = true
context_size = 4096
batch_size = 128  # Larger batch for throughput
prefill_batch_size = 256
tensor_parallel_size = 2  # Multi-GPU

[cache]
enabled = true
max_size_gb = 50
compression = "zstd"
persist = true
cache_ttl = 3600
prefetch_enabled = true

[performance]
async_workers = 8
io_threads = 4
compute_threads = 16
memory_pool_size = "32GB"
```

### Database Optimization

```bash
# PostgreSQL tuning (if using)
sudo -u postgres psql -c "
ALTER SYSTEM SET shared_buffers = '8GB';
ALTER SYSTEM SET effective_cache_size = '24GB';
ALTER SYSTEM SET maintenance_work_mem = '2GB';
ALTER SYSTEM SET checkpoint_completion_target = 0.9;
ALTER SYSTEM SET wal_buffers = '16MB';
ALTER SYSTEM SET default_statistics_target = 100;
ALTER SYSTEM SET random_page_cost = 1.1;
SELECT pg_reload_conf();
"

# Redis tuning
echo 'vm.overcommit_memory = 1' | sudo tee -a /etc/sysctl.conf
echo 'net.core.somaxconn = 65535' | sudo tee -a /etc/sysctl.conf
sudo sysctl -p
```

## High Availability and Disaster Recovery

### Backup Strategy

```bash
#!/bin/bash
# backup.sh - Comprehensive backup script

BACKUP_DIR="/backup/inferno/$(date +%Y%m%d_%H%M%S)"
mkdir -p "$BACKUP_DIR"

# Backup models
echo "Backing up models..."
rsync -av --progress /data/inferno/models/ "$BACKUP_DIR/models/"

# Backup configuration
echo "Backing up configuration..."
cp -r /etc/inferno/ "$BACKUP_DIR/config/"

# Backup database
echo "Backing up database..."
inferno admin export-db "$BACKUP_DIR/database.sql"

# Backup metrics (last 7 days)
echo "Backing up metrics..."
prometheus_backup --retention 7d "$BACKUP_DIR/metrics/"

# Create manifest
cat > "$BACKUP_DIR/manifest.json" << EOF
{
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "version": "$(inferno --version)",
  "models": $(inferno list --format json),
  "config_hash": "$(find /etc/inferno -type f -exec sha256sum {} \; | sha256sum | cut -d' ' -f1)"
}
EOF

# Compress backup
echo "Compressing backup..."
tar -czf "$BACKUP_DIR.tar.gz" -C "$(dirname "$BACKUP_DIR")" "$(basename "$BACKUP_DIR")"
rm -rf "$BACKUP_DIR"

# Upload to cloud storage
aws s3 cp "$BACKUP_DIR.tar.gz" s3://company-backups/inferno/

echo "Backup completed: $BACKUP_DIR.tar.gz"
```

### Disaster Recovery Plan

```bash
#!/bin/bash
# disaster_recovery.sh - Disaster recovery procedures

set -e

BACKUP_FILE="$1"
if [ -z "$BACKUP_FILE" ]; then
    echo "Usage: $0 <backup_file.tar.gz>"
    exit 1
fi

echo "Starting disaster recovery from $BACKUP_FILE..."

# Stop services
docker-compose down
systemctl stop inferno

# Create recovery directory
RECOVERY_DIR="/tmp/inferno_recovery_$(date +%s)"
mkdir -p "$RECOVERY_DIR"

# Extract backup
echo "Extracting backup..."
tar -xzf "$BACKUP_FILE" -C "$RECOVERY_DIR"

# Restore models
echo "Restoring models..."
rsync -av "$RECOVERY_DIR"/*/models/ /data/inferno/models/

# Restore configuration
echo "Restoring configuration..."
cp -r "$RECOVERY_DIR"/*/config/* /etc/inferno/

# Restore database
echo "Restoring database..."
inferno admin import-db "$RECOVERY_DIR"/*/database.sql

# Validate restoration
echo "Validating restoration..."
inferno validate --all

# Start services
systemctl start inferno
docker-compose up -d

# Health check
sleep 30
curl -f http://localhost:8080/health || {
    echo "Health check failed!"
    exit 1
}

echo "Disaster recovery completed successfully!"
```

### Failover Configuration

```yaml
# k8s/pod-disruption-budget.yaml
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: inferno-pdb
  namespace: inferno
spec:
  minAvailable: 2
  selector:
    matchLabels:
      app: inferno
```

## Maintenance Procedures

### Rolling Updates

```bash
#!/bin/bash
# rolling_update.sh - Zero-downtime updates

NEW_VERSION="$1"
if [ -z "$NEW_VERSION" ]; then
    echo "Usage: $0 <new_version>"
    exit 1
fi

echo "Starting rolling update to version $NEW_VERSION..."

# Update image
kubectl set image deployment/inferno inferno=inferno:$NEW_VERSION -n inferno

# Wait for rollout
kubectl rollout status deployment/inferno -n inferno --timeout=600s

# Verify deployment
kubectl get pods -n inferno -l app=inferno

# Run health checks
for i in {1..5}; do
    if curl -f http://api.yourcompany.com/health; then
        echo "Health check $i/5 passed"
        sleep 10
    else
        echo "Health check $i/5 failed"
        kubectl rollout undo deployment/inferno -n inferno
        exit 1
    fi
done

echo "Rolling update completed successfully!"
```

### Scaling Procedures

```bash
# Manual scaling
kubectl scale deployment inferno --replicas=10 -n inferno

# Update HPA limits
kubectl patch hpa inferno-hpa -n inferno -p '{"spec":{"maxReplicas":20}}'

# Vertical scaling (if using VPA)
kubectl patch vpa inferno-vpa -n inferno -p '{"spec":{"resourcePolicy":{"containerPolicies":[{"containerName":"inferno","maxAllowed":{"memory":"32Gi","cpu":"16"}}]}}}'
```

### Log Management

```bash
#!/bin/bash
# log_management.sh - Log rotation and cleanup

# Rotate logs
logrotate -f /etc/logrotate.d/inferno

# Clean old logs (keep 30 days)
find /var/log/inferno -name "*.log" -mtime +30 -delete

# Compress logs older than 7 days
find /var/log/inferno -name "*.log" -mtime +7 -exec gzip {} \;

# Export logs to centralized logging
filebeat -c /etc/filebeat/filebeat.yml
```

## Troubleshooting

### Common Production Issues

#### High Memory Usage

```bash
# Identify memory usage
inferno metrics show --filter memory
ps aux | grep inferno
cat /proc/$(pgrep inferno)/status

# Solutions
inferno config set backend_config.context_size 2048
inferno cache clear --force
inferno models unload --unused
```

#### High Latency

```bash
# Diagnose latency
inferno metrics show --filter latency
inferno bench --model current-model

# Solutions
inferno cache warm --popular
inferno config set backend_config.batch_size 256
inferno models optimize --quantization q4_0
```

#### GPU Issues

```bash
# Check GPU status
nvidia-smi
inferno gpu status

# Reset GPU
sudo nvidia-smi --gpu-reset
sudo systemctl restart nvidia-persistenced
```

### Health Monitoring

```bash
#!/bin/bash
# health_check.sh - Comprehensive health monitoring

echo "=== Inferno Health Check ==="

# Service status
echo "Service Status:"
systemctl status inferno

# API health
echo "API Health:"
curl -s http://localhost:8080/health | jq '.'

# Model status
echo "Model Status:"
inferno models status --all

# Resource usage
echo "Resource Usage:"
inferno metrics show --format compact

# GPU status
echo "GPU Status:"
nvidia-smi --query-gpu=name,utilization.gpu,memory.used,memory.total --format=csv

# Disk usage
echo "Disk Usage:"
df -h /data/inferno

# Network connectivity
echo "Network Connectivity:"
curl -s https://api.github.com > /dev/null && echo "External: OK" || echo "External: FAIL"

echo "=== Health Check Complete ==="
```

## Cost Optimization

### Resource Right-sizing

```bash
# Analyze resource usage
kubectl top pods -n inferno
kubectl describe hpa inferno-hpa -n inferno

# Optimize resource requests/limits
kubectl patch deployment inferno -n inferno -p '{"spec":{"template":{"spec":{"containers":[{"name":"inferno","resources":{"requests":{"memory":"6Gi","cpu":"3"},"limits":{"memory":"12Gi","cpu":"6"}}}]}}}}'
```

### Model Optimization

```bash
# Use quantized models for cost reduction
inferno install llama-2-7b-chat-q4_0  # 4-bit quantization
inferno remove llama-2-7b-chat-f16    # Remove full precision

# Enable model sharing
inferno config set cache.shared_models true
inferno config set cache.model_sharing_enabled true
```

This production deployment guide provides a comprehensive foundation for running Inferno at enterprise scale with high availability, security, and performance. Adjust the configurations based on your specific requirements and constraints.

## Next Steps

1. **[Security Configuration](security.md)** - Deep dive into security hardening
2. **[Monitoring and Observability](monitoring.md)** - Advanced monitoring setup
3. **[Performance Tuning](performance-tuning.md)** - Optimization strategies
4. **[Troubleshooting Guide](troubleshooting.md)** - Production issue resolution