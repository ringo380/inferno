# Inferno v0.8.0 Deployment Guide

## Overview

This guide covers deploying Inferno for development, staging, and production environments using Docker and Docker Compose.

## Quick Start

### Local Development

```bash
# Clone environment variables
cp .env.example .env

# Build and run
docker-compose up -d

# Check logs
docker-compose logs -f inferno

# Stop
docker-compose down
```

### Staging Deployment

```bash
# Run with staging configuration (2 replicas)
docker-compose -f docker-compose.yml -f docker-compose.staging.yml up -d

# Scale instances
docker-compose -f docker-compose.staging.yml up -d --scale inferno=2

# Check status
docker-compose ps
```

### Production Deployment

```bash
# Prepare data directories
mkdir -p data/{models,cache,queue,logs,prometheus}

# Build production image
docker buildx build --platform linux/amd64,linux/arm64 -t inferno:0.8.0 .

# Run with production configuration (3 replicas + LB + monitoring)
docker-compose -f docker-compose.prod.yml up -d

# Verify all services
docker-compose -f docker-compose.prod.yml ps

# Monitor logs
docker-compose -f docker-compose.prod.yml logs -f
```

---

## Configuration

### Environment Variables

Copy `.env.example` to `.env` and configure:

```bash
cp .env.example .env
nano .env
```

**Key Variables:**
- `INFERNO_MODELS_DIR` - Model storage location
- `INFERNO_CACHE_DIR` - Cache storage location
- `INFERNO_LOG_LEVEL` - Logging level (debug, info, warn, error)
- `INFERNO_GPU_ENABLED` - Enable GPU acceleration (true/false)

### Docker Compose Files

| File | Purpose | Use Case |
|------|---------|----------|
| `docker-compose.yml` | Base configuration | Development, single instance |
| `docker-compose.staging.yml` | 2-replica setup | Staging environment |
| `docker-compose.prod.yml` | Full production | Production with LB, monitoring |

### Volume Management

Inferno uses named volumes for persistence:

| Volume | Purpose | Persistence |
|--------|---------|-------------|
| `models` | Model files | ✅ Required |
| `cache` | Inference cache | ✅ Recommended |
| `queue` | Queue checkpoints | ✅ Recommended |
| `logs` | Application logs | Optional |

---

## Dockerfile Optimization

### Image Size

The multi-stage Dockerfile optimizes for size:

```
Builder Stage:
  - Full Rust compiler + dependencies
  - Compilation happens here
  - Discarded after compilation

Runtime Stage:
  - Minimal Debian slim base
  - Only runtime dependencies
  - Compiled binary only
  - ~300-400MB final size
```

### Health Checks

Docker health checks are configured:

```dockerfile
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8000/health || exit 1
```

- **Interval**: Check every 30 seconds
- **Timeout**: Wait max 5 seconds for response
- **Start Period**: Allow 10 seconds for startup
- **Retries**: Fail after 3 consecutive failures

### Non-Root User

Runs as non-root user for security:

```dockerfile
USER inferno (UID: 1000)
```

---

## Resource Management

### Memory Limits

Configure memory based on model size:

| Model Size | Memory Limit | Memory Reserve |
|-----------|--------------|-----------------|
| 7B | 4GB | 2GB |
| 13B | 8GB | 4GB |
| 70B | 16GB | 8GB |

In `docker-compose.yml`:

```yaml
deploy:
  resources:
    limits:
      memory: 4G
    reservations:
      memory: 2G
```

### CPU Allocation

```yaml
deploy:
  resources:
    limits:
      cpus: '2.0'    # Max 2 CPUs
    reservations:
      cpus: '1.0'    # Reserve 1 CPU
```

---

## Network Configuration

### Ports

| Port | Service | Purpose |
|------|---------|---------|
| 8000 | API | Main inference API |
| 9090 | Prometheus | Metrics collection |
| 80 | Nginx (Prod) | HTTP load balancer |
| 443 | Nginx (Prod) | HTTPS load balancer |

### Custom Networks

```yaml
networks:
  default:
    name: inferno-network
    driver: bridge
```

---

## Data Persistence

### Backup Strategy

```bash
# Backup models
tar -czf models-backup-$(date +%Y%m%d).tar.gz data/models/

# Backup queue state
tar -czf queue-backup-$(date +%Y%m%d).tar.gz data/queue/

# Restore
tar -xzf models-backup-20240101.tar.gz
```

### Volume Verification

```bash
# Check volume space
docker exec inferno df -h /home/inferno/.inferno

# List volumes
docker volume ls

# Inspect volume
docker volume inspect inferno_models
```

---

## Monitoring

### Health Checks

```bash
# Check container health
docker inspect --format='{{.State.Health.Status}}' inferno

# Test health endpoint
curl http://localhost:8000/health

# View logs
docker logs inferno
```

### Metrics Collection

In production (`docker-compose.prod.yml`):

```bash
# Prometheus is available at http://localhost:9090

# Query metrics
curl http://localhost:9090/api/v1/query?query=inferno_requests_total
```

### Logging

```bash
# View logs
docker-compose logs inferno

# Follow logs
docker-compose logs -f inferno

# Show last 100 lines
docker-compose logs --tail=100 inferno

# Export logs
docker logs inferno > inferno.log 2>&1
```

---

## Production Deployment Checklist

- [ ] SSL/TLS certificates obtained
- [ ] Environment variables configured (.env)
- [ ] Data directories created (data/)
- [ ] Backup strategy verified
- [ ] Monitoring configured (Prometheus)
- [ ] Load balancer tested
- [ ] Health checks passing
- [ ] Resource limits verified
- [ ] Logging configured
- [ ] Database backups scheduled
- [ ] Disaster recovery plan documented

---

## Troubleshooting

### Container won't start

```bash
# Check logs
docker-compose logs inferno

# Check image exists
docker images inferno

# Verify no port conflicts
docker ps | grep 8000

# Try building again
docker-compose build --no-cache
```

### Out of memory

```bash
# Check memory usage
docker stats inferno

# Increase limit in docker-compose.yml
# limits.memory: 8G

# Restart with new limits
docker-compose restart inferno
```

### Health checks failing

```bash
# Test health endpoint manually
docker exec inferno curl http://localhost:8000/health

# View detailed container health
docker inspect inferno | grep -A 10 Health

# Check if port 8000 is accessible
docker exec inferno netstat -tlnp | grep 8000
```

### Slow inference

```bash
# Check resource usage
docker stats inferno

# Monitor queue
docker exec inferno curl http://localhost:8000/metrics/queue/status

# Check logs for errors
docker-compose logs inferno | grep -i error
```

---

## Security Best Practices

### Image Security

1. **Non-root user** - Runs as `inferno` user (UID: 1000)
2. **Minimal image** - Only runtime dependencies
3. **Health checks** - Automatic restart on failure
4. **No secrets in image** - Use environment variables

### Container Security

```yaml
# Read-only filesystem (except volumes)
# security_opt:
#   - no-new-privileges:true

# Drop unnecessary capabilities
# cap_drop:
#   - ALL
# cap_add:
#   - NET_BIND_SERVICE
```

### Network Security

1. Use HTTPS in production (HTTPS_ENABLED=true)
2. Put behind load balancer with TLS termination
3. Use authenticated API keys (AUTH_ENABLED=true)
4. Implement rate limiting at load balancer

---

## Performance Tuning

### Optimization Tips

1. **Model Loading**
   - Pre-load frequently used models
   - Use GPU acceleration (INFERNO_GPU_ENABLED=true)
   - Increase batch size for throughput

2. **Caching**
   - Increase INFERNO_CACHE_SIZE_MB for high hit rates
   - Enable compression (INFERNO_CACHE_COMPRESSION_ENABLED=true)

3. **Worker Threads**
   - Set INFERNO_WORKER_THREADS = CPU cores
   - Increase for I/O-heavy workloads

4. **Streaming**
   - Adjust INFERNO_TOKEN_BATCH_SIZE (1 = low latency, 10+ = high throughput)
   - Enable compression for bandwidth-constrained networks

---

## Scaling

### Horizontal Scaling

```bash
# Scale to 5 instances
docker-compose up -d --scale inferno=5

# With load balancer (nginx)
docker-compose -f docker-compose.prod.yml up -d
```

### Vertical Scaling

Increase resources in docker-compose.yml:

```yaml
deploy:
  resources:
    limits:
      cpus: '4.0'
      memory: 8G
```

---

## Maintenance

### Regular Tasks

- **Daily**: Review logs for errors
- **Weekly**: Verify backups working
- **Monthly**: Update dependencies
- **Quarterly**: Security audit

### Cleanup

```bash
# Remove stopped containers
docker container prune

# Remove unused volumes
docker volume prune

# Remove unused images
docker image prune

# Full cleanup (be careful!)
docker system prune -a
```

---

## Support & Documentation

- **API Docs**: See `API_DOCUMENTATION.md`
- **Configuration**: See `.env.example` for all options
- **Troubleshooting**: See this section above
- **GitHub Issues**: https://github.com/ringo380/inferno/issues

---

**Last Updated**: 2024-Q4
**Inferno Version**: v0.8.0
