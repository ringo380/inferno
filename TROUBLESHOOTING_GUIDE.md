# Troubleshooting Guide

This comprehensive troubleshooting guide covers common issues and solutions for Inferno AI/ML runner's real implementations, including GGUF backend, ONNX Runtime integration, caching, conversion, and other advanced features.

## General Troubleshooting

### Enable Debug Logging

```bash
# Enable detailed logging
export INFERNO_LOG_LEVEL=debug
export RUST_BACKTRACE=1

# Start with debug output
inferno serve --log-level debug

# Enable tracing for specific components
export INFERNO_TRACE_COMPONENTS="backends,cache,conversion"
```

### Check System Status

```bash
# Comprehensive health check
inferno health check --detailed

# System diagnostics
inferno diagnostics --export diagnostics.json

# Verify configuration
inferno config validate --verbose

# Check resource usage
inferno resources monitor --duration 30s
```

## GGUF Backend Issues

### Model Loading Failures

**Issue**: `Failed to load GGUF model: missing GGUF magic bytes`

```bash
# Verify file integrity
file model.gguf
hexdump -C model.gguf | head -n 5

# Check file permissions
ls -la model.gguf
sudo chown $USER:$USER model.gguf

# Validate GGUF format
inferno validate model.gguf --detailed

# Re-download model if corrupted
wget -O model.gguf.new https://source/model.gguf
mv model.gguf.new model.gguf
```

**Issue**: `Model file too large for available memory`

```bash
# Check available memory
free -h

# Reduce model size with quantization
inferno convert model large-model.gguf small-model.gguf \
  --format gguf --quantization q4_0

# Enable memory mapping
inferno config set backend_config.memory_map true

# Reduce context size
inferno config set backend_config.context_size 2048
```

**Issue**: `GPU memory allocation failed`

```bash
# Check GPU memory
nvidia-smi

# Reduce GPU allocation
inferno config set backend_config.gpu_layers 20

# Use CPU fallback
inferno run --model model.gguf --cpu-only --prompt "test"

# Clear GPU memory
nvidia-smi --gpu-reset
```

### Inference Issues

**Issue**: `Slow inference performance`

```bash
# Check CPU usage
htop

# Enable GPU acceleration
inferno config set backend_config.gpu_enabled true

# Optimize context size
inferno benchmark context-size --model model.gguf

# Use larger batch size
inferno config set backend_config.batch_size 64

# Enable threading
export OMP_NUM_THREADS=8
```

**Issue**: `Tokenization errors`

```bash
# Check tokenizer configuration
inferno models info model.gguf --show-tokenizer

# Use fallback tokenization
inferno config set backend_config.fallback_tokenizer true

# Test with simple input
inferno run --model model.gguf --prompt "Hello"
```

## ONNX Backend Issues

### ONNX Runtime Errors

**Issue**: `ONNX Runtime initialization failed`

```bash
# Check ONNX Runtime installation
python -c "import onnxruntime; print(onnxruntime.__version__)"

# Verify library path
echo $ORT_LIB_LOCATION
ls -la $ORT_LIB_LOCATION

# Reinstall ONNX Runtime
cargo clean
ORT_STRATEGY=download cargo build --release --features onnx

# Use specific provider
inferno config set backend_config.execution_providers '["CPUExecutionProvider"]'
```

**Issue**: `GPU provider not available`

```bash
# Check available providers
inferno backends info onnx --providers

# Test GPU availability
nvidia-smi  # NVIDIA
rocm-smi   # AMD

# Install GPU drivers
# NVIDIA: sudo apt install nvidia-driver-520
# AMD: sudo apt install rocm-dev

# Verify CUDA installation
nvcc --version
```

**Issue**: `Model format not supported`

```bash
# Check ONNX model info
inferno models info model.onnx --detailed

# Validate ONNX format
python -c "import onnx; onnx.checker.check_model('model.onnx')"

# Convert to supported format
inferno convert model model.onnx model_compatible.onnx \
  --optimization balanced

# Check opset version
inferno models info model.onnx --opset
```

### Model Loading Issues

**Issue**: `Input/output shape mismatch`

```bash
# Analyze model structure
inferno models analyze model.onnx

# Check input requirements
inferno models info model.onnx --inputs

# Use dynamic shapes
inferno config set backend_config.dynamic_shapes true

# Reshape inputs
inferno config set backend_config.input_shape '[1, -1]'
```

## Model Conversion Issues

### Conversion Failures

**Issue**: `Unsupported model architecture for conversion`

```bash
# Check source model format
inferno validate source_model.pt --detailed

# Try different optimization level
inferno convert model source.pt target.gguf \
  --optimization fast

# Use compatible intermediate format
inferno convert model source.pt intermediate.onnx --format onnx
inferno convert model intermediate.onnx target.gguf --format gguf

# Check conversion logs
inferno convert model source.pt target.gguf \
  --format gguf --verbose 2>&1 | tee conversion.log
```

**Issue**: `Out of memory during conversion`

```bash
# Monitor memory usage
watch -n 1 'free -h'

# Reduce batch size
inferno convert model source.pt target.gguf \
  --batch-size 16

# Use quantization
inferno convert model source.pt target.gguf \
  --quantization q4_0

# Close other applications
sudo systemctl stop unnecessary-service
```

**Issue**: `Conversion takes too long`

```bash
# Use faster optimization
inferno convert model source.pt target.gguf \
  --optimization fast

# Skip optimization
inferno convert model source.pt target.gguf \
  --optimization none

# Run in background
nohup inferno convert model source.pt target.gguf &

# Monitor progress
inferno conversion status job_id
```

### Conversion Quality Issues

**Issue**: `Converted model produces poor results`

```bash
# Compare outputs
inferno compare models original.pt converted.gguf \
  --test-prompts test.txt

# Use higher precision
inferno convert model source.pt target.gguf \
  --precision fp16

# Disable quantization
inferno convert model source.pt target.gguf \
  --quantization none

# Use conservative optimization
inferno convert model source.pt target.gguf \
  --optimization balanced
```

## Caching Issues

### Cache Performance Problems

**Issue**: `Low cache hit rate`

```bash
# Analyze cache patterns
inferno cache analyze --period 24h

# Check cache configuration
inferno cache stats --detailed

# Increase cache size
inferno config set cache.max_size_gb 20

# Adjust TTL
inferno config set cache.ttl_hours 48

# Enable response deduplication
inferno config set cache.deduplication.enabled true
```

**Issue**: `Cache corruption detected`

```bash
# Verify cache integrity
inferno cache verify --detailed

# Repair corrupted entries
inferno cache repair --auto-fix

# Rebuild cache from backup
inferno cache restore --source /backup/cache

# Clear and rebuild
inferno cache clear --all
inferno cache warm --models /models/*.gguf
```

**Issue**: `Cache disk space issues`

```bash
# Check disk usage
df -h /var/cache/inferno

# Clean old entries
inferno cache cleanup --older-than 7d

# Increase compression
inferno config set cache.compression.algorithm zstd
inferno config set cache.compression.level 9

# Move cache to larger disk
inferno cache migrate --destination /large-disk/cache
```

### Cache Persistence Issues

**Issue**: `Cache not persisting across restarts`

```bash
# Check cache configuration
inferno config get cache

# Verify write permissions
ls -la /var/cache/inferno/
sudo chown -R $USER:$USER /var/cache/inferno/

# Enable persistent cache
inferno config set cache.type persistent

# Force sync
inferno cache sync --force
```

## Batch Processing Issues

### Job Scheduling Problems

**Issue**: `Cron jobs not executing`

```bash
# Check cron service
sudo systemctl status cron

# Validate cron expressions
inferno batch-queue validate-schedule "0 2 * * *"

# Check job status
inferno batch-queue list --status all

# Enable batch queue
inferno batch-queue enable

# Test manual execution
inferno batch-queue run job_id --now
```

**Issue**: `Jobs failing with resource limits`

```bash
# Check resource usage
inferno batch-queue resources --job job_id

# Increase limits
inferno batch-queue update job_id \
  --cpu-limit 4.0 \
  --memory-limit 8GB

# Monitor during execution
inferno batch-queue monitor job_id --real-time

# Check system resources
htop
iostat -x 1
```

### Job Queue Issues

**Issue**: `Jobs stuck in queue`

```bash
# Check queue status
inferno batch-queue stats

# Increase concurrent jobs
inferno config set batch_queue.max_concurrent_jobs 10

# Clear stuck jobs
inferno batch-queue clear --status stuck

# Restart queue processor
inferno batch-queue restart

# Check dependencies
inferno batch-queue dependencies job_id
```

## Audit System Issues

### Audit Log Problems

**Issue**: `Audit logs not being created`

```bash
# Check audit status
inferno audit status

# Enable audit logging
inferno audit enable --encryption

# Verify permissions
ls -la /var/log/inferno/audit/
sudo chown -R inferno:inferno /var/log/inferno/

# Check disk space
df -h /var/log

# Test audit logging
inferno audit test --write-test-entry
```

**Issue**: `Audit encryption errors`

```bash
# Check encryption key
ls -la /secure/audit.key

# Generate new key
inferno audit generate-key --output /secure/audit.key.new

# Test encryption
inferno audit test-encryption --key-file /secure/audit.key

# Disable encryption temporarily
inferno config set audit.encryption.enabled false
```

### Alert Issues

**Issue**: `Alerts not being sent`

```bash
# Test alert channels
inferno audit test-alerts --channels email,slack

# Check channel configuration
inferno config get audit.alerts

# Verify network connectivity
curl -I https://hooks.slack.com/services/...

# Check email configuration
telnet smtp.company.com 587

# View alert logs
inferno audit logs --filter alerts
```

## Dashboard API Issues

### API Connectivity Problems

**Issue**: `Dashboard endpoints not responding`

```bash
# Check server status
curl -I http://localhost:8080/health

# Test dashboard endpoint
curl http://localhost:8080/dashboard/stats

# Check authentication
curl -H "Authorization: Bearer $API_KEY" \
  http://localhost:8080/dashboard/stats

# Verify dashboard is enabled
inferno config get dashboard.enabled

# Check firewall rules
sudo ufw status
sudo iptables -L
```

**Issue**: `WebSocket connections failing`

```bash
# Test WebSocket endpoint
wscat -c ws://localhost:8080/dashboard/ws

# Check proxy configuration
# Nginx: proxy_set_header Upgrade $http_upgrade;
# Apache: ProxyPass ws://localhost:8080/

# Verify WebSocket support
curl -H "Upgrade: websocket" \
     -H "Connection: Upgrade" \
     http://localhost:8080/dashboard/ws
```

## Performance Issues

### High CPU Usage

```bash
# Profile CPU usage
perf record -g inferno serve
perf report

# Check thread usage
ps -eLf | grep inferno

# Reduce concurrent requests
inferno config set server.max_concurrent_requests 50

# Optimize backend settings
inferno config set backend_config.cpu_threads 4

# Use CPU affinity
taskset -c 0-7 inferno serve
```

### High Memory Usage

```bash
# Monitor memory usage
valgrind --tool=massif inferno serve

# Reduce cache size
inferno config set cache.max_size_gb 5

# Limit model loading
inferno config set models.max_loaded_models 2

# Enable garbage collection
export RUST_GC_FREQUENCY=100

# Check for memory leaks
inferno diagnostics memory-leak --duration 300s
```

### High Disk I/O

```bash
# Monitor I/O usage
iotop

# Use faster storage
sudo mount -o remount,noatime /var/cache/inferno

# Optimize cache settings
inferno config set cache.sync_interval_seconds 300

# Use RAM disk for temporary files
sudo mount -t tmpfs -o size=2G tmpfs /tmp/inferno

# Reduce log verbosity
inferno config set log_level warn
```

## Network Issues

### Connection Problems

**Issue**: `Cannot bind to address`

```bash
# Check port availability
sudo netstat -tulpn | grep 8080

# Use different port
inferno serve --port 8081

# Check firewall
sudo ufw allow 8080
sudo firewall-cmd --add-port=8080/tcp --permanent

# Bind to specific interface
inferno serve --bind 127.0.0.1:8080
```

**Issue**: `SSL/TLS errors`

```bash
# Check certificate
openssl x509 -in cert.pem -text -noout

# Verify private key
openssl rsa -in key.pem -check

# Test SSL connection
openssl s_client -connect localhost:8443

# Generate new certificate
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes

# Configure TLS
inferno config set server.tls.cert_file cert.pem
inferno config set server.tls.key_file key.pem
```

## GPU Issues

### NVIDIA GPU Problems

**Issue**: `CUDA out of memory`

```bash
# Check GPU memory
nvidia-smi

# Clear GPU cache
python -c "import torch; torch.cuda.empty_cache()"

# Reduce model size
inferno convert model large.gguf small.gguf --quantization q4_0

# Use CPU fallback
inferno config set backend_config.gpu_fallback true

# Limit GPU memory
export CUDA_VISIBLE_DEVICES=0
export CUDA_MEM_FRACTION=0.8
```

**Issue**: `CUDA driver version mismatch`

```bash
# Check CUDA version
nvidia-smi
nvcc --version

# Update NVIDIA drivers
sudo apt update && sudo apt install nvidia-driver-520

# Reinstall CUDA toolkit
sudo apt install cuda-toolkit-12-2

# Verify installation
nvidia-smi
```

### AMD GPU Problems

**Issue**: `ROCm not detected`

```bash
# Check ROCm installation
rocm-smi

# Install ROCm
sudo apt install rocm-dev rocm-libs

# Add user to render group
sudo usermod -a -G render $USER

# Set environment variables
export ROCM_PATH=/opt/rocm
export PATH=$ROCM_PATH/bin:$PATH

# Verify GPU detection
/opt/rocm/bin/rocminfo
```

## Security Issues

### Authentication Problems

**Issue**: `API key authentication failing`

```bash
# Verify API key format
echo $API_KEY | base64 -d

# Check key in database
inferno security api-key list

# Generate new API key
inferno security api-key create --user admin --name test

# Test authentication
curl -H "Authorization: Bearer $API_KEY" \
  http://localhost:8080/models

# Check permissions
inferno security user permissions admin
```

**Issue**: `Rate limiting issues`

```bash
# Check rate limits
inferno security rate-limit show --user admin

# Adjust limits
inferno security rate-limit set \
  --user admin \
  --requests-per-minute 2000

# Disable rate limiting temporarily
inferno config set security.rate_limiting_enabled false

# Check current usage
inferno security rate-limit stats
```

## Database Issues

### Database Connectivity

**Issue**: `Database connection failed`

```bash
# Check database status
sudo systemctl status postgresql

# Test connection
psql -h localhost -U inferno -d inferno_db

# Check connection pool
inferno diagnostics database --pool-stats

# Reset connection pool
inferno database reset-pool

# Migrate database
inferno database migrate --up
```

## Distributed System Issues

### Worker Connectivity

**Issue**: `Workers not connecting to coordinator`

```bash
# Check coordinator status
inferno distributed coordinator status

# Test worker connection
inferno distributed worker test-connection coordinator:9090

# Check network connectivity
telnet coordinator 9090

# Verify certificates
openssl s_client -connect coordinator:9090

# Update worker configuration
inferno config set distributed.coordinator_address coordinator:9090
```

**Issue**: `Load balancing not working`

```bash
# Check worker loads
inferno distributed workers list --detailed

# Monitor load distribution
inferno distributed monitor --real-time

# Adjust load balancing
inferno config set distributed.load_balancing_algorithm round_robin

# Force rebalancing
inferno distributed rebalance --force
```

## Logging and Debugging

### Advanced Debugging

```bash
# Enable component tracing
export RUST_LOG="inferno::backends=trace,inferno::cache=debug"

# Use debug builds for detailed errors
cargo build --features debug

# Enable memory debugging
export RUST_BACKTRACE=full
export MALLOC_CHECK_=2

# Profile performance
cargo flamegraph --bin inferno -- serve

# Generate debug report
inferno diagnostics generate-report --output debug_report.zip
```

### Log Analysis

```bash
# Search logs for errors
grep -i "error\|failed\|panic" /var/log/inferno/inferno.log

# Analyze log patterns
inferno logs analyze --pattern "inference.*latency" --period 24h

# Export structured logs
inferno logs export --format json --since 1h > logs.json

# Monitor logs in real-time
tail -f /var/log/inferno/inferno.log | grep -E "(ERROR|WARN|inference)"
```

## Getting Additional Help

### Collect Debug Information

```bash
# Generate comprehensive debug report
inferno diagnostics collect-all --output debug_$(date +%Y%m%d).tar.gz

# Include system information
inferno diagnostics system-info --detailed

# Export configuration (sanitized)
inferno config export --sanitize --output config_export.toml

# Generate performance profile
inferno diagnostics performance-profile --duration 300s
```

### Support Channels

1. **GitHub Issues**: Report bugs with debug information
2. **GitHub Discussions**: Community help from users and developers
3. **Documentation**: Check latest docs for updates
4. **Stack Overflow**: Tag questions with `inferno-ai`

### Before Reporting Issues

1. **Update to latest version**:
   ```bash
   git pull origin main
   cargo build --release
   ```

2. **Check known issues**:
   ```bash
   inferno known-issues --check
   ```

3. **Run diagnostics**:
   ```bash
   inferno diagnostics health-check --comprehensive
   ```

4. **Collect logs**:
   ```bash
   inferno logs export --since 24h --level error,warn
   ```

5. **Sanitize sensitive data** before sharing logs or configurations