# 🔧 Troubleshooting Guide

Comprehensive troubleshooting guide for common Inferno issues, from installation problems to production deployment challenges.

## Overview

This guide provides solutions for:
- ✅ **Installation and Setup Issues** - Get Inferno running smoothly
- ✅ **Model Management Problems** - Download, conversion, and loading issues
- ✅ **Performance Issues** - Slow inference, memory problems, GPU issues
- ✅ **API and Connectivity Problems** - Server errors, timeouts, authentication
- ✅ **Production Issues** - Scaling, monitoring, and deployment problems
- ✅ **System-Level Issues** - OS-specific problems and resource constraints

## Quick Diagnostic Commands

Start troubleshooting with these essential diagnostic commands:

```bash
# System health check
inferno --version
inferno config show
inferno gpu status
inferno models list

# Server status
curl http://localhost:8080/health
inferno metrics show

# Resource usage
inferno cache status
inferno memory analyze
df -h $(inferno config get models_dir)

# Logs analysis
tail -f ~/.local/share/inferno/logs/inferno.log
journalctl -u inferno -f  # For systemd
```

## Installation and Setup Issues

### Issue: "Command not found: inferno"

**Symptoms:**
```bash
$ inferno --help
bash: inferno: command not found
```

**Diagnosis:**
```bash
# Check if binary exists
which inferno
ls -la /usr/local/bin/inferno

# Check PATH
echo $PATH
```

**Solutions:**

1. **Add to PATH (most common):**
```bash
# Temporary fix
export PATH="$PATH:/usr/local/bin"

# Permanent fix
echo 'export PATH="$PATH:/usr/local/bin"' >> ~/.bashrc
source ~/.bashrc

# Or for zsh
echo 'export PATH="$PATH:/usr/local/bin"' >> ~/.zshrc
source ~/.zshrc
```

2. **Install/reinstall binary:**
```bash
# Download and install
wget https://github.com/ringo380/inferno/releases/latest/download/inferno-linux-x86_64.tar.gz
tar xzf inferno-linux-x86_64.tar.gz
sudo mv inferno /usr/local/bin/
sudo chmod +x /usr/local/bin/inferno
```

3. **Build from source:**
```bash
git clone https://github.com/ringo380/inferno.git
cd inferno
cargo build --release
sudo cp target/release/inferno /usr/local/bin/
```

### Issue: Permission Denied Errors

**Symptoms:**
```bash
$ inferno serve
Permission denied (os error 13)
```

**Diagnosis:**
```bash
# Check file permissions
ls -la /usr/local/bin/inferno

# Check directory permissions
ls -la ~/.local/share/inferno/
```

**Solutions:**

1. **Fix binary permissions:**
```bash
sudo chmod +x /usr/local/bin/inferno
```

2. **Fix data directory permissions:**
```bash
mkdir -p ~/.local/share/inferno/{models,cache,logs}
chmod -R 755 ~/.local/share/inferno/
```

3. **Run with correct user:**
```bash
# Don't run as root unless necessary
whoami  # Should not be root

# If running as service, check user
sudo systemctl status inferno
```

### Issue: Port Already in Use

**Symptoms:**
```bash
Error binding to 0.0.0.0:8080: Address already in use
```

**Diagnosis:**
```bash
# Check what's using the port
lsof -i :8080
netstat -tulpn | grep 8080

# Check if Inferno is already running
ps aux | grep inferno
```

**Solutions:**

1. **Kill existing process:**
```bash
# Find and kill the process
sudo kill $(lsof -t -i:8080)

# Or kill all inferno processes
pkill inferno
```

2. **Use different port:**
```bash
inferno serve --port 8081
# Or set in config
inferno config set server.port 8081
```

3. **Check systemd service:**
```bash
sudo systemctl status inferno
sudo systemctl stop inferno  # If running as service
```

### Issue: Configuration File Problems

**Symptoms:**
```bash
Error parsing config file: invalid TOML syntax
```

**Diagnosis:**
```bash
# Check config file syntax
inferno config validate

# Show current config
inferno config show

# Find config files
find ~ -name "*.toml" -path "*inferno*"
```

**Solutions:**

1. **Validate and fix TOML:**
```bash
# Check TOML syntax online or use tools
python3 -c "import toml; toml.load('~/.config/inferno/config.toml')"

# Regenerate default config
inferno config init --force
```

2. **Common TOML mistakes:**
```toml
# Wrong: Missing quotes
models_dir = /data/models

# Correct: Quoted paths
models_dir = "/data/models"

# Wrong: Invalid section
[backend.config]
gpu_enabled = true

# Correct: Valid section
[backend_config]
gpu_enabled = true
```

## Model Management Issues

### Issue: Model Download Failures

**Symptoms:**
```bash
Error downloading model: HTTP 404 Not Found
Error downloading model: Connection timeout
```

**Diagnosis:**
```bash
# Test internet connectivity
ping huggingface.co
curl -I https://huggingface.co

# Check repository configuration
inferno repo list
inferno repo test huggingface

# Check disk space
df -h $(inferno config get models_dir)
```

**Solutions:**

1. **Network connectivity issues:**
```bash
# Check proxy settings
echo $HTTP_PROXY $HTTPS_PROXY

# Test download manually
wget https://huggingface.co/gpt2/resolve/main/config.json

# Use different DNS
echo "nameserver 8.8.8.8" | sudo tee -a /etc/resolv.conf
```

2. **Repository authentication:**
```bash
# Set Hugging Face token
inferno repo auth huggingface --token your_hf_token

# Test authentication
inferno repo test huggingface
```

3. **Disk space issues:**
```bash
# Clean up space
inferno cache clear
inferno remove --unused

# Change models directory
inferno config set models_dir "/larger/disk/models"
```

4. **Retry with different options:**
```bash
# Force retry
inferno install model-name --retry --force

# Use different repository
inferno install model-name --repo ollama

# Manual download
inferno models download model-name --direct
```

### Issue: Model Loading Failures

**Symptoms:**
```bash
Error loading model: Invalid GGUF file format
Error loading model: Out of memory
Model validation failed: Checksum mismatch
```

**Diagnosis:**
```bash
# Validate model file
inferno validate model-name
file /path/to/model.gguf

# Check available memory
free -h
inferno memory status

# Check model info
inferno models info model-name
```

**Solutions:**

1. **Corrupted model files:**
```bash
# Re-download model
inferno remove model-name --purge
inferno install model-name --verify

# Check file integrity
inferno validate model-name --checksum
```

2. **Memory issues:**
```bash
# Use smaller context size
inferno config set backend_config.context_size 1024

# Enable memory mapping
inferno config set backend_config.memory_mapping true

# Use quantized model
inferno install model-name-q4_0
```

3. **Format issues:**
```bash
# Convert to compatible format
inferno convert model.pt model.gguf --format gguf

# Check supported formats
inferno models formats
```

### Issue: Model Conversion Problems

**Symptoms:**
```bash
Conversion failed: Unsupported source format
Conversion failed: Out of disk space
Conversion timeout after 300s
```

**Diagnosis:**
```bash
# Check source file
file input_model.pt
inferno validate input_model.pt

# Check disk space
df -h /tmp/
df -h $(pwd)

# Check conversion status
inferno convert status conversion-job-id
```

**Solutions:**

1. **Format compatibility:**
```bash
# Check supported conversions
inferno convert --list-formats

# Use intermediate format
inferno convert model.pt model.onnx --format onnx
inferno convert model.onnx model.gguf --format gguf
```

2. **Resource issues:**
```bash
# Increase timeout
inferno convert model.pt model.gguf --timeout 1800

# Use more memory
inferno convert model.pt model.gguf --memory 16GB

# Convert in chunks
inferno convert model.pt model.gguf --chunk-size 1GB
```

## Performance Issues

### Issue: Slow Inference

**Symptoms:**
- High latency (>5 seconds per request)
- Low throughput (<1 request/second)
- CPU usage consistently high

**Diagnosis:**
```bash
# Benchmark current performance
inferno bench --model current-model --detailed

# Check GPU usage
nvidia-smi  # or rocm-smi for AMD
inferno gpu status

# Profile inference
inferno profile --model current-model --duration 60s
```

**Solutions:**

1. **Enable GPU acceleration:**
```bash
# Check GPU availability
inferno gpu status

# Enable GPU
inferno config set backend_config.gpu_enabled true

# Verify GPU usage
inferno run --model gpt2 --prompt "test" --verbose
```

2. **Optimize model and settings:**
```bash
# Use quantized model
inferno install model-name-q4_0

# Optimize batch size
inferno config set backend_config.batch_size 64

# Increase GPU layers
inferno config set backend_config.gpu_layers 35
```

3. **System-level optimizations:**
```bash
# Set CPU governor to performance
echo 'performance' | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

# Increase worker threads
inferno config set server.workers 8

# Enable caching
inferno config set cache.enabled true
```

### Issue: High Memory Usage

**Symptoms:**
```bash
Out of memory error
System becomes unresponsive
Swap usage very high
```

**Diagnosis:**
```bash
# Check memory usage
free -h
ps aux | grep inferno
inferno memory analyze

# Check model sizes
inferno list --size
```

**Solutions:**

1. **Reduce memory usage:**
```bash
# Use smaller context size
inferno config set backend_config.context_size 2048

# Unload unused models
inferno models unload --unused

# Use quantized models
inferno remove large-model-f16
inferno install large-model-q4_0
```

2. **Optimize memory settings:**
```bash
# Enable memory mapping
inferno config set backend_config.memory_mapping true

# Reduce batch size
inferno config set backend_config.batch_size 16

# Enable lazy loading
inferno config set models.lazy_loading true
```

3. **System memory optimization:**
```bash
# Clear system cache
echo 3 | sudo tee /proc/sys/vm/drop_caches

# Adjust swappiness
echo 1 | sudo tee /proc/sys/vm/swappiness

# Add more swap if needed
sudo fallocate -l 8G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
```

### Issue: GPU Not Being Used

**Symptoms:**
- GPU utilization at 0%
- Slow inference despite GPU being available
- "Using CPU backend" in logs

**Diagnosis:**
```bash
# Check GPU status
nvidia-smi
inferno gpu status
inferno gpu list

# Check CUDA installation
nvcc --version
python3 -c "import torch; print(torch.cuda.is_available())"

# Check backend configuration
inferno config get backend_config.gpu_enabled
```

**Solutions:**

1. **GPU driver and CUDA issues:**
```bash
# Install/update NVIDIA drivers
sudo apt update
sudo apt install nvidia-driver-535

# Install CUDA toolkit
wget https://developer.download.nvidia.com/compute/cuda/repos/ubuntu2204/x86_64/cuda-keyring_1.0-1_all.deb
sudo dpkg -i cuda-keyring_1.0-1_all.deb
sudo apt update
sudo apt install cuda-toolkit-12-2

# Reboot system
sudo reboot
```

2. **Inferno configuration:**
```bash
# Enable GPU in config
inferno config set backend_config.gpu_enabled true

# Set GPU layers
inferno config set backend_config.gpu_layers 35

# Check model compatibility
inferno models info model-name --gpu-compatibility
```

3. **Container GPU access:**
```bash
# For Docker, use --gpus flag
docker run --gpus all inferno:latest

# Check GPU access in container
docker run --gpus all nvidia/cuda:11.8-base-ubuntu22.04 nvidia-smi
```

## API and Connectivity Issues

### Issue: Server Won't Start

**Symptoms:**
```bash
Server failed to start: bind error
Panic: database connection failed
```

**Diagnosis:**
```bash
# Check port availability
lsof -i :8080

# Check configuration
inferno config validate
inferno config show

# Check logs
tail -f ~/.local/share/inferno/logs/inferno.log
```

**Solutions:**

1. **Port conflicts:**
```bash
# Use different port
inferno serve --port 8081

# Kill conflicting process
sudo kill $(lsof -t -i:8080)
```

2. **Configuration issues:**
```bash
# Reset to defaults
inferno config reset

# Fix specific settings
inferno config set server.bind_address "127.0.0.1"
inferno config set server.port 8080
```

3. **Database issues:**
```bash
# Reset database
rm -rf ~/.local/share/inferno/db/
inferno serve  # Will recreate database
```

### Issue: API Requests Failing

**Symptoms:**
```bash
HTTP 500 Internal Server Error
Connection timeout
Authentication failed
```

**Diagnosis:**
```bash
# Test basic connectivity
curl http://localhost:8080/health

# Check authentication
curl -H "Authorization: Bearer your-key" http://localhost:8080/v1/models

# Test with verbose output
curl -v http://localhost:8080/v1/models
```

**Solutions:**

1. **Authentication issues:**
```bash
# Check if auth is enabled
inferno config get security.auth_enabled

# Disable auth for testing
inferno config set security.auth_enabled false

# Create new API key
inferno security key create --name test-key
```

2. **Rate limiting:**
```bash
# Check rate limit settings
inferno config get security.rate_limit

# Increase rate limit
inferno config set security.rate_limit 10000

# Disable rate limiting temporarily
inferno config set security.rate_limit 0
```

3. **Request format issues:**
```bash
# Test with minimal request
curl -X POST http://localhost:8080/v1/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"gpt2","prompt":"test","max_tokens":10}'

# Check request logs
tail -f ~/.local/share/inferno/logs/access.log
```

### Issue: Slow API Responses

**Symptoms:**
- Response times >10 seconds
- Timeouts on client side
- High server load

**Diagnosis:**
```bash
# Check server metrics
curl http://localhost:8080/metrics

# Monitor real-time performance
inferno monitor start --interval 5s

# Check active connections
netstat -an | grep :8080 | wc -l
```

**Solutions:**

1. **Server configuration:**
```bash
# Increase worker threads
inferno config set server.workers 16

# Optimize connection handling
inferno config set server.max_connections 2000
inferno config set server.keep_alive_timeout 60
```

2. **Model optimization:**
```bash
# Pre-load popular models
inferno cache warm --popular

# Use faster models
inferno install gpt2-medium-q4_0  # Instead of full precision
```

3. **Caching:**
```bash
# Enable response caching
inferno config set response_cache.enabled true
inferno config set response_cache.ttl 1800

# Check cache hit rate
inferno cache stats
```

## System-Level Issues

### Issue: Docker Container Problems

**Symptoms:**
```bash
Container exits immediately
GPU not accessible in container
Permission denied in container
```

**Diagnosis:**
```bash
# Check container logs
docker logs container-name

# Check GPU access
docker run --gpus all nvidia/cuda:11.8-base-ubuntu22.04 nvidia-smi

# Check container resources
docker stats container-name
```

**Solutions:**

1. **GPU access:**
```bash
# Enable GPU access
docker run --gpus all inferno:latest

# For AMD GPUs
docker run --device=/dev/kfd --device=/dev/dri inferno:latest

# Check GPU drivers on host
nvidia-smi
```

2. **Permission issues:**
```bash
# Run as non-root user
docker run --user $(id -u):$(id -g) inferno:latest

# Fix volume permissions
sudo chown -R $(id -u):$(id -g) /data/inferno/
```

3. **Resource limits:**
```bash
# Increase memory limit
docker run --memory=16g inferno:latest

# Remove resource constraints
docker run --ulimit nofile=65536:65536 inferno:latest
```

### Issue: Kubernetes Deployment Problems

**Symptoms:**
```bash
Pods in CrashLoopBackOff state
ImagePullBackOff errors
PVC mount failures
```

**Diagnosis:**
```bash
# Check pod status
kubectl get pods -n inferno
kubectl describe pod pod-name -n inferno

# Check events
kubectl get events -n inferno --sort-by='.lastTimestamp'

# Check logs
kubectl logs pod-name -n inferno
```

**Solutions:**

1. **Pod startup issues:**
```bash
# Check resource requests/limits
kubectl describe pod pod-name -n inferno

# Scale down for testing
kubectl scale deployment inferno --replicas=1 -n inferno

# Check node resources
kubectl describe node node-name
```

2. **Storage issues:**
```bash
# Check PVC status
kubectl get pvc -n inferno
kubectl describe pvc pvc-name -n inferno

# Check storage class
kubectl get storageclass

# Recreate PVC if needed
kubectl delete pvc pvc-name -n inferno
kubectl apply -f pvc.yaml
```

3. **GPU scheduling:**
```bash
# Check GPU nodes
kubectl get nodes -l accelerator=nvidia-tesla-k80

# Check GPU resources
kubectl describe node gpu-node-name

# Add node selector
kubectl patch deployment inferno -n inferno -p '{"spec":{"template":{"spec":{"nodeSelector":{"accelerator":"nvidia-tesla-k80"}}}}}'
```

### Issue: High CPU/Memory Usage

**Symptoms:**
- System becomes unresponsive
- High load average
- Out of memory killer (OOM) activating

**Diagnosis:**
```bash
# Check system resources
top
htop
free -h

# Check Inferno processes
ps aux | grep inferno
pmap $(pgrep inferno)

# Check system limits
ulimit -a
cat /proc/sys/vm/max_map_count
```

**Solutions:**

1. **Process limits:**
```bash
# Increase file descriptor limits
echo "* soft nofile 65536" | sudo tee -a /etc/security/limits.conf
echo "* hard nofile 65536" | sudo tee -a /etc/security/limits.conf

# Increase virtual memory
echo "vm.max_map_count=262144" | sudo tee -a /etc/sysctl.conf
sudo sysctl -p
```

2. **Resource optimization:**
```bash
# Limit Inferno workers
inferno config set server.workers 4

# Optimize memory usage
inferno config set backend_config.memory_pool_size "8GB"

# Enable memory monitoring
inferno monitor memory --threshold 80 --action scale-down
```

## Production Issues

### Issue: Load Balancer Problems

**Symptoms:**
- Uneven traffic distribution
- Health check failures
- SSL termination issues

**Diagnosis:**
```bash
# Check health endpoint
curl http://backend1:8080/health
curl http://backend2:8080/health

# Check load balancer logs
tail -f /var/log/nginx/access.log
tail -f /var/log/nginx/error.log

# Test SSL
openssl s_client -connect api.domain.com:443
```

**Solutions:**

1. **Health check configuration:**
```nginx
# Fix nginx health checks
upstream inferno_backend {
    server backend1:8080 max_fails=3 fail_timeout=30s;
    server backend2:8080 max_fails=3 fail_timeout=30s;
}

# Longer timeout for model loading
location /health {
    proxy_read_timeout 60s;
    proxy_pass http://inferno_backend;
}
```

2. **SSL issues:**
```bash
# Update certificates
sudo certbot renew

# Check certificate validity
openssl x509 -in /etc/ssl/certs/cert.pem -text -noout
```

### Issue: Monitoring and Alerting

**Symptoms:**
- Missing metrics
- False alarms
- No alerting on real issues

**Diagnosis:**
```bash
# Check metrics endpoint
curl http://localhost:8080/metrics

# Check Prometheus targets
curl http://prometheus:9090/api/v1/targets

# Check alert manager
curl http://alertmanager:9093/api/v1/alerts
```

**Solutions:**

1. **Metrics collection:**
```bash
# Enable metrics
inferno config set observability.prometheus_enabled true

# Check metrics port
inferno config set observability.metrics_port 9090

# Restart service
sudo systemctl restart inferno
```

2. **Alert configuration:**
```yaml
# Fix alert rules
- alert: InfernoHighLatency
  expr: inferno_request_duration_seconds{quantile="0.95"} > 2.0
  for: 5m
  labels:
    severity: warning
  annotations:
    summary: "High latency detected"
```

## Diagnostic Tools and Commands

### Log Analysis

```bash
# Application logs
tail -f ~/.local/share/inferno/logs/inferno.log

# System logs (systemd)
journalctl -u inferno -f

# Access logs
tail -f ~/.local/share/inferno/logs/access.log

# Error logs with context
grep -A5 -B5 "ERROR" ~/.local/share/inferno/logs/inferno.log

# Filter by timestamp
grep "2024-01-15" ~/.local/share/inferno/logs/inferno.log | grep ERROR
```

### Performance Profiling

```bash
# CPU profiling
inferno profile cpu --duration 60s --output cpu_profile.json

# Memory profiling
inferno profile memory --duration 60s --output mem_profile.json

# GPU profiling
nvidia-smi dmon -s pucvmet -d 1

# Network profiling
iftop -i eth0
```

### System Monitoring

```bash
# Real-time system monitoring
watch -n 1 'free -h; echo; df -h | head -5; echo; ps aux | grep inferno | head -5'

# Resource usage history
sar -u 1 60  # CPU usage
sar -r 1 60  # Memory usage
sar -d 1 60  # Disk I/O

# Network monitoring
ss -tuln | grep :8080
netstat -i
```

### Health Checks

```bash
#!/bin/bash
# comprehensive_health_check.sh

echo "=== Inferno Health Check ==="

# Basic service check
echo "1. Service Status:"
systemctl status inferno

# API health
echo "2. API Health:"
curl -s http://localhost:8080/health | jq '.' || echo "API not responding"

# Model status
echo "3. Model Status:"
inferno models list --status

# Resource usage
echo "4. Resource Usage:"
echo "Memory: $(free -h | grep Mem | awk '{print $3"/"$2}')"
echo "Disk: $(df -h $(inferno config get models_dir) | tail -1 | awk '{print $3"/"$2" ("$5" used)"}')"

# GPU status
echo "5. GPU Status:"
if command -v nvidia-smi &> /dev/null; then
    nvidia-smi --query-gpu=name,utilization.gpu,memory.used,memory.total --format=csv,noheader,nounits
else
    echo "No GPU or nvidia-smi not available"
fi

# Cache status
echo "6. Cache Status:"
inferno cache status

# Recent errors
echo "7. Recent Errors:"
tail -n 50 ~/.local/share/inferno/logs/inferno.log | grep ERROR | tail -5

echo "=== Health Check Complete ==="
```

## Getting Help

### Community Resources

- **📚 Documentation**: [Complete documentation](../README.md)
- **💬 GitHub Discussions**: [Community help and Q&A](https://github.com/ringo380/inferno/discussions)
- **🐛 Bug Reports**: [GitHub Issues](https://github.com/ringo380/inferno/issues)
- **📖 Wiki**: [Community wiki and examples](https://github.com/ringo380/inferno/wiki)

### Creating Effective Bug Reports

When reporting issues, include:

```bash
# System information
inferno --version
uname -a
cat /etc/os-release

# Configuration
inferno config show --sanitized

# Logs (last 50 lines)
tail -n 50 ~/.local/share/inferno/logs/inferno.log

# Error reproduction steps
# 1. Command that failed
# 2. Expected behavior
# 3. Actual behavior
# 4. Error message
```

### Debug Mode

```bash
# Enable maximum verbosity
export INFERNO_LOG_LEVEL=debug
export RUST_BACKTRACE=full

# Run with debug output
inferno --verbose serve

# Or for specific commands
inferno --debug models list
```

## Prevention and Best Practices

### Regular Maintenance

```bash
# Weekly maintenance script
#!/bin/bash

# Clean up old logs
find ~/.local/share/inferno/logs -name "*.log" -mtime +30 -delete

# Clear old cache entries
inferno cache cleanup --older-than 7d

# Update models if needed
inferno package upgrade --check

# Verify model integrity
inferno validate --all --quick

# Generate health report
inferno health report --output weekly_report.json
```

### Monitoring Setup

```bash
# Set up basic monitoring
inferno monitor start --metrics all --alerts

# Configure alerts
inferno alerts create --metric cpu_usage --threshold 80 --action email
inferno alerts create --metric memory_usage --threshold 90 --action email
inferno alerts create --metric disk_usage --threshold 85 --action email
```

### Backup Strategy

```bash
# Backup essential data
#!/bin/bash
BACKUP_DIR="/backup/inferno/$(date +%Y%m%d)"
mkdir -p "$BACKUP_DIR"

# Backup configuration
cp -r ~/.config/inferno/ "$BACKUP_DIR/config/"

# Backup models metadata (not the actual models)
inferno list --format json > "$BACKUP_DIR/models_list.json"

# Backup database
inferno export database "$BACKUP_DIR/database.sql"

echo "Backup completed: $BACKUP_DIR"
```

This comprehensive troubleshooting guide should help you resolve most common Inferno issues. Remember to always check the logs first, verify your configuration, and use the diagnostic tools provided to identify the root cause of problems.

## Next Steps

- **[Performance Optimization](../tutorials/performance-optimization.md)** - Optimize after resolving issues
- **[Production Deployment](production-deployment.md)** - Deploy reliably
- **[Monitoring Guide](monitoring.md)** - Set up comprehensive monitoring