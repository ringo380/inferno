/// Production-ready error recovery and resilience patterns for Inferno
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering},
        Arc, RwLock,
    },
    time::{Duration, Instant, SystemTime},
};
use tokio::{
    sync::{mpsc, oneshot, Semaphore},
    time::{sleep, timeout},
};
use tracing::{debug, error, info, warn};

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CircuitState {
    Closed,   // Normal operation
    Open,     // Failing, rejecting requests
    HalfOpen, // Testing if service recovered
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,   // Number of failures to trigger open state
    pub recovery_timeout_ms: u64, // Time to wait before trying half-open
    pub success_threshold: u32,   // Successes needed in half-open to close
    pub timeout_ms: u64,          // Request timeout
    pub max_concurrent_requests: usize, // Max concurrent requests
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout_ms: 60000, // 1 minute
            success_threshold: 3,
            timeout_ms: 30000, // 30 seconds
            max_concurrent_requests: 100,
        }
    }
}

/// Circuit breaker for service resilience
#[derive(Debug)]
pub struct CircuitBreaker {
    name: String,
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitState>>,
    failure_count: Arc<AtomicU64>,
    success_count: Arc<AtomicU64>,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
    semaphore: Arc<Semaphore>,
    metrics: CircuitBreakerMetrics,
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerMetrics {
    pub total_requests: Arc<AtomicU64>,
    pub successful_requests: Arc<AtomicU64>,
    pub failed_requests: Arc<AtomicU64>,
    pub rejected_requests: Arc<AtomicU64>,
    pub state_changes: Arc<AtomicU64>,
}

impl CircuitBreaker {
    pub fn new(name: String, config: CircuitBreakerConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_requests));

        Self {
            name,
            config,
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: Arc::new(AtomicU64::new(0)),
            success_count: Arc::new(AtomicU64::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
            semaphore,
            metrics: CircuitBreakerMetrics {
                total_requests: Arc::new(AtomicU64::new(0)),
                successful_requests: Arc::new(AtomicU64::new(0)),
                failed_requests: Arc::new(AtomicU64::new(0)),
                rejected_requests: Arc::new(AtomicU64::new(0)),
                state_changes: Arc::new(AtomicU64::new(0)),
            },
        }
    }

    /// Execute a function with circuit breaker protection
    pub async fn call<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        self.metrics.total_requests.fetch_add(1, Ordering::Relaxed);

        // Check if circuit is open
        if self.should_reject_request().await? {
            self.metrics
                .rejected_requests
                .fetch_add(1, Ordering::Relaxed);
            return Err(anyhow!("Circuit breaker {} is OPEN", self.name));
        }

        // Acquire semaphore permit
        let _permit = self.semaphore.acquire().await.map_err(|_| {
            anyhow!(
                "Failed to acquire semaphore permit for circuit breaker {}",
                self.name
            )
        })?;

        // Execute with timeout
        let result = timeout(Duration::from_millis(self.config.timeout_ms), operation()).await;

        match result {
            Ok(Ok(value)) => {
                self.on_success().await;
                self.metrics
                    .successful_requests
                    .fetch_add(1, Ordering::Relaxed);
                Ok(value)
            }
            Ok(Err(e)) => {
                self.on_failure().await;
                self.metrics.failed_requests.fetch_add(1, Ordering::Relaxed);
                Err(e)
            }
            Err(_) => {
                self.on_failure().await;
                self.metrics.failed_requests.fetch_add(1, Ordering::Relaxed);
                Err(anyhow!(
                    "Operation timed out in circuit breaker {}",
                    self.name
                ))
            }
        }
    }

    async fn should_reject_request(&self) -> Result<bool> {
        let state = self
            .state
            .read()
            .map_err(|_| anyhow!("Failed to read circuit state"))?;

        match *state {
            CircuitState::Open => {
                // Check if we should transition to half-open
                if let Some(last_failure) = *self
                    .last_failure_time
                    .read()
                    .map_err(|_| anyhow!("Failed to read last failure time"))?
                {
                    if last_failure.elapsed()
                        > Duration::from_millis(self.config.recovery_timeout_ms)
                    {
                        drop(state);
                        self.transition_to_half_open().await?;
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            CircuitState::HalfOpen => {
                // Allow limited requests in half-open state
                Ok(false)
            }
            CircuitState::Closed => Ok(false),
        }
    }

    async fn on_success(&self) {
        let state = {
            let state_guard = self.state.read().unwrap();
            state_guard.clone()
        };

        match state {
            CircuitState::HalfOpen => {
                let success_count = self.success_count.fetch_add(1, Ordering::Relaxed) + 1;
                if success_count >= self.config.success_threshold as u64 {
                    self.transition_to_closed().await.unwrap_or_else(|e| {
                        error!("Failed to transition circuit breaker to closed: {}", e);
                    });
                }
            }
            CircuitState::Closed => {
                self.failure_count.store(0, Ordering::Relaxed);
            }
            _ => {}
        }
    }

    async fn on_failure(&self) {
        let failure_count = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;

        *self.last_failure_time.write().unwrap() = Some(Instant::now());

        let state = {
            let state_guard = self.state.read().unwrap();
            state_guard.clone()
        };

        match state {
            CircuitState::Closed => {
                if failure_count >= self.config.failure_threshold as u64 {
                    self.transition_to_open().await.unwrap_or_else(|e| {
                        error!("Failed to transition circuit breaker to open: {}", e);
                    });
                }
            }
            CircuitState::HalfOpen => {
                self.transition_to_open().await.unwrap_or_else(|e| {
                    error!("Failed to transition circuit breaker to open: {}", e);
                });
            }
            _ => {}
        }
    }

    async fn transition_to_open(&self) -> Result<()> {
        let mut state = self
            .state
            .write()
            .map_err(|_| anyhow!("Failed to write circuit state"))?;
        if *state != CircuitState::Open {
            *state = CircuitState::Open;
            self.metrics.state_changes.fetch_add(1, Ordering::Relaxed);
            warn!("Circuit breaker {} transitioned to OPEN", self.name);
        }
        Ok(())
    }

    async fn transition_to_half_open(&self) -> Result<()> {
        let mut state = self
            .state
            .write()
            .map_err(|_| anyhow!("Failed to write circuit state"))?;
        if *state != CircuitState::HalfOpen {
            *state = CircuitState::HalfOpen;
            self.success_count.store(0, Ordering::Relaxed);
            self.metrics.state_changes.fetch_add(1, Ordering::Relaxed);
            info!("Circuit breaker {} transitioned to HALF-OPEN", self.name);
        }
        Ok(())
    }

    async fn transition_to_closed(&self) -> Result<()> {
        let mut state = self
            .state
            .write()
            .map_err(|_| anyhow!("Failed to write circuit state"))?;
        if *state != CircuitState::Closed {
            *state = CircuitState::Closed;
            self.failure_count.store(0, Ordering::Relaxed);
            self.success_count.store(0, Ordering::Relaxed);
            self.metrics.state_changes.fetch_add(1, Ordering::Relaxed);
            info!("Circuit breaker {} transitioned to CLOSED", self.name);
        }
        Ok(())
    }

    pub fn get_state(&self) -> CircuitState {
        self.state.read().unwrap().clone()
    }

    pub fn get_metrics(&self) -> CircuitBreakerMetrics {
        self.metrics.clone()
    }
}

/// Retry policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: usize,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
    pub jitter_enabled: bool,
    pub retry_on_timeout: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
            jitter_enabled: true,
            retry_on_timeout: true,
        }
    }
}

/// Retry mechanism with exponential backoff
#[derive(Debug)]
pub struct RetryPolicy {
    config: RetryConfig,
}

impl RetryPolicy {
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    /// Execute a function with retry logic
    pub async fn execute<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut last_error = None;

        for attempt in 1..=self.config.max_attempts {
            debug!("Retry attempt {} of {}", attempt, self.config.max_attempts);

            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);

                    if attempt < self.config.max_attempts {
                        let delay = self.calculate_delay(attempt);
                        debug!("Retrying in {}ms", delay.as_millis());
                        sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow!("All retry attempts failed")))
    }

    fn calculate_delay(&self, attempt: usize) -> Duration {
        let base_delay = self.config.initial_delay_ms as f64;
        let delay = base_delay * self.config.backoff_multiplier.powi(attempt as i32 - 1);
        let delay = delay.min(self.config.max_delay_ms as f64);

        let delay = if self.config.jitter_enabled {
            // Add jitter: ±25% of calculated delay
            let jitter = delay * 0.25 * (2.0 * rand::random::<f64>() - 1.0);
            (delay + jitter).max(0.0)
        } else {
            delay
        };

        Duration::from_millis(delay as u64)
    }
}

/// Bulkhead pattern for resource isolation
#[derive(Debug)]
pub struct Bulkhead {
    name: String,
    semaphore: Arc<Semaphore>,
    active_requests: Arc<AtomicUsize>,
    total_requests: Arc<AtomicU64>,
    rejected_requests: Arc<AtomicU64>,
}

impl Bulkhead {
    pub fn new(name: String, max_concurrent: usize) -> Self {
        Self {
            name,
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            active_requests: Arc::new(AtomicUsize::new(0)),
            total_requests: Arc::new(AtomicU64::new(0)),
            rejected_requests: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Execute operation with bulkhead protection
    pub async fn execute<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        self.total_requests.fetch_add(1, Ordering::Relaxed);

        // Try to acquire permit without blocking
        let permit = self.semaphore.try_acquire().map_err(|_| {
            self.rejected_requests.fetch_add(1, Ordering::Relaxed);
            anyhow!("Bulkhead {} is at capacity", self.name)
        })?;

        self.active_requests.fetch_add(1, Ordering::Relaxed);
        let result = operation().await;
        self.active_requests.fetch_sub(1, Ordering::Relaxed);
        drop(permit);

        result
    }

    pub fn get_active_requests(&self) -> usize {
        self.active_requests.load(Ordering::Relaxed)
    }

    pub fn get_total_requests(&self) -> u64 {
        self.total_requests.load(Ordering::Relaxed)
    }

    pub fn get_rejected_requests(&self) -> u64 {
        self.rejected_requests.load(Ordering::Relaxed)
    }
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub enabled: bool,
    pub check_interval_ms: u64,
    pub timeout_ms: u64,
    pub failure_threshold: u32,
    pub success_threshold: u32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval_ms: 30000, // 30 seconds
            timeout_ms: 5000,         // 5 seconds
            failure_threshold: 3,
            success_threshold: 1,
        }
    }
}

/// Health check status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Unknown,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub status: HealthStatus,
    pub timestamp: SystemTime,
    pub duration_ms: u64,
    pub message: Option<String>,
    pub details: HashMap<String, String>,
}

/// Service health monitor
#[derive(Debug)]
pub struct HealthMonitor {
    name: String,
    config: HealthCheckConfig,
    current_status: Arc<RwLock<HealthStatus>>,
    consecutive_failures: Arc<AtomicU32>,
    consecutive_successes: Arc<AtomicU32>,
    last_check: Arc<RwLock<Option<HealthCheckResult>>>,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl HealthMonitor {
    pub fn new(name: String, config: HealthCheckConfig) -> Self {
        Self {
            name,
            config,
            current_status: Arc::new(RwLock::new(HealthStatus::Unknown)),
            consecutive_failures: Arc::new(AtomicU32::new(0)),
            consecutive_successes: Arc::new(AtomicU32::new(0)),
            last_check: Arc::new(RwLock::new(None)),
            shutdown_tx: None,
        }
    }

    /// Start background health checking
    pub async fn start<F, Fut>(&mut self, health_check: F) -> Result<()>
    where
        F: Fn() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<HealthCheckResult>> + Send,
    {
        if !self.config.enabled {
            info!("Health monitoring disabled for {}", self.name);
            return Ok(());
        }

        let (shutdown_tx, mut shutdown_rx) = oneshot::channel();
        self.shutdown_tx = Some(shutdown_tx);

        let name = self.name.clone();
        let config = self.config.clone();
        let current_status = self.current_status.clone();
        let consecutive_failures = self.consecutive_failures.clone();
        let consecutive_successes = self.consecutive_successes.clone();
        let last_check = self.last_check.clone();

        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(Duration::from_millis(config.check_interval_ms));

            loop {
                tokio::select! {
                    _ = &mut shutdown_rx => {
                        info!("Health monitor for {} shutting down", name);
                        break;
                    }
                    _ = interval.tick() => {
                        let start_time = Instant::now();

                        let result = timeout(
                            Duration::from_millis(config.timeout_ms),
                            health_check()
                        ).await;

                        let check_result = match result {
                            Ok(Ok(result)) => result,
                            Ok(Err(e)) => HealthCheckResult {
                                status: HealthStatus::Unhealthy,
                                timestamp: SystemTime::now(),
                                duration_ms: start_time.elapsed().as_millis() as u64,
                                message: Some(format!("Health check failed: {}", e)),
                                details: HashMap::new(),
                            },
                            Err(_) => HealthCheckResult {
                                status: HealthStatus::Unhealthy,
                                timestamp: SystemTime::now(),
                                duration_ms: config.timeout_ms,
                                message: Some("Health check timed out".to_string()),
                                details: HashMap::new(),
                            },
                        };

                        // Update status based on consecutive results
                        match check_result.status {
                            HealthStatus::Healthy => {
                                let successes = consecutive_successes.fetch_add(1, Ordering::Relaxed) + 1;
                                consecutive_failures.store(0, Ordering::Relaxed);

                                if successes >= config.success_threshold {
                                    let mut status = current_status.write().unwrap();
                                    if *status != HealthStatus::Healthy {
                                        *status = HealthStatus::Healthy;
                                        info!("Service {} is now healthy", name);
                                    }
                                }
                            }
                            HealthStatus::Unhealthy => {
                                let failures = consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;
                                consecutive_successes.store(0, Ordering::Relaxed);

                                if failures >= config.failure_threshold {
                                    let mut status = current_status.write().unwrap();
                                    if *status != HealthStatus::Unhealthy {
                                        *status = HealthStatus::Unhealthy;
                                        warn!("Service {} is now unhealthy: {}",
                                              name,
                                              check_result.message.as_deref().unwrap_or("Unknown reason"));
                                    }
                                }
                            }
                            HealthStatus::Unknown => {}
                        }

                        *last_check.write().unwrap() = Some(check_result);
                    }
                }
            }
        });

        info!("Health monitoring started for {}", self.name);
        Ok(())
    }

    pub fn get_status(&self) -> HealthStatus {
        self.current_status.read().unwrap().clone()
    }

    pub fn get_last_check(&self) -> Option<HealthCheckResult> {
        self.last_check.read().unwrap().clone()
    }

    pub fn is_healthy(&self) -> bool {
        matches!(self.get_status(), HealthStatus::Healthy)
    }
}

impl Drop for HealthMonitor {
    fn drop(&mut self) {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
        }
    }
}

/// Resilience manager that coordinates all resilience patterns
#[derive(Debug)]
pub struct ResilienceManager {
    circuit_breakers: Arc<RwLock<HashMap<String, Arc<CircuitBreaker>>>>,
    bulkheads: Arc<RwLock<HashMap<String, Arc<Bulkhead>>>>,
    health_monitors: Arc<RwLock<HashMap<String, Arc<HealthMonitor>>>>,
    retry_policies: Arc<RwLock<HashMap<String, Arc<RetryPolicy>>>>,
}

impl ResilienceManager {
    pub fn new() -> Self {
        Self {
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
            bulkheads: Arc::new(RwLock::new(HashMap::new())),
            health_monitors: Arc::new(RwLock::new(HashMap::new())),
            retry_policies: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a circuit breaker
    pub fn add_circuit_breaker(&self, name: String, config: CircuitBreakerConfig) -> Result<()> {
        let circuit_breaker = Arc::new(CircuitBreaker::new(name.clone(), config));
        let mut breakers = self
            .circuit_breakers
            .write()
            .map_err(|_| anyhow!("Failed to acquire write lock"))?;
        breakers.insert(name.clone(), circuit_breaker);
        info!("Registered circuit breaker: {}", name);
        Ok(())
    }

    /// Register a bulkhead
    pub fn add_bulkhead(&self, name: String, max_concurrent: usize) -> Result<()> {
        let bulkhead = Arc::new(Bulkhead::new(name.clone(), max_concurrent));
        let mut bulkheads = self
            .bulkheads
            .write()
            .map_err(|_| anyhow!("Failed to acquire write lock"))?;
        bulkheads.insert(name.clone(), bulkhead);
        info!(
            "Registered bulkhead: {} with max concurrent: {}",
            name, max_concurrent
        );
        Ok(())
    }

    /// Register a retry policy
    pub fn add_retry_policy(&self, name: String, config: RetryConfig) -> Result<()> {
        let retry_policy = Arc::new(RetryPolicy::new(config));
        let mut policies = self
            .retry_policies
            .write()
            .map_err(|_| anyhow!("Failed to acquire write lock"))?;
        policies.insert(name.clone(), retry_policy);
        info!("Registered retry policy: {}", name);
        Ok(())
    }

    /// Get circuit breaker by name
    pub fn get_circuit_breaker(&self, name: &str) -> Option<Arc<CircuitBreaker>> {
        self.circuit_breakers.read().ok()?.get(name).cloned()
    }

    /// Get bulkhead by name
    pub fn get_bulkhead(&self, name: &str) -> Option<Arc<Bulkhead>> {
        self.bulkheads.read().ok()?.get(name).cloned()
    }

    /// Get retry policy by name
    pub fn get_retry_policy(&self, name: &str) -> Option<Arc<RetryPolicy>> {
        self.retry_policies.read().ok()?.get(name).cloned()
    }

    /// Execute operation with full resilience protection
    pub async fn execute_with_resilience<F, Fut, T>(
        &self,
        operation_name: &str,
        operation: F,
    ) -> Result<T>
    where
        F: Fn() -> Fut + Clone,
        Fut: std::future::Future<Output = Result<T>>,
    {
        // Try to get all resilience components
        let circuit_breaker = self.get_circuit_breaker(operation_name);
        let bulkhead = self.get_bulkhead(operation_name);
        let retry_policy = self.get_retry_policy(operation_name);

        // Execute with available resilience patterns
        match (circuit_breaker, bulkhead, retry_policy) {
            (Some(cb), Some(bh), Some(rp)) => {
                // Full protection: retry + circuit breaker + bulkhead
                rp.execute(|| {
                    let op = operation.clone();
                    let cb = cb.clone();
                    let bh = bh.clone();
                    async move {
                        cb.call(|| {
                            let op = op.clone();
                            let bh = bh.clone();
                            async move { bh.execute(&op).await }
                        })
                        .await
                    }
                })
                .await
            }
            (Some(cb), Some(bh), None) => {
                // Circuit breaker + bulkhead
                cb.call(|| {
                    let op = operation.clone();
                    let bh = bh.clone();
                    async move { bh.execute(&op).await }
                })
                .await
            }
            (Some(cb), None, Some(rp)) => {
                // Retry + circuit breaker
                rp.execute(|| {
                    let op = operation.clone();
                    let cb = cb.clone();
                    async move { cb.call(&op).await }
                })
                .await
            }
            (None, Some(bh), Some(rp)) => {
                // Retry + bulkhead
                rp.execute(|| {
                    let op = operation.clone();
                    let bh = bh.clone();
                    async move { bh.execute(&op).await }
                })
                .await
            }
            (Some(cb), None, None) => {
                // Circuit breaker only
                cb.call(&operation).await
            }
            (None, Some(bh), None) => {
                // Bulkhead only
                bh.execute(&operation).await
            }
            (None, None, Some(rp)) => {
                // Retry only
                rp.execute(&operation).await
            }
            (None, None, None) => {
                // No resilience patterns - execute directly
                operation().await
            }
        }
    }

    /// Get overall system health status
    pub fn get_system_health(&self) -> HashMap<String, HealthStatus> {
        let mut health_status = HashMap::new();

        if let Ok(monitors) = self.health_monitors.read() {
            for (name, monitor) in monitors.iter() {
                health_status.insert(name.clone(), monitor.get_status());
            }
        }

        health_status
    }

    /// Get resilience metrics for monitoring
    pub fn get_resilience_metrics(&self) -> HashMap<String, serde_json::Value> {
        let mut metrics = HashMap::new();

        // Circuit breaker metrics
        if let Ok(breakers) = self.circuit_breakers.read() {
            for (name, breaker) in breakers.iter() {
                let breaker_metrics = breaker.get_metrics();
                metrics.insert(
                    format!("circuit_breaker_{}", name),
                    serde_json::json!({
                        "state": breaker.get_state(),
                        "total_requests": breaker_metrics.total_requests.load(Ordering::Relaxed),
                        "successful_requests": breaker_metrics.successful_requests.load(Ordering::Relaxed),
                        "failed_requests": breaker_metrics.failed_requests.load(Ordering::Relaxed),
                        "rejected_requests": breaker_metrics.rejected_requests.load(Ordering::Relaxed),
                        "state_changes": breaker_metrics.state_changes.load(Ordering::Relaxed),
                    }),
                );
            }
        }

        // Bulkhead metrics
        if let Ok(bulkheads) = self.bulkheads.read() {
            for (name, bulkhead) in bulkheads.iter() {
                metrics.insert(
                    format!("bulkhead_{}", name),
                    serde_json::json!({
                        "active_requests": bulkhead.get_active_requests(),
                        "total_requests": bulkhead.get_total_requests(),
                        "rejected_requests": bulkhead.get_rejected_requests(),
                    }),
                );
            }
        }

        metrics
    }
}

impl Default for ResilienceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Graceful shutdown coordinator
#[derive(Debug)]
#[derive(Default)]
pub struct GracefulShutdown {
    shutdown_tx: Option<mpsc::UnboundedSender<()>>,
    shutdown_complete_rx: Option<oneshot::Receiver<()>>,
}

impl GracefulShutdown {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<()>, oneshot::Sender<()>) {
        let (shutdown_tx, shutdown_rx) = mpsc::unbounded_channel();
        let (shutdown_complete_tx, shutdown_complete_rx) = oneshot::channel();

        let shutdown = GracefulShutdown {
            shutdown_tx: Some(shutdown_tx),
            shutdown_complete_rx: Some(shutdown_complete_rx),
        };

        (shutdown, shutdown_rx, shutdown_complete_tx)
    }

    /// Initiate graceful shutdown
    pub async fn shutdown(&mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }

        if let Some(rx) = self.shutdown_complete_rx.take() {
            match timeout(Duration::from_secs(30), rx).await {
                Ok(Ok(())) => {
                    info!("Graceful shutdown completed successfully");
                    Ok(())
                }
                Ok(Err(_)) => {
                    warn!("Graceful shutdown channel closed unexpectedly");
                    Ok(())
                }
                Err(_) => {
                    error!("Graceful shutdown timed out after 30 seconds");
                    Err(anyhow!("Shutdown timeout"))
                }
            }
        } else {
            Ok(())
        }
    }
}

