use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tracing::{error, info, warn};

/// API Gateway configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiGatewayConfig {
    /// Enable API gateway
    pub enabled: bool,
    /// Gateway bind address
    pub bind_address: String,
    /// Gateway port
    pub port: u16,
    /// SSL/TLS configuration
    pub tls: Option<TlsConfig>,
    /// Load balancer configuration
    pub load_balancer: LoadBalancerConfig,
    /// Rate limiting configuration
    pub rate_limiting: RateLimitingConfig,
    /// Authentication configuration
    pub authentication: AuthenticationConfig,
    /// CORS configuration
    pub cors: CorsConfig,
    /// Middleware configuration
    pub middleware: MiddlewareConfig,
    /// Route configuration
    pub routes: Vec<RouteConfig>,
    /// Upstream services
    pub upstream_services: Vec<UpstreamService>,
    /// Health check configuration
    pub health_checks: HealthCheckConfig,
    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,
    /// Request timeout settings
    pub timeouts: TimeoutConfig,
    /// Logging and monitoring
    pub monitoring: GatewayMonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Enable TLS
    pub enabled: bool,
    /// Certificate file path
    pub cert_file: String,
    /// Private key file path
    pub key_file: String,
    /// CA certificate file path
    pub ca_file: Option<String>,
    /// Require client certificates
    pub client_auth: bool,
    /// TLS version (minimum)
    pub min_version: String,
    /// Cipher suites
    pub cipher_suites: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerConfig {
    /// Load balancing algorithm
    pub algorithm: LoadBalancingAlgorithm,
    /// Health check enabled
    pub health_check_enabled: bool,
    /// Health check interval in seconds
    pub health_check_interval: u64,
    /// Maximum retries
    pub max_retries: u32,
    /// Retry delay in milliseconds
    pub retry_delay: u64,
    /// Session affinity
    pub session_affinity: Option<SessionAffinityConfig>,
    /// Failover configuration
    pub failover: FailoverConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingAlgorithm {
    RoundRobin,
    WeightedRoundRobin,
    LeastConnections,
    WeightedLeastConnections,
    IPHash,
    Random,
    WeightedRandom,
    ConsistentHash,
    ResourceBased,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionAffinityConfig {
    /// Enable session affinity
    pub enabled: bool,
    /// Affinity type
    pub affinity_type: AffinityType,
    /// Session timeout in seconds
    pub session_timeout: u64,
    /// Cookie configuration
    pub cookie: Option<CookieConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AffinityType {
    ClientIP,
    Cookie,
    Header,
    JWTClaim,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieConfig {
    /// Cookie name
    pub name: String,
    /// Cookie domain
    pub domain: Option<String>,
    /// Cookie path
    pub path: String,
    /// Secure flag
    pub secure: bool,
    /// HTTP only flag
    pub http_only: bool,
    /// Same site policy
    pub same_site: SameSitePolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SameSitePolicy {
    Strict,
    Lax,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailoverConfig {
    /// Enable automatic failover
    pub enabled: bool,
    /// Failover threshold (failures before marking unhealthy)
    pub threshold: u32,
    /// Recovery threshold (successes before marking healthy)
    pub recovery_threshold: u32,
    /// Blacklist timeout in seconds
    pub blacklist_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitingConfig {
    /// Enable rate limiting
    pub enabled: bool,
    /// Default rate limit (requests per second)
    pub default_rate: u32,
    /// Default burst size
    pub default_burst: u32,
    /// Rate limiting algorithm
    pub algorithm: RateLimitingAlgorithm,
    /// Per-client rate limits
    pub per_client_limits: HashMap<String, ClientRateLimit>,
    /// Per-route rate limits
    pub per_route_limits: HashMap<String, RouteRateLimit>,
    /// Rate limit storage backend
    pub storage: RateLimitStorageConfig,
    /// Sliding window configuration
    pub sliding_window: SlidingWindowConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RateLimitingAlgorithm {
    TokenBucket,
    FixedWindow,
    SlidingWindow,
    SlidingWindowLog,
    LeakyBucket,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientRateLimit {
    /// Requests per second
    pub rate: u32,
    /// Burst size
    pub burst: u32,
    /// Time window in seconds
    pub window: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRateLimit {
    /// Route pattern
    pub pattern: String,
    /// Requests per second
    pub rate: u32,
    /// Burst size
    pub burst: u32,
    /// Time window in seconds
    pub window: u64,
    /// Per-client override
    pub per_client: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitStorageConfig {
    /// Storage backend type
    pub backend: StorageBackend,
    /// Redis configuration
    pub redis: Option<RedisConfig>,
    /// Memory configuration
    pub memory: Option<MemoryStorageConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageBackend {
    Memory,
    Redis,
    Database,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    /// Redis URL
    pub url: String,
    /// Key prefix
    pub key_prefix: String,
    /// Connection pool size
    pub pool_size: u32,
    /// Connection timeout
    pub timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStorageConfig {
    /// Maximum entries
    pub max_entries: usize,
    /// Cleanup interval in seconds
    pub cleanup_interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlidingWindowConfig {
    /// Window size in seconds
    pub window_size: u64,
    /// Number of sub-windows
    pub sub_windows: u32,
    /// Precision factor
    pub precision: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationConfig {
    /// Enable authentication
    pub enabled: bool,
    /// Authentication methods
    pub methods: Vec<AuthMethod>,
    /// JWT configuration
    pub jwt: Option<JwtConfig>,
    /// API key configuration
    pub api_key: Option<ApiKeyConfig>,
    /// OAuth configuration
    pub oauth: Option<OAuthConfig>,
    /// Basic auth configuration
    pub basic_auth: Option<BasicAuthConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    JWT,
    ApiKey,
    OAuth,
    BasicAuth,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// JWT secret
    pub secret: String,
    /// Algorithm
    pub algorithm: String,
    /// Token expiration in seconds
    pub expiration: u64,
    /// Issuer
    pub issuer: Option<String>,
    /// Audience
    pub audience: Option<String>,
    /// Claims validation
    pub required_claims: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyConfig {
    /// API key header name
    pub header_name: String,
    /// API key query parameter name
    pub query_param_name: Option<String>,
    /// Valid API keys
    pub keys: HashMap<String, ApiKeyInfo>,
    /// Key validation service
    pub validation_service: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyInfo {
    /// Key name/description
    pub name: String,
    /// Allowed permissions
    pub permissions: Vec<String>,
    /// Rate limit override
    pub rate_limit: Option<ClientRateLimit>,
    /// Expiration date
    pub expires_at: Option<DateTime<Utc>>,
    /// Created date
    pub created_at: DateTime<Utc>,
    /// Last used date
    pub last_used: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    /// OAuth provider
    pub provider: String,
    /// Client ID
    pub client_id: String,
    /// Client secret
    pub client_secret: String,
    /// Authorization endpoint
    pub auth_endpoint: String,
    /// Token endpoint
    pub token_endpoint: String,
    /// Userinfo endpoint
    pub userinfo_endpoint: String,
    /// Scopes
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicAuthConfig {
    /// Realm
    pub realm: String,
    /// User credentials
    pub users: HashMap<String, String>,
    /// External validation service
    pub validation_service: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    /// Enable CORS
    pub enabled: bool,
    /// Allowed origins
    pub allowed_origins: Vec<String>,
    /// Allowed methods
    pub allowed_methods: Vec<String>,
    /// Allowed headers
    pub allowed_headers: Vec<String>,
    /// Exposed headers
    pub expose_headers: Vec<String>,
    /// Allow credentials
    pub allow_credentials: bool,
    /// Max age in seconds
    pub max_age: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct MiddlewareConfig {
    /// Request logging
    pub request_logging: RequestLoggingConfig,
    /// Response transformation
    pub response_transform: ResponseTransformConfig,
    /// Request transformation
    pub request_transform: RequestTransformConfig,
    /// Custom middleware
    pub custom_middleware: Vec<CustomMiddlewareConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestLoggingConfig {
    /// Enable request logging
    pub enabled: bool,
    /// Log level
    pub level: String,
    /// Log format
    pub format: String,
    /// Include request body
    pub include_body: bool,
    /// Include response body
    pub include_response_body: bool,
    /// Maximum body size to log
    pub max_body_size: usize,
    /// Exclude paths
    pub exclude_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct ResponseTransformConfig {
    /// Enable response transformation
    pub enabled: bool,
    /// Transformation rules
    pub rules: Vec<TransformRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct RequestTransformConfig {
    /// Enable request transformation
    pub enabled: bool,
    /// Transformation rules
    pub rules: Vec<TransformRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformRule {
    /// Rule name
    pub name: String,
    /// Path pattern
    pub path_pattern: String,
    /// HTTP methods
    pub methods: Vec<String>,
    /// Transformation type
    pub transform_type: TransformationType,
    /// Transformation configuration
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransformationType {
    AddHeader,
    RemoveHeader,
    ModifyHeader,
    AddQueryParam,
    RemoveQueryParam,
    ModifyQueryParam,
    ModifyBody,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomMiddlewareConfig {
    /// Middleware name
    pub name: String,
    /// Middleware type
    pub middleware_type: String,
    /// Priority (lower numbers execute first)
    pub priority: u32,
    /// Configuration
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    /// Route ID
    pub id: String,
    /// Path pattern
    pub path: String,
    /// HTTP methods
    pub methods: Vec<String>,
    /// Upstream service
    pub upstream: String,
    /// Path rewrite rules
    pub rewrite: Option<PathRewriteConfig>,
    /// Route-specific middleware
    pub middleware: Vec<String>,
    /// Route-specific authentication
    pub auth: Option<RouteAuthConfig>,
    /// Route-specific rate limiting
    pub rate_limit: Option<RouteRateLimit>,
    /// Route-specific timeout
    pub timeout: Option<u64>,
    /// Route priority
    pub priority: u32,
    /// Route tags
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathRewriteConfig {
    /// Enable path rewriting
    pub enabled: bool,
    /// Rewrite pattern (regex)
    pub pattern: String,
    /// Replacement string
    pub replacement: String,
    /// Strip prefix
    pub strip_prefix: Option<String>,
    /// Add prefix
    pub add_prefix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteAuthConfig {
    /// Override global auth
    pub override_global: bool,
    /// Required methods
    pub required_methods: Vec<AuthMethod>,
    /// Required permissions
    pub required_permissions: Vec<String>,
    /// Required roles
    pub required_roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamService {
    /// Service ID
    pub id: String,
    /// Service name
    pub name: String,
    /// Service targets
    pub targets: Vec<ServiceTarget>,
    /// Service weight
    pub weight: u32,
    /// Health check configuration
    pub health_check: Option<ServiceHealthCheck>,
    /// Circuit breaker configuration
    pub circuit_breaker: Option<ServiceCircuitBreaker>,
    /// Service metadata
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceTarget {
    /// Target ID
    pub id: String,
    /// Target host
    pub host: String,
    /// Target port
    pub port: u16,
    /// Target weight
    pub weight: u32,
    /// Target health status
    pub healthy: bool,
    /// Target metadata
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealthCheck {
    /// Enable health checks
    pub enabled: bool,
    /// Health check path
    pub path: String,
    /// Health check interval in seconds
    pub interval: u64,
    /// Health check timeout in seconds
    pub timeout: u64,
    /// Healthy threshold
    pub healthy_threshold: u32,
    /// Unhealthy threshold
    pub unhealthy_threshold: u32,
    /// Expected status codes
    pub expected_status: Vec<u16>,
    /// Expected response body
    pub expected_body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCircuitBreaker {
    /// Enable circuit breaker
    pub enabled: bool,
    /// Failure threshold
    pub failure_threshold: u32,
    /// Recovery timeout in seconds
    pub recovery_timeout: u64,
    /// Half-open max calls
    pub half_open_max_calls: u32,
    /// Rolling window size in seconds
    pub rolling_window: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Enable health checks
    pub enabled: bool,
    /// Health check endpoint
    pub endpoint: String,
    /// Health check interval in seconds
    pub interval: u64,
    /// Health check timeout in seconds
    pub timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Enable circuit breaker
    pub enabled: bool,
    /// Default failure threshold
    pub default_failure_threshold: u32,
    /// Default recovery timeout in seconds
    pub default_recovery_timeout: u64,
    /// Default half-open max calls
    pub default_half_open_max_calls: u32,
    /// Default rolling window size in seconds
    pub default_rolling_window: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// Default request timeout in seconds
    pub request_timeout: u64,
    /// Default upstream timeout in seconds
    pub upstream_timeout: u64,
    /// Keep-alive timeout in seconds
    pub keep_alive_timeout: u64,
    /// Header timeout in seconds
    pub header_timeout: u64,
    /// Idle timeout in seconds
    pub idle_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayMonitoringConfig {
    /// Enable monitoring
    pub enabled: bool,
    /// Metrics collection
    pub metrics: GatewayMetricsConfig,
    /// Request tracing
    pub tracing: GatewayTracingConfig,
    /// Logging configuration
    pub logging: GatewayLoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayMetricsConfig {
    /// Enable metrics
    pub enabled: bool,
    /// Metrics endpoint
    pub endpoint: String,
    /// Collection interval in seconds
    pub collection_interval: u64,
    /// Include detailed metrics
    pub detailed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayTracingConfig {
    /// Enable tracing
    pub enabled: bool,
    /// Trace all requests
    pub trace_all: bool,
    /// Sample rate (0.0 to 1.0)
    pub sample_rate: f64,
    /// Include request/response bodies
    pub include_bodies: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayLoggingConfig {
    /// Log level
    pub level: String,
    /// Log format
    pub format: String,
    /// Log access requests
    pub access_log: bool,
    /// Log error details
    pub error_log: bool,
}

impl Default for ApiGatewayConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bind_address: "0.0.0.0".to_string(),
            port: 8090,
            tls: None,
            load_balancer: LoadBalancerConfig::default(),
            rate_limiting: RateLimitingConfig::default(),
            authentication: AuthenticationConfig::default(),
            cors: CorsConfig::default(),
            middleware: MiddlewareConfig::default(),
            routes: Vec::new(),
            upstream_services: Vec::new(),
            health_checks: HealthCheckConfig::default(),
            circuit_breaker: CircuitBreakerConfig::default(),
            timeouts: TimeoutConfig::default(),
            monitoring: GatewayMonitoringConfig::default(),
        }
    }
}

impl Default for LoadBalancerConfig {
    fn default() -> Self {
        Self {
            algorithm: LoadBalancingAlgorithm::RoundRobin,
            health_check_enabled: true,
            health_check_interval: 30,
            max_retries: 3,
            retry_delay: 1000,
            session_affinity: None,
            failover: FailoverConfig::default(),
        }
    }
}

impl Default for FailoverConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold: 3,
            recovery_threshold: 2,
            blacklist_timeout: 300,
        }
    }
}

impl Default for RateLimitingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_rate: 100,
            default_burst: 200,
            algorithm: RateLimitingAlgorithm::TokenBucket,
            per_client_limits: HashMap::new(),
            per_route_limits: HashMap::new(),
            storage: RateLimitStorageConfig::default(),
            sliding_window: SlidingWindowConfig::default(),
        }
    }
}

impl Default for RateLimitStorageConfig {
    fn default() -> Self {
        Self {
            backend: StorageBackend::Memory,
            redis: None,
            memory: Some(MemoryStorageConfig::default()),
        }
    }
}

impl Default for MemoryStorageConfig {
    fn default() -> Self {
        Self {
            max_entries: 10000,
            cleanup_interval: 60,
        }
    }
}

impl Default for SlidingWindowConfig {
    fn default() -> Self {
        Self {
            window_size: 60,
            sub_windows: 6,
            precision: 0.01,
        }
    }
}

impl Default for AuthenticationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            methods: vec![AuthMethod::ApiKey],
            jwt: None,
            api_key: None,
            oauth: None,
            basic_auth: None,
        }
    }
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
            ],
            allowed_headers: vec!["*".to_string()],
            expose_headers: Vec::new(),
            allow_credentials: false,
            max_age: 86400,
        }
    }
}


impl Default for RequestLoggingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            level: "info".to_string(),
            format: "combined".to_string(),
            include_body: false,
            include_response_body: false,
            max_body_size: 1024,
            exclude_paths: vec!["/health".to_string(), "/metrics".to_string()],
        }
    }
}



impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: "/health".to_string(),
            interval: 30,
            timeout: 10,
        }
    }
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_failure_threshold: 5,
            default_recovery_timeout: 60,
            default_half_open_max_calls: 3,
            default_rolling_window: 300,
        }
    }
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            request_timeout: 30,
            upstream_timeout: 25,
            keep_alive_timeout: 75,
            header_timeout: 5,
            idle_timeout: 180,
        }
    }
}

impl Default for GatewayMonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            metrics: GatewayMetricsConfig::default(),
            tracing: GatewayTracingConfig::default(),
            logging: GatewayLoggingConfig::default(),
        }
    }
}

impl Default for GatewayMetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: "/metrics".to_string(),
            collection_interval: 10,
            detailed: true,
        }
    }
}

impl Default for GatewayTracingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            trace_all: false,
            sample_rate: 0.1,
            include_bodies: false,
        }
    }
}

impl Default for GatewayLoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "json".to_string(),
            access_log: true,
            error_log: true,
        }
    }
}

/// Rate limiter implementation
pub struct RateLimiter {
    config: RateLimitingConfig,
    storage: Arc<dyn RateLimitStorage>,
    algorithm: Box<dyn RateLimitAlgorithm>,
}

pub trait RateLimitStorage: Send + Sync {
    fn increment(&self, key: &str, window: Duration) -> Result<u64>;
    fn get(&self, key: &str) -> Result<u64>;
    fn expire(&self, key: &str, ttl: Duration) -> Result<()>;
    fn cleanup(&self) -> Result<()>;
}

pub trait RateLimitAlgorithm: Send + Sync {
    fn check_rate_limit(
        &self,
        storage: &dyn RateLimitStorage,
        key: &str,
        limit: u32,
        window: Duration,
    ) -> Result<RateLimitResult>;
}

#[derive(Debug, Clone)]
pub struct RateLimitResult {
    pub allowed: bool,
    pub remaining: u32,
    pub reset_time: DateTime<Utc>,
    pub retry_after: Option<Duration>,
}

/// Memory-based rate limit storage
pub struct MemoryRateLimitStorage {
    storage: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
    cleanup_task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

#[derive(Debug, Clone)]
struct RateLimitEntry {
    count: u64,
    window_start: Instant,
    expires_at: Instant,
}

impl MemoryRateLimitStorage {
    pub fn new(config: &MemoryStorageConfig) -> Self {
        let storage = Arc::new(RwLock::new(HashMap::<String, RateLimitEntry>::new()));
        let cleanup_task = Arc::new(Mutex::new(None::<tokio::task::JoinHandle<()>>));

        // Start cleanup task
        if config.cleanup_interval > 0 {
            let storage_clone = Arc::clone(&storage);
            let interval = Duration::from_secs(config.cleanup_interval);
            let task = tokio::spawn(async move {
                let mut interval_timer = tokio::time::interval(interval);
                loop {
                    interval_timer.tick().await;
                    let mut storage = storage_clone.write().await;
                    let now = Instant::now();
                    storage.retain(|_, entry| entry.expires_at > now);
                }
            });

            let cleanup_task_clone = Arc::clone(&cleanup_task);
            tokio::spawn(async move {
                let mut task_guard = cleanup_task_clone.lock().await;
                *task_guard = Some(task);
            });
        }

        Self {
            storage,
            cleanup_task,
        }
    }
}

impl RateLimitStorage for MemoryRateLimitStorage {
    fn increment(&self, key: &str, window: Duration) -> Result<u64> {
        let now = Instant::now();
        let window_start = now;
        let expires_at = now + window;

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let mut storage = self.storage.write().await;

                let entry = storage
                    .entry(key.to_string())
                    .or_insert_with(|| RateLimitEntry {
                        count: 0,
                        window_start,
                        expires_at,
                    });

                // Reset if window expired
                if now >= entry.expires_at {
                    entry.count = 0;
                    entry.window_start = window_start;
                    entry.expires_at = expires_at;
                }

                entry.count += 1;
                Ok(entry.count)
            })
        })
    }

    fn get(&self, key: &str) -> Result<u64> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let storage = self.storage.read().await;
                Ok(storage.get(key).map(|entry| entry.count).unwrap_or(0))
            })
        })
    }

    fn expire(&self, key: &str, _ttl: Duration) -> Result<()> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let mut storage = self.storage.write().await;
                storage.remove(key);
                Ok(())
            })
        })
    }

    fn cleanup(&self) -> Result<()> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let mut storage = self.storage.write().await;
                let now = Instant::now();
                storage.retain(|_, entry| entry.expires_at > now);
                Ok(())
            })
        })
    }
}

/// Token bucket rate limiting algorithm
pub struct TokenBucketAlgorithm;

impl RateLimitAlgorithm for TokenBucketAlgorithm {
    fn check_rate_limit(
        &self,
        storage: &dyn RateLimitStorage,
        key: &str,
        limit: u32,
        window: Duration,
    ) -> Result<RateLimitResult> {
        let current_count = storage.increment(key, window)?;
        let allowed = current_count <= limit as u64;
        let remaining = if current_count <= limit as u64 {
            limit - current_count as u32
        } else {
            0
        };

        let reset_time = Utc::now() + chrono::Duration::from_std(window).unwrap();
        let retry_after = if !allowed { Some(window) } else { None };

        Ok(RateLimitResult {
            allowed,
            remaining,
            reset_time,
            retry_after,
        })
    }
}

/// Load balancer implementation
pub struct LoadBalancer {
    config: LoadBalancerConfig,
    services: Arc<RwLock<HashMap<String, UpstreamService>>>,
    targets: Arc<RwLock<HashMap<String, Vec<ServiceTarget>>>>,
    current_target: Arc<RwLock<HashMap<String, usize>>>,
    health_checker: Arc<HealthChecker>,
    circuit_breakers: Arc<RwLock<HashMap<String, CircuitBreaker>>>,
}

impl LoadBalancer {
    pub fn new(config: LoadBalancerConfig, services: Vec<UpstreamService>) -> Self {
        let mut targets = HashMap::new();
        let mut service_map = HashMap::new();

        for service in services {
            targets.insert(service.id.clone(), service.targets.clone());
            service_map.insert(service.id.clone(), service);
        }

        Self {
            config,
            services: Arc::new(RwLock::new(service_map)),
            targets: Arc::new(RwLock::new(targets)),
            current_target: Arc::new(RwLock::new(HashMap::new())),
            health_checker: Arc::new(HealthChecker::new()),
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn select_target(&self, service_id: &str) -> Result<Option<ServiceTarget>> {
        let targets = self.targets.read().await;
        let service_targets = match targets.get(service_id) {
            Some(targets) => targets,
            None => return Ok(None),
        };

        if service_targets.is_empty() {
            return Ok(None);
        }

        // Filter healthy targets
        let healthy_targets: Vec<&ServiceTarget> = service_targets
            .iter()
            .filter(|target| target.healthy)
            .collect();

        if healthy_targets.is_empty() {
            warn!("No healthy targets available for service: {}", service_id);
            return Ok(None);
        }

        match self.config.algorithm {
            LoadBalancingAlgorithm::RoundRobin => {
                self.round_robin_selection(service_id, &healthy_targets)
                    .await
            }
            LoadBalancingAlgorithm::WeightedRoundRobin => {
                self.weighted_round_robin_selection(service_id, &healthy_targets)
                    .await
            }
            LoadBalancingAlgorithm::LeastConnections => {
                self.least_connections_selection(&healthy_targets).await
            }
            LoadBalancingAlgorithm::Random => self.random_selection(&healthy_targets).await,
            LoadBalancingAlgorithm::IPHash => {
                // Would need client IP for proper implementation
                self.round_robin_selection(service_id, &healthy_targets)
                    .await
            }
            _ => {
                // Default to round robin
                self.round_robin_selection(service_id, &healthy_targets)
                    .await
            }
        }
    }

    async fn round_robin_selection(
        &self,
        service_id: &str,
        targets: &[&ServiceTarget],
    ) -> Result<Option<ServiceTarget>> {
        let mut current = self.current_target.write().await;
        let index = current.entry(service_id.to_string()).or_insert(0);

        *index = (*index + 1) % targets.len();
        Ok(Some(targets[*index].clone()))
    }

    async fn weighted_round_robin_selection(
        &self,
        service_id: &str,
        targets: &[&ServiceTarget],
    ) -> Result<Option<ServiceTarget>> {
        // Simple implementation - could be more sophisticated
        let total_weight: u32 = targets.iter().map(|t| t.weight).sum();
        if total_weight == 0 {
            return self.round_robin_selection(service_id, targets).await;
        }

        let mut current = self.current_target.write().await;
        let index = current.entry(service_id.to_string()).or_insert(0);

        *index = (*index + 1) % targets.len();
        Ok(Some(targets[*index].clone()))
    }

    async fn least_connections_selection(
        &self,
        targets: &[&ServiceTarget],
    ) -> Result<Option<ServiceTarget>> {
        // For now, just select the first target
        // In a real implementation, this would track active connections
        Ok(targets.first().map(|t| (*t).clone()))
    }

    async fn random_selection(&self, targets: &[&ServiceTarget]) -> Result<Option<ServiceTarget>> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..targets.len());
        Ok(Some(targets[index].clone()))
    }

    pub async fn update_target_health(
        &self,
        service_id: &str,
        target_id: &str,
        healthy: bool,
    ) -> Result<()> {
        let mut targets = self.targets.write().await;
        if let Some(service_targets) = targets.get_mut(service_id) {
            for target in service_targets.iter_mut() {
                if target.id == target_id {
                    target.healthy = healthy;
                    break;
                }
            }
        }
        Ok(())
    }
}

/// Health checker implementation
pub struct HealthChecker {
    active_checks: Arc<RwLock<HashMap<String, tokio::task::JoinHandle<()>>>>,
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            active_checks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start_health_checks(
        &self,
        service: &UpstreamService,
        load_balancer: Arc<LoadBalancer>,
    ) -> Result<()> {
        if let Some(health_config) = &service.health_check {
            if !health_config.enabled {
                return Ok(());
            }

            for target in &service.targets {
                self.start_target_health_check(
                    &service.id,
                    target,
                    health_config,
                    Arc::clone(&load_balancer),
                )
                .await?;
            }
        }
        Ok(())
    }

    async fn start_target_health_check(
        &self,
        service_id: &str,
        target: &ServiceTarget,
        config: &ServiceHealthCheck,
        load_balancer: Arc<LoadBalancer>,
    ) -> Result<()> {
        let check_id = format!("{}:{}", service_id, target.id);
        let url = format!("http://{}:{}{}", target.host, target.port, config.path);
        let interval = Duration::from_secs(config.interval);
        let timeout = Duration::from_secs(config.timeout);
        let expected_status = config.expected_status.clone();
        let expected_body = config.expected_body.clone();
        let service_id = service_id.to_string();
        let target_id = target.id.clone();
        let healthy_threshold = config.healthy_threshold;
        let unhealthy_threshold = config.unhealthy_threshold;

        let task = tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            let mut consecutive_failures = 0;
            let mut consecutive_successes = 0;

            loop {
                interval_timer.tick().await;

                let health_result = Self::perform_health_check(
                    &url,
                    timeout,
                    &expected_status,
                    expected_body.as_deref(),
                )
                .await;

                match health_result {
                    Ok(healthy) => {
                        if healthy {
                            consecutive_successes += 1;
                            consecutive_failures = 0;

                            if consecutive_successes >= healthy_threshold {
                                if let Err(e) = load_balancer
                                    .update_target_health(&service_id, &target_id, true)
                                    .await
                                {
                                    error!("Failed to update target health: {}", e);
                                }
                            }
                        } else {
                            consecutive_failures += 1;
                            consecutive_successes = 0;

                            if consecutive_failures >= unhealthy_threshold {
                                if let Err(e) = load_balancer
                                    .update_target_health(&service_id, &target_id, false)
                                    .await
                                {
                                    error!("Failed to update target health: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Health check failed for {}: {}", url, e);
                        consecutive_failures += 1;
                        consecutive_successes = 0;

                        if consecutive_failures >= unhealthy_threshold {
                            if let Err(e) = load_balancer
                                .update_target_health(&service_id, &target_id, false)
                                .await
                            {
                                error!("Failed to update target health: {}", e);
                            }
                        }
                    }
                }
            }
        });

        let mut active_checks = self.active_checks.write().await;
        active_checks.insert(check_id, task);

        Ok(())
    }

    async fn perform_health_check(
        url: &str,
        timeout: Duration,
        expected_status: &[u16],
        expected_body: Option<&str>,
    ) -> Result<bool> {
        #[cfg(feature = "reqwest")]
        {
            let client = reqwest::Client::new();
            let response = tokio::time::timeout(timeout, client.get(url).send())
                .await
                .map_err(|_| anyhow::anyhow!("Health check timeout"))?
                .map_err(|e| anyhow::anyhow!("Health check request failed: {}", e))?;

            let status_ok = if expected_status.is_empty() {
                response.status().is_success()
            } else {
                expected_status.contains(&response.status().as_u16())
            };

            if !status_ok {
                return Ok(false);
            }

            if let Some(expected) = expected_body {
                let body = response
                    .text()
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to read response body: {}", e))?;
                return Ok(body.contains(expected));
            }

            Ok(true)
        }

        #[cfg(not(feature = "reqwest"))]
        {
            // Mock implementation
            info!("Health check for {} (mock)", url);
            Ok(true)
        }
    }

    pub async fn stop_health_checks(&self) -> Result<()> {
        let mut active_checks = self.active_checks.write().await;
        for (_, task) in active_checks.drain() {
            task.abort();
        }
        Ok(())
    }
}

/// Circuit breaker implementation
pub struct CircuitBreaker {
    config: ServiceCircuitBreaker,
    state: Arc<RwLock<CircuitBreakerState>>,
    failure_count: Arc<RwLock<u32>>,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
    half_open_calls: Arc<RwLock<u32>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

impl CircuitBreaker {
    pub fn new(config: ServiceCircuitBreaker) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(CircuitBreakerState::Closed)),
            failure_count: Arc::new(RwLock::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
            half_open_calls: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn should_allow_request(&self) -> bool {
        let state = self.state.read().await;
        match *state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                drop(state);
                self.check_recovery_timeout().await
            }
            CircuitBreakerState::HalfOpen => {
                drop(state);
                let half_open_calls = self.half_open_calls.read().await;
                *half_open_calls < self.config.half_open_max_calls
            }
        }
    }

    pub async fn record_success(&self) {
        let mut state = self.state.write().await;
        match *state {
            CircuitBreakerState::HalfOpen => {
                *state = CircuitBreakerState::Closed;
                let mut failure_count = self.failure_count.write().await;
                *failure_count = 0;
                let mut half_open_calls = self.half_open_calls.write().await;
                *half_open_calls = 0;
                info!("Circuit breaker closed after successful request");
            }
            CircuitBreakerState::Closed => {
                let mut failure_count = self.failure_count.write().await;
                *failure_count = 0;
            }
            _ => {}
        }
    }

    pub async fn record_failure(&self) {
        let mut failure_count = self.failure_count.write().await;
        *failure_count += 1;
        let mut last_failure_time = self.last_failure_time.write().await;
        *last_failure_time = Some(Instant::now());

        if *failure_count >= self.config.failure_threshold {
            let mut state = self.state.write().await;
            if *state != CircuitBreakerState::Open {
                *state = CircuitBreakerState::Open;
                warn!("Circuit breaker opened after {} failures", *failure_count);
            }
        }
    }

    async fn check_recovery_timeout(&self) -> bool {
        let last_failure = self.last_failure_time.read().await;
        if let Some(last_time) = *last_failure {
            if last_time.elapsed() >= Duration::from_secs(self.config.recovery_timeout) {
                drop(last_failure);
                let mut state = self.state.write().await;
                if *state == CircuitBreakerState::Open {
                    *state = CircuitBreakerState::HalfOpen;
                    let mut half_open_calls = self.half_open_calls.write().await;
                    *half_open_calls = 0;
                    info!("Circuit breaker transitioned to half-open");
                    return true;
                }
            }
        }
        false
    }

    pub async fn record_call(&self) {
        let state = self.state.read().await;
        if *state == CircuitBreakerState::HalfOpen {
            drop(state);
            let mut half_open_calls = self.half_open_calls.write().await;
            *half_open_calls += 1;
        }
    }
}

/// Main API Gateway implementation
pub struct ApiGateway {
    config: ApiGatewayConfig,
    rate_limiter: Arc<RateLimiter>,
    load_balancer: Arc<LoadBalancer>,
    health_checker: Arc<HealthChecker>,
    circuit_breakers: Arc<RwLock<HashMap<String, Arc<CircuitBreaker>>>>,
}

impl ApiGateway {
    pub async fn new(config: ApiGatewayConfig) -> Result<Self> {
        // Initialize storage
        let storage: Arc<dyn RateLimitStorage> = match config.rate_limiting.storage.backend {
            StorageBackend::Memory => {
                let memory_config = config
                    .rate_limiting
                    .storage
                    .memory
                    .as_ref()
                    .cloned()
                    .unwrap_or_else(MemoryStorageConfig::default);
                Arc::new(MemoryRateLimitStorage::new(&memory_config))
            }
            _ => {
                return Err(anyhow::anyhow!("Unsupported storage backend"));
            }
        };

        // Initialize rate limiting algorithm
        let algorithm: Box<dyn RateLimitAlgorithm> = match config.rate_limiting.algorithm {
            RateLimitingAlgorithm::TokenBucket => Box::new(TokenBucketAlgorithm),
            _ => {
                return Err(anyhow::anyhow!("Unsupported rate limiting algorithm"));
            }
        };

        let rate_limiter = Arc::new(RateLimiter {
            config: config.rate_limiting.clone(),
            storage,
            algorithm,
        });

        // Initialize load balancer
        let load_balancer = Arc::new(LoadBalancer::new(
            config.load_balancer.clone(),
            config.upstream_services.clone(),
        ));

        // Initialize health checker
        let health_checker = Arc::new(HealthChecker::new());

        // Initialize circuit breakers
        let mut circuit_breakers = HashMap::new();
        for service in &config.upstream_services {
            if let Some(cb_config) = &service.circuit_breaker {
                if cb_config.enabled {
                    circuit_breakers.insert(
                        service.id.clone(),
                        Arc::new(CircuitBreaker::new(cb_config.clone())),
                    );
                }
            }
        }

        Ok(Self {
            config,
            rate_limiter,
            load_balancer,
            health_checker,
            circuit_breakers: Arc::new(RwLock::new(circuit_breakers)),
        })
    }

    pub async fn start(&self) -> Result<()> {
        info!(
            "Starting API Gateway on {}:{}",
            self.config.bind_address, self.config.port
        );

        // Start health checks
        for service in &self.config.upstream_services {
            self.health_checker
                .start_health_checks(service, Arc::clone(&self.load_balancer))
                .await?;
        }

        info!("API Gateway started successfully");
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        info!("Stopping API Gateway");

        // Stop health checks
        self.health_checker.stop_health_checks().await?;

        info!("API Gateway stopped");
        Ok(())
    }

    pub async fn check_rate_limit(&self, client_id: &str, route: &str) -> Result<RateLimitResult> {
        // Check route-specific rate limit first
        if let Some(route_limit) = self.config.rate_limiting.per_route_limits.get(route) {
            let key = if route_limit.per_client {
                format!("route:{}:client:{}", route, client_id)
            } else {
                format!("route:{}", route)
            };

            return self.rate_limiter.algorithm.check_rate_limit(
                self.rate_limiter.storage.as_ref(),
                &key,
                route_limit.rate,
                Duration::from_secs(route_limit.window),
            );
        }

        // Check client-specific rate limit
        if let Some(client_limit) = self.config.rate_limiting.per_client_limits.get(client_id) {
            let key = format!("client:{}", client_id);
            return self.rate_limiter.algorithm.check_rate_limit(
                self.rate_limiter.storage.as_ref(),
                &key,
                client_limit.rate,
                Duration::from_secs(client_limit.window),
            );
        }

        // Use default rate limit
        let key = format!("client:{}", client_id);
        self.rate_limiter.algorithm.check_rate_limit(
            self.rate_limiter.storage.as_ref(),
            &key,
            self.config.rate_limiting.default_rate,
            Duration::from_secs(60), // Default 1 minute window
        )
    }

    pub async fn route_request(&self, route_id: &str) -> Result<Option<ServiceTarget>> {
        // Find route configuration
        let route = self.config.routes.iter().find(|r| r.id == route_id);
        if let Some(route_config) = route {
            // Check circuit breaker
            let circuit_breakers = self.circuit_breakers.read().await;
            if let Some(circuit_breaker) = circuit_breakers.get(&route_config.upstream) {
                if !circuit_breaker.should_allow_request().await {
                    return Err(anyhow::anyhow!(
                        "Circuit breaker is open for service: {}",
                        route_config.upstream
                    ));
                }
            }

            // Select target from load balancer
            self.load_balancer
                .select_target(&route_config.upstream)
                .await
        } else {
            Err(anyhow::anyhow!("Route not found: {}", route_id))
        }
    }

    pub async fn record_request_result(&self, service_id: &str, success: bool) -> Result<()> {
        let circuit_breakers = self.circuit_breakers.read().await;
        if let Some(circuit_breaker) = circuit_breakers.get(service_id) {
            if success {
                circuit_breaker.record_success().await;
            } else {
                circuit_breaker.record_failure().await;
            }
        }
        Ok(())
    }

    pub async fn get_status(&self) -> Result<GatewayStatus> {
        let routes_count = self.config.routes.len();
        let services_count = self.config.upstream_services.len();

        let circuit_breakers = self.circuit_breakers.read().await;
        let mut circuit_breaker_status = HashMap::new();
        for (service_id, cb) in circuit_breakers.iter() {
            let state = cb.state.read().await;
            circuit_breaker_status.insert(service_id.clone(), format!("{:?}", *state));
        }

        Ok(GatewayStatus {
            enabled: self.config.enabled,
            routes_count,
            services_count,
            rate_limiting_enabled: self.config.rate_limiting.enabled,
            authentication_enabled: self.config.authentication.enabled,
            circuit_breaker_status,
            uptime: Instant::now(), // Would track actual uptime
        })
    }
}

#[derive(Debug, Serialize)]
pub struct GatewayStatus {
    pub enabled: bool,
    pub routes_count: usize,
    pub services_count: usize,
    pub rate_limiting_enabled: bool,
    pub authentication_enabled: bool,
    pub circuit_breaker_status: HashMap<String, String>,
    #[serde(skip_serializing)]
    pub uptime: Instant,
}
