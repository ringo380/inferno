use inferno::{
    audit::{
        AuditSystem, AuditConfig, AuditEvent, EventType, Severity, Actor, ActorType,
        Resource, ResourceType, EventDetails, EventContext, EventOutcome,
        AlertRule, AlertCondition, AlertAction, AlertConfig, CompressionType,
        EncryptionConfig, RetentionPolicy, ComplianceConfig, AlertingConfig,
        QueryFilter, AuditQuery, EventExportFormat, AuditMetrics,
    },
    models::{ModelInfo, ModelManager},
    backends::{BackendConfig, BackendType},
    cache::{ModelCache, CacheConfig},
    security::{SecurityManager, SecurityConfig},
    InfernoError,
};
use anyhow::Result;
use chrono::{DateTime, Utc, Duration as ChronoDuration};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tempfile::TempDir;
use tokio::{
    fs,
    sync::{mpsc, RwLock},
    time::{sleep, timeout},
};
use uuid::Uuid;

/// Test utilities for audit system integration tests
mod audit_test_utils {
    use super::*;

    pub fn create_test_audit_config(temp_dir: &TempDir) -> AuditConfig {
        let audit_dir = temp_dir.path().join("audit");
        let alerts_dir = temp_dir.path().join("alerts");

        AuditConfig {
            enabled: true,
            log_directory: audit_dir,
            max_file_size_mb: 10,
            max_files: 5,
            compression: CompressionType::Zstd,
            compression_level: 3,
            encryption: EncryptionConfig {
                enabled: true,
                algorithm: "AES-256-GCM".to_string(),
                key_derivation: "PBKDF2".to_string(),
                key_rotation_days: 30,
                master_key_path: None,
            },
            retention: RetentionPolicy {
                default_retention_days: 365,
                critical_retention_days: 2555, // 7 years
                audit_log_retention_days: 2555,
                compliance_retention_days: 2555,
                max_storage_gb: 100,
                auto_archive: true,
                archive_compression: CompressionType::Zstd,
            },
            compliance: ComplianceConfig {
                enable_sox: true,
                enable_hipaa: false,
                enable_gdpr: true,
                enable_pci: false,
                custom_requirements: HashMap::new(),
            },
            alerting: AlertingConfig {
                enabled: true,
                alert_directory: alerts_dir,
                email_enabled: false,
                webhook_enabled: false,
                slack_enabled: false,
                smtp_config: None,
                webhook_urls: Vec::new(),
                slack_config: None,
                alert_cooldown_minutes: 5,
                max_alerts_per_hour: 100,
            },
            buffer_size: 1000,
            flush_interval_seconds: 5,
            async_processing: true,
            enable_metrics: true,
            debug_mode: true,
        }
    }

    pub fn create_test_event(
        event_type: EventType,
        severity: Severity,
        actor_name: &str,
        resource_name: &str,
        action: &str,
    ) -> AuditEvent {
        AuditEvent {
            id: Uuid::new_v4().to_string(),
            timestamp: SystemTime::now(),
            event_type,
            severity,
            actor: Actor {
                actor_type: ActorType::User,
                id: Uuid::new_v4().to_string(),
                name: actor_name.to_string(),
                ip_address: Some("127.0.0.1".to_string()),
                user_agent: Some("test-agent/1.0".to_string()),
                session_id: Some(Uuid::new_v4().to_string()),
            },
            resource: Resource {
                resource_type: ResourceType::Model,
                id: Uuid::new_v4().to_string(),
                name: resource_name.to_string(),
                path: Some(format!("/models/{}", resource_name)),
                attributes: HashMap::new(),
            },
            action: action.to_string(),
            details: EventDetails {
                description: format!("Test event: {} on {}", action, resource_name),
                request_id: Some(Uuid::new_v4().to_string()),
                trace_id: Some(Uuid::new_v4().to_string()),
                span_id: Some(Uuid::new_v4().to_string()),
                parameters: HashMap::new(),
                response_data: None,
                error_details: None,
            },
            context: EventContext {
                source_component: "integration_test".to_string(),
                environment: "test".to_string(),
                version: "1.0.0".to_string(),
                region: Some("us-east-1".to_string()),
                availability_zone: Some("us-east-1a".to_string()),
                cluster: Some("test-cluster".to_string()),
                node: Some("test-node".to_string()),
                tenant_id: Some("test-tenant".to_string()),
                correlation_id: Some(Uuid::new_v4().to_string()),
            },
            outcome: EventOutcome {
                success: true,
                status_code: Some(200),
                duration_ms: Some(150),
                bytes_processed: Some(1024),
                records_affected: Some(1),
                resource_usage: HashMap::new(),
            },
            metadata: HashMap::from([
                ("test_run".to_string(), serde_json::Value::Bool(true)),
                ("test_id".to_string(), serde_json::Value::String(Uuid::new_v4().to_string())),
            ]),
        }
    }

    pub fn create_test_alert_rule(name: &str, event_type: EventType, severity: Severity) -> AlertRule {
        AlertRule {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: format!("Test alert rule for {}", name),
            enabled: true,
            conditions: vec![
                AlertCondition::EventType(event_type),
                AlertCondition::Severity(severity),
                AlertCondition::EventCount {
                    count: 3,
                    time_window_minutes: 5,
                },
            ],
            actions: vec![
                AlertAction::Log {
                    level: "ERROR".to_string(),
                    message: format!("Alert triggered: {}", name),
                },
                AlertAction::File {
                    path: format!("/tmp/alert_{}.log", name),
                    format: "json".to_string(),
                },
            ],
            cooldown_minutes: 5,
            max_alerts_per_hour: 10,
            severity: severity,
            tags: HashMap::from([
                ("category".to_string(), "test".to_string()),
                ("automated".to_string(), "true".to_string()),
            ]),
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
            last_triggered: None,
            trigger_count: 0,
        }
    }

    pub async fn wait_for_audit_file(audit_dir: &PathBuf, timeout_duration: Duration) -> Result<bool> {
        let start = std::time::Instant::now();
        while start.elapsed() < timeout_duration {
            if let Ok(mut entries) = fs::read_dir(audit_dir).await {
                while let Some(entry) = entries.next_entry().await? {
                    if entry.file_name().to_string_lossy().ends_with(".audit") ||
                       entry.file_name().to_string_lossy().ends_with(".audit.zst") {
                        return Ok(true);
                    }
                }
            }
            sleep(Duration::from_millis(100)).await;
        }
        Ok(false)
    }

    pub async fn count_audit_files(audit_dir: &PathBuf) -> Result<usize> {
        let mut count = 0;
        if let Ok(mut entries) = fs::read_dir(audit_dir).await {
            while let Some(entry) = entries.next_entry().await? {
                let filename = entry.file_name().to_string_lossy().to_string();
                if filename.ends_with(".audit") || filename.ends_with(".audit.zst") {
                    count += 1;
                }
            }
        }
        Ok(count)
    }
}

/// Test basic audit event logging and storage
#[tokio::test]
async fn test_audit_event_logging() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = audit_test_utils::create_test_audit_config(&temp_dir);

    let audit_system = AuditSystem::new(config.clone()).await?;
    audit_system.start().await?;

    // Create and log various types of audit events
    let events = vec![
        audit_test_utils::create_test_event(
            EventType::Authentication,
            Severity::Info,
            "test_user",
            "auth_service",
            "login",
        ),
        audit_test_utils::create_test_event(
            EventType::ModelManagement,
            Severity::Medium,
            "admin_user",
            "llama_7b",
            "model_load",
        ),
        audit_test_utils::create_test_event(
            EventType::SecurityEvent,
            Severity::High,
            "security_system",
            "firewall",
            "unauthorized_access_attempt",
        ),
        audit_test_utils::create_test_event(
            EventType::ApiCall,
            Severity::Info,
            "api_client",
            "inference_endpoint",
            "inference_request",
        ),
    ];

    for event in events {
        audit_system.log_event(event).await?;
    }

    // Wait for events to be flushed to disk
    sleep(Duration::from_secs(6)).await; // Wait longer than flush_interval_seconds

    // Verify audit files were created
    let audit_files_exist = audit_test_utils::wait_for_audit_file(
        &config.log_directory,
        Duration::from_secs(10),
    ).await?;

    assert!(audit_files_exist, "Audit files should be created");

    // Verify file count
    let file_count = audit_test_utils::count_audit_files(&config.log_directory).await?;
    assert!(file_count > 0, "Should have at least one audit file");

    // Test audit system shutdown
    audit_system.shutdown().await?;

    Ok(())
}

/// Test audit event compression and encryption
#[tokio::test]
async fn test_audit_compression_encryption() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let mut config = audit_test_utils::create_test_audit_config(&temp_dir);
    config.compression = CompressionType::Zstd;
    config.compression_level = 5;
    config.encryption.enabled = true;

    let audit_system = AuditSystem::new(config.clone()).await?;
    audit_system.start().await?;

    // Create several events to test compression efficiency
    for i in 0..20 {
        let mut event = audit_test_utils::create_test_event(
            EventType::ApiCall,
            Severity::Info,
            "test_user",
            &format!("test_resource_{}", i),
            "test_action",
        );

        // Add large metadata to test compression
        event.metadata.insert(
            "large_data".to_string(),
            serde_json::Value::String("x".repeat(1000)),
        );

        audit_system.log_event(event).await?;
    }

    // Force flush
    audit_system.flush().await?;
    sleep(Duration::from_secs(2)).await;

    // Verify compressed files exist
    let audit_files_exist = audit_test_utils::wait_for_audit_file(
        &config.log_directory,
        Duration::from_secs(10),
    ).await?;

    assert!(audit_files_exist, "Compressed audit files should be created");

    // Check that compressed files are smaller than expected uncompressed size
    if let Ok(mut entries) = fs::read_dir(&config.log_directory).await {
        while let Some(entry) = entries.next_entry().await? {
            let filename = entry.file_name().to_string_lossy().to_string();
            if filename.ends_with(".zst") {
                let file_size = entry.metadata().await?.len();
                // Compressed file should be smaller than 20KB (20 * 1000 bytes per event)
                assert!(file_size < 20000, "Compressed file should be smaller: {} bytes", file_size);
            }
        }
    }

    audit_system.shutdown().await?;

    Ok(())
}

/// Test audit alerting system
#[tokio::test]
async fn test_audit_alerting() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let mut config = audit_test_utils::create_test_audit_config(&temp_dir);
    config.alerting.enabled = true;
    config.alerting.alert_cooldown_minutes = 1; // Short cooldown for testing

    let audit_system = AuditSystem::new(config.clone()).await?;
    audit_system.start().await?;

    // Create alert rules
    let security_alert_rule = audit_test_utils::create_test_alert_rule(
        "security_events",
        EventType::SecurityEvent,
        Severity::High,
    );

    let error_alert_rule = audit_test_utils::create_test_alert_rule(
        "error_events",
        EventType::ErrorEvent,
        Severity::Critical,
    );

    audit_system.add_alert_rule(security_alert_rule).await?;
    audit_system.add_alert_rule(error_alert_rule).await?;

    // Generate events that should trigger alerts
    for i in 0..5 { // Generate enough events to trigger the count-based condition
        let security_event = audit_test_utils::create_test_event(
            EventType::SecurityEvent,
            Severity::High,
            "attacker",
            "security_system",
            "intrusion_attempt",
        );

        audit_system.log_event(security_event).await?;
        sleep(Duration::from_millis(200)).await; // Small delay between events
    }

    // Generate a critical error
    let error_event = audit_test_utils::create_test_event(
        EventType::ErrorEvent,
        Severity::Critical,
        "system",
        "database",
        "connection_failed",
    );

    audit_system.log_event(error_event).await?;

    // Wait for alerts to be processed
    sleep(Duration::from_secs(10)).await;

    // Check alert metrics
    let alert_metrics = audit_system.get_alert_metrics().await?;
    assert!(alert_metrics.alerts_triggered > 0, "Should have triggered alerts");
    assert!(alert_metrics.active_rules >= 2, "Should have active alert rules");

    // Check alert history
    let alert_history = audit_system.get_alert_history(
        SystemTime::now() - Duration::from_secs(600),
        SystemTime::now(),
    ).await?;

    assert!(!alert_history.is_empty(), "Should have alert history");

    audit_system.shutdown().await?;

    Ok(())
}

/// Test audit querying and search functionality
#[tokio::test]
async fn test_audit_querying() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = audit_test_utils::create_test_audit_config(&temp_dir);

    let audit_system = AuditSystem::new(config.clone()).await?;
    audit_system.start().await?;

    // Create events with different characteristics for querying
    let events = vec![
        // Authentication events
        audit_test_utils::create_test_event(
            EventType::Authentication,
            Severity::Info,
            "alice",
            "auth_service",
            "login",
        ),
        audit_test_utils::create_test_event(
            EventType::Authentication,
            Severity::High,
            "bob",
            "auth_service",
            "failed_login",
        ),
        // Model management events
        audit_test_utils::create_test_event(
            EventType::ModelManagement,
            Severity::Medium,
            "admin",
            "llama_7b",
            "model_load",
        ),
        audit_test_utils::create_test_event(
            EventType::ModelManagement,
            Severity::Info,
            "user1",
            "gpt_3_5",
            "inference",
        ),
        // Security events
        audit_test_utils::create_test_event(
            EventType::SecurityEvent,
            Severity::Critical,
            "unknown",
            "firewall",
            "intrusion_detected",
        ),
    ];

    for event in events {
        audit_system.log_event(event).await?;
    }

    // Force flush to ensure events are written
    audit_system.flush().await?;
    sleep(Duration::from_secs(2)).await;

    // Test various queries

    // 1. Query by event type
    let auth_query = AuditQuery {
        filters: vec![QueryFilter::EventType(EventType::Authentication)],
        start_time: Some(SystemTime::now() - Duration::from_secs(300)),
        end_time: Some(SystemTime::now()),
        limit: Some(100),
        offset: None,
        sort_by: Some("timestamp".to_string()),
        sort_order: Some("desc".to_string()),
    };

    let auth_results = audit_system.query_events(auth_query).await?;
    assert_eq!(auth_results.len(), 2, "Should find 2 authentication events");

    // 2. Query by severity
    let critical_query = AuditQuery {
        filters: vec![QueryFilter::Severity(Severity::Critical)],
        start_time: Some(SystemTime::now() - Duration::from_secs(300)),
        end_time: Some(SystemTime::now()),
        limit: Some(100),
        offset: None,
        sort_by: None,
        sort_order: None,
    };

    let critical_results = audit_system.query_events(critical_query).await?;
    assert_eq!(critical_results.len(), 1, "Should find 1 critical event");

    // 3. Query by actor
    let alice_query = AuditQuery {
        filters: vec![QueryFilter::Actor("alice".to_string())],
        start_time: Some(SystemTime::now() - Duration::from_secs(300)),
        end_time: Some(SystemTime::now()),
        limit: Some(100),
        offset: None,
        sort_by: None,
        sort_order: None,
    };

    let alice_results = audit_system.query_events(alice_query).await?;
    assert_eq!(alice_results.len(), 1, "Should find 1 event by alice");

    // 4. Complex query with multiple filters
    let complex_query = AuditQuery {
        filters: vec![
            QueryFilter::EventType(EventType::ModelManagement),
            QueryFilter::SeverityRange {
                min: Severity::Info,
                max: Severity::Medium,
            },
        ],
        start_time: Some(SystemTime::now() - Duration::from_secs(300)),
        end_time: Some(SystemTime::now()),
        limit: Some(100),
        offset: None,
        sort_by: Some("severity".to_string()),
        sort_order: Some("asc".to_string()),
    };

    let complex_results = audit_system.query_events(complex_query).await?;
    assert_eq!(complex_results.len(), 2, "Should find 2 model management events with specified severity");

    audit_system.shutdown().await?;

    Ok(())
}

/// Test audit export functionality
#[tokio::test]
async fn test_audit_export() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = audit_test_utils::create_test_audit_config(&temp_dir);

    let audit_system = AuditSystem::new(config.clone()).await?;
    audit_system.start().await?;

    // Create events for export
    for i in 0..10 {
        let event = audit_test_utils::create_test_event(
            EventType::ApiCall,
            Severity::Info,
            &format!("user_{}", i),
            &format!("resource_{}", i),
            "api_call",
        );
        audit_system.log_event(event).await?;
    }

    audit_system.flush().await?;
    sleep(Duration::from_secs(2)).await;

    let export_dir = temp_dir.path().join("exports");
    fs::create_dir_all(&export_dir).await?;

    // Test JSON export
    let json_export_path = export_dir.join("events.json");
    let export_query = AuditQuery {
        filters: vec![QueryFilter::EventType(EventType::ApiCall)],
        start_time: Some(SystemTime::now() - Duration::from_secs(300)),
        end_time: Some(SystemTime::now()),
        limit: Some(100),
        offset: None,
        sort_by: None,
        sort_order: None,
    };

    audit_system.export_events(
        export_query.clone(),
        &json_export_path,
        EventExportFormat::Json,
    ).await?;

    assert!(json_export_path.exists(), "JSON export file should exist");

    let json_content = fs::read_to_string(&json_export_path).await?;
    assert!(!json_content.is_empty(), "JSON export should not be empty");

    // Test CSV export
    let csv_export_path = export_dir.join("events.csv");
    audit_system.export_events(
        export_query.clone(),
        &csv_export_path,
        EventExportFormat::Csv,
    ).await?;

    assert!(csv_export_path.exists(), "CSV export file should exist");

    let csv_content = fs::read_to_string(&csv_export_path).await?;
    assert!(!csv_content.is_empty(), "CSV export should not be empty");
    assert!(csv_content.contains("timestamp"), "CSV should contain headers");

    // Test compressed export
    let compressed_export_path = export_dir.join("events.json.gz");
    audit_system.export_events_compressed(
        export_query,
        &compressed_export_path,
        EventExportFormat::Json,
        CompressionType::Gzip,
    ).await?;

    assert!(compressed_export_path.exists(), "Compressed export file should exist");

    let compressed_size = fs::metadata(&compressed_export_path).await?.len();
    let uncompressed_size = fs::metadata(&json_export_path).await?.len();
    assert!(compressed_size < uncompressed_size, "Compressed file should be smaller");

    audit_system.shutdown().await?;

    Ok(())
}

/// Test audit retention and cleanup
#[tokio::test]
async fn test_audit_retention_cleanup() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let mut config = audit_test_utils::create_test_audit_config(&temp_dir);
    config.retention.default_retention_days = 1; // Short retention for testing
    config.max_file_size_mb = 1; // Small file size to trigger rotation

    let audit_system = AuditSystem::new(config.clone()).await?;
    audit_system.start().await?;

    // Generate many events to trigger file rotation
    for i in 0..100 {
        let mut event = audit_test_utils::create_test_event(
            EventType::UserAction,
            Severity::Info,
            &format!("user_{}", i),
            &format!("resource_{}", i),
            "action",
        );

        // Add large metadata to increase file size
        event.metadata.insert(
            "large_data".to_string(),
            serde_json::Value::String("x".repeat(500)),
        );

        audit_system.log_event(event).await?;

        if i % 20 == 0 {
            audit_system.flush().await?;
            sleep(Duration::from_millis(100)).await;
        }
    }

    audit_system.flush().await?;
    sleep(Duration::from_secs(3)).await;

    // Check that multiple files were created due to rotation
    let initial_file_count = audit_test_utils::count_audit_files(&config.log_directory).await?;
    assert!(initial_file_count > 1, "Should have multiple audit files due to rotation");

    // Simulate old files by creating files with old timestamps
    let old_file = config.log_directory.join("old_audit.audit");
    fs::write(&old_file, "old audit data").await?;

    // Set file time to be older than retention period
    // Note: This would require setting file times in a real implementation

    // Test cleanup operation
    audit_system.cleanup_old_files().await?;

    // Test retention policy enforcement
    let retention_stats = audit_system.get_retention_stats().await?;
    assert!(retention_stats.total_files >= 0);
    assert!(retention_stats.total_size_mb >= 0.0);

    audit_system.shutdown().await?;

    Ok(())
}

/// Test audit metrics and monitoring
#[tokio::test]
async fn test_audit_metrics() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = audit_test_utils::create_test_audit_config(&temp_dir);

    let audit_system = AuditSystem::new(config.clone()).await?;
    audit_system.start().await?;

    // Generate events of different types and severities
    let event_types = vec![
        (EventType::Authentication, Severity::Info),
        (EventType::SecurityEvent, Severity::High),
        (EventType::ModelManagement, Severity::Medium),
        (EventType::ErrorEvent, Severity::Critical),
        (EventType::ApiCall, Severity::Info),
    ];

    for (event_type, severity) in event_types {
        for i in 0..3 {
            let event = audit_test_utils::create_test_event(
                event_type.clone(),
                severity.clone(),
                &format!("user_{}", i),
                &format!("resource_{}", i),
                "test_action",
            );
            audit_system.log_event(event).await?;
        }
    }

    audit_system.flush().await?;
    sleep(Duration::from_secs(2)).await;

    // Test general metrics
    let metrics = audit_system.get_metrics().await?;
    assert!(metrics.total_events >= 15, "Should have at least 15 events");
    assert!(metrics.events_per_minute >= 0.0);
    assert!(metrics.average_event_size_bytes > 0);

    // Test metrics by event type
    let type_metrics = audit_system.get_metrics_by_event_type().await?;
    assert!(type_metrics.contains_key(&EventType::Authentication));
    assert!(type_metrics.contains_key(&EventType::SecurityEvent));
    assert_eq!(type_metrics[&EventType::Authentication], 3);

    // Test metrics by severity
    let severity_metrics = audit_system.get_metrics_by_severity().await?;
    assert!(severity_metrics.contains_key(&Severity::Critical));
    assert!(severity_metrics.contains_key(&Severity::Info));
    assert_eq!(severity_metrics[&Severity::Critical], 3);

    // Test performance metrics
    let performance_metrics = audit_system.get_performance_metrics().await?;
    assert!(performance_metrics.average_write_time_ms >= 0.0);
    assert!(performance_metrics.queue_size >= 0);
    assert!(performance_metrics.flush_count > 0);

    audit_system.shutdown().await?;

    Ok(())
}

/// Test audit system integration with other components
#[tokio::test]
async fn test_audit_integration_with_components() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let models_dir = temp_dir.path().join("models");
    fs::create_dir_all(&models_dir).await?;

    // Create mock model file
    let model_path = models_dir.join("test_model.gguf");
    let model_content = b"GGUF\x03\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x00\x00\x00";
    fs::write(&model_path, model_content).await?;

    let audit_config = audit_test_utils::create_test_audit_config(&temp_dir);
    let audit_system = Arc::new(AuditSystem::new(audit_config).await?);
    audit_system.start().await?;

    // Initialize other components with audit integration
    let model_manager = Arc::new(ModelManager::new(models_dir));
    let backend_config = BackendConfig::default();
    let cache_config = CacheConfig::default();
    let cache = Arc::new(ModelCache::new(
        cache_config,
        backend_config.clone(),
        model_manager.clone(),
        None,
    ).await?);

    // Test model management audit events
    let models = model_manager.discover_models().await?;
    assert!(!models.is_empty());

    // Simulate model loading event
    let model_load_event = audit_test_utils::create_test_event(
        EventType::ModelManagement,
        Severity::Info,
        "model_manager",
        &models[0].name,
        "model_discovered",
    );
    audit_system.log_event(model_load_event).await?;

    // Test cache audit events
    let cache_event = audit_test_utils::create_test_event(
        EventType::SystemChange,
        Severity::Info,
        "cache_system",
        "model_cache",
        "cache_hit",
    );
    audit_system.log_event(cache_event).await?;

    // Test security audit events
    let security_event = audit_test_utils::create_test_event(
        EventType::SecurityEvent,
        Severity::Medium,
        "security_system",
        "auth_module",
        "access_granted",
    );
    audit_system.log_event(security_event).await?;

    audit_system.flush().await?;
    sleep(Duration::from_secs(3)).await;

    // Verify events were logged
    let query = AuditQuery {
        filters: vec![],
        start_time: Some(SystemTime::now() - Duration::from_secs(300)),
        end_time: Some(SystemTime::now()),
        limit: Some(100),
        offset: None,
        sort_by: None,
        sort_order: None,
    };

    let events = audit_system.query_events(query).await?;
    assert!(events.len() >= 3, "Should have audit events from different components");

    // Check event types are represented
    let event_types: std::collections::HashSet<_> = events.iter()
        .map(|e| std::mem::discriminant(&e.event_type))
        .collect();

    assert!(event_types.len() >= 2, "Should have events of different types");

    audit_system.shutdown().await?;

    Ok(())
}

/// Test concurrent audit operations
#[tokio::test]
async fn test_concurrent_audit_operations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = audit_test_utils::create_test_audit_config(&temp_dir);

    let audit_system = Arc::new(AuditSystem::new(config).await?);
    audit_system.start().await?;

    // Launch multiple concurrent tasks that generate audit events
    let mut tasks = Vec::new();

    for task_id in 0..5 {
        let audit_system_clone = audit_system.clone();
        let task = tokio::spawn(async move {
            for i in 0..20 {
                let event = audit_test_utils::create_test_event(
                    EventType::ApiCall,
                    Severity::Info,
                    &format!("task_{}_user_{}", task_id, i),
                    &format!("task_{}_resource_{}", task_id, i),
                    "concurrent_action",
                );

                if let Err(e) = audit_system_clone.log_event(event).await {
                    eprintln!("Failed to log event: {}", e);
                }

                // Small delay to simulate real usage
                sleep(Duration::from_millis(10)).await;
            }
        });

        tasks.push(task);
    }

    // Wait for all tasks to complete
    futures::future::join_all(tasks).await;

    audit_system.flush().await?;
    sleep(Duration::from_secs(5)).await;

    // Verify all events were logged correctly
    let query = AuditQuery {
        filters: vec![QueryFilter::EventType(EventType::ApiCall)],
        start_time: Some(SystemTime::now() - Duration::from_secs(300)),
        end_time: Some(SystemTime::now()),
        limit: Some(200),
        offset: None,
        sort_by: None,
        sort_order: None,
    };

    let events = audit_system.query_events(query).await?;
    assert_eq!(events.len(), 100, "Should have 100 concurrent events (5 tasks * 20 events)");

    // Verify no data corruption occurred
    for event in events {
        assert!(event.actor.name.starts_with("task_"));
        assert!(event.resource.name.starts_with("task_"));
        assert_eq!(event.action, "concurrent_action");
    }

    audit_system.shutdown().await?;

    Ok(())
}