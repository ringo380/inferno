use crate::config::Config;
use anyhow::{Context, Result};
use axum::{
    extract::{ws::WebSocket, Path, Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use sysinfo::SystemExt;
use tokio::sync::{broadcast, RwLock};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};
use uuid::Uuid;

/// Configuration for the web dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    /// Enable the web dashboard
    pub enabled: bool,
    /// Dashboard server binding address
    pub bind_address: String,
    /// Dashboard server port
    pub port: u16,
    /// Static assets directory
    pub assets_dir: PathBuf,
    /// Authentication settings
    pub auth: DashboardAuthConfig,
    /// UI customization
    pub ui: UiConfig,
    /// Real-time updates
    pub realtime: RealtimeConfig,
    /// Security settings
    pub security: DashboardSecurityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardAuthConfig {
    /// Enable authentication
    pub enabled: bool,
    /// Authentication provider
    pub provider: AuthProvider,
    /// Session timeout in minutes
    pub session_timeout_minutes: u32,
    /// JWT secret for token signing
    pub jwt_secret: Option<String>,
    /// Admin users
    pub admin_users: Vec<String>,
    /// Read-only users
    pub readonly_users: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthProvider {
    Local,
    OAuth2,
    LDAP,
    SAML,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Dashboard title
    pub title: String,
    /// Theme configuration
    pub theme: ThemeConfig,
    /// Layout settings
    pub layout: LayoutConfig,
    /// Feature toggles
    pub features: FeatureConfig,
    /// Custom branding
    pub branding: BrandingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Default theme (light, dark, auto)
    pub default_theme: String,
    /// Allow theme switching
    pub allow_switching: bool,
    /// Custom CSS file
    pub custom_css: Option<PathBuf>,
    /// Color scheme overrides
    pub colors: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    /// Show sidebar by default
    pub sidebar_expanded: bool,
    /// Dashboard refresh interval in seconds
    pub refresh_interval: u32,
    /// Items per page for lists
    pub items_per_page: u32,
    /// Show advanced features
    pub show_advanced: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    /// Enable model management
    pub model_management: bool,
    /// Enable metrics dashboard
    pub metrics: bool,
    /// Enable federated learning UI
    pub federated_learning: bool,
    /// Enable marketplace integration
    pub marketplace: bool,
    /// Enable deployment management
    pub deployment: bool,
    /// Enable system monitoring
    pub monitoring: bool,
    /// Enable user management
    pub user_management: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandingConfig {
    /// Organization name
    pub organization: String,
    /// Logo URL or path
    pub logo: Option<String>,
    /// Favicon URL or path
    pub favicon: Option<String>,
    /// Custom header HTML
    pub custom_header: Option<String>,
    /// Custom footer HTML
    pub custom_footer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeConfig {
    /// Enable real-time updates
    pub enabled: bool,
    /// WebSocket endpoint path
    pub ws_path: String,
    /// Update frequency in milliseconds
    pub update_frequency_ms: u64,
    /// Maximum concurrent connections
    pub max_connections: u32,
    /// Message buffer size
    pub buffer_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSecurityConfig {
    /// Enable HTTPS
    pub https_enabled: bool,
    /// TLS certificate path
    pub cert_path: Option<PathBuf>,
    /// TLS private key path
    pub key_path: Option<PathBuf>,
    /// Content Security Policy
    pub csp_header: Option<String>,
    /// Rate limiting settings
    pub rate_limit: RateLimitConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Enable rate limiting
    pub enabled: bool,
    /// Requests per minute per IP
    pub requests_per_minute: u32,
    /// Burst allowance
    pub burst_size: u32,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        let assets_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("inferno")
            .join("dashboard");

        Self {
            enabled: true,
            bind_address: "127.0.0.1".to_string(),
            port: 8080,
            assets_dir,
            auth: DashboardAuthConfig::default(),
            ui: UiConfig::default(),
            realtime: RealtimeConfig::default(),
            security: DashboardSecurityConfig::default(),
        }
    }
}

impl Default for DashboardAuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: AuthProvider::Local,
            session_timeout_minutes: 480, // 8 hours
            jwt_secret: None,
            admin_users: vec!["admin".to_string()],
            readonly_users: vec![],
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            title: "Inferno AI Dashboard".to_string(),
            theme: ThemeConfig::default(),
            layout: LayoutConfig::default(),
            features: FeatureConfig::default(),
            branding: BrandingConfig::default(),
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            default_theme: "auto".to_string(),
            allow_switching: true,
            custom_css: None,
            colors: HashMap::new(),
        }
    }
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            sidebar_expanded: true,
            refresh_interval: 30,
            items_per_page: 25,
            show_advanced: false,
        }
    }
}

impl Default for FeatureConfig {
    fn default() -> Self {
        Self {
            model_management: true,
            metrics: true,
            federated_learning: true,
            marketplace: true,
            deployment: true,
            monitoring: true,
            user_management: false,
        }
    }
}

impl Default for BrandingConfig {
    fn default() -> Self {
        Self {
            organization: "Inferno AI".to_string(),
            logo: None,
            favicon: None,
            custom_header: None,
            custom_footer: None,
        }
    }
}

impl Default for RealtimeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ws_path: "/ws".to_string(),
            update_frequency_ms: 5000,
            max_connections: 100,
            buffer_size: 1024,
        }
    }
}

impl Default for DashboardSecurityConfig {
    fn default() -> Self {
        Self {
            https_enabled: false,
            cert_path: None,
            key_path: None,
            csp_header: Some("default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'".to_string()),
            rate_limit: RateLimitConfig::default(),
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            requests_per_minute: 60,
            burst_size: 10,
        }
    }
}

impl DashboardConfig {
    pub fn from_config(config: &Config) -> Result<Self> {
        Ok(config.dashboard.clone())
    }
}

/// Dashboard application state
#[derive(Clone)]
pub struct DashboardState {
    pub config: DashboardConfig,
    pub models: Arc<RwLock<Vec<ModelInfo>>>,
    pub metrics: Arc<RwLock<SystemMetrics>>,
    pub nodes: Arc<RwLock<Vec<NodeInfo>>>,
    pub deployments: Arc<RwLock<Vec<DeploymentInfo>>>,
    pub users: Arc<RwLock<Vec<UserInfo>>>,
    pub notifications: broadcast::Sender<NotificationMessage>,
    pub security_manager: Arc<crate::security::SecurityManager>,
    pub marketplace: Arc<crate::marketplace::ModelMarketplace>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub format: String,
    pub size_mb: f64,
    pub accuracy: Option<f64>,
    pub status: ModelStatus,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub usage_count: u64,
    pub tags: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelStatus {
    Available,
    Loading,
    Training,
    Deployed,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: DateTime<Utc>,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub gpu_usage: Option<f32>,
    pub disk_usage: f32,
    pub network_io: NetworkIO,
    pub inference_stats: InferenceStats,
    pub model_stats: ModelStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIO {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_latency_ms: f64,
    pub requests_per_second: f64,
    pub tokens_per_second: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStats {
    pub active_models: u32,
    pub total_models: u32,
    pub models_loading: u32,
    pub models_deployed: u32,
    pub total_parameters: u64,
    pub total_size_gb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: String,
    pub name: String,
    pub role: String,
    pub status: NodeStatus,
    pub endpoint: String,
    pub capabilities: NodeCapabilities,
    pub current_load: f32,
    pub last_seen: DateTime<Utc>,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeStatus {
    Online,
    Offline,
    Training,
    Deploying,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCapabilities {
    pub cpu_cores: u32,
    pub memory_gb: f64,
    pub gpu_count: u32,
    pub storage_gb: f64,
    pub supported_formats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentInfo {
    pub id: String,
    pub model_id: String,
    pub environment: String,
    pub status: DeploymentStatus,
    pub replicas: u32,
    pub target_replicas: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub health_checks: Vec<HealthCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentStatus {
    Pending,
    Deploying,
    Running,
    Scaling,
    Failed,
    Terminated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub name: String,
    pub status: HealthStatus,
    pub last_check: DateTime<Utc>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub last_login: Option<DateTime<Utc>>,
    pub active: bool,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserRole {
    Admin,
    User,
    ReadOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationMessage {
    pub id: String,
    pub level: NotificationLevel,
    pub title: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub category: String,
    pub actions: Vec<NotificationAction>,
}

// API Request/Response structures
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateModelRequest {
    pub name: String,
    pub version: String,
    pub format: String,
    pub description: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateModelRequest {
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeployModelRequest {
    pub environment: String,
    pub replicas: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDeploymentRequest {
    pub model_id: String,
    pub environment: String,
    pub replicas: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateDeploymentRequest {
    pub environment: Option<String>,
    pub target_replicas: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScaleDeploymentRequest {
    pub replicas: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserProfile,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub role: String,
    pub last_login: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: Option<String>,
    pub password: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserListResponse {
    pub users: Vec<UserSummary>,
    pub total: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSummary {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub role: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDetailResponse {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub role: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub permissions: Vec<String>,
}

/// Authentication context extracted from JWT
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: String,
    pub username: String,
    pub role: crate::security::UserRole,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemConfigResponse {
    pub dashboard: DashboardConfigSummary,
    pub security: SecurityConfigSummary,
    pub server: ServerConfigSummary,
    pub features: FeatureConfigSummary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemConfigUpdateRequest {
    pub dashboard: Option<DashboardConfigUpdate>,
    pub security: Option<SecurityConfigUpdate>,
    pub server: Option<ServerConfigUpdate>,
    pub features: Option<FeatureConfigUpdate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardConfigSummary {
    pub enabled: bool,
    pub port: u16,
    pub bind_address: String,
    pub auth_enabled: bool,
    pub theme: String,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityConfigSummary {
    pub auth_enabled: bool,
    pub rate_limiting_enabled: bool,
    pub max_requests_per_minute: u32,
    pub token_expiry_hours: i64,
    pub tls_required: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfigSummary {
    pub max_concurrent_requests: usize,
    pub request_timeout_seconds: u64,
    pub enable_cors: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureConfigSummary {
    pub model_management: bool,
    pub metrics: bool,
    pub marketplace: bool,
    pub deployment: bool,
    pub user_management: bool,
    pub monitoring: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardConfigUpdate {
    pub enabled: Option<bool>,
    pub port: Option<u16>,
    pub bind_address: Option<String>,
    pub theme: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityConfigUpdate {
    pub rate_limiting_enabled: Option<bool>,
    pub max_requests_per_minute: Option<u32>,
    pub token_expiry_hours: Option<i64>,
    pub tls_required: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfigUpdate {
    pub max_concurrent_requests: Option<usize>,
    pub request_timeout_seconds: Option<u64>,
    pub enable_cors: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureConfigUpdate {
    pub model_management: Option<bool>,
    pub metrics: Option<bool>,
    pub marketplace: Option<bool>,
    pub deployment: Option<bool>,
    pub user_management: Option<bool>,
    pub monitoring: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketplaceSearchRequest {
    pub query: Option<String>,
    pub format: Option<String>,   // gguf, onnx, pytorch, etc.
    pub category: Option<String>, // llm, embedding, vision, etc.
    pub size_limit_gb: Option<f64>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketplaceSearchResponse {
    pub models: Vec<MarketplaceModel>,
    pub total: usize,
    pub has_more: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketplaceModel {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub format: String,
    pub size_gb: f64,
    pub category: String,
    pub license: String,
    pub downloads: u64,
    pub rating: f32,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub download_url: String,
    pub homepage_url: Option<String>,
    pub documentation_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeaturedModelsResponse {
    pub featured: Vec<MarketplaceModel>,
    pub trending: Vec<MarketplaceModel>,
    pub recent: Vec<MarketplaceModel>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelDownloadInfo {
    pub model_id: String,
    pub download_url: String,
    pub checksum: String,
    pub checksum_type: String, // sha256, md5, etc.
    pub size_bytes: u64,
    pub expires_at: Option<DateTime<Utc>>,
    pub download_instructions: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub details: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub data: T,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsHistoryQuery {
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub interval: Option<String>, // "1m", "5m", "1h", "1d"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsExportQuery {
    pub format: Option<String>, // "json", "csv", "prometheus"
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub actions: Vec<NotificationAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
    Primary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationAction {
    pub label: String,
    pub action: String,
    pub style: ActionStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionStyle {
    Primary,
    Secondary,
    Success,
    Warning,
    Danger,
}

/// Main dashboard server
pub struct DashboardServer {
    config: DashboardConfig,
    state: DashboardState,
}

impl DashboardServer {
    pub fn new(config: DashboardConfig) -> Result<Self> {
        let (notification_tx, _) = broadcast::channel(1000);

        let state = DashboardState {
            config: config.clone(),
            models: Arc::new(RwLock::new(vec![])),
            metrics: Arc::new(RwLock::new(SystemMetrics::default())),
            nodes: Arc::new(RwLock::new(vec![])),
            deployments: Arc::new(RwLock::new(vec![])),
            users: Arc::new(RwLock::new(vec![])),
            notifications: notification_tx,
            security_manager: Arc::new(crate::security::SecurityManager::new(Default::default())),
            marketplace: Arc::new(crate::marketplace::ModelMarketplace::new(
                Default::default(),
            )?),
        };

        Ok(Self { config, state })
    }

    pub async fn start(&self) -> Result<()> {
        info!(
            "Starting dashboard server on {}:{}",
            self.config.bind_address, self.config.port
        );

        let app = self.create_router().await?;

        let addr = format!("{}:{}", self.config.bind_address, self.config.port);
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .context("Failed to bind dashboard server")?;

        info!("Dashboard server listening on http://{}", addr);

        axum::serve(listener, app)
            .await
            .context("Dashboard server error")?;

        Ok(())
    }

    async fn create_router(&self) -> Result<Router> {
        let app = Router::new()
            // Main dashboard pages
            .route("/", get(dashboard_index))
            .route("/models", get(models_page))
            .route("/metrics", get(metrics_page))
            .route("/nodes", get(nodes_page))
            .route("/deployments", get(deployments_page))
            .route("/marketplace", get(marketplace_page))
            .route("/settings", get(settings_page))
            // API endpoints
            .nest("/api/v1", self.create_api_router())
            // WebSocket for real-time updates
            .route(&self.config.realtime.ws_path, get(websocket_handler))
            // Static assets
            .nest("/assets", self.create_static_router())
            // State and middleware
            .with_state(self.state.clone())
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(CorsLayer::permissive()),
            );

        Ok(app)
    }

    fn create_api_router(&self) -> Router<DashboardState> {
        Router::new()
            // Models API
            .route("/models", get(api_list_models).post(api_create_model))
            .route(
                "/models/:id",
                get(api_get_model)
                    .put(api_update_model)
                    .delete(api_delete_model),
            )
            .route("/models/:id/deploy", post(api_deploy_model))
            .route("/models/:id/metrics", get(api_model_metrics))
            // Metrics API
            .route("/metrics", get(api_system_metrics))
            .route("/metrics/history", get(api_metrics_history))
            .route("/metrics/export", get(api_export_metrics))
            // Nodes API
            .route("/nodes", get(api_list_nodes))
            .route("/nodes/:id", get(api_get_node))
            .route("/nodes/:id/status", get(api_node_status))
            // Deployments API
            .route(
                "/deployments",
                get(api_list_deployments).post(api_create_deployment),
            )
            .route(
                "/deployments/:id",
                get(api_get_deployment)
                    .put(api_update_deployment)
                    .delete(api_delete_deployment),
            )
            .route("/deployments/:id/scale", post(api_scale_deployment))
            .route("/deployments/:id/logs", get(api_deployment_logs))
            // Marketplace API
            .route("/marketplace/search", get(api_marketplace_search))
            .route("/marketplace/featured", get(api_marketplace_featured))
            .route("/marketplace/downloads", get(api_marketplace_downloads))
            // System API
            .route("/system/info", get(api_system_info))
            .route("/system/health", get(api_health_check))
            .route("/system/config", get(api_get_config).put(api_update_config))
            // User management API
            .route("/users", get(api_list_users).post(api_create_user))
            .route(
                "/users/:id",
                get(api_get_user)
                    .put(api_update_user)
                    .delete(api_delete_user),
            )
            .route("/auth/login", post(api_login))
            .route("/auth/logout", post(api_logout))
            .route("/auth/profile", get(api_profile))
    }

    fn create_static_router(&self) -> Router<DashboardState> {
        // In a real implementation, this would serve static files from the assets directory
        Router::new()
            .route("/css/*path", get(serve_static_css))
            .route("/js/*path", get(serve_static_js))
            .route("/images/*path", get(serve_static_images))
            .route("/fonts/*path", get(serve_static_fonts))
    }

    pub async fn load_initial_data(&self) -> Result<()> {
        info!("Loading initial dashboard data");

        // Load models
        {
            let mut models = self.state.models.write().await;
            models.push(ModelInfo {
                id: "llama-7b".to_string(),
                name: "LLaMA 7B".to_string(),
                version: "v1.0".to_string(),
                format: "GGUF".to_string(),
                size_mb: 7168.0,
                accuracy: Some(0.85),
                status: ModelStatus::Available,
                created_at: Utc::now(),
                last_used: Some(Utc::now()),
                usage_count: 1250,
                tags: vec!["language-model".to_string(), "chat".to_string()],
                description: "7 billion parameter language model optimized for chat".to_string(),
            });
        }

        // Load nodes
        {
            let mut nodes = self.state.nodes.write().await;
            nodes.push(NodeInfo {
                id: "node-001".to_string(),
                name: "Primary Node".to_string(),
                role: "Coordinator".to_string(),
                status: NodeStatus::Online,
                endpoint: "http://localhost:8090".to_string(),
                capabilities: NodeCapabilities {
                    cpu_cores: 16,
                    memory_gb: 64.0,
                    gpu_count: 2,
                    storage_gb: 2048.0,
                    supported_formats: vec!["GGUF".to_string(), "ONNX".to_string()],
                },
                current_load: 45.2,
                last_seen: Utc::now(),
                version: "0.1.0".to_string(),
            });
        }

        // Load deployments
        {
            let mut deployments = self.state.deployments.write().await;
            deployments.push(DeploymentInfo {
                id: "deploy-001".to_string(),
                model_id: "llama-7b".to_string(),
                environment: "production".to_string(),
                status: DeploymentStatus::Running,
                replicas: 3,
                target_replicas: 3,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                health_checks: vec![HealthCheck {
                    name: "HTTP Health".to_string(),
                    status: HealthStatus::Healthy,
                    last_check: Utc::now(),
                    message: None,
                }],
            });
        }

        Ok(())
    }

    pub async fn start_background_tasks(&self) -> Result<()> {
        info!("Starting dashboard background tasks");

        // Start metrics collection
        let state = self.state.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(
                state.config.realtime.update_frequency_ms,
            ));

            loop {
                interval.tick().await;
                if let Err(e) = update_system_metrics(&state).await {
                    warn!("Failed to update system metrics: {}", e);
                }
            }
        });

        // Start real-time notifications
        let state = self.state.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));

            loop {
                interval.tick().await;
                if let Err(e) = check_system_alerts(&state).await {
                    warn!("Failed to check system alerts: {}", e);
                }
            }
        });

        Ok(())
    }
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            timestamp: Utc::now(),
            cpu_usage: 0.0,
            memory_usage: 0.0,
            gpu_usage: None,
            disk_usage: 0.0,
            network_io: NetworkIO {
                bytes_sent: 0,
                bytes_received: 0,
                packets_sent: 0,
                packets_received: 0,
            },
            inference_stats: InferenceStats {
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                average_latency_ms: 0.0,
                requests_per_second: 0.0,
                tokens_per_second: 0.0,
            },
            model_stats: ModelStats {
                active_models: 0,
                total_models: 0,
                models_loading: 0,
                models_deployed: 0,
                total_parameters: 0,
                total_size_gb: 0.0,
            },
        }
    }
}

// Dashboard page handlers
async fn dashboard_index(State(state): State<DashboardState>) -> impl IntoResponse {
    Html(generate_dashboard_html(&state.config, "dashboard").await)
}

async fn models_page(State(state): State<DashboardState>) -> impl IntoResponse {
    Html(generate_dashboard_html(&state.config, "models").await)
}

async fn metrics_page(State(state): State<DashboardState>) -> impl IntoResponse {
    Html(generate_dashboard_html(&state.config, "metrics").await)
}

async fn nodes_page(State(state): State<DashboardState>) -> impl IntoResponse {
    Html(generate_dashboard_html(&state.config, "nodes").await)
}

async fn deployments_page(State(state): State<DashboardState>) -> impl IntoResponse {
    Html(generate_dashboard_html(&state.config, "deployments").await)
}

async fn marketplace_page(State(state): State<DashboardState>) -> impl IntoResponse {
    Html(generate_dashboard_html(&state.config, "marketplace").await)
}

async fn settings_page(State(state): State<DashboardState>) -> impl IntoResponse {
    Html(generate_dashboard_html(&state.config, "settings").await)
}

// API handlers
async fn api_list_models(State(state): State<DashboardState>) -> impl IntoResponse {
    let models = state.models.read().await;
    Json(models.clone())
}

async fn api_get_model(
    State(state): State<DashboardState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let models = state.models.read().await;
    if let Some(model) = models.iter().find(|m| m.id == id) {
        Json(serde_json::json!(model.clone()))
    } else {
        Json(serde_json::json!({"error": "Model not found"}))
    }
}

async fn api_system_metrics(State(state): State<DashboardState>) -> impl IntoResponse {
    let metrics = state.metrics.read().await;
    Json(metrics.clone())
}

async fn api_list_nodes(State(state): State<DashboardState>) -> impl IntoResponse {
    let nodes = state.nodes.read().await;
    Json(nodes.clone())
}

async fn api_list_deployments(State(state): State<DashboardState>) -> impl IntoResponse {
    let deployments = state.deployments.read().await;
    Json(deployments.clone())
}

async fn api_health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": Utc::now(),
        "version": std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.1.0".to_string())
    }))
}

// Model management API handlers
async fn api_create_model(
    State(state): State<DashboardState>,
    Json(request): Json<CreateModelRequest>,
) -> impl IntoResponse {
    // Validate input
    if request.name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Model name cannot be empty",
                "details": null
            })),
        );
    }

    // Check if model with same name already exists
    {
        let models = state.models.read().await;
        if models.iter().any(|m| m.name == request.name) {
            return (
                StatusCode::CONFLICT,
                Json(serde_json::json!({
                    "error": "Model with this name already exists",
                    "details": format!("Model '{}' already exists", request.name)
                })),
            );
        }
    }

    // Create new model
    let model = ModelInfo {
        id: Uuid::new_v4().to_string(),
        name: request.name.clone(),
        version: request.version,
        format: request.format,
        size_mb: 0.0, // Will be updated when actual model file is processed
        accuracy: None,
        status: ModelStatus::Available,
        created_at: Utc::now(),
        last_used: None,
        usage_count: 0,
        tags: request.tags,
        description: request.description,
    };

    // Add to models list
    {
        let mut models = state.models.write().await;
        models.push(model.clone());
    }

    // Send notification
    let _ = state.notifications.send(NotificationMessage {
        id: Uuid::new_v4().to_string(),
        level: NotificationLevel::Success,
        title: "Model Created".to_string(),
        message: format!("Model '{}' has been created successfully", model.name),
        timestamp: Utc::now(),
        category: "models".to_string(),
        actions: vec![],
    });

    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "data": model,
            "message": "Model created successfully"
        })),
    )
}
async fn api_update_model(
    State(state): State<DashboardState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateModelRequest>,
) -> impl IntoResponse {
    let mut models = state.models.write().await;

    if let Some(model) = models.iter_mut().find(|m| m.id == id) {
        // Update fields if provided
        if let Some(name) = request.name {
            model.name = name;
        }
        if let Some(version) = request.version {
            model.version = version;
        }
        if let Some(description) = request.description {
            model.description = description;
        }
        if let Some(tags) = request.tags {
            model.tags = tags;
        }

        let updated_model = model.clone();
        drop(models); // Release the lock

        // Send notification
        let _ = state.notifications.send(NotificationMessage {
            id: Uuid::new_v4().to_string(),
            level: NotificationLevel::Primary,
            title: "Model Updated".to_string(),
            message: format!("Model '{}' has been updated", updated_model.name),
            timestamp: Utc::now(),
            category: "models".to_string(),
            actions: vec![],
        });

        (
            StatusCode::OK,
            Json(serde_json::json!({
                "data": updated_model,
                "message": "Model updated successfully"
            })),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Model not found",
                "details": format!("Model with ID '{}' does not exist", id)
            })),
        )
    }
}
async fn api_delete_model(
    State(state): State<DashboardState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut models = state.models.write().await;

    if let Some(pos) = models.iter().position(|m| m.id == id) {
        let model = models.remove(pos);
        drop(models); // Release the lock

        // Check if model is being used in deployments
        {
            let deployments = state.deployments.read().await;
            let active_deployments = deployments.iter().filter(|d| d.model_id == id).count();
            if active_deployments > 0 {
                return (
                    StatusCode::CONFLICT,
                    Json(serde_json::json!({
                        "error": "Cannot delete model",
                        "details": format!("Model is currently used in {} active deployment(s)", active_deployments)
                    })),
                );
            }
        }

        // Send notification
        let _ = state.notifications.send(NotificationMessage {
            id: Uuid::new_v4().to_string(),
            level: NotificationLevel::Warning,
            title: "Model Deleted".to_string(),
            message: format!("Model '{}' has been deleted", model.name),
            timestamp: Utc::now(),
            category: "models".to_string(),
            actions: vec![],
        });

        (
            StatusCode::OK,
            Json(serde_json::json!({
                "message": "Model deleted successfully",
                "deleted_model": {
                    "id": model.id,
                    "name": model.name
                }
            })),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Model not found",
                "details": format!("Model with ID '{}' does not exist", id)
            })),
        )
    }
}
async fn api_deploy_model(
    State(state): State<DashboardState>,
    Path(id): Path<String>,
    Json(request): Json<DeployModelRequest>,
) -> impl IntoResponse {
    // Check if model exists
    let model_exists = {
        let models = state.models.read().await;
        models.iter().any(|m| m.id == id)
    };

    if !model_exists {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Model not found",
                "details": format!("Model with ID '{}' does not exist", id)
            })),
        );
    }

    // Create new deployment
    let deployment = DeploymentInfo {
        id: Uuid::new_v4().to_string(),
        model_id: id.clone(),
        environment: request.environment.clone(),
        status: DeploymentStatus::Deploying,
        replicas: 0,
        target_replicas: request.replicas,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        health_checks: vec![],
    };

    // Add to deployments list
    {
        let mut deployments = state.deployments.write().await;
        deployments.push(deployment.clone());
    }

    // Update model status to deployed
    {
        let mut models = state.models.write().await;
        if let Some(model) = models.iter_mut().find(|m| m.id == id) {
            model.status = ModelStatus::Deployed;
        }
    }

    // Send notification
    let _ = state.notifications.send(NotificationMessage {
        id: Uuid::new_v4().to_string(),
        level: NotificationLevel::Success,
        title: "Model Deployment Started".to_string(),
        message: format!("Model deployment '{}' has been initiated", deployment.id),
        timestamp: Utc::now(),
        category: "deployments".to_string(),
        actions: vec![],
    });

    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "data": deployment,
            "message": "Model deployment initiated successfully"
        })),
    )
}
async fn api_model_metrics(
    State(state): State<DashboardState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // Check if model exists
    let model_exists = {
        let models = state.models.read().await;
        models.iter().any(|m| m.id == id)
    };

    if !model_exists {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Model not found",
                "details": format!("Model with ID '{}' does not exist", id)
            })),
        );
    }

    // Generate model-specific metrics
    let model_metrics = serde_json::json!({
        "model_id": id,
        "timestamp": Utc::now(),
        "inference_metrics": {
            "total_requests": 1245,
            "successful_requests": 1198,
            "failed_requests": 47,
            "average_latency_ms": 245.6,
            "p95_latency_ms": 478.2,
            "p99_latency_ms": 892.1,
            "requests_per_second": 12.4,
            "tokens_per_second": 85.7
        },
        "resource_usage": {
            "cpu_usage_percent": 34.2,
            "memory_usage_mb": 2048.5,
            "gpu_usage_percent": 78.9,
            "gpu_memory_usage_mb": 4096.0
        },
        "error_breakdown": {
            "timeout_errors": 23,
            "validation_errors": 15,
            "system_errors": 9
        },
        "usage_patterns": {
            "peak_hours": ["09:00-11:00", "14:00-16:00"],
            "avg_request_size_tokens": 156,
            "avg_response_size_tokens": 342
        }
    });

    (StatusCode::OK, Json(model_metrics))
}
async fn api_metrics_history(
    State(_state): State<DashboardState>,
    Query(query): Query<MetricsHistoryQuery>,
) -> impl IntoResponse {
    let start_time = query
        .start_time
        .unwrap_or_else(|| Utc::now() - chrono::Duration::hours(24));
    let end_time = query.end_time.unwrap_or_else(Utc::now);
    let interval = query.interval.unwrap_or_else(|| "5m".to_string());

    // Validate time range
    if start_time >= end_time {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid time range",
                "details": "Start time must be before end time"
            })),
        );
    }

    // Generate historical metrics data
    let mut historical_data = Vec::new();
    let mut current_time = start_time;
    let step = match interval.as_str() {
        "1m" => chrono::Duration::minutes(1),
        "5m" => chrono::Duration::minutes(5),
        "1h" => chrono::Duration::hours(1),
        "1d" => chrono::Duration::days(1),
        _ => chrono::Duration::minutes(5), // Default to 5 minutes
    };

    while current_time < end_time {
        // Simulate realistic metrics data
        let base_cpu = 45.0;
        let cpu_variation = (current_time.timestamp() as f32 * 0.001).sin() * 20.0;
        let cpu_usage = (base_cpu + cpu_variation).max(10.0).min(90.0);

        let base_memory = 65.0;
        let memory_variation = (current_time.timestamp() as f32 * 0.002).cos() * 15.0;
        let memory_usage = (base_memory + memory_variation).max(20.0).min(95.0);

        historical_data.push(serde_json::json!({
            "timestamp": current_time,
            "cpu_usage": cpu_usage,
            "memory_usage": memory_usage,
            "gpu_usage": if current_time.timestamp() % 3 == 0 { Some(cpu_usage * 1.2) } else { None },
            "requests_per_second": (cpu_usage / 4.0).max(1.0),
            "average_latency_ms": (200.0 + (100.0 - cpu_usage) * 2.0).max(50.0),
            "active_connections": ((cpu_usage / 10.0) as u32).max(1)
        }));

        current_time += step;
    }

    let response = serde_json::json!({
        "start_time": start_time,
        "end_time": end_time,
        "interval": interval,
        "data_points": historical_data.len(),
        "metrics": historical_data
    });

    (StatusCode::OK, Json(response))
}
async fn api_export_metrics(
    State(state): State<DashboardState>,
    Query(query): Query<MetricsExportQuery>,
) -> impl IntoResponse {
    let format = query.format.unwrap_or_else(|| "json".to_string());
    let start_time = query
        .start_time
        .unwrap_or_else(|| Utc::now() - chrono::Duration::hours(24));
    let end_time = query.end_time.unwrap_or_else(Utc::now);

    let metrics = state.metrics.read().await;

    match format.as_str() {
        "json" => {
            let export_data = serde_json::json!({
                "export_info": {
                    "format": "json",
                    "generated_at": Utc::now(),
                    "start_time": start_time,
                    "end_time": end_time,
                    "version": std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.1.0".to_string())
                },
                "current_metrics": *metrics,
                "summary": {
                    "total_models": 5,
                    "total_deployments": 3,
                    "total_requests_24h": 15420,
                    "average_latency_24h": 234.5,
                    "success_rate_24h": 0.962
                }
            });
            (StatusCode::OK, Json(export_data))
        }
        "csv" => {
            let csv_data = format!(
                "timestamp,cpu_usage,memory_usage,gpu_usage,disk_usage,requests_per_second,avg_latency_ms\n{},{},{},{},{},{},{}\n",
                Utc::now(),
                metrics.cpu_usage,
                metrics.memory_usage,
                metrics.gpu_usage.unwrap_or(0.0),
                metrics.disk_usage,
                metrics.inference_stats.requests_per_second,
                metrics.inference_stats.average_latency_ms
            );
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "format": "csv",
                    "data": csv_data,
                    "filename": format!("inferno_metrics_{}.csv", Utc::now().format("%Y%m%d_%H%M%S"))
                })),
            )
        }
        "prometheus" => {
            let prometheus_data = format!(
                "# HELP inferno_cpu_usage CPU usage percentage\n# TYPE inferno_cpu_usage gauge\ninferno_cpu_usage {}\n# HELP inferno_memory_usage Memory usage percentage\n# TYPE inferno_memory_usage gauge\ninferno_memory_usage {}\n# HELP inferno_requests_per_second Requests per second\n# TYPE inferno_requests_per_second gauge\ninferno_requests_per_second {}\n",
                metrics.cpu_usage,
                metrics.memory_usage,
                metrics.inference_stats.requests_per_second
            );
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "format": "prometheus",
                    "data": prometheus_data,
                    "content_type": "text/plain; version=0.0.4"
                })),
            )
        }
        _ => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Unsupported export format",
                "details": format!("Format '{}' is not supported. Use: json, csv, prometheus", format)
            })),
        ),
    }
}
async fn api_get_node(
    State(state): State<DashboardState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let nodes = state.nodes.read().await;

    if let Some(node) = nodes.iter().find(|n| n.id == id) {
        let detailed_node = serde_json::json!({
            "node": node,
            "detailed_metrics": {
                "uptime_seconds": 86400, // 24 hours
                "last_heartbeat": Utc::now() - chrono::Duration::seconds(5),
                "active_models": 2,
                "queued_requests": 15,
                "processed_requests_24h": 5420,
                "error_count_24h": 23,
                "network_latency_ms": 12.4,
                "disk_io_mb_per_sec": 45.2
            },
            "health_status": {
                "overall": "healthy",
                "checks": [
                    {
                        "name": "CPU Health",
                        "status": "healthy",
                        "value": node.current_load,
                        "threshold": 80.0
                    },
                    {
                        "name": "Memory Health",
                        "status": "healthy",
                        "usage_percent": 65.2,
                        "available_gb": node.capabilities.memory_gb * 0.35
                    },
                    {
                        "name": "Storage Health",
                        "status": "healthy",
                        "usage_percent": 42.1,
                        "available_gb": node.capabilities.storage_gb * 0.58
                    }
                ]
            }
        });

        (StatusCode::OK, Json(detailed_node))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Node not found",
                "details": format!("Node with ID '{}' does not exist", id)
            })),
        )
    }
}
async fn api_node_status(
    State(state): State<DashboardState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let nodes = state.nodes.read().await;

    if let Some(node) = nodes.iter().find(|n| n.id == id) {
        // Simulate real-time status check
        let status_response = serde_json::json!({
            "node_id": node.id,
            "status": node.status,
            "last_updated": Utc::now(),
            "connectivity": {
                "reachable": true,
                "response_time_ms": 23.4,
                "last_successful_ping": Utc::now() - chrono::Duration::seconds(2)
            },
            "resource_status": {
                "cpu": {
                    "usage_percent": node.current_load,
                    "status": if node.current_load > 80.0 { "warning" } else { "healthy" },
                    "cores_available": node.capabilities.cpu_cores
                },
                "memory": {
                    "usage_percent": 68.4,
                    "status": "healthy",
                    "total_gb": node.capabilities.memory_gb,
                    "available_gb": node.capabilities.memory_gb * 0.316
                },
                "gpu": {
                    "count": node.capabilities.gpu_count,
                    "usage_percent": if node.capabilities.gpu_count > 0 { Some(45.2) } else { None },
                    "status": if node.capabilities.gpu_count > 0 { "healthy" } else { "not_available" }
                },
                "storage": {
                    "usage_percent": 35.8,
                    "status": "healthy",
                    "total_gb": node.capabilities.storage_gb,
                    "available_gb": node.capabilities.storage_gb * 0.642
                }
            },
            "services": {
                "inference_engine": {
                    "status": "running",
                    "port": 8090,
                    "version": node.version.clone()
                },
                "monitoring_agent": {
                    "status": "running",
                    "last_report": Utc::now() - chrono::Duration::seconds(10)
                },
                "model_loader": {
                    "status": "idle",
                    "active_loads": 0
                }
            },
            "performance": {
                "requests_per_second": 15.4,
                "average_latency_ms": 145.6,
                "error_rate_percent": 0.8,
                "queue_depth": 3
            }
        });

        (StatusCode::OK, Json(status_response))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Node not found",
                "details": format!("Node with ID '{}' does not exist", id)
            })),
        )
    }
}
async fn api_create_deployment(
    State(state): State<DashboardState>,
    Json(request): Json<CreateDeploymentRequest>,
) -> impl IntoResponse {
    // Validate that the model exists
    let model_exists = {
        let models = state.models.read().await;
        models.iter().any(|m| m.id == request.model_id)
    };

    if !model_exists {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid model ID",
                "details": format!("Model with ID '{}' does not exist", request.model_id)
            })),
        );
    }

    // Validate replicas count
    if request.replicas == 0 || request.replicas > 100 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid replica count",
                "details": "Replica count must be between 1 and 100"
            })),
        );
    }

    // Create new deployment
    let deployment = DeploymentInfo {
        id: Uuid::new_v4().to_string(),
        model_id: request.model_id.clone(),
        environment: request.environment.clone(),
        status: DeploymentStatus::Pending,
        replicas: 0, // Starting with 0, will scale up
        target_replicas: request.replicas,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        health_checks: vec![
            HealthCheck {
                name: "HTTP Health Check".to_string(),
                status: HealthStatus::Unknown,
                last_check: Utc::now(),
                message: Some("Deployment initializing".to_string()),
            },
            HealthCheck {
                name: "Model Ready Check".to_string(),
                status: HealthStatus::Unknown,
                last_check: Utc::now(),
                message: Some("Waiting for model to load".to_string()),
            },
        ],
    };

    // Add to deployments list
    {
        let mut deployments = state.deployments.write().await;
        deployments.push(deployment.clone());
    }

    // Send notification
    let _ = state.notifications.send(NotificationMessage {
        id: Uuid::new_v4().to_string(),
        level: NotificationLevel::Primary,
        title: "Deployment Created".to_string(),
        message: format!(
            "Deployment '{}' created for environment '{}'",
            deployment.id, request.environment
        ),
        timestamp: Utc::now(),
        category: "deployments".to_string(),
        actions: vec![],
    });

    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "data": deployment,
            "message": "Deployment created successfully"
        })),
    )
}
async fn api_get_deployment(
    State(state): State<DashboardState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let deployments = state.deployments.read().await;

    if let Some(deployment) = deployments.iter().find(|d| d.id == id) {
        let detailed_deployment = serde_json::json!({
            "deployment": deployment,
            "runtime_info": {
                "uptime_seconds": 3600, // 1 hour
                "total_requests": 2450,
                "successful_requests": 2398,
                "failed_requests": 52,
                "average_response_time_ms": 234.5,
                "p95_response_time_ms": 456.2,
                "current_rps": 12.4,
                "peak_rps": 28.7
            },
            "resource_usage": {
                "cpu_usage_percent": 45.2,
                "memory_usage_mb": 2048.5,
                "network_in_mbps": 12.4,
                "network_out_mbps": 8.7
            },
            "instances": [
                {
                    "id": "instance-1",
                    "node_id": "node-001",
                    "status": "running",
                    "health": "healthy",
                    "started_at": deployment.created_at,
                    "requests_handled": 1225
                },
                {
                    "id": "instance-2",
                    "node_id": "node-001",
                    "status": "running",
                    "health": "healthy",
                    "started_at": deployment.created_at + chrono::Duration::minutes(5),
                    "requests_handled": 1173
                }
            ],
            "recent_events": [
                {
                    "timestamp": Utc::now() - chrono::Duration::minutes(5),
                    "type": "scaling",
                    "message": "Scaled to 2 replicas"
                },
                {
                    "timestamp": deployment.created_at,
                    "type": "creation",
                    "message": "Deployment created"
                }
            ]
        });

        (StatusCode::OK, Json(detailed_deployment))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Deployment not found",
                "details": format!("Deployment with ID '{}' does not exist", id)
            })),
        )
    }
}
async fn api_update_deployment(
    State(state): State<DashboardState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateDeploymentRequest>,
) -> impl IntoResponse {
    let mut deployments = state.deployments.write().await;

    if let Some(deployment) = deployments.iter_mut().find(|d| d.id == id) {
        let mut updated = false;
        let mut changes = Vec::new();

        // Update environment if provided
        if let Some(environment) = request.environment {
            if deployment.environment != environment {
                deployment.environment = environment.clone();
                changes.push(format!("Environment changed to '{}'", environment));
                updated = true;
            }
        }

        // Update target replicas if provided
        if let Some(target_replicas) = request.target_replicas {
            if target_replicas == 0 || target_replicas > 100 {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": "Invalid replica count",
                        "details": "Replica count must be between 1 and 100"
                    })),
                );
            }

            if deployment.target_replicas != target_replicas {
                let old_replicas = deployment.target_replicas;
                deployment.target_replicas = target_replicas;
                deployment.status = DeploymentStatus::Scaling;
                changes.push(format!(
                    "Target replicas changed from {} to {}",
                    old_replicas, target_replicas
                ));
                updated = true;
            }
        }

        if updated {
            deployment.updated_at = Utc::now();
            let updated_deployment = deployment.clone();
            drop(deployments); // Release the lock

            // Send notification
            let _ = state.notifications.send(NotificationMessage {
                id: Uuid::new_v4().to_string(),
                level: NotificationLevel::Primary,
                title: "Deployment Updated".to_string(),
                message: format!(
                    "Deployment '{}' updated: {}",
                    updated_deployment.id,
                    changes.join(", ")
                ),
                timestamp: Utc::now(),
                category: "deployments".to_string(),
                actions: vec![],
            });

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "data": updated_deployment,
                    "message": "Deployment updated successfully"
                })),
            )
        } else {
            let deployment_copy = deployment.clone();
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "data": deployment_copy,
                    "message": "No changes made to deployment"
                })),
            )
        }
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Deployment not found",
                "details": format!("Deployment with ID '{}' does not exist", id)
            })),
        )
    }
}
async fn api_delete_deployment(
    State(state): State<DashboardState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut deployments = state.deployments.write().await;

    if let Some(pos) = deployments.iter().position(|d| d.id == id) {
        let mut deployment = deployments.remove(pos);
        deployment.status = DeploymentStatus::Terminated;
        let deployment_info = deployment.clone();
        drop(deployments); // Release the lock

        // Send notification
        let _ = state.notifications.send(NotificationMessage {
            id: Uuid::new_v4().to_string(),
            level: NotificationLevel::Warning,
            title: "Deployment Deleted".to_string(),
            message: format!(
                "Deployment '{}' has been terminated and deleted",
                deployment_info.id
            ),
            timestamp: Utc::now(),
            category: "deployments".to_string(),
            actions: vec![],
        });

        (
            StatusCode::OK,
            Json(serde_json::json!({
                "message": "Deployment deleted successfully",
                "deleted_deployment": {
                    "id": deployment_info.id,
                    "model_id": deployment_info.model_id,
                    "environment": deployment_info.environment,
                    "terminated_at": Utc::now()
                }
            })),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Deployment not found",
                "details": format!("Deployment with ID '{}' does not exist", id)
            })),
        )
    }
}
async fn api_scale_deployment(
    State(state): State<DashboardState>,
    Path(id): Path<String>,
    Json(request): Json<ScaleDeploymentRequest>,
) -> impl IntoResponse {
    // Validate replica count
    if request.replicas == 0 || request.replicas > 100 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid replica count",
                "details": "Replica count must be between 1 and 100"
            })),
        );
    }

    let mut deployments = state.deployments.write().await;

    if let Some(deployment) = deployments.iter_mut().find(|d| d.id == id) {
        let old_replicas = deployment.target_replicas;
        let new_replicas = request.replicas;

        if old_replicas == new_replicas {
            let deployment_copy = deployment.clone();
            return (
                StatusCode::OK,
                Json(serde_json::json!({
                    "data": deployment_copy,
                    "message": "Deployment is already at the requested replica count"
                })),
            );
        }

        // Update deployment
        deployment.target_replicas = new_replicas;
        deployment.status = DeploymentStatus::Scaling;
        deployment.updated_at = Utc::now();

        // Simulate gradual scaling
        if new_replicas > old_replicas {
            // Scaling up: set current replicas to halfway point
            deployment.replicas = old_replicas + ((new_replicas - old_replicas) / 2);
        } else {
            // Scaling down: set current replicas to halfway point
            deployment.replicas = old_replicas - ((old_replicas - new_replicas) / 2);
        }

        let scaled_deployment = deployment.clone();
        drop(deployments); // Release the lock

        // Send notification
        let scale_action = if new_replicas > old_replicas {
            "up"
        } else {
            "down"
        };
        let _ = state.notifications.send(NotificationMessage {
            id: Uuid::new_v4().to_string(),
            level: NotificationLevel::Primary,
            title: "Deployment Scaling".to_string(),
            message: format!(
                "Deployment '{}' is scaling {} from {} to {} replicas",
                scaled_deployment.id, scale_action, old_replicas, new_replicas
            ),
            timestamp: Utc::now(),
            category: "deployments".to_string(),
            actions: vec![],
        });

        let response_data = serde_json::json!({
            "deployment": scaled_deployment,
            "scaling_info": {
                "previous_replicas": old_replicas,
                "target_replicas": new_replicas,
                "current_replicas": scaled_deployment.replicas,
                "scaling_direction": scale_action,
                "estimated_completion_time": Utc::now() + chrono::Duration::minutes(5)
            }
        });

        (
            StatusCode::OK,
            Json(serde_json::json!({
                "data": response_data,
                "message": format!("Deployment scaling initiated: {} -> {} replicas", old_replicas, new_replicas)
            })),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Deployment not found",
                "details": format!("Deployment with ID '{}' does not exist", id)
            })),
        )
    }
}
async fn api_deployment_logs(
    State(state): State<DashboardState>,
    Path(deployment_id): Path<String>,
    Query(params): Query<DeploymentLogsQuery>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    // Verify authentication
    let _auth = match extract_auth_context(&state, &headers).await {
        Ok(auth) => auth,
        Err(error_response) => return error_response.into_response(),
    };

    // Validate deployment exists (mock validation)
    if deployment_id.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Invalid deployment ID"})),
        )
            .into_response();
    }

    // Parse query parameters
    let level = params.level.unwrap_or_else(|| "info".to_string());
    let lines = params.lines.unwrap_or(100).min(1000); // Cap at 1000 lines
    let since = params
        .since
        .unwrap_or_else(|| Utc::now() - chrono::Duration::hours(24));
    let follow = params.follow.unwrap_or(false);

    // Mock deployment logs - in real implementation, this would fetch from logging system
    let log_entries = generate_mock_deployment_logs(&deployment_id, &level, lines, since);

    let response = serde_json::json!({
        "deployment_id": deployment_id,
        "total_lines": log_entries.len(),
        "requested_lines": lines,
        "level_filter": level,
        "since": since,
        "follow": follow,
        "logs": log_entries,
        "metadata": {
            "container_id": format!("container-{}", deployment_id),
            "pod_name": format!("inferno-{}", deployment_id),
            "namespace": "default",
            "node": "worker-node-01",
            "log_source": "container",
            "last_updated": Utc::now()
        }
    });

    (StatusCode::OK, Json(response)).into_response()
}

#[derive(serde::Deserialize)]
struct DeploymentLogsQuery {
    #[serde(default)]
    level: Option<String>,
    #[serde(default)]
    lines: Option<usize>,
    #[serde(default)]
    since: Option<chrono::DateTime<Utc>>,
    #[serde(default)]
    follow: Option<bool>,
}

fn generate_mock_deployment_logs(
    deployment_id: &str,
    level: &str,
    lines: usize,
    since: chrono::DateTime<Utc>,
) -> Vec<serde_json::Value> {
    let mut logs = Vec::new();
    let base_time = since;

    // Generate realistic deployment logs
    let log_templates = vec![
        (
            "info",
            "Model loading started for deployment {}",
            "model_loading",
        ),
        ("info", "Backend initialization completed", "backend_init"),
        (
            "info",
            "Health check endpoint registered on port 8080",
            "health_check",
        ),
        (
            "info",
            "Processing inference request from user {}",
            "inference_request",
        ),
        (
            "debug",
            "Token processing: {} tokens/sec",
            "token_processing",
        ),
        (
            "info",
            "Model inference completed in {}ms",
            "inference_complete",
        ),
        ("warn", "High memory usage detected: {}%", "memory_warning"),
        (
            "info",
            "Scaling event triggered: current replicas {}",
            "scaling",
        ),
        ("error", "Failed to process request: {}", "request_error"),
        ("info", "Graceful shutdown initiated", "shutdown"),
    ];

    for i in 0..lines.min(200) {
        let template = &log_templates[i % log_templates.len()];
        let timestamp = base_time + chrono::Duration::minutes(i as i64);

        // Filter by log level
        if !should_include_log_level(template.0, level) {
            continue;
        }

        let message = match template.2 {
            "model_loading" => format!("Model loading started for deployment {}", deployment_id),
            "inference_request" => format!(
                "Processing inference request from user {}",
                format!("user_{}", i % 5 + 1)
            ),
            "token_processing" => format!("Token processing: {} tokens/sec", 450 + (i % 100)),
            "inference_complete" => format!("Model inference completed in {}ms", 125 + (i % 50)),
            "memory_warning" => format!("High memory usage detected: {}%", 75 + (i % 20)),
            "scaling" => format!("Scaling event triggered: current replicas {}", 2 + (i % 3)),
            "request_error" => format!("Failed to process request: {}", "timeout after 30s"),
            _ => template.1.to_string(),
        };

        logs.push(serde_json::json!({
            "timestamp": timestamp,
            "level": template.0.to_uppercase(),
            "message": message,
            "source": "inferno-worker",
            "deployment_id": deployment_id,
            "request_id": format!("req-{:04x}", i),
            "thread": format!("worker-{}", i % 4 + 1),
            "module": template.2
        }));
    }

    logs
}

fn should_include_log_level(log_level: &str, filter_level: &str) -> bool {
    let level_priority = |level: &str| match level.to_lowercase().as_str() {
        "trace" => 0,
        "debug" => 1,
        "info" => 2,
        "warn" => 3,
        "error" => 4,
        _ => 2, // default to info
    };

    level_priority(log_level) >= level_priority(filter_level)
}
async fn api_marketplace_search(
    State(state): State<DashboardState>,
    Query(params): Query<MarketplaceSearchRequest>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    // Verify authentication (marketplace browsing requires login)
    let _auth = match extract_auth_context(&state, &headers).await {
        Ok(auth) => auth,
        Err(error_response) => return error_response.into_response(),
    };

    // Convert dashboard search request to marketplace search filters
    let query = params.query.as_deref().unwrap_or("");
    let page = params.offset.unwrap_or(0) / params.limit.unwrap_or(20);
    let per_page = params.limit.unwrap_or(20);

    // Map category from string to ModelCategory enum if needed
    let category_filter =
        params
            .category
            .as_ref()
            .and_then(|cat| match cat.to_lowercase().as_str() {
                "llm" => Some(crate::marketplace::ModelCategory::Language),
                "embedding" => Some(crate::marketplace::ModelCategory::Embedding),
                "vision" => Some(crate::marketplace::ModelCategory::Vision),
                "audio" => Some(crate::marketplace::ModelCategory::Audio),
                _ => None,
            });

    let filters = Some(crate::marketplace::SearchFilters {
        category: category_filter,
        publisher: None,
        license: None,
        min_rating: None,
        max_size_gb: params.size_limit_gb,
        tags: vec![],
        frameworks: params
            .format
            .as_ref()
            .map(|f| vec![f.clone()])
            .unwrap_or_default(),
        languages: vec![],
        platforms: vec![],
        free_only: false,
        verified_only: false,
    });

    // Query the real marketplace
    let search_result = match state
        .marketplace
        .search_models(query, filters, page, per_page)
        .await
    {
        Ok(result) => result,
        Err(e) => {
            warn!("Marketplace search failed: {}", e);
            // Return empty result on error
            crate::marketplace::SearchResult {
                models: vec![],
                total_count: 0,
                page,
                per_page,
                total_pages: 0,
                facets: crate::marketplace::SearchFacets::default(),
            }
        }
    };

    // Convert marketplace ModelListing to dashboard MarketplaceModel
    let mut marketplace_models = Vec::new();
    for model in search_result.models {
        marketplace_models.push(MarketplaceModel {
            id: model.id,
            name: model.name,
            description: model.description,
            author: model.publisher,
            version: model.version,
            format: "gguf".to_string(), // Default format, could be derived from model metadata
            size_gb: model.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0),
            category: format!("{:?}", model.category).to_lowercase(),
            license: model.license,
            downloads: model.downloads,
            rating: model.rating.unwrap_or(0.0),
            tags: model.tags,
            created_at: model.published_at,
            updated_at: model.updated_at,
            download_url: model.download_url,
            homepage_url: None,      // Could be derived from model metadata
            documentation_url: None, // Could be derived from model metadata
        });
    }

    // Create response using actual search results
    let response = MarketplaceSearchResponse {
        models: marketplace_models,
        total: search_result.total_count,
        has_more: search_result.page < search_result.total_pages - 1,
    };

    (StatusCode::OK, Json(response)).into_response()
}

async fn api_marketplace_featured(
    State(state): State<DashboardState>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    // Verify authentication
    let _auth = match extract_auth_context(&state, &headers).await {
        Ok(auth) => auth,
        Err(error_response) => return error_response.into_response(),
    };

    // In a real implementation, this would query featured models from the marketplace
    // For demo purposes, we'll return curated lists of mock models

    let featured_models = vec![
        MarketplaceModel {
            id: "llama-2-7b-chat".to_string(),
            name: "Llama 2 7B Chat".to_string(),
            description: "Editor's choice: Best general-purpose chat model for most use cases"
                .to_string(),
            author: "Meta".to_string(),
            version: "1.0.0".to_string(),
            format: "gguf".to_string(),
            size_gb: 4.1,
            category: "llm".to_string(),
            license: "Custom".to_string(),
            downloads: 1250000,
            rating: 4.8,
            tags: vec![
                "featured".to_string(),
                "chat".to_string(),
                "recommended".to_string(),
            ],
            created_at: Utc::now() - chrono::Duration::days(180),
            updated_at: Utc::now() - chrono::Duration::days(30),
            download_url: "https://huggingface.co/meta-llama/Llama-2-7b-chat-hf".to_string(),
            homepage_url: Some("https://ai.meta.com/llama/".to_string()),
            documentation_url: Some("https://github.com/facebookresearch/llama".to_string()),
        },
        MarketplaceModel {
            id: "bert-base-uncased".to_string(),
            name: "BERT Base Uncased".to_string(),
            description: "Staff pick: Most reliable embeddings model for text understanding"
                .to_string(),
            author: "Google".to_string(),
            version: "1.0.0".to_string(),
            format: "onnx".to_string(),
            size_gb: 0.42,
            category: "embedding".to_string(),
            license: "Apache 2.0".to_string(),
            downloads: 2100000,
            rating: 4.6,
            tags: vec![
                "featured".to_string(),
                "embeddings".to_string(),
                "reliable".to_string(),
            ],
            created_at: Utc::now() - chrono::Duration::days(900),
            updated_at: Utc::now() - chrono::Duration::days(60),
            download_url: "https://huggingface.co/bert-base-uncased".to_string(),
            homepage_url: Some(
                "https://ai.googleblog.com/2018/11/open-sourcing-bert-state-of-art-pre.html"
                    .to_string(),
            ),
            documentation_url: Some(
                "https://huggingface.co/docs/transformers/model_doc/bert".to_string(),
            ),
        },
    ];

    let trending_models = vec![
        MarketplaceModel {
            id: "whisper-large-v3".to_string(),
            name: "Whisper Large v3".to_string(),
            description: "🔥 Trending: Latest speech recognition breakthrough from OpenAI"
                .to_string(),
            author: "OpenAI".to_string(),
            version: "3.0".to_string(),
            format: "onnx".to_string(),
            size_gb: 2.9,
            category: "audio".to_string(),
            license: "MIT".to_string(),
            downloads: 750000,
            rating: 4.9,
            tags: vec![
                "trending".to_string(),
                "new".to_string(),
                "speech-to-text".to_string(),
            ],
            created_at: Utc::now() - chrono::Duration::days(60),
            updated_at: Utc::now() - chrono::Duration::days(5),
            download_url: "https://huggingface.co/openai/whisper-large-v3".to_string(),
            homepage_url: Some("https://openai.com/research/whisper".to_string()),
            documentation_url: Some("https://github.com/openai/whisper".to_string()),
        },
        MarketplaceModel {
            id: "mistral-7b-instruct".to_string(),
            name: "Mistral 7B Instruct".to_string(),
            description: "📈 Hot: Outperforming larger models on reasoning tasks".to_string(),
            author: "Mistral AI".to_string(),
            version: "0.2".to_string(),
            format: "gguf".to_string(),
            size_gb: 4.4,
            category: "llm".to_string(),
            license: "Apache 2.0".to_string(),
            downloads: 890000,
            rating: 4.7,
            tags: vec![
                "trending".to_string(),
                "reasoning".to_string(),
                "efficient".to_string(),
            ],
            created_at: Utc::now() - chrono::Duration::days(120),
            updated_at: Utc::now() - chrono::Duration::days(15),
            download_url: "https://huggingface.co/mistralai/Mistral-7B-Instruct-v0.2".to_string(),
            homepage_url: Some("https://mistral.ai/".to_string()),
            documentation_url: Some("https://docs.mistral.ai/".to_string()),
        },
    ];

    let recent_models = vec![MarketplaceModel {
        id: "phi-3-mini".to_string(),
        name: "Phi-3 Mini".to_string(),
        description: "🆕 Just released: Compact yet powerful 3.8B parameter model from Microsoft"
            .to_string(),
        author: "Microsoft".to_string(),
        version: "1.0.0".to_string(),
        format: "onnx".to_string(),
        size_gb: 2.3,
        category: "llm".to_string(),
        license: "MIT".to_string(),
        downloads: 45000,
        rating: 4.5,
        tags: vec![
            "new".to_string(),
            "compact".to_string(),
            "mobile-ready".to_string(),
        ],
        created_at: Utc::now() - chrono::Duration::days(7),
        updated_at: Utc::now() - chrono::Duration::days(2),
        download_url: "https://huggingface.co/microsoft/Phi-3-mini-4k-instruct".to_string(),
        homepage_url: Some(
            "https://azure.microsoft.com/en-us/products/ai-services/phi-3".to_string(),
        ),
        documentation_url: Some("https://github.com/microsoft/Phi-3CookBook".to_string()),
    }];

    let response = FeaturedModelsResponse {
        featured: featured_models,
        trending: trending_models,
        recent: recent_models,
    };

    (StatusCode::OK, Json(response)).into_response()
}
async fn api_marketplace_downloads(
    State(state): State<DashboardState>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    // Verify authentication
    let _auth = match extract_auth_context(&state, &headers).await {
        Ok(auth) => auth,
        Err(error_response) => return error_response.into_response(),
    };

    // Mock download statistics - in a real implementation, this would fetch from marketplace service
    let downloads = serde_json::json!({
        "recent_downloads": [
            {
                "model_id": "llama-2-7b-chat",
                "name": "Llama 2 7B Chat",
                "version": "v1.0.0",
                "downloaded_at": "2024-01-15T14:30:00Z",
                "download_count": 15420,
                "size_gb": 3.5,
                "status": "completed"
            },
            {
                "model_id": "codellama-7b-instruct",
                "name": "Code Llama 7B Instruct",
                "version": "v1.1.0",
                "downloaded_at": "2024-01-14T09:15:00Z",
                "download_count": 8930,
                "size_gb": 3.8,
                "status": "completed"
            },
            {
                "model_id": "mistral-7b-v0.1",
                "name": "Mistral 7B v0.1",
                "version": "v0.1.0",
                "downloaded_at": "2024-01-13T16:45:00Z",
                "download_count": 12750,
                "size_gb": 4.1,
                "status": "completed"
            }
        ],
        "popular_downloads": [
            {
                "model_id": "llama-2-13b-chat",
                "name": "Llama 2 13B Chat",
                "total_downloads": 45230,
                "growth_rate": 23.5,
                "size_gb": 7.2
            },
            {
                "model_id": "vicuna-13b-v1.3",
                "name": "Vicuna 13B v1.3",
                "total_downloads": 38920,
                "growth_rate": 18.2,
                "size_gb": 6.8
            }
        ],
        "download_stats": {
            "total_downloads_today": 1247,
            "total_downloads_week": 8934,
            "total_downloads_month": 34521,
            "total_bandwidth_gb": 156.8,
            "unique_users": 3421,
            "peak_download_hour": "14:00-15:00"
        },
        "top_categories": [
            {"name": "Chat Models", "downloads": 18743, "percentage": 42.1},
            {"name": "Code Generation", "downloads": 12456, "percentage": 28.0},
            {"name": "Text Completion", "downloads": 8921, "percentage": 20.0},
            {"name": "Embeddings", "downloads": 4356, "percentage": 9.9}
        ]
    });

    (StatusCode::OK, Json(downloads)).into_response()
}
async fn api_system_info(
    State(state): State<DashboardState>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    // Verify authentication
    let _auth = match extract_auth_context(&state, &headers).await {
        Ok(auth) => auth,
        Err(error_response) => return error_response.into_response(),
    };

    // Gather real system information
    let sys = sysinfo::System::new_all();
    let cpu_count = num_cpus::get();
    let uptime = sys.uptime();

    let system_info = serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "build_date": option_env!("VERGEN_BUILD_DATE").unwrap_or("unknown"),
        "git_commit": option_env!("VERGEN_GIT_SHA").unwrap_or("unknown"),
        "platform": {
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
            "name": sys.name().unwrap_or_else(|| "Unknown".to_string()),
            "kernel_version": sys.kernel_version().unwrap_or_else(|| "Unknown".to_string()),
            "os_version": sys.os_version().unwrap_or_else(|| "Unknown".to_string()),
            "host_name": sys.host_name().unwrap_or_else(|| "Unknown".to_string())
        },
        "hardware": {
            "cpu_count": cpu_count,
            "total_memory_gb": sys.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0),
            "available_memory_gb": sys.available_memory() as f64 / (1024.0 * 1024.0 * 1024.0),
            "total_swap_gb": sys.total_swap() as f64 / (1024.0 * 1024.0 * 1024.0),
            "used_swap_gb": sys.used_swap() as f64 / (1024.0 * 1024.0 * 1024.0)
        },
        "runtime": {
            "uptime_seconds": uptime,
            "uptime_formatted": format_duration(uptime),
            "models_loaded": state.models.read().await.len(),
            "config_source": "config.toml",
            "log_level": std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string())
        },
        "features": {
            "gguf_backend": cfg!(feature = "gguf"),
            "onnx_backend": cfg!(feature = "onnx"),
            "gpu_metal": cfg!(feature = "gpu-metal"),
            "gpu_vulkan": cfg!(feature = "gpu-vulkan"),
            "tauri_app": cfg!(feature = "desktop"),
            "download_support": cfg!(feature = "download")
        },
        "endpoints": {
            "dashboard": format!("http://{}:{}", state.config.bind_address, state.config.port),
            "api": format!("http://{}:{}/api", state.config.bind_address, state.config.port),
            "websocket": format!("ws://{}:{}/ws", state.config.bind_address, state.config.port)
        }
    });

    (StatusCode::OK, Json(system_info)).into_response()
}

fn format_duration(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if days > 0 {
        format!("{}d {}h {}m {}s", days, hours, minutes, secs)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}
async fn api_get_config(
    State(state): State<DashboardState>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    // Verify authentication and admin access
    let auth = match extract_auth_context(&state, &headers).await {
        Ok(auth) => auth,
        Err(error_response) => return error_response.into_response(),
    };

    if let Err(error_response) = require_admin(&auth) {
        return error_response.into_response();
    }

    // Build configuration response from current state
    let config = &state.config;

    let response = SystemConfigResponse {
        dashboard: DashboardConfigSummary {
            enabled: config.enabled,
            port: config.port,
            bind_address: config.bind_address.clone(),
            auth_enabled: config.auth.enabled,
            theme: config.ui.theme.default_theme.clone(),
            title: config.ui.title.clone(),
        },
        security: SecurityConfigSummary {
            auth_enabled: config.auth.enabled,
            rate_limiting_enabled: config.security.rate_limit.enabled,
            max_requests_per_minute: config.security.rate_limit.requests_per_minute,
            token_expiry_hours: config.auth.session_timeout_minutes as i64 / 60, // Convert to hours
            tls_required: config.security.https_enabled,
        },
        server: ServerConfigSummary {
            max_concurrent_requests: 1000, // Default value - would come from server config
            request_timeout_seconds: 30,   // Default value - would come from server config
            enable_cors: true,             // Default value - would come from server config
        },
        features: FeatureConfigSummary {
            model_management: config.ui.features.model_management,
            metrics: config.ui.features.metrics,
            marketplace: config.ui.features.marketplace,
            deployment: config.ui.features.deployment,
            user_management: config.ui.features.user_management,
            monitoring: config.ui.features.monitoring,
        },
    };

    (StatusCode::OK, Json(response)).into_response()
}
async fn api_update_config(
    State(state): State<DashboardState>,
    headers: axum::http::HeaderMap,
    Json(request): Json<SystemConfigUpdateRequest>,
) -> impl IntoResponse {
    // Verify authentication and admin access
    let auth = match extract_auth_context(&state, &headers).await {
        Ok(auth) => auth,
        Err(error_response) => return error_response.into_response(),
    };

    if let Err(error_response) = require_admin(&auth) {
        return error_response.into_response();
    }

    // Note: In a real implementation, you would update persistent configuration storage
    // For this demo, we'll simulate config updates and return success

    // Validate inputs
    if let Some(ref dashboard) = request.dashboard {
        if let Some(port) = dashboard.port {
            if !(1024..=65535).contains(&port) {
                let error = ApiError {
                    error: "Port must be between 1024 and 65535".to_string(),
                    details: None,
                };
                return (StatusCode::BAD_REQUEST, Json(error)).into_response();
            }
        }

        if let Some(ref bind_address) = dashboard.bind_address {
            if bind_address.trim().is_empty() {
                let error = ApiError {
                    error: "Bind address cannot be empty".to_string(),
                    details: None,
                };
                return (StatusCode::BAD_REQUEST, Json(error)).into_response();
            }
        }

        if let Some(ref theme) = dashboard.theme {
            if !["light", "dark", "auto"].contains(&theme.as_str()) {
                let error = ApiError {
                    error: "Theme must be one of: light, dark, auto".to_string(),
                    details: None,
                };
                return (StatusCode::BAD_REQUEST, Json(error)).into_response();
            }
        }
    }

    if let Some(ref security) = request.security {
        if let Some(max_requests) = security.max_requests_per_minute {
            if max_requests == 0 || max_requests > 10000 {
                let error = ApiError {
                    error: "Max requests per minute must be between 1 and 10000".to_string(),
                    details: None,
                };
                return (StatusCode::BAD_REQUEST, Json(error)).into_response();
            }
        }

        if let Some(token_expiry) = security.token_expiry_hours {
            if !(1..=168).contains(&token_expiry) {
                // 1 hour to 1 week
                let error = ApiError {
                    error: "Token expiry must be between 1 and 168 hours".to_string(),
                    details: None,
                };
                return (StatusCode::BAD_REQUEST, Json(error)).into_response();
            }
        }
    }

    if let Some(ref server) = request.server {
        if let Some(max_requests) = server.max_concurrent_requests {
            if max_requests == 0 || max_requests > 100000 {
                let error = ApiError {
                    error: "Max concurrent requests must be between 1 and 100000".to_string(),
                    details: None,
                };
                return (StatusCode::BAD_REQUEST, Json(error)).into_response();
            }
        }

        if let Some(timeout) = server.request_timeout_seconds {
            if timeout == 0 || timeout > 300 {
                // 5 minutes max
                let error = ApiError {
                    error: "Request timeout must be between 1 and 300 seconds".to_string(),
                    details: None,
                };
                return (StatusCode::BAD_REQUEST, Json(error)).into_response();
            }
        }
    }

    // In a real implementation, you would:
    // 1. Update the configuration in persistent storage (file, database, etc.)
    // 2. Apply the changes to the running system
    // 3. Potentially restart services if needed

    // For now, simulate successful configuration update
    info!("Configuration updated by admin user: {}", auth.username);

    // Return the updated configuration (in a real impl, re-read from storage)
    let config = &state.config;
    let response = SystemConfigResponse {
        dashboard: DashboardConfigSummary {
            enabled: request
                .dashboard
                .as_ref()
                .and_then(|d| d.enabled)
                .unwrap_or(config.enabled),
            port: request
                .dashboard
                .as_ref()
                .and_then(|d| d.port)
                .unwrap_or(config.port),
            bind_address: request
                .dashboard
                .as_ref()
                .and_then(|d| d.bind_address.clone())
                .unwrap_or_else(|| config.bind_address.clone()),
            auth_enabled: config.auth.enabled,
            theme: request
                .dashboard
                .as_ref()
                .and_then(|d| d.theme.clone())
                .unwrap_or_else(|| config.ui.theme.default_theme.clone()),
            title: request
                .dashboard
                .as_ref()
                .and_then(|d| d.title.clone())
                .unwrap_or_else(|| config.ui.title.clone()),
        },
        security: SecurityConfigSummary {
            auth_enabled: config.auth.enabled,
            rate_limiting_enabled: request
                .security
                .as_ref()
                .and_then(|s| s.rate_limiting_enabled)
                .unwrap_or(config.security.rate_limit.enabled),
            max_requests_per_minute: request
                .security
                .as_ref()
                .and_then(|s| s.max_requests_per_minute)
                .unwrap_or(config.security.rate_limit.requests_per_minute),
            token_expiry_hours: request
                .security
                .as_ref()
                .and_then(|s| s.token_expiry_hours)
                .unwrap_or(config.auth.session_timeout_minutes as i64 / 60),
            tls_required: request
                .security
                .as_ref()
                .and_then(|s| s.tls_required)
                .unwrap_or(config.security.https_enabled),
        },
        server: ServerConfigSummary {
            max_concurrent_requests: request
                .server
                .as_ref()
                .and_then(|s| s.max_concurrent_requests)
                .unwrap_or(1000),
            request_timeout_seconds: request
                .server
                .as_ref()
                .and_then(|s| s.request_timeout_seconds)
                .unwrap_or(30),
            enable_cors: request
                .server
                .as_ref()
                .and_then(|s| s.enable_cors)
                .unwrap_or(true),
        },
        features: FeatureConfigSummary {
            model_management: request
                .features
                .as_ref()
                .and_then(|f| f.model_management)
                .unwrap_or(config.ui.features.model_management),
            metrics: request
                .features
                .as_ref()
                .and_then(|f| f.metrics)
                .unwrap_or(config.ui.features.metrics),
            marketplace: request
                .features
                .as_ref()
                .and_then(|f| f.marketplace)
                .unwrap_or(config.ui.features.marketplace),
            deployment: request
                .features
                .as_ref()
                .and_then(|f| f.deployment)
                .unwrap_or(config.ui.features.deployment),
            user_management: request
                .features
                .as_ref()
                .and_then(|f| f.user_management)
                .unwrap_or(config.ui.features.user_management),
            monitoring: request
                .features
                .as_ref()
                .and_then(|f| f.monitoring)
                .unwrap_or(config.ui.features.monitoring),
        },
    };

    (StatusCode::OK, Json(response)).into_response()
}
async fn api_list_users(
    State(state): State<DashboardState>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    // Verify authentication and admin access
    let auth = match extract_auth_context(&state, &headers).await {
        Ok(auth) => auth,
        Err(error_response) => return error_response.into_response(),
    };

    if let Err(error_response) = require_admin(&auth) {
        return error_response.into_response();
    }

    // Get all users from security manager
    let users = state.security_manager.get_all_users().await;
    let user_summaries: Vec<UserSummary> = users
        .iter()
        .map(|user| UserSummary {
            id: user.id.clone(),
            username: user.username.clone(),
            email: user.email.clone(),
            role: format!("{:?}", user.role),
            is_active: user.is_active,
            created_at: user.created_at,
            last_login: user.last_login,
        })
        .collect();

    let response = UserListResponse {
        total: user_summaries.len(),
        users: user_summaries,
    };

    (StatusCode::OK, Json(response)).into_response()
}
async fn api_create_user(
    State(state): State<DashboardState>,
    headers: axum::http::HeaderMap,
    Json(request): Json<CreateUserRequest>,
) -> impl IntoResponse {
    // Verify authentication and admin access
    let auth = match extract_auth_context(&state, &headers).await {
        Ok(auth) => auth,
        Err(error_response) => return error_response.into_response(),
    };

    if let Err(error_response) = require_admin(&auth) {
        return error_response.into_response();
    }

    // Validate input
    if request.username.trim().is_empty() {
        let error = ApiError {
            error: "Username cannot be empty".to_string(),
            details: None,
        };
        return (StatusCode::BAD_REQUEST, Json(error)).into_response();
    }

    if request.password.len() < 8 {
        let error = ApiError {
            error: "Password must be at least 8 characters".to_string(),
            details: None,
        };
        return (StatusCode::BAD_REQUEST, Json(error)).into_response();
    }

    // Parse role
    let role = match request.role.to_lowercase().as_str() {
        "admin" => crate::security::UserRole::Admin,
        "user" => crate::security::UserRole::User,
        "guest" => crate::security::UserRole::Guest,
        "service" => crate::security::UserRole::Service,
        _ => {
            let error = ApiError {
                error: "Invalid role. Must be one of: admin, user, guest, service".to_string(),
                details: None,
            };
            return (StatusCode::BAD_REQUEST, Json(error)).into_response();
        }
    };

    // Hash password
    let password_hash = match state.security_manager.hash_password(&request.password) {
        Ok(hash) => hash,
        Err(e) => {
            let error = ApiError {
                error: "Failed to hash password".to_string(),
                details: Some(e.to_string()),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response();
        }
    };

    // Create user
    let user_id = uuid::Uuid::new_v4().to_string();
    let new_user = crate::security::User {
        id: user_id.clone(),
        username: request.username.trim().to_string(),
        email: request.email,
        password_hash: Some(password_hash),
        role,
        api_keys: vec![],
        created_at: chrono::Utc::now(),
        last_login: None,
        is_active: true,
        permissions: std::collections::HashSet::new(),
        rate_limit_override: None,
    };

    // Add user to security manager
    match state.security_manager.create_user(new_user.clone()).await {
        Ok(_) => {
            let response = UserDetailResponse {
                id: new_user.id,
                username: new_user.username,
                email: new_user.email,
                role: format!("{:?}", new_user.role),
                is_active: new_user.is_active,
                created_at: new_user.created_at,
                last_login: new_user.last_login,
                permissions: vec![], // TODO: Convert permissions to strings
            };
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => {
            let error = ApiError {
                error: "Failed to create user".to_string(),
                details: Some(e.to_string()),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}
async fn api_get_user(
    State(state): State<DashboardState>,
    Path(user_id): Path<String>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    // Verify authentication and admin access
    let auth = match extract_auth_context(&state, &headers).await {
        Ok(auth) => auth,
        Err(error_response) => return error_response.into_response(),
    };

    if let Err(error_response) = require_admin(&auth) {
        return error_response.into_response();
    }

    // Get user from security manager
    let user = match state.security_manager.get_user_by_id(&user_id).await {
        Some(user) => user,
        None => {
            let error = ApiError {
                error: "User not found".to_string(),
                details: None,
            };
            return (StatusCode::NOT_FOUND, Json(error)).into_response();
        }
    };

    let permissions: Vec<String> = user
        .permissions
        .iter()
        .map(|p| format!("{:?}", p))
        .collect();

    let response = UserDetailResponse {
        id: user.id.clone(),
        username: user.username.clone(),
        email: user.email.clone(),
        role: format!("{:?}", user.role),
        is_active: user.is_active,
        created_at: user.created_at,
        last_login: user.last_login,
        permissions,
    };

    (StatusCode::OK, Json(response)).into_response()
}
async fn api_update_user(
    State(state): State<DashboardState>,
    Path(user_id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(request): Json<UpdateUserRequest>,
) -> impl IntoResponse {
    // Verify authentication and admin access
    let auth = match extract_auth_context(&state, &headers).await {
        Ok(auth) => auth,
        Err(error_response) => return error_response.into_response(),
    };

    if let Err(error_response) = require_admin(&auth) {
        return error_response.into_response();
    }

    // Get and update user
    let mut users = state.security_manager.users.write().await;
    let user = match users.get_mut(&user_id) {
        Some(user) => user,
        None => {
            let error = ApiError {
                error: "User not found".to_string(),
                details: None,
            };
            return (StatusCode::NOT_FOUND, Json(error)).into_response();
        }
    };

    // Prevent self-deactivation for admins
    if user_id == auth.user_id && request.is_active == Some(false) {
        let error = ApiError {
            error: "Cannot deactivate your own account".to_string(),
            details: None,
        };
        return (StatusCode::BAD_REQUEST, Json(error)).into_response();
    }

    // Update fields if provided
    if let Some(username) = &request.username {
        if username.trim().is_empty() {
            let error = ApiError {
                error: "Username cannot be empty".to_string(),
                details: None,
            };
            return (StatusCode::BAD_REQUEST, Json(error)).into_response();
        }
        user.username = username.trim().to_string();
    }

    if let Some(email) = &request.email {
        user.email = Some(email.clone());
    }

    if let Some(role_str) = &request.role {
        let role = match role_str.to_lowercase().as_str() {
            "admin" => crate::security::UserRole::Admin,
            "user" => crate::security::UserRole::User,
            "guest" => crate::security::UserRole::Guest,
            "service" => crate::security::UserRole::Service,
            _ => {
                let error = ApiError {
                    error: "Invalid role. Must be one of: admin, user, guest, service".to_string(),
                    details: None,
                };
                return (StatusCode::BAD_REQUEST, Json(error)).into_response();
            }
        };

        // Prevent removing admin role from yourself
        if user_id == auth.user_id && !matches!(role, crate::security::UserRole::Admin) {
            let error = ApiError {
                error: "Cannot remove admin role from your own account".to_string(),
                details: None,
            };
            return (StatusCode::BAD_REQUEST, Json(error)).into_response();
        }

        user.role = role;
    }

    if let Some(is_active) = request.is_active {
        user.is_active = is_active;
    }

    let permissions: Vec<String> = user
        .permissions
        .iter()
        .map(|p| format!("{:?}", p))
        .collect();

    let response = UserDetailResponse {
        id: user.id.clone(),
        username: user.username.clone(),
        email: user.email.clone(),
        role: format!("{:?}", user.role),
        is_active: user.is_active,
        created_at: user.created_at,
        last_login: user.last_login,
        permissions,
    };

    (StatusCode::OK, Json(response)).into_response()
}
async fn api_delete_user(
    State(state): State<DashboardState>,
    Path(user_id): Path<String>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    // Verify authentication and admin access
    let auth = match extract_auth_context(&state, &headers).await {
        Ok(auth) => auth,
        Err(error_response) => return error_response.into_response(),
    };

    if let Err(error_response) = require_admin(&auth) {
        return error_response.into_response();
    }

    // Prevent self-deletion
    if user_id == auth.user_id {
        let error = ApiError {
            error: "Cannot delete your own account".to_string(),
            details: None,
        };
        return (StatusCode::BAD_REQUEST, Json(error)).into_response();
    }

    // Delete user from security manager
    match state.security_manager.delete_user(&user_id).await {
        Ok(_) => {
            let response = serde_json::json!({
                "message": "User deleted successfully",
                "user_id": user_id
            });
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            // Check if it's a "user not found" error
            let error = if e.to_string().contains("not found") {
                ApiError {
                    error: "User not found".to_string(),
                    details: None,
                }
            } else {
                ApiError {
                    error: "Failed to delete user".to_string(),
                    details: Some(e.to_string()),
                }
            };

            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };

            (status, Json(error)).into_response()
        }
    }
}
async fn api_login(
    State(state): State<DashboardState>,
    Json(request): Json<LoginRequest>,
) -> impl IntoResponse {
    // Authenticate user
    match state
        .security_manager
        .authenticate_user(&request.username, &request.password)
        .await
    {
        Ok(Some(user)) => {
            // Generate JWT token
            match state.security_manager.generate_jwt_token(&user).await {
                Ok(token) => {
                    let profile = UserProfile {
                        id: user.id,
                        username: user.username,
                        email: user.email,
                        role: format!("{:?}", user.role),
                        last_login: user.last_login,
                    };

                    let response = LoginResponse {
                        token,
                        user: profile,
                    };

                    (StatusCode::OK, Json(response)).into_response()
                }
                Err(e) => {
                    let error = ApiError {
                        error: "Token generation failed".to_string(),
                        details: Some(e.to_string()),
                    };
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
                }
            }
        }
        Ok(None) => {
            let error = ApiError {
                error: "Invalid credentials".to_string(),
                details: None,
            };
            (StatusCode::UNAUTHORIZED, Json(error)).into_response()
        }
        Err(e) => {
            let error = ApiError {
                error: "Authentication failed".to_string(),
                details: Some(e.to_string()),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

async fn api_logout(
    State(state): State<DashboardState>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    // Extract JWT from Authorization header
    if let Some(auth_header) = headers.get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                // Verify token and get claims to extract JTI
                if let Ok(claims) = state.security_manager.verify_jwt_token(token).await {
                    // Revoke the token
                    if let Err(e) = state.security_manager.revoke_token(claims.jti).await {
                        warn!("Failed to revoke token: {}", e);
                    }
                }
            }
        }
    }

    // Return success regardless - logout should always succeed from client perspective
    Json(serde_json::json!({"message": "Logged out successfully"}))
}

async fn api_profile(
    State(state): State<DashboardState>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    // Extract JWT from Authorization header
    if let Some(auth_header) = headers.get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                // Verify token
                match state.security_manager.verify_jwt_token(token).await {
                    Ok(claims) => {
                        // Get user details
                        let users = state.security_manager.users.read().await;
                        if let Some(user) = users.get(&claims.sub) {
                            let profile = UserProfile {
                                id: user.id.clone(),
                                username: user.username.clone(),
                                email: user.email.clone(),
                                role: format!("{:?}", user.role),
                                last_login: user.last_login,
                            };
                            return (StatusCode::OK, Json(profile)).into_response();
                        }
                    }
                    Err(e) => {
                        let error = ApiError {
                            error: "Invalid token".to_string(),
                            details: Some(e.to_string()),
                        };
                        return (StatusCode::UNAUTHORIZED, Json(error)).into_response();
                    }
                }
            }
        }
    }

    let error = ApiError {
        error: "Authorization header missing or invalid".to_string(),
        details: None,
    };
    (StatusCode::UNAUTHORIZED, Json(error)).into_response()
}

/// Extract authentication context from request headers
async fn extract_auth_context(
    state: &DashboardState,
    headers: &axum::http::HeaderMap,
) -> Result<AuthContext, (StatusCode, Json<ApiError>)> {
    // Extract JWT from Authorization header
    let auth_header = headers.get("Authorization").ok_or_else(|| {
        let error = ApiError {
            error: "Authorization header missing".to_string(),
            details: None,
        };
        (StatusCode::UNAUTHORIZED, Json(error))
    })?;

    let auth_str = auth_header.to_str().map_err(|_| {
        let error = ApiError {
            error: "Invalid Authorization header".to_string(),
            details: None,
        };
        (StatusCode::UNAUTHORIZED, Json(error))
    })?;

    let token = auth_str.strip_prefix("Bearer ").ok_or_else(|| {
        let error = ApiError {
            error: "Authorization header must be Bearer token".to_string(),
            details: None,
        };
        (StatusCode::UNAUTHORIZED, Json(error))
    })?;

    // Verify token
    let claims = state
        .security_manager
        .verify_jwt_token(token)
        .await
        .map_err(|e| {
            let error = ApiError {
                error: "Invalid token".to_string(),
                details: Some(e.to_string()),
            };
            (StatusCode::UNAUTHORIZED, Json(error))
        })?;

    Ok(AuthContext {
        user_id: claims.sub,
        username: claims.username,
        role: claims.role,
    })
}

/// Check if user has admin role
fn require_admin(auth: &AuthContext) -> Result<(), (StatusCode, Json<ApiError>)> {
    match auth.role {
        crate::security::UserRole::Admin => Ok(()),
        _ => {
            let error = ApiError {
                error: "Admin access required".to_string(),
                details: None,
            };
            Err((StatusCode::FORBIDDEN, Json(error)))
        }
    }
}

// Static file handlers
async fn serve_static_css() -> impl IntoResponse {
    "/* CSS content */"
}
async fn serve_static_js() -> impl IntoResponse {
    "// JavaScript content"
}
async fn serve_static_images() -> impl IntoResponse {
    "Image content"
}
async fn serve_static_fonts() -> impl IntoResponse {
    "Font content"
}

// WebSocket handler for real-time updates
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<DashboardState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_websocket(socket, state))
}

async fn handle_websocket(mut socket: WebSocket, state: DashboardState) {
    info!("New WebSocket connection established");

    let mut rx = state.notifications.subscribe();

    loop {
        tokio::select! {
            msg = rx.recv() => {
                match msg {
                    Ok(notification) => {
                        let json = serde_json::to_string(&notification).unwrap_or_default();
                        if socket.send(axum::extract::ws::Message::Text(json)).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(axum::extract::ws::Message::Close(_))) => break,
                    Some(Err(_)) => break,
                    None => break,
                    _ => {} // Ignore other message types
                }
            }
        }
    }

    info!("WebSocket connection closed");
}

// HTML generation
async fn generate_dashboard_html(config: &DashboardConfig, page: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{} - {}</title>
    <link rel="stylesheet" href="/assets/css/dashboard.css">
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.0.0/css/all.min.css">
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <script src="https://unpkg.com/vue@3/dist/vue.global.js"></script>
</head>
<body>
    <div id="app">
        <nav class="sidebar">
            <div class="sidebar-header">
                <h2><i class="fas fa-fire"></i> {}</h2>
            </div>
            <ul class="sidebar-menu">
                <li><a href="/" class="{}"><i class="fas fa-tachometer-alt"></i> Dashboard</a></li>
                <li><a href="/models" class="{}"><i class="fas fa-brain"></i> Models</a></li>
                <li><a href="/metrics" class="{}"><i class="fas fa-chart-bar"></i> Metrics</a></li>
                <li><a href="/nodes" class="{}"><i class="fas fa-server"></i> Nodes</a></li>
                <li><a href="/deployments" class="{}"><i class="fas fa-rocket"></i> Deployments</a></li>
                <li><a href="/marketplace" class="{}"><i class="fas fa-store"></i> Marketplace</a></li>
                <li><a href="/settings" class="{}"><i class="fas fa-cog"></i> Settings</a></li>
            </ul>
        </nav>

        <main class="main-content">
            <header class="header">
                <div class="header-left">
                    <h1>{}</h1>
                </div>
                <div class="header-right">
                    <div class="notifications">
                        <i class="fas fa-bell"></i>
                        <span class="notification-count">3</span>
                    </div>
                    <div class="user-menu">
                        <i class="fas fa-user-circle"></i>
                        <span>Admin</span>
                    </div>
                </div>
            </header>

            <div class="content">
                {}
            </div>
        </main>
    </div>

    <script src="/assets/js/dashboard.js"></script>
    <script>
        // Initialize WebSocket connection
        const ws = new WebSocket('ws://localhost:{}/ws');
        ws.onmessage = function(event) {{
            const notification = JSON.parse(event.data);
            console.log('Received notification:', notification);
            // Handle real-time updates
        }};
    </script>
</body>
</html>"#,
        page.replace("_", " ").replace("-", " "), // page title
        config.ui.title,
        config.ui.branding.organization,
        if page == "dashboard" { "active" } else { "" },
        if page == "models" { "active" } else { "" },
        if page == "metrics" { "active" } else { "" },
        if page == "nodes" { "active" } else { "" },
        if page == "deployments" { "active" } else { "" },
        if page == "marketplace" { "active" } else { "" },
        if page == "settings" { "active" } else { "" },
        page.replace("_", " ").replace("-", " "), // header title
        generate_page_content(page),
        config.port
    )
}

fn generate_page_content(page: &str) -> String {
    match page {
        "dashboard" => r#"
            <div class="dashboard-grid">
                <div class="card">
                    <h3>System Overview</h3>
                    <div class="metrics-grid">
                        <div class="metric">
                            <div class="metric-value">85%</div>
                            <div class="metric-label">CPU Usage</div>
                        </div>
                        <div class="metric">
                            <div class="metric-value">12.5GB</div>
                            <div class="metric-label">Memory</div>
                        </div>
                        <div class="metric">
                            <div class="metric-value">3</div>
                            <div class="metric-label">Active Models</div>
                        </div>
                        <div class="metric">
                            <div class="metric-value">1,234</div>
                            <div class="metric-label">Requests/min</div>
                        </div>
                    </div>
                </div>

                <div class="card">
                    <h3>Performance Chart</h3>
                    <canvas id="performanceChart"></canvas>
                </div>

                <div class="card">
                    <h3>Recent Activity</h3>
                    <ul class="activity-list">
                        <li><i class="fas fa-upload"></i> Model "llama-7b" deployed to production</li>
                        <li><i class="fas fa-warning"></i> High memory usage detected on node-02</li>
                        <li><i class="fas fa-check"></i> Federated round 15 completed successfully</li>
                    </ul>
                </div>
            </div>
        "#.to_string(),

        "models" => r#"
            <div class="models-container">
                <div class="models-header">
                    <button class="btn btn-primary"><i class="fas fa-plus"></i> Upload Model</button>
                    <button class="btn btn-secondary"><i class="fas fa-download"></i> Import from Marketplace</button>
                </div>

                <div class="models-table">
                    <table>
                        <thead>
                            <tr>
                                <th>Model</th>
                                <th>Version</th>
                                <th>Format</th>
                                <th>Size</th>
                                <th>Status</th>
                                <th>Usage</th>
                                <th>Actions</th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr>
                                <td>LLaMA 7B</td>
                                <td>v1.0</td>
                                <td>GGUF</td>
                                <td>7.2 GB</td>
                                <td><span class="status-badge status-available">Available</span></td>
                                <td>1,250 requests</td>
                                <td>
                                    <button class="btn-icon"><i class="fas fa-play"></i></button>
                                    <button class="btn-icon"><i class="fas fa-download"></i></button>
                                    <button class="btn-icon"><i class="fas fa-trash"></i></button>
                                </td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>
        "#.to_string(),

        _ => format!("<h2>{} Page</h2><p>Content for {} page would be here.</p>",
                    page.replace("_", " ").replace("-", " "), page)
    }
}

// Background task functions
async fn update_system_metrics(state: &DashboardState) -> Result<()> {
    // Mock metrics update
    let mut metrics = state.metrics.write().await;
    metrics.timestamp = Utc::now();
    metrics.cpu_usage = (Utc::now().timestamp() as f32 * 0.001).sin().abs() * 100.0;
    metrics.memory_usage = (Utc::now().timestamp() as f32 * 0.002).cos().abs() * 100.0;
    // Update other metrics...

    Ok(())
}

async fn check_system_alerts(state: &DashboardState) -> Result<()> {
    let metrics = state.metrics.read().await;

    if metrics.cpu_usage > 90.0 {
        let notification = NotificationMessage {
            id: Uuid::new_v4().to_string(),
            level: NotificationLevel::Warning,
            title: "High CPU Usage".to_string(),
            message: format!("CPU usage is at {:.1}%", metrics.cpu_usage),
            timestamp: Utc::now(),
            category: "system".to_string(),
            actions: vec![],
        };

        let _ = state.notifications.send(notification);
    }

    Ok(())
}
