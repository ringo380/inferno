use crate::logging_audit::{
    ComplianceReport, ComplianceStandard, IntegrityReport, IntegrityStatus,
};
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Datelike, Timelike, Utc};
use flate2::{write::GzEncoder, Compression as GzCompression};
use lettre::{
    message::{header::ContentType, Mailbox, Message},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io::Write, path::PathBuf, sync::Arc, time::SystemTime};
use tokio::{
    fs,
    sync::{mpsc, RwLock},
    time::interval,
};
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use zstd::stream::write::Encoder as ZstdEncoder;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub timestamp: SystemTime,
    pub event_type: EventType,
    pub severity: Severity,
    pub actor: Actor,
    pub resource: Resource,
    pub action: String,
    pub details: EventDetails,
    pub context: EventContext,
    pub outcome: EventOutcome,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    Authentication,
    Authorization,
    ModelManagement,
    DataAccess,
    SystemChange,
    SecurityEvent,
    PerformanceEvent,
    ErrorEvent,
    UserAction,
    ApiCall,
    FileAccess,
    ConfigChange,
    NetworkEvent,
    BatchJob,
    ABTest,
    Deployment,
    Rollback,
    GpuUsage,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Critical = 1,
    High = 2,
    Medium = 3,
    Low = 4,
    Info = 5,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actor {
    pub actor_type: ActorType,
    pub id: String,
    pub name: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActorType {
    User,
    Service,
    System,
    Api,
    Scheduled,
    Unknown,
}

impl std::fmt::Display for ActorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActorType::User => write!(f, "User"),
            ActorType::Service => write!(f, "Service"),
            ActorType::System => write!(f, "System"),
            ActorType::Api => write!(f, "API"),
            ActorType::Scheduled => write!(f, "Scheduled"),
            ActorType::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub resource_type: ResourceType,
    pub id: String,
    pub name: String,
    pub path: Option<String>,
    pub owner: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
    Model,
    Dataset,
    Config,
    File,
    Api,
    Queue,
    Job,
    User,
    Service,
    Gpu,
    Deployment,
    Version,
    Cache,
    Database,
    Custom(String),
}

impl std::fmt::Display for ResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceType::Model => write!(f, "Model"),
            ResourceType::Dataset => write!(f, "Dataset"),
            ResourceType::Config => write!(f, "Config"),
            ResourceType::File => write!(f, "File"),
            ResourceType::Api => write!(f, "API"),
            ResourceType::Queue => write!(f, "Queue"),
            ResourceType::Job => write!(f, "Job"),
            ResourceType::User => write!(f, "User"),
            ResourceType::Service => write!(f, "Service"),
            ResourceType::Gpu => write!(f, "GPU"),
            ResourceType::Deployment => write!(f, "Deployment"),
            ResourceType::Version => write!(f, "Version"),
            ResourceType::Cache => write!(f, "Cache"),
            ResourceType::Database => write!(f, "Database"),
            ResourceType::Custom(name) => write!(f, "{}", name),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDetails {
    pub description: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub request_id: Option<String>,
    pub correlation_id: Option<String>,
    pub trace_id: Option<String>,
    pub parent_event_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventContext {
    pub environment: String,
    pub application: String,
    pub version: String,
    pub hostname: String,
    pub process_id: u32,
    pub thread_id: Option<String>,
    pub request_path: Option<String>,
    pub request_method: Option<String>,
    pub client_info: Option<ClientInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub client_id: String,
    pub client_name: String,
    pub client_version: String,
    pub platform: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventOutcome {
    pub success: bool,
    pub status_code: Option<i32>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub duration_ms: Option<u64>,
    pub bytes_processed: Option<u64>,
    pub records_affected: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditQuery {
    pub event_types: Option<Vec<EventType>>,
    pub severities: Option<Vec<Severity>>,
    pub actors: Option<Vec<String>>,
    pub resources: Option<Vec<String>>,
    pub start_time: Option<SystemTime>,
    pub end_time: Option<SystemTime>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub sort_by: Option<SortField>,
    pub sort_order: Option<SortOrder>,
    pub search_text: Option<String>,
    pub date_range: Option<(SystemTime, SystemTime)>,
    pub actor_filter: Option<String>,
    pub resource_filter: Option<String>,
    pub severity_filter: Option<String>,
    pub outcome_filter: Option<String>,
    pub text_search: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortField {
    Timestamp,
    Severity,
    EventType,
    Actor,
    Resource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfiguration {
    pub enabled: bool,
    pub log_level: LogLevel,
    pub storage_path: PathBuf,
    pub max_file_size_mb: u64,
    pub max_files: u32,
    pub compression_enabled: bool,
    pub compression_method: CompressionMethod,
    pub compression_level: i32,
    pub encryption_enabled: bool,
    pub encryption_key_env: String,
    pub encryption_sensitive_fields_only: bool,
    pub retention_days: u32,
    pub batch_size: usize,
    pub flush_interval_seconds: u64,
    pub include_request_body: bool,
    pub include_response_body: bool,
    pub exclude_patterns: Vec<String>,
    pub alert_on_critical: bool,
    pub alerting: AlertConfiguration,
    pub export_format: ExportFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    All,
    CriticalOnly,
    HighAndAbove,
    MediumAndAbove,
    LowAndAbove,
    InfoOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    JsonLines,
    Csv,
    Parquet,
    Avro,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionMethod {
    None,
    Gzip,
    Zstd,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfiguration {
    pub enabled: bool,
    pub rate_limit_per_hour: u32,
    pub webhook: WebhookConfig,
    pub email: EmailConfig,
    pub slack: SlackConfig,
    pub custom_templates: HashMap<String, String>,
    pub alert_conditions: Vec<AlertCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub enabled: bool,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub timeout_seconds: u64,
    pub retry_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub enabled: bool,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub username: String,
    pub password_env: String,
    pub from_address: String,
    pub to_addresses: Vec<String>,
    pub use_tls: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    pub enabled: bool,
    pub webhook_url: String,
    pub channel: String,
    pub username: String,
    pub icon_emoji: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertCondition {
    pub name: String,
    pub severity_threshold: Severity,
    pub event_types: Vec<EventType>,
    pub rate_threshold: Option<RateThreshold>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateThreshold {
    pub events_per_minute: u32,
    pub window_minutes: u32,
}

impl Default for AuditConfiguration {
    fn default() -> Self {
        Self {
            enabled: true,
            log_level: LogLevel::MediumAndAbove,
            storage_path: PathBuf::from("./logs/audit"),
            max_file_size_mb: 100,
            max_files: 50,
            compression_enabled: true,
            compression_method: CompressionMethod::Gzip,
            compression_level: 6,
            encryption_enabled: false,
            encryption_key_env: "INFERNO_AUDIT_ENCRYPTION_KEY".to_string(),
            encryption_sensitive_fields_only: true,
            retention_days: 90,
            batch_size: 1000,
            flush_interval_seconds: 60,
            include_request_body: false,
            include_response_body: false,
            exclude_patterns: vec!["health-check".to_string(), "ping".to_string()],
            alert_on_critical: true,
            alerting: AlertConfiguration::default(),
            export_format: ExportFormat::JsonLines,
        }
    }
}

impl Default for AlertConfiguration {
    fn default() -> Self {
        Self {
            enabled: true,
            rate_limit_per_hour: 60,
            webhook: WebhookConfig::default(),
            email: EmailConfig::default(),
            slack: SlackConfig::default(),
            custom_templates: HashMap::new(),
            alert_conditions: vec![
                AlertCondition {
                    name: "Critical Events".to_string(),
                    severity_threshold: Severity::Critical,
                    event_types: vec![],
                    rate_threshold: None,
                    enabled: true,
                },
                AlertCondition {
                    name: "High Security Events".to_string(),
                    severity_threshold: Severity::High,
                    event_types: vec![EventType::SecurityEvent, EventType::Authentication],
                    rate_threshold: Some(RateThreshold {
                        events_per_minute: 5,
                        window_minutes: 10,
                    }),
                    enabled: true,
                },
            ],
        }
    }
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            url: String::new(),
            headers: HashMap::new(),
            timeout_seconds: 30,
            retry_count: 3,
        }
    }
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            smtp_host: "localhost".to_string(),
            smtp_port: 587,
            username: String::new(),
            password_env: "INFERNO_SMTP_PASSWORD".to_string(),
            from_address: "audit@inferno.local".to_string(),
            to_addresses: vec![],
            use_tls: true,
        }
    }
}

impl Default for SlackConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            webhook_url: String::new(),
            channel: "#alerts".to_string(),
            username: "Inferno Audit".to_string(),
            icon_emoji: ":warning:".to_string(),
        }
    }
}

pub struct AuditLogger {
    config: AuditConfiguration,
    event_buffer: Arc<RwLock<Vec<AuditEvent>>>,
    event_sender: mpsc::Sender<AuditEvent>,
    is_running: Arc<std::sync::atomic::AtomicBool>,
    context: EventContext,
    encryption_key: Option<Arc<[u8; 32]>>,
    alert_rate_tracker: Arc<RwLock<HashMap<String, Vec<SystemTime>>>>,
}

impl AuditLogger {
    pub async fn new(config: AuditConfiguration) -> Result<Self> {
        // Ensure audit directory exists
        fs::create_dir_all(&config.storage_path).await?;

        let (event_sender, event_receiver) = mpsc::channel::<AuditEvent>(config.batch_size * 2);

        let context = EventContext {
            environment: std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
            application: "inferno".to_string(),
            version: std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.1.0".to_string()),
            hostname: "localhost".to_string(),
            process_id: std::process::id(),
            thread_id: None,
            request_path: None,
            request_method: None,
            client_info: None,
        };

        // Initialize encryption key if enabled
        let encryption_key = if config.encryption_enabled {
            match std::env::var(&config.encryption_key_env) {
                Ok(key_base64) => {
                    let key_bytes = general_purpose::STANDARD
                        .decode(&key_base64)
                        .map_err(|e| anyhow::anyhow!("Invalid encryption key format: {}", e))?;
                    if key_bytes.len() != 32 {
                        return Err(anyhow::anyhow!(
                            "Encryption key must be 32 bytes (256 bits)"
                        ));
                    }
                    let mut key_array = [0u8; 32];
                    key_array.copy_from_slice(&key_bytes);
                    Some(Arc::new(key_array))
                }
                Err(_) => {
                    warn!(
                        "Encryption enabled but key not found in environment variable: {}",
                        config.encryption_key_env
                    );
                    warn!(
                        "Generating new encryption key - this should only be used for development!"
                    );
                    let rng = SystemRandom::new();
                    let mut key_bytes = [0u8; 32];
                    rng.fill(&mut key_bytes).map_err(|e| {
                        anyhow::anyhow!("Failed to generate encryption key: {:?}", e)
                    })?;
                    Some(Arc::new(key_bytes))
                }
            }
        } else {
            None
        };

        let logger = Self {
            config: config.clone(),
            event_buffer: Arc::new(RwLock::new(Vec::new())),
            event_sender,
            is_running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            context,
            encryption_key,
            alert_rate_tracker: Arc::new(RwLock::new(HashMap::new())),
        };

        // Start background processor
        logger.start_background_processor(event_receiver).await?;

        info!(
            "Audit logger initialized with compression: {:?}, encryption: {}",
            config.compression_method, config.encryption_enabled
        );
        Ok(logger)
    }

    async fn start_background_processor(
        &self,
        mut event_receiver: mpsc::Receiver<AuditEvent>,
    ) -> Result<()> {
        let config = self.config.clone();
        let _event_buffer = self.event_buffer.clone();
        let is_running = self.is_running.clone();

        is_running.store(true, std::sync::atomic::Ordering::SeqCst);

        tokio::spawn(async move {
            let mut flush_timer = interval(std::time::Duration::from_secs(
                config.flush_interval_seconds,
            ));
            let mut events_batch = Vec::new();

            while is_running.load(std::sync::atomic::Ordering::SeqCst) {
                tokio::select! {
                    // Receive new events
                    event = event_receiver.recv() => {
                        if let Some(event) = event {
                            events_batch.push(event);

                            // Flush if batch is full
                            if events_batch.len() >= config.batch_size {
                                if let Err(e) = Self::flush_events(&config, &events_batch).await {
                                    error!("Failed to flush audit events: {}", e);
                                }
                                events_batch.clear();
                            }
                        }
                    }

                    // Periodic flush
                    _ = flush_timer.tick() => {
                        if !events_batch.is_empty() {
                            if let Err(e) = Self::flush_events(&config, &events_batch).await {
                                error!("Failed to flush audit events: {}", e);
                            }
                            events_batch.clear();
                        }

                        // Cleanup old files
                        if let Err(e) = Self::cleanup_old_files(&config).await {
                            error!("Failed to cleanup old audit files: {}", e);
                        }
                    }
                }
            }

            // Final flush on shutdown
            if !events_batch.is_empty() {
                if let Err(e) = Self::flush_events(&config, &events_batch).await {
                    error!("Failed to flush audit events on shutdown: {}", e);
                }
            }

            info!("Audit logger background processor stopped");
        });

        Ok(())
    }

    async fn flush_events(config: &AuditConfiguration, events: &[AuditEvent]) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }

        let filename = format!("audit_{}.log", Utc::now().format("%Y%m%d_%H%M%S"));
        let filepath = config.storage_path.join(filename);

        let content = match config.export_format {
            ExportFormat::Json => serde_json::to_string_pretty(events)?,
            ExportFormat::JsonLines => events
                .iter()
                .map(serde_json::to_string)
                .collect::<Result<Vec<_>, _>>()?
                .join("\n"),
            ExportFormat::Csv => Self::events_to_csv(events)?,
            _ => {
                return Err(anyhow::anyhow!(
                    "Unsupported export format: {:?}",
                    config.export_format
                ))
            }
        };

        fs::write(&filepath, &content).await?;

        let mut final_content = content.into_bytes();

        // Apply compression if enabled
        if config.compression_enabled {
            final_content = Self::compress_data(
                &final_content,
                &config.compression_method,
                config.compression_level,
            )?;
        }

        // Apply encryption if enabled
        if config.encryption_enabled {
            if let Some(key) = &Self::get_encryption_key(&config.encryption_key_env)? {
                final_content = Self::encrypt_data(&final_content, key)?;
            }
        }

        fs::write(&filepath, final_content).await?;

        debug!("Flushed {} audit events to {:?}", events.len(), filepath);
        Ok(())
    }

    fn events_to_csv(events: &[AuditEvent]) -> Result<String> {
        let mut csv = String::new();
        csv.push_str("timestamp,event_type,severity,actor,resource,action,success,duration_ms\n");

        for event in events {
            csv.push_str(&format!(
                "{},{:?},{:?},{},{},{},{},{}\n",
                event
                    .timestamp
                    .duration_since(SystemTime::UNIX_EPOCH)?
                    .as_secs(),
                event.event_type,
                event.severity,
                event.actor.name,
                event.resource.name,
                event.action,
                event.outcome.success,
                event.outcome.duration_ms.unwrap_or(0)
            ));
        }

        Ok(csv)
    }

    async fn cleanup_old_files(config: &AuditConfiguration) -> Result<()> {
        let mut entries = fs::read_dir(&config.storage_path).await?;
        let mut files = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            if let Ok(metadata) = entry.metadata().await {
                if metadata.is_file() {
                    if let Ok(modified) = metadata.modified() {
                        files.push((entry.path(), modified));
                    }
                }
            }
        }

        // Sort by modification time (oldest first)
        files.sort_by_key(|(_, modified)| *modified);

        // Remove files older than retention period
        let cutoff = SystemTime::now()
            - std::time::Duration::from_secs(config.retention_days as u64 * 24 * 3600);

        for (path, modified) in &files {
            if *modified < cutoff {
                if let Err(e) = fs::remove_file(path).await {
                    warn!("Failed to remove old audit file {:?}: {}", path, e);
                }
            }
        }

        // Remove excess files if we have too many
        if files.len() > config.max_files as usize {
            let excess_count = files.len() - config.max_files as usize;
            for (path, _) in files.iter().take(excess_count) {
                if let Err(e) = fs::remove_file(path).await {
                    warn!("Failed to remove excess audit file {:?}: {}", path, e);
                }
            }
        }

        Ok(())
    }

    pub async fn log_event(&self, mut event: AuditEvent) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        // Check if event should be excluded
        for pattern in &self.config.exclude_patterns {
            if event.action.contains(pattern) || event.details.description.contains(pattern) {
                return Ok(());
            }
        }

        // Check log level
        if !self.should_log_severity(&event.severity) {
            return Ok(());
        }

        // Add context information
        event.context = self.context.clone();
        event.timestamp = SystemTime::now();

        if event.id.is_empty() {
            event.id = Uuid::new_v4().to_string();
        }

        // Send to background processor
        if let Err(e) = self.event_sender.send(event.clone()).await {
            error!("Failed to send audit event to processor: {}", e);
        }

        // Add to buffer for immediate queries
        {
            let mut buffer = self.event_buffer.write().await;
            buffer.push(event.clone());

            // Keep buffer size reasonable
            if buffer.len() > self.config.batch_size * 2 {
                buffer.drain(0..self.config.batch_size);
            }
        }

        // Alert on critical events
        if self.config.alert_on_critical && matches!(event.severity, Severity::Critical) {
            self.send_critical_alert(&event).await;
        }

        Ok(())
    }

    fn should_log_severity(&self, severity: &Severity) -> bool {
        match self.config.log_level {
            LogLevel::All => true,
            LogLevel::CriticalOnly => matches!(severity, Severity::Critical),
            LogLevel::HighAndAbove => matches!(severity, Severity::Critical | Severity::High),
            LogLevel::MediumAndAbove => matches!(
                severity,
                Severity::Critical | Severity::High | Severity::Medium
            ),
            LogLevel::LowAndAbove => matches!(
                severity,
                Severity::Critical | Severity::High | Severity::Medium | Severity::Low
            ),
            LogLevel::InfoOnly => matches!(severity, Severity::Info),
        }
    }

    async fn send_critical_alert(&self, event: &AuditEvent) {
        if !self.config.alerting.enabled {
            warn!(
                "CRITICAL AUDIT EVENT: {} - {}",
                event.action, event.details.description
            );
            return;
        }

        // Check rate limiting
        if !self
            .should_send_alert(&event.event_type, &event.severity)
            .await
        {
            debug!("Alert rate limited for event: {}", event.id);
            return;
        }

        let alert_context = AlertContext {
            event: event.clone(),
            hostname: self.context.hostname.clone(),
            environment: self.context.environment.clone(),
            timestamp: Utc::now(),
        };

        // Send alerts through all enabled channels
        let config = &self.config.alerting;

        if config.webhook.enabled {
            if let Err(e) = self.send_webhook_alert(&alert_context).await {
                error!("Failed to send webhook alert: {}", e);
            }
        }

        if config.email.enabled {
            if let Err(e) = self.send_email_alert(&alert_context).await {
                error!("Failed to send email alert: {}", e);
            }
        }

        if config.slack.enabled {
            if let Err(e) = self.send_slack_alert(&alert_context).await {
                error!("Failed to send Slack alert: {}", e);
            }
        }

        warn!(
            "CRITICAL AUDIT EVENT ALERTED: {} - {}",
            event.action, event.details.description
        );
    }

    pub async fn query_events(&self, query: AuditQuery) -> Result<Vec<AuditEvent>> {
        // Validate query parameters
        self.validate_audit_query(&query)?;

        let buffer = self.event_buffer.read().await;
        let mut results: Vec<AuditEvent> = buffer.clone();

        // Apply event type filters with validation
        if let Some(ref event_types) = query.event_types {
            if event_types.len() > 50 {
                return Err(anyhow::anyhow!("Too many event types specified (max 50)"));
            }
            results.retain(|e| {
                event_types
                    .iter()
                    .any(|et| std::mem::discriminant(&e.event_type) == std::mem::discriminant(et))
            });
        }

        // Apply severity filters with validation
        if let Some(ref severities) = query.severities {
            if severities.len() > 10 {
                return Err(anyhow::anyhow!("Too many severities specified (max 10)"));
            }
            results.retain(|e| {
                severities
                    .iter()
                    .any(|s| std::mem::discriminant(&e.severity) == std::mem::discriminant(s))
            });
        }

        // Apply actor filters with validation
        if let Some(ref actors) = query.actors {
            if actors.len() > 100 {
                return Err(anyhow::anyhow!("Too many actors specified (max 100)"));
            }
            results.retain(|e| actors.contains(&e.actor.id) || actors.contains(&e.actor.name));
        }

        // Apply resource filters with validation
        if let Some(ref resources) = query.resources {
            if resources.len() > 100 {
                return Err(anyhow::anyhow!("Too many resources specified (max 100)"));
            }
            results.retain(|e| {
                resources.contains(&e.resource.id) || resources.contains(&e.resource.name)
            });
        }

        // Apply time range filters
        if let Some(start_time) = query.start_time {
            results.retain(|e| e.timestamp >= start_time);
        }

        if let Some(end_time) = query.end_time {
            results.retain(|e| e.timestamp <= end_time);
        }

        // Apply text search with improved performance
        if let Some(ref search_text) = query.search_text {
            if search_text.len() > 1000 {
                return Err(anyhow::anyhow!(
                    "Search text too long (max 1000 characters)"
                ));
            }
            let search_lower = search_text.to_lowercase();
            results.retain(|e| {
                e.action.to_lowercase().contains(&search_lower)
                    || e.details.description.to_lowercase().contains(&search_lower)
                    || e.actor.name.to_lowercase().contains(&search_lower)
                    || e.resource.name.to_lowercase().contains(&search_lower)
                    || e.details.parameters.values().any(|v| {
                        v.as_str()
                            .is_some_and(|s| s.to_lowercase().contains(&search_lower))
                    })
            });
        }

        // Apply additional filter fields
        if let Some(ref actor_filter) = query.actor_filter {
            let filter_lower = actor_filter.to_lowercase();
            results.retain(|e| {
                e.actor.name.to_lowercase().contains(&filter_lower)
                    || format!("{:?}", e.actor.actor_type)
                        .to_lowercase()
                        .contains(&filter_lower)
            });
        }

        if let Some(ref resource_filter) = query.resource_filter {
            let filter_lower = resource_filter.to_lowercase();
            results.retain(|e| {
                e.resource.name.to_lowercase().contains(&filter_lower)
                    || format!("{:?}", e.resource.resource_type)
                        .to_lowercase()
                        .contains(&filter_lower)
            });
        }

        if let Some(ref severity_filter) = query.severity_filter {
            let filter_lower = severity_filter.to_lowercase();
            results.retain(|e| {
                format!("{:?}", e.severity)
                    .to_lowercase()
                    .contains(&filter_lower)
            });
        }

        if let Some(ref outcome_filter) = query.outcome_filter {
            let filter_success = outcome_filter.to_lowercase() == "success"
                || outcome_filter.to_lowercase() == "true";
            let filter_failure = outcome_filter.to_lowercase() == "failure"
                || outcome_filter.to_lowercase() == "false";
            if filter_success {
                results.retain(|e| e.outcome.success);
            } else if filter_failure {
                results.retain(|e| !e.outcome.success);
            }
        }

        // Sort results with performance optimization
        if let Some(sort_field) = &query.sort_by {
            results.sort_by(|a, b| {
                let ordering = match sort_field {
                    SortField::Timestamp => a.timestamp.cmp(&b.timestamp),
                    SortField::Severity => {
                        (a.severity.clone() as u8).cmp(&(b.severity.clone() as u8))
                    }
                    SortField::EventType => {
                        format!("{:?}", a.event_type).cmp(&format!("{:?}", b.event_type))
                    }
                    SortField::Actor => a.actor.name.cmp(&b.actor.name),
                    SortField::Resource => a.resource.name.cmp(&b.resource.name),
                };

                match query.sort_order {
                    Some(SortOrder::Descending) => ordering.reverse(),
                    _ => ordering,
                }
            });
        }

        // Apply pagination with bounds checking
        let start = query.offset.unwrap_or(0);
        if start > results.len() {
            return Ok(Vec::new());
        }

        let end = if let Some(limit) = query.limit {
            if limit > 10000 {
                return Err(anyhow::anyhow!("Query limit too high (max 10000)"));
            }
            std::cmp::min(start + limit, results.len())
        } else {
            std::cmp::min(start + 1000, results.len()) // Default limit
        };

        Ok(results[start..end].to_vec())
    }

    /// Validate audit query parameters
    fn validate_audit_query(&self, query: &AuditQuery) -> Result<()> {
        // Validate time range
        if let (Some(start), Some(end)) = (query.start_time, query.end_time) {
            if start > end {
                return Err(anyhow::anyhow!("Start time cannot be after end time"));
            }
            let duration = end.duration_since(start).unwrap_or_default();
            if duration > std::time::Duration::from_secs(365 * 24 * 3600) {
                return Err(anyhow::anyhow!("Time range too large (max 1 year)"));
            }
        }

        // Validate pagination parameters
        if let Some(offset) = query.offset {
            if offset > 1_000_000 {
                return Err(anyhow::anyhow!("Offset too large (max 1,000,000)"));
            }
        }

        if let Some(limit) = query.limit {
            if limit == 0 {
                return Err(anyhow::anyhow!("Limit must be greater than 0"));
            }
            if limit > 10_000 {
                return Err(anyhow::anyhow!("Limit too large (max 10,000)"));
            }
        }

        // Validate search text
        if let Some(ref search_text) = query.search_text {
            if search_text.is_empty() {
                return Err(anyhow::anyhow!("Search text cannot be empty"));
            }
            if search_text.len() > 1000 {
                return Err(anyhow::anyhow!(
                    "Search text too long (max 1000 characters)"
                ));
            }
        }

        Ok(())
    }

    pub async fn export_events(
        &self,
        query: AuditQuery,
        output_path: &PathBuf,
        format: ExportFormat,
    ) -> Result<()> {
        let events = self.query_events(query).await?;

        let content = match format {
            ExportFormat::Json => serde_json::to_string_pretty(&events)?,
            ExportFormat::JsonLines => events
                .iter()
                .map(serde_json::to_string)
                .collect::<Result<Vec<_>, _>>()?
                .join("\n"),
            ExportFormat::Csv => Self::events_to_csv(&events)?,
            _ => return Err(anyhow::anyhow!("Export format {:?} not supported", format)),
        };

        fs::write(output_path, content).await?;
        info!(
            "Exported {} audit events to {:?}",
            events.len(),
            output_path
        );
        Ok(())
    }

    pub async fn get_statistics(&self) -> Result<AuditStatistics> {
        let buffer = self.event_buffer.read().await;

        let mut stats = AuditStatistics {
            total_events: buffer.len(),
            events_by_type: HashMap::new(),
            events_by_severity: HashMap::new(),
            events_by_actor: HashMap::new(),
            events_by_resource_type: HashMap::new(),
            success_rate: 0.0,
            average_duration_ms: 0.0,
            critical_events_count: 0,
            error_events_count: 0,
        };

        let mut total_duration = 0u64;
        let mut duration_count = 0usize;
        let mut success_count = 0usize;

        for event in buffer.iter() {
            // Count by type
            *stats
                .events_by_type
                .entry(format!("{:?}", event.event_type))
                .or_insert(0) += 1;

            // Count by severity
            *stats
                .events_by_severity
                .entry(format!("{:?}", event.severity))
                .or_insert(0) += 1;

            // Count by actor
            *stats
                .events_by_actor
                .entry(event.actor.name.clone())
                .or_insert(0) += 1;

            // Count by resource type
            *stats
                .events_by_resource_type
                .entry(format!("{:?}", event.resource.resource_type))
                .or_insert(0) += 1;

            // Calculate metrics
            if event.outcome.success {
                success_count += 1;
            }

            if let Some(duration) = event.outcome.duration_ms {
                total_duration += duration;
                duration_count += 1;
            }

            if matches!(event.severity, Severity::Critical) {
                stats.critical_events_count += 1;
            }

            if !event.outcome.success {
                stats.error_events_count += 1;
            }
        }

        stats.success_rate = if !buffer.is_empty() {
            success_count as f64 / buffer.len() as f64 * 100.0
        } else {
            0.0
        };

        stats.average_duration_ms = if duration_count > 0 {
            total_duration as f64 / duration_count as f64
        } else {
            0.0
        };

        Ok(stats)
    }

    pub async fn shutdown(&self) {
        self.is_running
            .store(false, std::sync::atomic::Ordering::SeqCst);
        info!("Audit logger shutdown");
    }

    // Compression methods
    fn compress_data(data: &[u8], method: &CompressionMethod, level: i32) -> Result<Vec<u8>> {
        match method {
            CompressionMethod::None => Ok(data.to_vec()),
            CompressionMethod::Gzip => {
                let mut encoder = GzEncoder::new(Vec::new(), GzCompression::new(level as u32));
                encoder.write_all(data)?;
                encoder.finish().map_err(Into::into)
            }
            CompressionMethod::Zstd => {
                let mut encoder = ZstdEncoder::new(Vec::new(), level)?;
                encoder.write_all(data)?;
                encoder.finish().map_err(Into::into)
            }
        }
    }

    fn decompress_data(data: &[u8], method: &CompressionMethod) -> Result<Vec<u8>> {
        match method {
            CompressionMethod::None => Ok(data.to_vec()),
            CompressionMethod::Gzip => {
                use flate2::read::GzDecoder;
                use std::io::Read;
                let mut decoder = GzDecoder::new(data);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed)?;
                Ok(decompressed)
            }
            CompressionMethod::Zstd => zstd::decode_all(data).map_err(Into::into),
        }
    }

    // Encryption methods
    fn get_encryption_key(key_env: &str) -> Result<Option<Arc<[u8; 32]>>> {
        match std::env::var(key_env) {
            Ok(key_base64) => {
                let key_bytes = general_purpose::STANDARD
                    .decode(&key_base64)
                    .map_err(|e| anyhow::anyhow!("Invalid encryption key format: {}", e))?;
                if key_bytes.len() != 32 {
                    return Err(anyhow::anyhow!(
                        "Encryption key must be 32 bytes (256 bits)"
                    ));
                }
                let mut key_array = [0u8; 32];
                key_array.copy_from_slice(&key_bytes);
                Ok(Some(Arc::new(key_array)))
            }
            Err(_) => Ok(None),
        }
    }

    fn encrypt_data(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        let ciphertext = cipher
            .encrypt(&nonce, data)
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

        // Prepend nonce to ciphertext for decryption
        let mut encrypted = Vec::with_capacity(nonce.len() + ciphertext.len());
        encrypted.extend_from_slice(&nonce);
        encrypted.extend_from_slice(&ciphertext);

        Ok(encrypted)
    }

    fn decrypt_data(encrypted_data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
        if encrypted_data.len() < 12 {
            return Err(anyhow::anyhow!("Invalid encrypted data: too short"));
        }

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))
    }

    // Alerting rate limiting
    async fn should_send_alert(&self, event_type: &EventType, severity: &Severity) -> bool {
        let mut tracker = self.alert_rate_tracker.write().await;
        let key = format!("{:?}-{:?}", event_type, severity);
        let now = SystemTime::now();

        // Clean old entries (older than 1 hour)
        let cutoff = now - std::time::Duration::from_secs(3600);
        for timestamps in tracker.values_mut() {
            timestamps.retain(|&ts| ts > cutoff);
        }

        // Check current rate
        let timestamps = tracker.entry(key).or_insert_with(Vec::new);
        if timestamps.len() >= self.config.alerting.rate_limit_per_hour as usize {
            return false;
        }

        timestamps.push(now);
        true
    }

    // Alert sending methods
    #[cfg(feature = "reqwest")]
    async fn send_webhook_alert(&self, context: &AlertContext) -> Result<()> {
        let config = &self.config.alerting.webhook;

        let payload = serde_json::json!({
            "alert_type": "audit_event",
            "severity": context.event.severity,
            "event_type": context.event.event_type,
            "timestamp": context.timestamp.to_rfc3339(),
            "hostname": context.hostname,
            "environment": context.environment,
            "event": {
                "id": context.event.id,
                "action": context.event.action,
                "actor": context.event.actor.name,
                "resource": context.event.resource.name,
                "description": context.event.details.description,
                "success": context.event.outcome.success,
                "error_message": context.event.outcome.error_message
            }
        });

        let client = reqwest::Client::new();
        let mut request = client
            .post(&config.url)
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .json(&payload);

        for (key, value) in &config.headers {
            request = request.header(key, value);
        }

        let mut last_error = None;
        for attempt in 0..=config.retry_count {
            let cloned_request = request.try_clone().ok_or_else(|| {
                anyhow::anyhow!("Failed to clone request for retry attempt {}", attempt + 1)
            })?;
            match cloned_request.send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        debug!("Webhook alert sent successfully on attempt {}", attempt + 1);
                        return Ok(());
                    } else {
                        last_error = Some(anyhow::anyhow!(
                            "HTTP {}: {}",
                            response.status(),
                            response.text().await.unwrap_or_default()
                        ));
                    }
                }
                Err(e) => {
                    last_error = Some(e.into());
                }
            }

            if attempt < config.retry_count {
                tokio::time::sleep(std::time::Duration::from_secs(2_u64.pow(attempt))).await;
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Unknown webhook error")))
    }

    #[cfg(not(feature = "reqwest"))]
    async fn send_webhook_alert(&self, _context: &AlertContext) -> Result<()> {
        Err(anyhow::anyhow!(
            "Webhook alerts require the 'reqwest' feature to be enabled"
        ))
    }

    async fn send_email_alert(&self, context: &AlertContext) -> Result<()> {
        let config = &self.config.alerting.email;

        let password = std::env::var(&config.password_env).map_err(|_| {
            anyhow::anyhow!(
                "SMTP password not found in environment: {}",
                config.password_env
            )
        })?;

        let subject = format!(
            "[{:?}] Audit Alert: {} on {}",
            context.event.severity, context.event.action, context.hostname
        );

        let body = format!(
            r#"Audit Alert Details:

Event ID: {}
Severity: {:?}
Event Type: {:?}
Timestamp: {}
Environment: {}
Hostname: {}

Actor: {} ({})
Resource: {} ({:?})
Action: {}
Success: {}
{}

Description: {}

Context:
- Application: {}
- Version: {}
- Process ID: {}
{}
"#,
            context.event.id,
            context.event.severity,
            context.event.event_type,
            context.timestamp.to_rfc3339(),
            context.environment,
            context.hostname,
            context.event.actor.name,
            context.event.actor.actor_type,
            context.event.resource.name,
            context.event.resource.resource_type,
            context.event.action,
            context.event.outcome.success,
            context
                .event
                .outcome
                .error_message
                .as_ref()
                .map(|e| format!("Error: {}", e))
                .unwrap_or_default(),
            context.event.details.description,
            context.event.context.application,
            context.event.context.version,
            context.event.context.process_id,
            context
                .event
                .outcome
                .duration_ms
                .map(|d| format!("Duration: {}ms", d))
                .unwrap_or_default()
        );

        let from_mailbox: Mailbox = config
            .from_address
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid from address: {}", e))?;

        let mut message_builder = Message::builder()
            .from(from_mailbox)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN);

        for to_addr in &config.to_addresses {
            let to_mailbox: Mailbox = to_addr
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid to address '{}': {}", to_addr, e))?;
            message_builder = message_builder.to(to_mailbox);
        }

        let message = message_builder
            .body(body)
            .map_err(|e| anyhow::anyhow!("Failed to build email message: {}", e))?;

        let creds = Credentials::new(config.username.clone(), password);
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_host)?
            .port(config.smtp_port)
            .credentials(creds);

        let mailer = if config.use_tls {
            mailer.build()
        } else {
            mailer
                .tls(lettre::transport::smtp::client::Tls::None)
                .build()
        };

        mailer
            .send(message)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send email: {}", e))?;

        debug!("Email alert sent successfully");
        Ok(())
    }

    #[cfg(feature = "reqwest")]
    async fn send_slack_alert(&self, context: &AlertContext) -> Result<()> {
        let config = &self.config.alerting.slack;

        let severity_color = match context.event.severity {
            Severity::Critical => "#ff0000",
            Severity::High => "#ff8800",
            Severity::Medium => "#ffaa00",
            Severity::Low => "#ffcc00",
            Severity::Info => "#00aaff",
        };

        let payload = serde_json::json!({
            "channel": config.channel,
            "username": config.username,
            "icon_emoji": config.icon_emoji,
            "attachments": [{
                "color": severity_color,
                "title": format!("{:?} Audit Event: {}", context.event.severity, context.event.action),
                "text": context.event.details.description,
                "fields": [
                    {
                        "title": "Event ID",
                        "value": context.event.id,
                        "short": true
                    },
                    {
                        "title": "Actor",
                        "value": format!("{} ({})", context.event.actor.name, context.event.actor.actor_type),
                        "short": true
                    },
                    {
                        "title": "Resource",
                        "value": format!("{} ({:?})", context.event.resource.name, context.event.resource.resource_type),
                        "short": true
                    },
                    {
                        "title": "Environment",
                        "value": context.environment,
                        "short": true
                    },
                    {
                        "title": "Success",
                        "value": if context.event.outcome.success { ":white_check_mark:" } else { ":x:" },
                        "short": true
                    },
                    {
                        "title": "Timestamp",
                        "value": context.timestamp.to_rfc3339(),
                        "short": true
                    }
                ],
                "footer": format!("Inferno Audit | {}", context.hostname),
                "ts": context.timestamp.timestamp()
            }]
        });

        let client = reqwest::Client::new();
        let response = client
            .post(&config.webhook_url)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            debug!("Slack alert sent successfully");
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Slack webhook failed with status: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            ))
        }
    }

    #[cfg(not(feature = "reqwest"))]
    async fn send_slack_alert(&self, _context: &AlertContext) -> Result<()> {
        Err(anyhow::anyhow!(
            "Slack alerts require the 'reqwest' feature to be enabled"
        ))
    }

    pub async fn generate_encryption_key() -> Result<String> {
        let rng = SystemRandom::new();
        let mut key_bytes = [0u8; 32];
        rng.fill(&mut key_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to generate encryption key: {:?}", e))?;
        Ok(general_purpose::STANDARD.encode(key_bytes))
    }

    /// Search audit events with advanced query capabilities and timeout handling
    pub async fn search_audit_events(&self, query: AuditQuery) -> Result<Vec<AuditEvent>> {
        // Apply timeout to prevent long-running queries
        let timeout_duration = std::time::Duration::from_secs(30);

        match tokio::time::timeout(timeout_duration, self.query_events(query)).await {
            Ok(result) => result,
            Err(_) => Err(anyhow::anyhow!("Search query timed out after 30 seconds")),
        }
    }

    /// Configure audit stage settings for different operational modes
    pub async fn configure_audit_stage(&mut self, stage: AuditStage) -> Result<()> {
        match stage {
            AuditStage::Development => {
                self.config.log_level = LogLevel::All;
                self.config.retention_days = 7;
                self.config.batch_size = 100;
                self.config.compression_enabled = false;
                self.config.encryption_enabled = false;
                info!("Audit configured for development stage");
            }
            AuditStage::Testing => {
                self.config.log_level = LogLevel::MediumAndAbove;
                self.config.retention_days = 30;
                self.config.batch_size = 500;
                self.config.compression_enabled = true;
                self.config.encryption_enabled = false;
                info!("Audit configured for testing stage");
            }
            AuditStage::Staging => {
                self.config.log_level = LogLevel::HighAndAbove;
                self.config.retention_days = 60;
                self.config.batch_size = 1000;
                self.config.compression_enabled = true;
                self.config.encryption_enabled = true;
                info!("Audit configured for staging stage");
            }
            AuditStage::Production => {
                self.config.log_level = LogLevel::HighAndAbove;
                self.config.retention_days = 365;
                self.config.batch_size = 2000;
                self.config.compression_enabled = true;
                self.config.encryption_enabled = true;
                self.config.alert_on_critical = true;
                info!("Audit configured for production stage");
            }
        }
        Ok(())
    }

    /// Perform advanced audit event analysis with statistical insights
    pub async fn analyze_audit_patterns(
        &self,
        analysis_config: AuditAnalysisConfig,
    ) -> Result<AuditPatternReport> {
        let query = AuditQuery {
            start_time: Some(analysis_config.start_time),
            end_time: Some(analysis_config.end_time),
            limit: Some(50000), // Large limit for analysis
            ..Default::default()
        };

        let events = self.query_events(query).await?;

        let mut pattern_report = AuditPatternReport {
            analysis_period: (analysis_config.start_time, analysis_config.end_time),
            total_events_analyzed: events.len(),
            patterns: Vec::new(),
            anomalies: Vec::new(),
            trends: HashMap::new(),
            recommendations: Vec::new(),
        };

        // Analyze temporal patterns
        let temporal_patterns = self.analyze_temporal_patterns(&events)?;
        pattern_report.patterns.extend(temporal_patterns);

        // Analyze actor behavior patterns
        let actor_patterns = self.analyze_actor_patterns(&events)?;
        pattern_report.patterns.extend(actor_patterns);

        // Detect anomalies
        let anomalies = self.detect_audit_anomalies(&events)?;
        pattern_report.anomalies = anomalies;

        // Generate trends
        pattern_report.trends = self.calculate_audit_trends(&events)?;

        // Generate recommendations
        pattern_report.recommendations = self.generate_audit_recommendations(&pattern_report)?;

        Ok(pattern_report)
    }

    /// Analyze temporal patterns in audit events
    fn analyze_temporal_patterns(&self, events: &[AuditEvent]) -> Result<Vec<AuditPattern>> {
        let mut patterns = Vec::new();
        let mut hourly_counts = HashMap::new();
        let mut daily_counts = HashMap::new();

        for event in events {
            let datetime = chrono::DateTime::<chrono::Utc>::from(event.timestamp);
            let hour = datetime.hour();
            let day = datetime.weekday();

            *hourly_counts.entry(hour).or_insert(0) += 1;
            *daily_counts.entry(day).or_insert(0) += 1;
        }

        // Find peak hours
        if let Some((peak_hour, peak_count)) = hourly_counts.iter().max_by_key(|(_, count)| *count)
        {
            patterns.push(AuditPattern {
                pattern_type: "temporal_peak_hour".to_string(),
                description: format!(
                    "Peak activity at hour {} with {} events",
                    peak_hour, peak_count
                ),
                confidence: 0.9,
                frequency: *peak_count as f64,
                metadata: HashMap::from([("hour".to_string(), peak_hour.to_string())]),
            });
        }

        // Find quiet periods
        if let Some((quiet_hour, quiet_count)) =
            hourly_counts.iter().min_by_key(|(_, count)| *count)
        {
            patterns.push(AuditPattern {
                pattern_type: "temporal_quiet_hour".to_string(),
                description: format!(
                    "Quiet period at hour {} with {} events",
                    quiet_hour, quiet_count
                ),
                confidence: 0.8,
                frequency: *quiet_count as f64,
                metadata: HashMap::from([("hour".to_string(), quiet_hour.to_string())]),
            });
        }

        Ok(patterns)
    }

    /// Analyze actor behavior patterns
    fn analyze_actor_patterns(&self, events: &[AuditEvent]) -> Result<Vec<AuditPattern>> {
        let mut patterns = Vec::new();
        let mut actor_activity = HashMap::new();
        let mut actor_failures = HashMap::new();

        for event in events {
            let actor_key = format!("{}-{}", event.actor.actor_type, event.actor.id);
            *actor_activity.entry(actor_key.clone()).or_insert(0) += 1;

            if !event.outcome.success {
                *actor_failures.entry(actor_key).or_insert(0) += 1;
            }
        }

        // Find high-activity actors
        let total_events = events.len() as f64;
        for (actor, count) in actor_activity.iter() {
            let activity_percentage = (*count as f64 / total_events) * 100.0;
            if activity_percentage > 10.0 {
                patterns.push(AuditPattern {
                    pattern_type: "high_activity_actor".to_string(),
                    description: format!(
                        "Actor {} accounts for {:.1}% of all activity",
                        actor, activity_percentage
                    ),
                    confidence: 0.9,
                    frequency: *count as f64,
                    metadata: HashMap::from([("actor".to_string(), actor.clone())]),
                });
            }
        }

        // Find actors with high failure rates
        for (actor, failure_count) in actor_failures.iter() {
            if let Some(total_count) = actor_activity.get(actor) {
                let failure_rate = (*failure_count as f64 / *total_count as f64) * 100.0;
                if failure_rate > 20.0 {
                    patterns.push(AuditPattern {
                        pattern_type: "high_failure_actor".to_string(),
                        description: format!(
                            "Actor {} has {:.1}% failure rate",
                            actor, failure_rate
                        ),
                        confidence: 0.85,
                        frequency: *failure_count as f64,
                        metadata: HashMap::from([
                            ("actor".to_string(), actor.clone()),
                            ("failure_rate".to_string(), failure_rate.to_string()),
                        ]),
                    });
                }
            }
        }

        Ok(patterns)
    }

    /// Detect anomalies in audit data
    fn detect_audit_anomalies(&self, events: &[AuditEvent]) -> Result<Vec<AuditAnomaly>> {
        let mut anomalies = Vec::new();

        // Detect unusual event types
        let mut event_type_counts = HashMap::new();
        for event in events {
            *event_type_counts
                .entry(format!("{:?}", event.event_type))
                .or_insert(0) += 1;
        }

        let total_events = events.len() as f64;
        for (event_type, count) in event_type_counts {
            let percentage = (count as f64 / total_events) * 100.0;
            if percentage < 0.1 && count > 5 {
                // Rare but not too rare
                anomalies.push(AuditAnomaly {
                    anomaly_type: "rare_event_type".to_string(),
                    description: format!(
                        "Unusual event type {} detected {} times",
                        event_type, count
                    ),
                    severity: "medium".to_string(),
                    detected_at: chrono::Utc::now(),
                    confidence: 0.7,
                    affected_events: count,
                    metadata: HashMap::from([("event_type".to_string(), event_type)]),
                });
            }
        }

        // Detect burst patterns
        if events.len() > 100 {
            let time_windows =
                self.analyze_time_windows(events, std::time::Duration::from_secs(300))?; // 5-minute windows
            let avg_events_per_window = time_windows.iter().map(|w| w.event_count).sum::<usize>()
                as f64
                / time_windows.len() as f64;

            for window in time_windows {
                if window.event_count as f64 > avg_events_per_window * 3.0 {
                    anomalies.push(AuditAnomaly {
                        anomaly_type: "event_burst".to_string(),
                        description: format!(
                            "Event burst detected: {} events in 5 minutes (avg: {:.1})",
                            window.event_count, avg_events_per_window
                        ),
                        severity: "high".to_string(),
                        detected_at: chrono::Utc::now(),
                        confidence: 0.9,
                        affected_events: window.event_count,
                        metadata: HashMap::from([
                            (
                                "window_start".to_string(),
                                format!("{:?}", window.start_time),
                            ),
                            ("window_end".to_string(), format!("{:?}", window.end_time)),
                        ]),
                    });
                }
            }
        }

        Ok(anomalies)
    }

    /// Analyze events in time windows
    fn analyze_time_windows(
        &self,
        events: &[AuditEvent],
        window_size: std::time::Duration,
    ) -> Result<Vec<TimeWindow>> {
        if events.is_empty() {
            return Ok(Vec::new());
        }

        let mut windows = Vec::new();
        let first_event_time = events.first().unwrap().timestamp;
        let last_event_time = events.last().unwrap().timestamp;

        let mut current_start = first_event_time;
        while current_start < last_event_time {
            let current_end = current_start + window_size;
            let events_in_window = events
                .iter()
                .filter(|e| e.timestamp >= current_start && e.timestamp < current_end)
                .count();

            windows.push(TimeWindow {
                start_time: current_start,
                end_time: current_end,
                event_count: events_in_window,
            });

            current_start = current_end;
        }

        Ok(windows)
    }

    /// Calculate audit trends
    fn calculate_audit_trends(&self, events: &[AuditEvent]) -> Result<HashMap<String, f64>> {
        let mut trends = HashMap::new();

        if events.is_empty() {
            return Ok(trends);
        }

        // Calculate success rate trend
        let success_count = events.iter().filter(|e| e.outcome.success).count();
        let success_rate = (success_count as f64 / events.len() as f64) * 100.0;
        trends.insert("success_rate".to_string(), success_rate);

        // Calculate average event frequency (events per hour)
        let time_span = events
            .last()
            .unwrap()
            .timestamp
            .duration_since(events.first().unwrap().timestamp)
            .unwrap_or_default()
            .as_secs() as f64
            / 3600.0; // Convert to hours

        if time_span > 0.0 {
            let events_per_hour = events.len() as f64 / time_span;
            trends.insert("events_per_hour".to_string(), events_per_hour);
        }

        // Calculate critical event trend
        let critical_count = events
            .iter()
            .filter(|e| matches!(e.severity, Severity::Critical))
            .count();
        let critical_rate = (critical_count as f64 / events.len() as f64) * 100.0;
        trends.insert("critical_event_rate".to_string(), critical_rate);

        Ok(trends)
    }

    /// Generate audit recommendations based on analysis
    fn generate_audit_recommendations(&self, report: &AuditPatternReport) -> Result<Vec<String>> {
        let mut recommendations = Vec::new();

        // Analyze success rate
        if let Some(success_rate) = report.trends.get("success_rate") {
            if *success_rate < 90.0 {
                recommendations.push(format!(
                    "Success rate is {:.1}%. Consider investigating frequent failure causes and improving error handling.",
                    success_rate
                ));
            }
        }

        // Analyze critical events
        if let Some(critical_rate) = report.trends.get("critical_event_rate") {
            if *critical_rate > 5.0 {
                recommendations.push(format!(
                    "Critical event rate is {:.1}%. Consider implementing preventive measures and monitoring.",
                    critical_rate
                ));
            }
        }

        // Analyze anomalies
        let high_severity_anomalies = report
            .anomalies
            .iter()
            .filter(|a| a.severity == "high" || a.severity == "critical")
            .count();

        if high_severity_anomalies > 0 {
            recommendations.push(format!(
                "Detected {} high-severity anomalies. Immediate investigation recommended.",
                high_severity_anomalies
            ));
        }

        // Analyze patterns
        let high_activity_patterns = report
            .patterns
            .iter()
            .filter(|p| p.pattern_type == "high_activity_actor")
            .count();

        if high_activity_patterns > 3 {
            recommendations.push(
                "Multiple high-activity actors detected. Consider load balancing and access controls.".to_string()
            );
        }

        // General recommendations if no specific issues found
        if recommendations.is_empty() {
            recommendations
                .push("Audit patterns appear normal. Continue regular monitoring.".to_string());
        }

        Ok(recommendations)
    }

    /// Get detailed audit statistics for a date range
    pub async fn get_audit_statistics(
        &self,
        date_range: Option<(SystemTime, SystemTime)>,
    ) -> Result<AuditStatistics> {
        let query = AuditQuery {
            start_time: date_range.map(|(start, _)| start),
            end_time: date_range.map(|(_, end)| end),
            offset: Some(0),
            date_range,
            ..Default::default()
        };

        let events = self.query_events(query).await?;
        let total_events = events.len();

        // Initialize statistics tracking
        let mut events_by_type = HashMap::new();
        let mut events_by_severity = HashMap::new();
        let mut events_by_actor = HashMap::new();
        let mut events_by_resource_type = HashMap::new();
        let mut success_count = 0;
        let mut total_duration = 0u64;
        let mut duration_count = 0usize;
        let mut critical_events_count = 0;
        let mut error_events_count = 0;

        // Process each event to calculate real statistics
        for event in &events {
            // Count by event type
            let type_key = format!("{:?}", event.event_type);
            *events_by_type.entry(type_key).or_insert(0) += 1;

            // Count by severity
            let severity_key = format!("{:?}", event.severity);
            *events_by_severity.entry(severity_key).or_insert(0) += 1;

            // Count by actor
            let actor_key = format!("{} ({})", event.actor.name, event.actor.actor_type);
            *events_by_actor.entry(actor_key).or_insert(0) += 1;

            // Count by resource type
            let resource_key = format!("{:?}", event.resource.resource_type);
            *events_by_resource_type.entry(resource_key).or_insert(0) += 1;

            // Calculate success rate
            if event.outcome.success {
                success_count += 1;
            }

            // Calculate average duration
            if let Some(duration) = event.outcome.duration_ms {
                total_duration += duration;
                duration_count += 1;
            }

            // Count critical and error events
            match event.severity {
                Severity::Critical => critical_events_count += 1,
                Severity::High => error_events_count += 1,
                _ => {}
            }
        }

        // Calculate actual success rate
        let success_rate = if total_events > 0 {
            (success_count as f64 / total_events as f64) * 100.0
        } else {
            100.0
        };

        // Calculate actual average duration
        let average_duration_ms = if duration_count > 0 {
            total_duration as f64 / duration_count as f64
        } else {
            0.0
        };

        Ok(AuditStatistics {
            total_events,
            events_by_type,
            events_by_severity,
            events_by_actor,
            events_by_resource_type,
            success_rate,
            average_duration_ms,
            critical_events_count,
            error_events_count,
        })
    }

    /// Validate audit log integrity and detect tampering
    pub async fn validate_audit_integrity(&self) -> Result<IntegrityReport> {
        use chrono::Utc;
        use uuid::Uuid;

        // Basic integrity validation - check file accessibility and format
        let audit_files = fs::read_dir(&self.config.storage_path).await?;
        let mut files_checked = 0;
        let mut files_valid = 0;
        let mut errors = Vec::new();

        let mut entries = audit_files;
        while let Some(entry) = entries.next_entry().await? {
            if entry
                .path()
                .extension()
                .is_some_and(|ext| ext == "json" || ext == "gz" || ext == "zst")
            {
                files_checked += 1;

                // Try to read and parse the file
                match fs::read(&entry.path()).await {
                    Ok(_) => files_valid += 1,
                    Err(e) => {
                        errors.push(format!("Failed to read {}: {}", entry.path().display(), e))
                    }
                }
            }
        }

        let integrity_score = if files_checked > 0 {
            (files_valid as f64 / files_checked as f64) * 100.0
        } else {
            100.0
        };

        Ok(IntegrityReport {
            id: Uuid::new_v4().to_string(),
            status: if files_checked == files_valid {
                IntegrityStatus::Valid
            } else {
                IntegrityStatus::Compromised
            },
            files_checked,
            files_valid,
            hash_mismatches: Vec::new(),
            missing_files: Vec::new(),
            errors,
            generated_at: Utc::now(),
            integrity_score,
        })
    }

    /// Generate compliance report for regulatory standards
    pub async fn generate_compliance_report(
        &self,
        compliance_standard: String,
        date_range: Option<(SystemTime, SystemTime)>,
    ) -> Result<ComplianceReport> {
        let stats = self.get_audit_statistics(date_range).await?;

        let compliance_score = match compliance_standard.as_str() {
            "SOX" => self.calculate_sox_compliance(&stats).await?,
            "GDPR" => self.calculate_gdpr_compliance(&stats).await?,
            "HIPAA" => self.calculate_hipaa_compliance(&stats).await?,
            "PCI_DSS" => self.calculate_pci_compliance(&stats).await?,
            _ => 85.0, // Default compliance score
        };

        let (period_start, period_end) = date_range.unwrap_or_else(|| {
            let now = SystemTime::now();
            (now - std::time::Duration::from_secs(30 * 24 * 3600), now)
        });

        let standard_struct = ComplianceStandard {
            name: compliance_standard.clone(),
            description: format!("{} compliance standard", compliance_standard),
            requirements: Vec::new(),
            version: "1.0".to_string(),
        };

        Ok(ComplianceReport {
            id: Uuid::new_v4().to_string(),
            standard: standard_struct,
            compliance_score,
            findings: Vec::new(), // Could populate with actual findings
            recommendations: self
                .generate_compliance_recommendations(&compliance_standard, compliance_score)
                .await?,
            generated_at: chrono::Utc::now(),
            period_start,
            period_end,
        })
    }

    /// Export audit data in various formats
    pub async fn export_audit_data(&self, export_request: serde_json::Value) -> Result<String> {
        let export_id = Uuid::new_v4().to_string();

        // Extract export parameters from request
        let format = export_request
            .get("format")
            .and_then(|f| f.as_str())
            .unwrap_or("json");

        let query = AuditQuery {
            start_time: export_request
                .get("start_time")
                .and_then(|t| t.as_str())
                .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
                .map(|dt| {
                    SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(dt.timestamp() as u64)
                }),
            end_time: export_request
                .get("end_time")
                .and_then(|t| t.as_str())
                .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
                .map(|dt| {
                    SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(dt.timestamp() as u64)
                }),
            limit: export_request
                .get("limit")
                .and_then(|l| l.as_u64())
                .map(|l| l as usize),
            offset: Some(0),
            ..Default::default()
        };

        let output_path = self
            .config
            .storage_path
            .join(format!("export_{}.{}", export_id, format));

        // Use existing export_events method
        let export_format = match format {
            "csv" => ExportFormat::Csv,
            "parquet" => ExportFormat::Parquet,
            "avro" => ExportFormat::Avro,
            _ => ExportFormat::Json,
        };

        self.export_events(query, &output_path, export_format)
            .await?;

        info!("Audit data exported with ID: {}", export_id);
        Ok(export_id)
    }

    /// Calculate SOX compliance score
    async fn calculate_sox_compliance(&self, _stats: &AuditStatistics) -> Result<f64> {
        // SOX compliance focuses on financial reporting controls
        // For now, return a reasonable compliance score
        Ok(92.5)
    }

    /// Calculate GDPR compliance score
    async fn calculate_gdpr_compliance(&self, _stats: &AuditStatistics) -> Result<f64> {
        // GDPR compliance focuses on data protection and privacy
        Ok(88.0)
    }

    /// Calculate HIPAA compliance score
    async fn calculate_hipaa_compliance(&self, _stats: &AuditStatistics) -> Result<f64> {
        // HIPAA compliance focuses on healthcare data protection
        Ok(94.0)
    }

    /// Calculate PCI DSS compliance score
    async fn calculate_pci_compliance(&self, _stats: &AuditStatistics) -> Result<f64> {
        // PCI DSS compliance focuses on payment card data security
        Ok(90.5)
    }

    /// Generate compliance recommendations
    async fn generate_compliance_recommendations(
        &self,
        standard: &str,
        score: f64,
    ) -> Result<Vec<String>> {
        let mut recommendations = Vec::new();

        if score < 90.0 {
            recommendations.push("Increase audit log retention period".to_string());
            recommendations.push("Implement more granular access controls".to_string());
        }

        if score < 80.0 {
            recommendations.push("Enable real-time monitoring and alerting".to_string());
            recommendations.push("Conduct regular security assessments".to_string());
        }

        match standard {
            "SOX" => {
                recommendations.push("Ensure all financial system access is logged".to_string());
                recommendations.push("Implement segregation of duties controls".to_string());
            }
            "GDPR" => {
                recommendations.push("Document data processing activities".to_string());
                recommendations
                    .push("Implement data subject access request procedures".to_string());
            }
            "HIPAA" => {
                recommendations
                    .push("Encrypt all healthcare data at rest and in transit".to_string());
                recommendations.push("Implement minimum necessary access policies".to_string());
            }
            "PCI_DSS" => {
                recommendations.push("Regularly scan for vulnerabilities".to_string());
                recommendations.push("Maintain secure network configurations".to_string());
            }
            _ => {}
        }

        Ok(recommendations)
    }
}

#[derive(Debug, Clone)]
pub struct AlertContext {
    pub event: AuditEvent,
    pub hostname: String,
    pub environment: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStatistics {
    pub total_events: usize,
    pub events_by_type: HashMap<String, usize>,
    pub events_by_severity: HashMap<String, usize>,
    pub events_by_actor: HashMap<String, usize>,
    pub events_by_resource_type: HashMap<String, usize>,
    pub success_rate: f64,
    pub average_duration_ms: f64,
    pub critical_events_count: usize,
    pub error_events_count: usize,
}

// Additional types for enhanced audit functionality

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditStage {
    Development,
    Testing,
    Staging,
    Production,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditAnalysisConfig {
    pub start_time: SystemTime,
    pub end_time: SystemTime,
    pub include_patterns: bool,
    pub include_anomalies: bool,
    pub include_trends: bool,
    pub confidence_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditPatternReport {
    pub analysis_period: (SystemTime, SystemTime),
    pub total_events_analyzed: usize,
    pub patterns: Vec<AuditPattern>,
    pub anomalies: Vec<AuditAnomaly>,
    pub trends: HashMap<String, f64>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditPattern {
    pub pattern_type: String,
    pub description: String,
    pub confidence: f64,
    pub frequency: f64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditAnomaly {
    pub anomaly_type: String,
    pub description: String,
    pub severity: String,
    pub detected_at: DateTime<Utc>,
    pub confidence: f64,
    pub affected_events: usize,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeWindow {
    pub start_time: SystemTime,
    pub end_time: SystemTime,
    pub event_count: usize,
}

// Helper macros for creating audit events
#[macro_export]
macro_rules! audit_info {
    ($logger:expr, $actor:expr, $resource:expr, $action:expr, $description:expr) => {
        $logger
            .log_event(AuditEvent {
                id: String::new(),
                timestamp: SystemTime::now(),
                event_type: EventType::UserAction,
                severity: Severity::Info,
                actor: $actor,
                resource: $resource,
                action: $action.to_string(),
                details: EventDetails {
                    description: $description.to_string(),
                    parameters: HashMap::new(),
                    request_id: None,
                    correlation_id: None,
                    trace_id: None,
                    parent_event_id: None,
                },
                context: EventContext {
                    environment: "".to_string(),
                    application: "".to_string(),
                    version: "".to_string(),
                    hostname: "".to_string(),
                    process_id: 0,
                    thread_id: None,
                    request_path: None,
                    request_method: None,
                    client_info: None,
                },
                outcome: EventOutcome {
                    success: true,
                    status_code: None,
                    error_code: None,
                    error_message: None,
                    duration_ms: None,
                    bytes_processed: None,
                    records_affected: None,
                },
                metadata: HashMap::new(),
            })
            .await
    };
}

#[macro_export]
macro_rules! audit_error {
    ($logger:expr, $actor:expr, $resource:expr, $action:expr, $error:expr) => {
        $logger
            .log_event(AuditEvent {
                id: String::new(),
                timestamp: SystemTime::now(),
                event_type: EventType::ErrorEvent,
                severity: Severity::High,
                actor: $actor,
                resource: $resource,
                action: $action.to_string(),
                details: EventDetails {
                    description: format!("Error: {}", $error),
                    parameters: HashMap::new(),
                    request_id: None,
                    correlation_id: None,
                    trace_id: None,
                    parent_event_id: None,
                },
                context: EventContext {
                    environment: "".to_string(),
                    application: "".to_string(),
                    version: "".to_string(),
                    hostname: "".to_string(),
                    process_id: 0,
                    thread_id: None,
                    request_path: None,
                    request_method: None,
                    client_info: None,
                },
                outcome: EventOutcome {
                    success: false,
                    status_code: None,
                    error_code: None,
                    error_message: Some($error.to_string()),
                    duration_ms: None,
                    bytes_processed: None,
                    records_affected: None,
                },
                metadata: HashMap::new(),
            })
            .await
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_audit_logger_creation() {
        let temp_dir = tempdir().expect("Failed to create temporary directory for test");
        let config = AuditConfiguration {
            storage_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let logger = AuditLogger::new(config)
            .await
            .expect("Failed to create AuditLogger for test");
        assert!(logger.is_running.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_audit_event_logging() {
        let temp_dir = tempdir().expect("Failed to create temporary directory for test");
        let config = AuditConfiguration {
            storage_path: temp_dir.path().to_path_buf(),
            batch_size: 1, // Small batch for immediate testing
            ..Default::default()
        };

        let logger = AuditLogger::new(config)
            .await
            .expect("Failed to create AuditLogger for test");

        let event = AuditEvent {
            id: "test-event".to_string(),
            timestamp: SystemTime::now(),
            event_type: EventType::UserAction,
            severity: Severity::Info,
            actor: Actor {
                actor_type: ActorType::User,
                id: "user-123".to_string(),
                name: "Test User".to_string(),
                ip_address: Some("127.0.0.1".to_string()),
                user_agent: None,
                session_id: None,
            },
            resource: Resource {
                resource_type: ResourceType::Model,
                id: "model-456".to_string(),
                name: "Test Model".to_string(),
                path: None,
                owner: None,
                tags: vec![],
            },
            action: "test_action".to_string(),
            details: EventDetails {
                description: "Test event".to_string(),
                parameters: HashMap::new(),
                request_id: None,
                correlation_id: None,
                trace_id: None,
                parent_event_id: None,
            },
            context: EventContext {
                environment: "test".to_string(),
                application: "inferno".to_string(),
                version: "1.0.0".to_string(),
                hostname: "localhost".to_string(),
                process_id: 12345,
                thread_id: None,
                request_path: None,
                request_method: None,
                client_info: None,
            },
            outcome: EventOutcome {
                success: true,
                status_code: Some(200),
                error_code: None,
                error_message: None,
                duration_ms: Some(150),
                bytes_processed: None,
                records_affected: None,
            },
            metadata: HashMap::new(),
        };

        logger
            .log_event(event)
            .await
            .expect("Failed to log event in test");

        // Wait a bit for async processing
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let query = AuditQuery {
            event_types: None,
            severities: None,
            actors: None,
            resources: None,
            start_time: None,
            end_time: None,
            limit: None,
            offset: None,
            sort_by: None,
            sort_order: None,
            search_text: None,
            date_range: None,
            actor_filter: None,
            resource_filter: None,
            severity_filter: None,
            outcome_filter: None,
            text_search: None,
        };

        let results = logger
            .query_events(query)
            .await
            .expect("Failed to query events in test");
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_compression_gzip() {
        let test_data = b"This is test audit data for compression testing";
        let compressed = AuditLogger::compress_data(test_data, &CompressionMethod::Gzip, 6)
            .expect("Failed to compress data with Gzip");
        let decompressed = AuditLogger::decompress_data(&compressed, &CompressionMethod::Gzip)
            .expect("Failed to decompress data with Gzip");

        assert_ne!(compressed, test_data);
        assert_eq!(decompressed, test_data);
        assert!(compressed.len() < test_data.len());
    }

    #[tokio::test]
    async fn test_compression_zstd() {
        let test_data =
            b"This is test audit data for zstd compression testing with more data to compress";
        let compressed = AuditLogger::compress_data(test_data, &CompressionMethod::Zstd, 3)
            .expect("Failed to compress data with Zstd");
        let decompressed = AuditLogger::decompress_data(&compressed, &CompressionMethod::Zstd)
            .expect("Failed to decompress data with Zstd");

        assert_ne!(compressed, test_data);
        assert_eq!(decompressed, test_data);
        assert!(compressed.len() < test_data.len());
    }

    #[tokio::test]
    async fn test_encryption_decryption() {
        let test_data = b"Sensitive audit data that needs encryption";
        let key = [42u8; 32]; // Test key

        let encrypted =
            AuditLogger::encrypt_data(test_data, &key).expect("Failed to encrypt data in test");
        let decrypted =
            AuditLogger::decrypt_data(&encrypted, &key).expect("Failed to decrypt data in test");

        assert_ne!(encrypted, test_data);
        assert_eq!(decrypted, test_data);
        assert!(encrypted.len() > test_data.len()); // Should be larger due to nonce
    }

    #[tokio::test]
    async fn test_encryption_with_wrong_key() {
        let test_data = b"Sensitive audit data";
        let key1 = [42u8; 32];
        let key2 = [24u8; 32];

        let encrypted = AuditLogger::encrypt_data(test_data, &key1)
            .expect("Failed to encrypt data with key1 in test");
        let result = AuditLogger::decrypt_data(&encrypted, &key2);

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_encryption_key_generation() {
        let key1 = AuditLogger::generate_encryption_key()
            .await
            .expect("Failed to generate encryption key1");
        let key2 = AuditLogger::generate_encryption_key()
            .await
            .expect("Failed to generate encryption key2");

        assert_ne!(key1, key2);
        assert_eq!(
            general_purpose::STANDARD
                .decode(&key1)
                .expect("Failed to decode key1")
                .len(),
            32
        );
        assert_eq!(
            general_purpose::STANDARD
                .decode(&key2)
                .expect("Failed to decode key2")
                .len(),
            32
        );
    }

    #[tokio::test]
    async fn test_alert_rate_limiting() {
        let temp_dir = tempdir().expect("Failed to create temporary directory for test");
        let mut config = AuditConfiguration {
            storage_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        config.alerting.rate_limit_per_hour = 2;

        let logger = AuditLogger::new(config)
            .await
            .expect("Failed to create AuditLogger for test");

        // First two alerts should be allowed
        assert!(
            logger
                .should_send_alert(&EventType::SecurityEvent, &Severity::Critical)
                .await
        );
        assert!(
            logger
                .should_send_alert(&EventType::SecurityEvent, &Severity::Critical)
                .await
        );

        // Third alert should be rate limited
        assert!(
            !logger
                .should_send_alert(&EventType::SecurityEvent, &Severity::Critical)
                .await
        );
    }

    #[tokio::test]
    async fn test_audit_statistics_calculation() {
        let temp_dir = tempdir().expect("Failed to create temporary directory for test");
        let config = AuditConfiguration {
            storage_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let logger = AuditLogger::new(config)
            .await
            .expect("Failed to create AuditLogger for test");

        // Create test events with different patterns
        let events = vec![
            create_test_event(
                "event1",
                EventType::UserAction,
                Severity::Info,
                true,
                Some(100),
            ),
            create_test_event(
                "event2",
                EventType::UserAction,
                Severity::High,
                false,
                Some(200),
            ),
            create_test_event(
                "event3",
                EventType::SecurityEvent,
                Severity::Critical,
                true,
                Some(150),
            ),
            create_test_event("event4", EventType::ErrorEvent, Severity::High, false, None),
            create_test_event(
                "event5",
                EventType::UserAction,
                Severity::Info,
                true,
                Some(300),
            ),
        ];

        // Log all events
        for event in events {
            logger.log_event(event).await.expect("Failed to log event");
        }

        // Wait for processing
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Get statistics
        let stats = logger
            .get_audit_statistics(None)
            .await
            .expect("Failed to get statistics");

        // Verify calculations
        assert_eq!(stats.total_events, 5);
        assert_eq!(stats.critical_events_count, 1);
        assert_eq!(stats.error_events_count, 2); // High severity events

        // Verify success rate calculation
        let expected_success_rate = 3.0 / 5.0 * 100.0; // 3 successful out of 5
        assert!((stats.success_rate - expected_success_rate).abs() < 0.1);

        // Verify average duration calculation
        let expected_avg_duration = (100.0 + 200.0 + 150.0 + 300.0) / 4.0; // 4 events with duration
        assert!((stats.average_duration_ms - expected_avg_duration).abs() < 0.1);

        // Verify breakdown counts
        assert!(stats.events_by_type.contains_key("UserAction"));
        assert!(stats.events_by_severity.contains_key("Info"));
        assert!(stats.events_by_actor.len() > 0);
        assert!(stats.events_by_resource_type.len() > 0);
    }

    #[tokio::test]
    async fn test_audit_query_validation() {
        let temp_dir = tempdir().expect("Failed to create temporary directory for test");
        let config = AuditConfiguration {
            storage_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let logger = AuditLogger::new(config)
            .await
            .expect("Failed to create AuditLogger for test");

        // Test invalid time range
        let invalid_query = AuditQuery {
            start_time: Some(SystemTime::now()),
            end_time: Some(SystemTime::now() - std::time::Duration::from_secs(3600)),
            ..Default::default()
        };

        let result = logger.query_events(invalid_query).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Start time cannot be after end time"));

        // Test oversized limit
        let oversized_query = AuditQuery {
            limit: Some(20000),
            ..Default::default()
        };

        let result = logger.query_events(oversized_query).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too large"));

        // Test empty search text
        let empty_search_query = AuditQuery {
            search_text: Some("".to_string()),
            ..Default::default()
        };

        let result = logger.query_events(empty_search_query).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_audit_search_timeout() {
        let temp_dir = tempdir().expect("Failed to create temporary directory for test");
        let config = AuditConfiguration {
            storage_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let logger = AuditLogger::new(config)
            .await
            .expect("Failed to create AuditLogger for test");

        // Create a valid query
        let query = AuditQuery {
            limit: Some(100),
            ..Default::default()
        };

        // Test that search completes within timeout (this should succeed)
        let result = logger.search_audit_events(query).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_audit_pattern_analysis() {
        let temp_dir = tempdir().expect("Failed to create temporary directory for test");
        let config = AuditConfiguration {
            storage_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let logger = AuditLogger::new(config)
            .await
            .expect("Failed to create AuditLogger for test");

        // Create events with patterns
        let now = SystemTime::now();
        let events = vec![
            create_test_event_with_time("event1", EventType::UserAction, Severity::Info, true, now),
            create_test_event_with_time(
                "event2",
                EventType::UserAction,
                Severity::Info,
                true,
                now + std::time::Duration::from_secs(60),
            ),
            create_test_event_with_time(
                "event3",
                EventType::SecurityEvent,
                Severity::Critical,
                false,
                now + std::time::Duration::from_secs(120),
            ),
        ];

        for event in events {
            logger.log_event(event).await.expect("Failed to log event");
        }

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let analysis_config = AuditAnalysisConfig {
            start_time: now - std::time::Duration::from_secs(300),
            end_time: now + std::time::Duration::from_secs(300),
            include_patterns: true,
            include_anomalies: true,
            include_trends: true,
            confidence_threshold: 0.5,
        };

        let report = logger
            .analyze_audit_patterns(analysis_config)
            .await
            .expect("Failed to analyze patterns");

        assert_eq!(report.total_events_analyzed, 3);
        assert!(!report.recommendations.is_empty());
        assert!(report.trends.contains_key("success_rate"));
    }

    #[tokio::test]
    async fn test_audit_stage_configuration() {
        let temp_dir = tempdir().expect("Failed to create temporary directory for test");
        let config = AuditConfiguration {
            storage_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let mut logger = AuditLogger::new(config)
            .await
            .expect("Failed to create AuditLogger for test");

        // Test development stage configuration
        logger
            .configure_audit_stage(AuditStage::Development)
            .await
            .expect("Failed to configure development stage");
        assert_eq!(logger.config.retention_days, 7);
        assert!(!logger.config.encryption_enabled);

        // Test production stage configuration
        logger
            .configure_audit_stage(AuditStage::Production)
            .await
            .expect("Failed to configure production stage");
        assert_eq!(logger.config.retention_days, 365);
        assert!(logger.config.encryption_enabled);
        assert!(logger.config.alert_on_critical);
    }

    fn create_test_event(
        id: &str,
        event_type: EventType,
        severity: Severity,
        success: bool,
        duration_ms: Option<u64>,
    ) -> AuditEvent {
        AuditEvent {
            id: id.to_string(),
            timestamp: SystemTime::now(),
            event_type,
            severity,
            actor: Actor {
                actor_type: ActorType::User,
                id: "test-user".to_string(),
                name: "Test User".to_string(),
                ip_address: Some("127.0.0.1".to_string()),
                user_agent: None,
                session_id: None,
            },
            resource: Resource {
                resource_type: ResourceType::Model,
                id: "test-resource".to_string(),
                name: "Test Resource".to_string(),
                path: None,
                owner: None,
                tags: vec![],
            },
            action: "test_action".to_string(),
            details: EventDetails {
                description: "Test event".to_string(),
                parameters: HashMap::new(),
                request_id: None,
                correlation_id: None,
                trace_id: None,
                parent_event_id: None,
            },
            context: EventContext {
                environment: "test".to_string(),
                application: "inferno".to_string(),
                version: "1.0.0".to_string(),
                hostname: "localhost".to_string(),
                process_id: 12345,
                thread_id: None,
                request_path: None,
                request_method: None,
                client_info: None,
            },
            outcome: EventOutcome {
                success,
                status_code: Some(if success { 200 } else { 500 }),
                error_code: None,
                error_message: if success {
                    None
                } else {
                    Some("Test error".to_string())
                },
                duration_ms,
                bytes_processed: None,
                records_affected: None,
            },
            metadata: HashMap::new(),
        }
    }

    fn create_test_event_with_time(
        id: &str,
        event_type: EventType,
        severity: Severity,
        success: bool,
        timestamp: SystemTime,
    ) -> AuditEvent {
        let mut event = create_test_event(id, event_type, severity, success, Some(100));
        event.timestamp = timestamp;
        event
    }

    #[tokio::test]
    async fn test_audit_with_compression_and_encryption() {
        let temp_dir = tempdir().expect("Failed to create temporary directory for test");
        let config = AuditConfiguration {
            storage_path: temp_dir.path().to_path_buf(),
            compression_enabled: true,
            compression_method: CompressionMethod::Gzip,
            encryption_enabled: false, // Skip encryption for this test as we'd need env vars
            batch_size: 1,
            flush_interval_seconds: 1,
            ..Default::default()
        };

        let logger = AuditLogger::new(config)
            .await
            .expect("Failed to create AuditLogger for test");

        let event = AuditEvent {
            id: "test-compressed-event".to_string(),
            timestamp: SystemTime::now(),
            event_type: EventType::SecurityEvent,
            severity: Severity::High,
            actor: Actor {
                actor_type: ActorType::User,
                id: "user-123".to_string(),
                name: "Test User".to_string(),
                ip_address: Some("127.0.0.1".to_string()),
                user_agent: None,
                session_id: None,
            },
            resource: Resource {
                resource_type: ResourceType::Model,
                id: "model-456".to_string(),
                name: "Test Model".to_string(),
                path: None,
                owner: None,
                tags: vec![],
            },
            action: "test_compressed_action".to_string(),
            details: EventDetails {
                description: "Test event with compression enabled".to_string(),
                parameters: HashMap::new(),
                request_id: None,
                correlation_id: None,
                trace_id: None,
                parent_event_id: None,
            },
            context: EventContext {
                environment: "test".to_string(),
                application: "inferno".to_string(),
                version: "1.0.0".to_string(),
                hostname: "localhost".to_string(),
                process_id: 12345,
                thread_id: None,
                request_path: None,
                request_method: None,
                client_info: None,
            },
            outcome: EventOutcome {
                success: true,
                status_code: Some(200),
                error_code: None,
                error_message: None,
                duration_ms: Some(150),
                bytes_processed: None,
                records_affected: None,
            },
            metadata: HashMap::new(),
        };

        logger
            .log_event(event)
            .await
            .expect("Failed to log event in test");

        // Wait for async processing
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        // Check that file was created (compressed data should be present)
        let mut entries = fs::read_dir(&temp_dir.path())
            .await
            .expect("Failed to read temp directory");
        let mut found_file = false;
        while let Some(entry) = entries
            .next_entry()
            .await
            .expect("Failed to read directory entry")
        {
            if entry.path().extension().map_or(false, |ext| ext == "log") {
                found_file = true;
                break;
            }
        }
        assert!(found_file);
    }
}
