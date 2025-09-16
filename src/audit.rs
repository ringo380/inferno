use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub encryption_enabled: bool,
    pub retention_days: u32,
    pub batch_size: usize,
    pub flush_interval_seconds: u64,
    pub include_request_body: bool,
    pub include_response_body: bool,
    pub exclude_patterns: Vec<String>,
    pub alert_on_critical: bool,
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

impl Default for AuditConfiguration {
    fn default() -> Self {
        Self {
            enabled: true,
            log_level: LogLevel::MediumAndAbove,
            storage_path: PathBuf::from("./logs/audit"),
            max_file_size_mb: 100,
            max_files: 50,
            compression_enabled: true,
            encryption_enabled: false,
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
            export_format: ExportFormat::JsonLines,
        }
    }
}

pub struct AuditLogger {
    config: AuditConfiguration,
    event_buffer: Arc<RwLock<Vec<AuditEvent>>>,
    event_sender: mpsc::Sender<AuditEvent>,
    is_running: Arc<std::sync::atomic::AtomicBool>,
    context: EventContext,
}

impl AuditLogger {
    pub async fn new(config: AuditConfiguration) -> Result<Self> {
        // Ensure audit directory exists
        fs::create_dir_all(&config.storage_path).await?;

        let (event_sender, event_receiver) = mpsc::channel::<AuditEvent>(config.batch_size * 2);

        let context = EventContext {
            environment: std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
            application: "inferno".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            hostname: "localhost".to_string(),
            process_id: std::process::id(),
            thread_id: None,
            request_path: None,
            request_method: None,
            client_info: None,
        };

        let logger = Self {
            config: config.clone(),
            event_buffer: Arc::new(RwLock::new(Vec::new())),
            event_sender,
            is_running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            context,
        };

        // Start background processor
        logger.start_background_processor(event_receiver).await?;

        info!("Audit logger initialized");
        Ok(logger)
    }

    async fn start_background_processor(&self, mut event_receiver: mpsc::Receiver<AuditEvent>) -> Result<()> {
        let config = self.config.clone();
        let event_buffer = self.event_buffer.clone();
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

        fs::write(&filepath, content).await?;

        if config.compression_enabled {
            // TODO: Implement compression
        }

        if config.encryption_enabled {
            // TODO: Implement encryption
        }

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
        // TODO: Implement alerting mechanism (email, webhook, etc.)
        warn!("CRITICAL AUDIT EVENT: {} - {}", event.action, event.details.description);
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
}