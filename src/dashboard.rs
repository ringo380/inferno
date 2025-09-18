use crate::config::Config;
use anyhow::{Context, Result};
use axum::{
    extract::{Path, Query, State, WebSocketUpgrade, ws::WebSocket},
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
use tokio::sync::{broadcast, RwLock};
use tower::ServiceBuilder;
use uuid::Uuid;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

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
        };

        Ok(Self { config, state })
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting dashboard server on {}:{}", self.config.bind_address, self.config.port);

        let app = self.create_router().await?;

        let addr = format!("{}:{}", self.config.bind_address, self.config.port);
        let listener = tokio::net::TcpListener::bind(&addr).await
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
                    .layer(CorsLayer::permissive())
            );

        Ok(app)
    }

    fn create_api_router(&self) -> Router<DashboardState> {
        Router::new()
            // Models API
            .route("/models", get(api_list_models).post(api_create_model))
            .route("/models/:id", get(api_get_model).put(api_update_model).delete(api_delete_model))
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
            .route("/deployments", get(api_list_deployments).post(api_create_deployment))
            .route("/deployments/:id", get(api_get_deployment).put(api_update_deployment).delete(api_delete_deployment))
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
            .route("/users/:id", get(api_get_user).put(api_update_user).delete(api_delete_user))
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
                health_checks: vec![
                    HealthCheck {
                        name: "HTTP Health".to_string(),
                        status: HealthStatus::Healthy,
                        last_check: Utc::now(),
                        message: None,
                    }
                ],
            });
        }

        Ok(())
    }

    pub async fn start_background_tasks(&self) -> Result<()> {
        info!("Starting dashboard background tasks");

        // Start metrics collection
        let state = self.state.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_millis(state.config.realtime.update_frequency_ms)
            );

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
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": "Model name cannot be empty",
            "details": null
        })));
    }

    // Check if model with same name already exists
    {
        let models = state.models.read().await;
        if models.iter().any(|m| m.name == request.name) {
            return (StatusCode::CONFLICT, Json(serde_json::json!({
                "error": "Model with this name already exists",
                "details": format!("Model '{}' already exists", request.name)
            })));
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

    (StatusCode::CREATED, Json(serde_json::json!({
        "data": model,
        "message": "Model created successfully"
    })))
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

        (StatusCode::OK, Json(serde_json::json!({
            "data": updated_model,
            "message": "Model updated successfully"
        })))
    } else {
        (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": "Model not found",
            "details": format!("Model with ID '{}' does not exist", id)
        })))
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
                return (StatusCode::CONFLICT, Json(serde_json::json!({
                    "error": "Cannot delete model",
                    "details": format!("Model is currently used in {} active deployment(s)", active_deployments)
                })));
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

        (StatusCode::OK, Json(serde_json::json!({
            "message": "Model deleted successfully",
            "deleted_model": {
                "id": model.id,
                "name": model.name
            }
        })))
    } else {
        (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": "Model not found",
            "details": format!("Model with ID '{}' does not exist", id)
        })))
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
        return (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": "Model not found",
            "details": format!("Model with ID '{}' does not exist", id)
        })));
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

    (StatusCode::CREATED, Json(serde_json::json!({
        "data": deployment,
        "message": "Model deployment initiated successfully"
    })))
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
        return (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": "Model not found",
            "details": format!("Model with ID '{}' does not exist", id)
        })));
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
    let start_time = query.start_time.unwrap_or_else(|| Utc::now() - chrono::Duration::hours(24));
    let end_time = query.end_time.unwrap_or_else(|| Utc::now());
    let interval = query.interval.unwrap_or_else(|| "5m".to_string());

    // Validate time range
    if start_time >= end_time {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": "Invalid time range",
            "details": "Start time must be before end time"
        })));
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

        current_time = current_time + step;
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
    let start_time = query.start_time.unwrap_or_else(|| Utc::now() - chrono::Duration::hours(24));
    let end_time = query.end_time.unwrap_or_else(|| Utc::now());

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
        },
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
            (StatusCode::OK, Json(serde_json::json!({
                "format": "csv",
                "data": csv_data,
                "filename": format!("inferno_metrics_{}.csv", Utc::now().format("%Y%m%d_%H%M%S"))
            })))
        },
        "prometheus" => {
            let prometheus_data = format!(
                "# HELP inferno_cpu_usage CPU usage percentage\n# TYPE inferno_cpu_usage gauge\ninferno_cpu_usage {}\n# HELP inferno_memory_usage Memory usage percentage\n# TYPE inferno_memory_usage gauge\ninferno_memory_usage {}\n# HELP inferno_requests_per_second Requests per second\n# TYPE inferno_requests_per_second gauge\ninferno_requests_per_second {}\n",
                metrics.cpu_usage,
                metrics.memory_usage,
                metrics.inference_stats.requests_per_second
            );
            (StatusCode::OK, Json(serde_json::json!({
                "format": "prometheus",
                "data": prometheus_data,
                "content_type": "text/plain; version=0.0.4"
            })))
        },
        _ => {
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": "Unsupported export format",
                "details": format!("Format '{}' is not supported. Use: json, csv, prometheus", format)
            })))
        }
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
        (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": "Node not found",
            "details": format!("Node with ID '{}' does not exist", id)
        })))
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
        (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": "Node not found",
            "details": format!("Node with ID '{}' does not exist", id)
        })))
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
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": "Invalid model ID",
            "details": format!("Model with ID '{}' does not exist", request.model_id)
        })));
    }

    // Validate replicas count
    if request.replicas == 0 || request.replicas > 100 {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": "Invalid replica count",
            "details": "Replica count must be between 1 and 100"
        })));
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
            }
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
        message: format!("Deployment '{}' created for environment '{}'", deployment.id, request.environment),
        timestamp: Utc::now(),
        category: "deployments".to_string(),
        actions: vec![],
    });

    (StatusCode::CREATED, Json(serde_json::json!({
        "data": deployment,
        "message": "Deployment created successfully"
    })))
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
        (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": "Deployment not found",
            "details": format!("Deployment with ID '{}' does not exist", id)
        })))
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
                return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                    "error": "Invalid replica count",
                    "details": "Replica count must be between 1 and 100"
                })));
            }

            if deployment.target_replicas != target_replicas {
                let old_replicas = deployment.target_replicas;
                deployment.target_replicas = target_replicas;
                deployment.status = DeploymentStatus::Scaling;
                changes.push(format!("Target replicas changed from {} to {}", old_replicas, target_replicas));
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
                message: format!("Deployment '{}' updated: {}", updated_deployment.id, changes.join(", ")),
                timestamp: Utc::now(),
                category: "deployments".to_string(),
                actions: vec![],
            });

            (StatusCode::OK, Json(serde_json::json!({
                "data": updated_deployment,
                "message": "Deployment updated successfully"
            })))
        } else {
            let deployment_copy = deployment.clone();
            (StatusCode::OK, Json(serde_json::json!({
                "data": deployment_copy,
                "message": "No changes made to deployment"
            })))
        }
    } else {
        (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": "Deployment not found",
            "details": format!("Deployment with ID '{}' does not exist", id)
        })))
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
            message: format!("Deployment '{}' has been terminated and deleted", deployment_info.id),
            timestamp: Utc::now(),
            category: "deployments".to_string(),
            actions: vec![],
        });

        (StatusCode::OK, Json(serde_json::json!({
            "message": "Deployment deleted successfully",
            "deleted_deployment": {
                "id": deployment_info.id,
                "model_id": deployment_info.model_id,
                "environment": deployment_info.environment,
                "terminated_at": Utc::now()
            }
        })))
    } else {
        (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": "Deployment not found",
            "details": format!("Deployment with ID '{}' does not exist", id)
        })))
    }
}
async fn api_scale_deployment(
    State(state): State<DashboardState>,
    Path(id): Path<String>,
    Json(request): Json<ScaleDeploymentRequest>,
) -> impl IntoResponse {
    // Validate replica count
    if request.replicas == 0 || request.replicas > 100 {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": "Invalid replica count",
            "details": "Replica count must be between 1 and 100"
        })));
    }

    let mut deployments = state.deployments.write().await;

    if let Some(deployment) = deployments.iter_mut().find(|d| d.id == id) {
        let old_replicas = deployment.target_replicas;
        let new_replicas = request.replicas;

        if old_replicas == new_replicas {
            let deployment_copy = deployment.clone();
            return (StatusCode::OK, Json(serde_json::json!({
                "data": deployment_copy,
                "message": "Deployment is already at the requested replica count"
            })));
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
        let scale_action = if new_replicas > old_replicas { "up" } else { "down" };
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

        (StatusCode::OK, Json(serde_json::json!({
            "data": response_data,
            "message": format!("Deployment scaling initiated: {} -> {} replicas", old_replicas, new_replicas)
        })))
    } else {
        (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": "Deployment not found",
            "details": format!("Deployment with ID '{}' does not exist", id)
        })))
    }
}
async fn api_deployment_logs() -> impl IntoResponse { Json(serde_json::json!({"message": "Not implemented"})) }
async fn api_marketplace_search() -> impl IntoResponse { Json(serde_json::json!({"message": "Not implemented"})) }
async fn api_marketplace_featured() -> impl IntoResponse { Json(serde_json::json!({"message": "Not implemented"})) }
async fn api_marketplace_downloads() -> impl IntoResponse { Json(serde_json::json!({"message": "Not implemented"})) }
async fn api_system_info() -> impl IntoResponse { Json(serde_json::json!({"message": "Not implemented"})) }
async fn api_get_config() -> impl IntoResponse { Json(serde_json::json!({"message": "Not implemented"})) }
async fn api_update_config() -> impl IntoResponse { Json(serde_json::json!({"message": "Not implemented"})) }
async fn api_list_users() -> impl IntoResponse { Json(serde_json::json!({"message": "Not implemented"})) }
async fn api_create_user() -> impl IntoResponse { Json(serde_json::json!({"message": "Not implemented"})) }
async fn api_get_user() -> impl IntoResponse { Json(serde_json::json!({"message": "Not implemented"})) }
async fn api_update_user() -> impl IntoResponse { Json(serde_json::json!({"message": "Not implemented"})) }
async fn api_delete_user() -> impl IntoResponse { Json(serde_json::json!({"message": "Not implemented"})) }
async fn api_login() -> impl IntoResponse { Json(serde_json::json!({"message": "Not implemented"})) }
async fn api_logout() -> impl IntoResponse { Json(serde_json::json!({"message": "Not implemented"})) }
async fn api_profile() -> impl IntoResponse { Json(serde_json::json!({"message": "Not implemented"})) }

// Static file handlers
async fn serve_static_css() -> impl IntoResponse { "/* CSS content */" }
async fn serve_static_js() -> impl IntoResponse { "// JavaScript content" }
async fn serve_static_images() -> impl IntoResponse { "Image content" }
async fn serve_static_fonts() -> impl IntoResponse { "Font content" }

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
    format!(r#"<!DOCTYPE html>
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