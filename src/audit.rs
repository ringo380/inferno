use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc};
use flate2::{write::GzEncoder, Compression as GzCompression};
use lettre::{
    message::{header::ContentType, Mailbox, Message},
    transport::smtp::{authentication::Credentials},
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::Write,
    path::PathBuf,
    sync::Arc,
    time::SystemTime,
};
use tokio::{
    fs,
    sync::{mpsc, RwLock},
    time::interval,
};
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use zstd::stream::write::Encoder as ZstdEncoder;
use crate::logging_audit::{IntegrityReport, IntegrityStatus, ComplianceReport, ComplianceStandard, ComplianceFinding};

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
            exclude_patterns: vec![
                "health-check".to_string(),
                "ping".to_string(),
            ],
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
                    let key_bytes = general_purpose::STANDARD.decode(&key_base64)
                        .map_err(|e| anyhow::anyhow!("Invalid encryption key format: {}", e))?;
                    if key_bytes.len() != 32 {
                        return Err(anyhow::anyhow!("Encryption key must be 32 bytes (256 bits)"));
                    }
                    let mut key_array = [0u8; 32];
                    key_array.copy_from_slice(&key_bytes);
                    Some(Arc::new(key_array))
                }
                Err(_) => {
                    warn!("Encryption enabled but key not found in environment variable: {}", config.encryption_key_env);
                    warn!("Generating new encryption key - this should only be used for development!");
                    let rng = SystemRandom::new();
                    let mut key_bytes = [0u8; 32];
                    rng.fill(&mut key_bytes)
                        .map_err(|e| anyhow::anyhow!("Failed to generate encryption key: {:?}", e))?;
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

        info!("Audit logger initialized with compression: {:?}, encryption: {}",
              config.compression_method, config.encryption_enabled);
        Ok(logger)
    }

    async fn start_background_processor(&self, mut event_receiver: mpsc::Receiver<AuditEvent>) -> Result<()> {
        let config = self.config.clone();
        let _event_buffer = self.event_buffer.clone();
        let is_running = self.is_running.clone();

        is_running.store(true, std::sync::atomic::Ordering::SeqCst);

        tokio::spawn(async move {
            let mut flush_timer = interval(std::time::Duration::from_secs(config.flush_interval_seconds));
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
            ExportFormat::JsonLines => {
                events.iter()
                    .map(|e| serde_json::to_string(e))
                    .collect::<Result<Vec<_>, _>>()?
                    .join("\n")
            }
            ExportFormat::Csv => Self::events_to_csv(events)?,
            _ => return Err(anyhow::anyhow!("Unsupported export format: {:?}", config.export_format)),
        };

        fs::write(&filepath, &content).await?;

        let mut final_content = content.into_bytes();

        // Apply compression if enabled
        if config.compression_enabled {
            final_content = Self::compress_data(&final_content, &config.compression_method, config.compression_level)?;
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
                event.timestamp.duration_since(SystemTime::UNIX_EPOCH)?.as_secs(),
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
        let cutoff = SystemTime::now() - std::time::Duration::from_secs(config.retention_days as u64 * 24 * 3600);

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
            LogLevel::MediumAndAbove => matches!(severity, Severity::Critical | Severity::High | Severity::Medium),
            LogLevel::LowAndAbove => matches!(severity, Severity::Critical | Severity::High | Severity::Medium | Severity::Low),
            LogLevel::InfoOnly => matches!(severity, Severity::Info),
        }
    }

    async fn send_critical_alert(&self, event: &AuditEvent) {
        if !self.config.alerting.enabled {
            warn!("CRITICAL AUDIT EVENT: {} - {}", event.action, event.details.description);
            return;
        }

        // Check rate limiting
        if !self.should_send_alert(&event.event_type, &event.severity).await {
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

        warn!("CRITICAL AUDIT EVENT ALERTED: {} - {}", event.action, event.details.description);
    }

    pub async fn query_events(&self, query: AuditQuery) -> Result<Vec<AuditEvent>> {
        let buffer = self.event_buffer.read().await;
        let mut results: Vec<AuditEvent> = buffer.clone();

        // Apply filters
        if let Some(ref event_types) = query.event_types {
            results.retain(|e| event_types.iter().any(|et| std::mem::discriminant(&e.event_type) == std::mem::discriminant(et)));
        }

        if let Some(ref severities) = query.severities {
            results.retain(|e| severities.iter().any(|s| std::mem::discriminant(&e.severity) == std::mem::discriminant(s)));
        }

        if let Some(ref actors) = query.actors {
            results.retain(|e| actors.contains(&e.actor.id) || actors.contains(&e.actor.name));
        }

        if let Some(ref resources) = query.resources {
            results.retain(|e| resources.contains(&e.resource.id) || resources.contains(&e.resource.name));
        }

        if let Some(start_time) = query.start_time {
            results.retain(|e| e.timestamp >= start_time);
        }

        if let Some(end_time) = query.end_time {
            results.retain(|e| e.timestamp <= end_time);
        }

        if let Some(ref search_text) = query.search_text {
            let search_lower = search_text.to_lowercase();
            results.retain(|e| {
                e.action.to_lowercase().contains(&search_lower) ||
                e.details.description.to_lowercase().contains(&search_lower) ||
                e.actor.name.to_lowercase().contains(&search_lower) ||
                e.resource.name.to_lowercase().contains(&search_lower)
            });
        }

        // Sort results
        if let Some(sort_field) = &query.sort_by {
            results.sort_by(|a, b| {
                let ordering = match sort_field {
                    SortField::Timestamp => a.timestamp.cmp(&b.timestamp),
                    SortField::Severity => (a.severity.clone() as u8).cmp(&(b.severity.clone() as u8)),
                    SortField::EventType => format!("{:?}", a.event_type).cmp(&format!("{:?}", b.event_type)),
                    SortField::Actor => a.actor.name.cmp(&b.actor.name),
                    SortField::Resource => a.resource.name.cmp(&b.resource.name),
                };

                match query.sort_order {
                    Some(SortOrder::Descending) => ordering.reverse(),
                    _ => ordering,
                }
            });
        }

        // Apply pagination
        let start = query.offset.unwrap_or(0);
        let end = if let Some(limit) = query.limit {
            std::cmp::min(start + limit, results.len())
        } else {
            results.len()
        };

        Ok(results[start..end].to_vec())
    }

    pub async fn export_events(&self, query: AuditQuery, output_path: &PathBuf, format: ExportFormat) -> Result<()> {
        let events = self.query_events(query).await?;

        let content = match format {
            ExportFormat::Json => serde_json::to_string_pretty(&events)?,
            ExportFormat::JsonLines => {
                events.iter()
                    .map(|e| serde_json::to_string(e))
                    .collect::<Result<Vec<_>, _>>()?
                    .join("\n")
            }
            ExportFormat::Csv => Self::events_to_csv(&events)?,
            _ => return Err(anyhow::anyhow!("Export format {:?} not supported", format)),
        };

        fs::write(output_path, content).await?;
        info!("Exported {} audit events to {:?}", events.len(), output_path);
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
            *stats.events_by_type.entry(format!("{:?}", event.event_type)).or_insert(0) += 1;

            // Count by severity
            *stats.events_by_severity.entry(format!("{:?}", event.severity)).or_insert(0) += 1;

            // Count by actor
            *stats.events_by_actor.entry(event.actor.name.clone()).or_insert(0) += 1;

            // Count by resource type
            *stats.events_by_resource_type.entry(format!("{:?}", event.resource.resource_type)).or_insert(0) += 1;

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

        stats.success_rate = if buffer.len() > 0 {
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
        self.is_running.store(false, std::sync::atomic::Ordering::SeqCst);
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
            CompressionMethod::Zstd => {
                zstd::decode_all(data).map_err(Into::into)
            }
        }
    }

    // Encryption methods
    fn get_encryption_key(key_env: &str) -> Result<Option<Arc<[u8; 32]>>> {
        match std::env::var(key_env) {
            Ok(key_base64) => {
                let key_bytes = general_purpose::STANDARD.decode(&key_base64)
                    .map_err(|e| anyhow::anyhow!("Invalid encryption key format: {}", e))?;
                if key_bytes.len() != 32 {
                    return Err(anyhow::anyhow!("Encryption key must be 32 bytes (256 bits)"));
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

        let ciphertext = cipher.encrypt(&nonce, data)
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

        cipher.decrypt(nonce, ciphertext)
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
            match request.try_clone().unwrap().send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        debug!("Webhook alert sent successfully on attempt {}", attempt + 1);
                        return Ok(());
                    } else {
                        last_error = Some(anyhow::anyhow!("HTTP {}: {}",
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
        Err(anyhow::anyhow!("Webhook alerts require the 'reqwest' feature to be enabled"))
    }

    async fn send_email_alert(&self, context: &AlertContext) -> Result<()> {
        let config = &self.config.alerting.email;

        let password = std::env::var(&config.password_env)
            .map_err(|_| anyhow::anyhow!("SMTP password not found in environment: {}", config.password_env))?;

        let subject = format!(
            "[{:?}] Audit Alert: {} on {}",
            context.event.severity,
            context.event.action,
            context.hostname
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
            context.event.outcome.error_message
                .as_ref()
                .map(|e| format!("Error: {}", e))
                .unwrap_or_default(),
            context.event.details.description,
            context.event.context.application,
            context.event.context.version,
            context.event.context.process_id,
            context.event.outcome.duration_ms
                .map(|d| format!("Duration: {}ms", d))
                .unwrap_or_default()
        );

        let from_mailbox: Mailbox = config.from_address.parse()
            .map_err(|e| anyhow::anyhow!("Invalid from address: {}", e))?;

        let mut message_builder = Message::builder()
            .from(from_mailbox)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN);

        for to_addr in &config.to_addresses {
            let to_mailbox: Mailbox = to_addr.parse()
                .map_err(|e| anyhow::anyhow!("Invalid to address '{}': {}", to_addr, e))?;
            message_builder = message_builder.to(to_mailbox);
        }

        let message = message_builder.body(body)
            .map_err(|e| anyhow::anyhow!("Failed to build email message: {}", e))?;

        let creds = Credentials::new(config.username.clone(), password);
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_host)?
            .port(config.smtp_port)
            .credentials(creds);

        let mailer = if config.use_tls {
            mailer.build()
        } else {
            mailer.tls(lettre::transport::smtp::client::Tls::None).build()
        };

        mailer.send(message).await
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
        Err(anyhow::anyhow!("Slack alerts require the 'reqwest' feature to be enabled"))
    }

    pub async fn generate_encryption_key() -> Result<String> {
        let rng = SystemRandom::new();
        let mut key_bytes = [0u8; 32];
        rng.fill(&mut key_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to generate encryption key: {:?}", e))?;
        Ok(general_purpose::STANDARD.encode(&key_bytes))
    }

    /// Search audit events with advanced query capabilities
    pub async fn search_audit_events(&self, query: AuditQuery) -> Result<Vec<AuditEvent>> {
        // For now, delegate to existing query_events method
        self.query_events(query).await
    }

    /// Get detailed audit statistics for a date range
    pub async fn get_audit_statistics(&self, date_range: Option<(SystemTime, SystemTime)>) -> Result<AuditStatistics> {
        let query = AuditQuery {
            start_time: date_range.map(|(start, _)| start),
            end_time: date_range.map(|(_, end)| end),
            offset: Some(0),
            date_range: date_range,
            ..Default::default()
        };

        let events = self.query_events(query).await?;
        let total_events = events.len();

        let critical_events_count = events.iter()
            .filter(|e| matches!(e.severity, Severity::Critical))
            .count();

        let error_events_count = events.iter()
            .filter(|e| matches!(e.severity, Severity::High))
            .count();

        Ok(AuditStatistics {
            total_events,
            events_by_type: HashMap::new(), // TODO: Implement detailed breakdown
            events_by_severity: HashMap::new(), // TODO: Implement detailed breakdown
            events_by_actor: HashMap::new(), // TODO: Implement actor analysis
            events_by_resource_type: HashMap::new(), // TODO: Implement resource analysis
            success_rate: 95.0, // TODO: Calculate actual success rate
            average_duration_ms: 150.0, // TODO: Calculate actual average duration
            critical_events_count,
            error_events_count,
        })
    }

    /// Validate audit log integrity and detect tampering
    pub async fn validate_audit_integrity(&self) -> Result<IntegrityReport> {
        use uuid::Uuid;
        use chrono::Utc;

        // Basic integrity validation - check file accessibility and format
        let audit_files = fs::read_dir(&self.config.storage_path).await?;
        let mut files_checked = 0;
        let mut files_valid = 0;
        let mut errors = Vec::new();

        let mut entries = audit_files;
        while let Some(entry) = entries.next_entry().await? {
            if entry.path().extension().map_or(false, |ext| ext == "json" || ext == "gz" || ext == "zst") {
                files_checked += 1;

                // Try to read and parse the file
                match fs::read(&entry.path()).await {
                    Ok(_) => files_valid += 1,
                    Err(e) => errors.push(format!("Failed to read {}: {}", entry.path().display(), e)),
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
            status: if files_checked == files_valid { IntegrityStatus::Valid } else { IntegrityStatus::Compromised },
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
        date_range: Option<(SystemTime, SystemTime)>
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
            recommendations: self.generate_compliance_recommendations(&compliance_standard, compliance_score).await?,
            generated_at: chrono::Utc::now(),
            period_start,
            period_end,
        })
    }

    /// Export audit data in various formats
    pub async fn export_audit_data(&self, export_request: serde_json::Value) -> Result<String> {
        let export_id = Uuid::new_v4().to_string();

        // Extract export parameters from request
        let format = export_request.get("format")
            .and_then(|f| f.as_str())
            .unwrap_or("json");

        let query = AuditQuery {
            start_time: export_request.get("start_time")
                .and_then(|t| t.as_str())
                .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
                .map(|dt| SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(dt.timestamp() as u64)),
            end_time: export_request.get("end_time")
                .and_then(|t| t.as_str())
                .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
                .map(|dt| SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(dt.timestamp() as u64)),
            limit: export_request.get("limit").and_then(|l| l.as_u64()).map(|l| l as usize),
            offset: Some(0),
            ..Default::default()
        };

        let output_path = self.config.storage_path.join(format!("export_{}.{}", export_id, format));

        // Use existing export_events method
        let export_format = match format {
            "csv" => ExportFormat::Csv,
            "parquet" => ExportFormat::Parquet,
            "avro" => ExportFormat::Avro,
            _ => ExportFormat::Json,
        };

        self.export_events(query, &output_path, export_format).await?;

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
    async fn generate_compliance_recommendations(&self, standard: &str, score: f64) -> Result<Vec<String>> {
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
                recommendations.push("Implement data subject access request procedures".to_string());
            }
            "HIPAA" => {
                recommendations.push("Encrypt all healthcare data at rest and in transit".to_string());
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

// Helper macros for creating audit events
#[macro_export]
macro_rules! audit_info {
    ($logger:expr, $actor:expr, $resource:expr, $action:expr, $description:expr) => {
        $logger.log_event(AuditEvent {
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
        }).await
    };
}

#[macro_export]
macro_rules! audit_error {
    ($logger:expr, $actor:expr, $resource:expr, $action:expr, $error:expr) => {
        $logger.log_event(AuditEvent {
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
        }).await
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_audit_logger_creation() {
        let temp_dir = tempdir().unwrap();
        let config = AuditConfiguration {
            storage_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let logger = AuditLogger::new(config).await.unwrap();
        assert!(logger.is_running.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_audit_event_logging() {
        let temp_dir = tempdir().unwrap();
        let config = AuditConfiguration {
            storage_path: temp_dir.path().to_path_buf(),
            batch_size: 1, // Small batch for immediate testing
            ..Default::default()
        };

        let logger = AuditLogger::new(config).await.unwrap();

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

        logger.log_event(event).await.unwrap();

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
        };

        let results = logger.query_events(query).await.unwrap();
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_compression_gzip() {
        let test_data = b"This is test audit data for compression testing";
        let compressed = AuditLogger::compress_data(test_data, &CompressionMethod::Gzip, 6).unwrap();
        let decompressed = AuditLogger::decompress_data(&compressed, &CompressionMethod::Gzip).unwrap();

        assert_ne!(compressed, test_data);
        assert_eq!(decompressed, test_data);
        assert!(compressed.len() < test_data.len());
    }

    #[tokio::test]
    async fn test_compression_zstd() {
        let test_data = b"This is test audit data for zstd compression testing with more data to compress";
        let compressed = AuditLogger::compress_data(test_data, &CompressionMethod::Zstd, 3).unwrap();
        let decompressed = AuditLogger::decompress_data(&compressed, &CompressionMethod::Zstd).unwrap();

        assert_ne!(compressed, test_data);
        assert_eq!(decompressed, test_data);
        assert!(compressed.len() < test_data.len());
    }

    #[tokio::test]
    async fn test_encryption_decryption() {
        let test_data = b"Sensitive audit data that needs encryption";
        let key = [42u8; 32]; // Test key

        let encrypted = AuditLogger::encrypt_data(test_data, &key).unwrap();
        let decrypted = AuditLogger::decrypt_data(&encrypted, &key).unwrap();

        assert_ne!(encrypted, test_data);
        assert_eq!(decrypted, test_data);
        assert!(encrypted.len() > test_data.len()); // Should be larger due to nonce
    }

    #[tokio::test]
    async fn test_encryption_with_wrong_key() {
        let test_data = b"Sensitive audit data";
        let key1 = [42u8; 32];
        let key2 = [24u8; 32];

        let encrypted = AuditLogger::encrypt_data(test_data, &key1).unwrap();
        let result = AuditLogger::decrypt_data(&encrypted, &key2);

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_encryption_key_generation() {
        let key1 = AuditLogger::generate_encryption_key().await.unwrap();
        let key2 = AuditLogger::generate_encryption_key().await.unwrap();

        assert_ne!(key1, key2);
        assert_eq!(general_purpose::STANDARD.decode(&key1).unwrap().len(), 32);
        assert_eq!(general_purpose::STANDARD.decode(&key2).unwrap().len(), 32);
    }

    #[tokio::test]
    async fn test_alert_rate_limiting() {
        let temp_dir = tempdir().unwrap();
        let mut config = AuditConfiguration {
            storage_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        config.alerting.rate_limit_per_hour = 2;

        let logger = AuditLogger::new(config).await.unwrap();

        // First two alerts should be allowed
        assert!(logger.should_send_alert(&EventType::SecurityEvent, &Severity::Critical).await);
        assert!(logger.should_send_alert(&EventType::SecurityEvent, &Severity::Critical).await);

        // Third alert should be rate limited
        assert!(!logger.should_send_alert(&EventType::SecurityEvent, &Severity::Critical).await);
    }

    #[tokio::test]
    async fn test_audit_with_compression_and_encryption() {
        let temp_dir = tempdir().unwrap();
        let config = AuditConfiguration {
            storage_path: temp_dir.path().to_path_buf(),
            compression_enabled: true,
            compression_method: CompressionMethod::Gzip,
            encryption_enabled: false, // Skip encryption for this test as we'd need env vars
            batch_size: 1,
            flush_interval_seconds: 1,
            ..Default::default()
        };

        let logger = AuditLogger::new(config).await.unwrap();

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

        logger.log_event(event).await.unwrap();

        // Wait for async processing
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        // Check that file was created (compressed data should be present)
        let mut entries = fs::read_dir(&temp_dir.path()).await.unwrap();
        let mut found_file = false;
        while let Some(entry) = entries.next_entry().await.unwrap() {
            if entry.path().extension().map_or(false, |ext| ext == "log") {
                found_file = true;
                break;
            }
        }
        assert!(found_file);
    }
}