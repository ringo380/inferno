# Troubleshooting Guide

This comprehensive troubleshooting guide covers common issues and solutions for Inferno AI/ML runner's real implementations, including GGUF backend, ONNX Runtime integration, caching, conversion, and other advanced features.

## General Troubleshooting

### Enable Debug Logging

```bash
# Enable detailed logging
export INFERNO_LOG_LEVEL=debug
export RUST_BACKTRACE=1

# Enable per-component tracing (Rust tracing filter)
export RUST_LOG="inferno::backends=trace,inferno::cache=debug"

# Start the server with logging enabled
inferno serve
```

### Check System Status

```bash
# Observability component health check
inferno observability health

# Current monitoring status and metrics
inferno monitor status

# Verify configuration
inferno config validate

# Watch performance in real-time (Ctrl-C to stop)
inferno monitor watch
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
inferno validate model.gguf --deep

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
  --format gguf --quantization q4-0

# Enable memory mapping in ~/.inferno.toml:
#   [backend_config]
#   memory_map = true

# Reduce context size in ~/.inferno.toml:
#   [backend_config]
#   context_size = 2048
```

**Issue**: `GPU memory allocation failed`

```bash
# Check GPU memory
nvidia-smi

# Reduce GPU allocation in ~/.inferno.toml:
#   [backend_config]
#   gpu_layers = 20

# Disable GPU in ~/.inferno.toml to force CPU:
#   [backend_config]
#   gpu_enabled = false
inferno run --model model.gguf --prompt "test"

# Clear GPU memory
nvidia-smi --gpu-reset
```

### Inference Issues

**Issue**: `Slow inference performance`

```bash
# Check CPU usage
htop

# Enable GPU acceleration in ~/.inferno.toml:
#   [backend_config]
#   gpu_enabled = true

# Profile the model to find bottlenecks
inferno optimization profile --model model.gguf --detailed

# Use a larger batch size in ~/.inferno.toml:
#   [backend_config]
#   batch_size = 64

# Enable threading
export OMP_NUM_THREADS=8
```

**Issue**: `Tokenization errors`

```bash
# Check model details
inferno models info model.gguf

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

# Set execution providers in ~/.inferno.toml:
#   [backend_config]
#   execution_providers = ["CPUExecutionProvider"]
```

**Issue**: `GPU provider not available`

```bash
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
# Analyze the ONNX model
inferno convert analyze model.onnx --detailed

# Validate ONNX format
python -c "import onnx; onnx.checker.check_model('model.onnx')"

# Convert to a compatible format
inferno convert model model.onnx model_compatible.onnx \
  --format onnx --optimization balanced
```

### Model Loading Issues

**Issue**: `Input/output shape mismatch`

```bash
# Analyze model structure
inferno convert analyze model.onnx --detailed

# Validate the file
inferno validate model.onnx --deep
```

## Model Conversion Issues

### Conversion Failures

**Issue**: `Unsupported model architecture for conversion`

```bash
# Check source model format
inferno validate source_model.pt --deep

# Try a different optimization level
inferno convert model source.pt target.gguf \
  --format gguf --optimization basic

# Use a compatible intermediate format
inferno convert model source.pt intermediate.onnx --format onnx
inferno convert model intermediate.onnx target.gguf --format gguf

# Check conversion logs
inferno convert model source.pt target.gguf \
  --format gguf 2>&1 | tee conversion.log
```

**Issue**: `Out of memory during conversion`

```bash
# Monitor memory usage
watch -n 1 'free -h'

# Reduce batch size
inferno convert model source.pt target.gguf \
  --format gguf --batch-size 16

# Use quantization
inferno convert model source.pt target.gguf \
  --format gguf --quantization q4-0

# Close other applications
sudo systemctl stop unnecessary-service
```

**Issue**: `Conversion takes too long`

```bash
# Use a faster optimization level
inferno convert model source.pt target.gguf \
  --format gguf --optimization basic

# Skip optimization
inferno convert model source.pt target.gguf \
  --format gguf --optimization none

# Run in background
nohup inferno convert model source.pt target.gguf --format gguf &
```

### Conversion Quality Issues

**Issue**: `Converted model produces poor results`

```bash
# Compare outputs by running the same prompt through each model
inferno run --model original.pt --prompt "test prompt" > original.txt
inferno run --model converted.gguf --prompt "test prompt" > converted.txt
diff original.txt converted.txt

# Use higher precision
inferno convert model source.pt target.gguf \
  --format gguf --precision float16

# Convert without quantization (omit --quantization)
inferno convert model source.pt target.gguf \
  --format gguf

# Use conservative optimization
inferno convert model source.pt target.gguf \
  --format gguf --optimization balanced
```

## Caching Issues

### Cache Performance Problems

**Issue**: `Low cache hit rate`

```bash
# Check cache statistics
inferno cache stats

# Monitor cache usage in real-time
inferno cache monitor --detailed

# Increase cache limits
inferno cache configure --max-models 10 --max-memory-mb 20480

# Adjust model TTL
inferno cache configure --ttl-seconds 172800
```

**Issue**: `Cache corruption detected`

```bash
# Check cache status
inferno cache stats

# Clear and warm up the cache again
inferno cache clear --force
inferno cache warmup model-a model-b
```

**Issue**: `Cache disk space issues`

```bash
# Check disk usage
df -h /var/cache/inferno

# Clear the cache
inferno cache clear --force

# Reduce cache limits
inferno cache configure --max-models 3 --max-memory-mb 4096
```

### Cache Persistence Issues

**Issue**: `Cache not persisting across restarts`

```bash
# Check cache configuration and current state
inferno cache stats

# Verify write permissions
ls -la /var/cache/inferno/
sudo chown -R $USER:$USER /var/cache/inferno/

# Configure warmup so hot models reload on start
inferno cache configure --warmup true --strategy usage-based
```

## Batch Processing Issues

### Job Scheduling Problems

**Issue**: `Cron jobs not executing`

```bash
# Check cron service
sudo systemctl status cron

# Schedule a recurring job with a cron expression
inferno queue schedule --name nightly \
  --schedule-type cron --expression "0 2 * * *" \
  --input-file jobs.jsonl --model my-model my-queue

# Check job status in a queue
inferno queue list-jobs my-queue

# Inspect a specific job
inferno queue job-status my-queue <job-id>
```

**Issue**: `Jobs failing with resource limits`

```bash
# Check queue metrics
inferno queue metrics my-queue

# Monitor queue activity during execution
inferno queue monitor my-queue

# Check system resources
htop
iostat -x 1
```

### Job Queue Issues

**Issue**: `Jobs stuck in queue`

```bash
# Check queue metrics
inferno queue metrics my-queue

# Configure queue concurrency
inferno queue configure --max-concurrent 10 my-queue

# Clear completed jobs
inferno queue clear my-queue

# Cancel or retry a specific job
inferno queue cancel my-queue <job-id>
inferno queue retry my-queue <job-id>
```

## Audit System Issues

### Audit Log Problems

**Issue**: `Audit logs not being created`

```bash
# Check audit statistics
inferno audit stats

# Enable audit logging
inferno audit configure --enable true

# Verify permissions
ls -la /var/log/inferno/audit/
sudo chown -R inferno:inferno /var/log/inferno/

# Check disk space
df -h /var/log

# Write a test audit event
inferno audit log --help
```

**Issue**: `Audit configuration problems`

```bash
# Show current audit configuration
inferno audit configure --show

# Set retention and compression
inferno audit configure --retention-days 30 --compression true

# Validate audit log integrity
inferno audit validate
```

### Alert Issues

**Issue**: `Alerts not being sent`

```bash
# Test the alert system
inferno monitor test-alerts

# View active alerts
inferno monitor alerts

# Verify network connectivity to a webhook
curl -I https://hooks.slack.com/services/...

# Check email configuration
telnet smtp.company.com 587
```

## HTTP API Issues

### API Connectivity Problems

**Issue**: `API endpoints not responding`

```bash
# Check server health
curl -I http://localhost:8080/health

# Check the metrics endpoint
curl http://localhost:8080/metrics

# List available models via the OpenAI-compatible API
curl http://localhost:8080/v1/models

# Check server status
curl http://localhost:8080/v1/status

# Check firewall rules
sudo ufw status
sudo iptables -L
```

**Issue**: `Streaming or WebSocket connections failing`

```bash
# Test the streaming WebSocket endpoint
wscat -c ws://localhost:8080/ws/stream

# Or use SSE streaming on the chat endpoint
curl -N http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"my-model","messages":[{"role":"user","content":"hi"}],"stream":true}'

# Check proxy configuration
# Nginx: proxy_set_header Upgrade $http_upgrade;
# Apache: ProxyPass ws://localhost:8080/
```

## Performance Issues

### High CPU Usage

```bash
# Profile CPU usage
perf record -g inferno serve
perf report

# Check thread usage
ps -eLf | grep inferno

# Reduce concurrent requests in ~/.inferno.toml:
#   [server]
#   max_concurrent_requests = 50

# Set CPU threads in ~/.inferno.toml:
#   [backend_config]
#   cpu_threads = 4

# Use CPU affinity
taskset -c 0-7 inferno serve
```

### High Memory Usage

```bash
# Monitor memory usage
valgrind --tool=massif inferno serve

# Reduce cache memory limit
inferno cache configure --max-memory-mb 5120

# Profile inference memory usage over time
inferno performance-benchmark memory-profile --model model.gguf --track

# Enable full backtraces for debugging
export RUST_BACKTRACE=full
```

### High Disk I/O

```bash
# Monitor I/O usage
iotop

# Use faster storage
sudo mount -o remount,noatime /var/cache/inferno

# Use RAM disk for temporary files
sudo mount -t tmpfs -o size=2G tmpfs /tmp/inferno

# Reduce log verbosity
export INFERNO_LOG_LEVEL=warn
```

## Network Issues

### Connection Problems

**Issue**: `Cannot bind to address`

```bash
# Check port availability
sudo netstat -tulpn | grep 8080

# Use a different bind address/port
inferno serve --bind 127.0.0.1:8081

# Check firewall
sudo ufw allow 8080
sudo firewall-cmd --add-port=8080/tcp --permanent

# Bind to a specific interface
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

# Generate a new certificate
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes
```

## GPU Issues

### GPU Detection

```bash
# List detected GPUs
inferno gpu list

# Detailed info for a specific GPU (requires the GPU id)
inferno gpu info 0 --capabilities

# GPU health check
inferno gpu health --detailed
```

### NVIDIA GPU Problems

**Issue**: `CUDA out of memory`

```bash
# Check GPU memory
nvidia-smi

# Clear GPU cache
python -c "import torch; torch.cuda.empty_cache()"

# Reduce model size
inferno convert model large.gguf small.gguf --format gguf --quantization q4-0

# Limit GPU memory
export CUDA_VISIBLE_DEVICES=0
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
# List API keys for a user
inferno security api-key list --user admin

# Generate a new API key
inferno security api-key generate --user admin --name test

# Test authentication
curl -H "Authorization: Bearer $API_KEY" \
  http://localhost:8080/v1/models

# Test an API key
inferno security api-key test --help
```

**Issue**: `Rate limiting issues`

```bash
# Check rate limit status
inferno security rate-limit status --identifier admin

# Adjust limits
inferno security rate-limit set --user admin --per-minute 2000

# Reset rate limit counters
inferno security rate-limit reset --help

# Test rate limiting
inferno security rate-limit test --help
```

## Distributed System Issues

### Worker Connectivity

**Issue**: `Workers not connecting`

```bash
# Show worker statistics
inferno distributed stats

# Start distributed inference with a worker pool
inferno distributed start --workers 4 --load-balancing

# Test a single inference request against the pool
inferno distributed test --model my-model --input "Hello, world!"

# Check network connectivity
telnet coordinator 9090
```

**Issue**: `Load balancing not working`

```bash
# Show worker statistics
inferno distributed stats

# Restart the pool with load balancing enabled
inferno distributed start --workers 4 --load-balancing --max-concurrent 8

# Benchmark distributed inference performance
inferno distributed benchmark --help
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
```

### Log Analysis

```bash
# Logs are written to stdout/stderr (and any file you redirect to).
# If running under systemd, use journalctl:
journalctl -u inferno -f

# Search a log file for errors
grep -i "error\|failed\|panic" /var/log/inferno/inferno.log

# Monitor logs in real-time
tail -f /var/log/inferno/inferno.log | grep -E "(ERROR|WARN|inference)"
```

## Getting Additional Help

### Collect Debug Information

```bash
# Capture system and GPU information
inferno gpu list
inferno config show

# Profile inference performance
inferno optimization profile --model model.gguf --detailed

# Export configuration (redact secrets before sharing)
inferno config show > config_export.toml

# Export a monitoring/metrics snapshot
inferno metrics snapshot
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

2. **Run health checks**:
   ```bash
   inferno observability health
   inferno monitor status
   ```

3. **Collect logs** (from stdout, your log file, or journalctl) and

4. **Sanitize sensitive data** before sharing logs or configurations
