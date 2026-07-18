//! Integration tests for the audit logging subsystem.
//!
//! These exercise the real `inferno::audit::AuditLogger` API end to end:
//! construct a logger over a temp directory, log events, then query, export,
//! and summarize them. Assertions are anchored to observable behavior
//! (retrieved events, statistics, export contents) rather than internal state.
//!
//! Notes on the real API that shape these tests:
//! - `AuditLogger::log_event` pushes the event into an in-memory buffer
//!   synchronously (under a write lock) before returning, so a subsequent
//!   `query_events`/`get_statistics` sees it without any sleep.
//! - The default `log_level` is `MediumAndAbove`, which drops `Info`/`Low`
//!   events. Tests that log `Info` set `log_level: LogLevel::All`.
//! - Query/statistics read the in-memory buffer, which only drains once it
//!   exceeds `batch_size * 2` (default 1000), so counts below that are exact.

use anyhow::Result;
use inferno::{
    audit::{
        Actor, ActorType, AuditConfiguration, AuditEvent, AuditLogger, AuditQuery, EventContext,
        EventDetails, EventOutcome, EventType, ExportFormat, LogLevel, Resource, ResourceType,
        Severity,
    },
    backends::BackendConfig,
    cache::{CacheConfig, ModelCache},
    models::ModelManager,
};
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tempfile::TempDir;
use tokio::fs;

/// Build an `AuditConfiguration` rooted at a temp dir with the given log level.
/// Everything else stays at defaults (alerting channels are all disabled by
/// default, so nothing reaches the network).
fn make_config(temp_dir: &TempDir, log_level: LogLevel) -> AuditConfiguration {
    AuditConfiguration {
        storage_path: temp_dir.path().join("audit"),
        log_level,
        // Never attempt an out-of-band alert during tests.
        alert_on_critical: false,
        ..Default::default()
    }
}

/// Construct a fully-populated `AuditEvent`. `AuditLogger::log_event` overwrites
/// `context` and `timestamp` and fills an empty `id`, but preserves `actor`,
/// `resource`, `action`, `details`, `outcome`, and `metadata`, which is what
/// these tests assert on.
fn make_event(
    event_type: EventType,
    severity: Severity,
    actor_name: &str,
    resource_name: &str,
    action: &str,
) -> AuditEvent {
    let success = !matches!(severity, Severity::Critical);
    AuditEvent {
        id: String::new(),
        timestamp: std::time::SystemTime::now(),
        event_type,
        severity,
        actor: Actor {
            actor_type: ActorType::User,
            id: format!("id-{actor_name}"),
            name: actor_name.to_string(),
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("integration-test/1.0".to_string()),
            session_id: None,
        },
        resource: Resource {
            resource_type: ResourceType::Model,
            id: format!("id-{resource_name}"),
            name: resource_name.to_string(),
            path: Some(format!("/models/{resource_name}")),
            owner: None,
            tags: vec![],
        },
        action: action.to_string(),
        details: EventDetails {
            description: format!("{action} on {resource_name}"),
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
            process_id: std::process::id(),
            thread_id: None,
            request_path: None,
            request_method: None,
            client_info: None,
        },
        outcome: EventOutcome {
            success,
            status_code: Some(if success { 200 } else { 500 }),
            error_code: None,
            error_message: None,
            duration_ms: Some(150),
            bytes_processed: Some(1024),
            records_affected: Some(1),
        },
        metadata: HashMap::new(),
    }
}

/// A query with a generous limit and no filters. Callers set the specific
/// filter fields they want to test.
fn query_all() -> AuditQuery {
    AuditQuery {
        limit: Some(500),
        ..Default::default()
    }
}

/// Log a handful of events, then read them all back.
#[tokio::test]
async fn test_log_and_query_all() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let logger = AuditLogger::new(make_config(&temp_dir, LogLevel::All)).await?;

    let events = vec![
        make_event(
            EventType::Authentication,
            Severity::Info,
            "alice",
            "auth_service",
            "login",
        ),
        make_event(
            EventType::ModelManagement,
            Severity::Medium,
            "admin",
            "llama_7b",
            "model_load",
        ),
        make_event(
            EventType::SecurityEvent,
            Severity::High,
            "watchdog",
            "firewall",
            "unauthorized_access",
        ),
    ];
    for event in events {
        logger.log_event(event).await?;
    }

    let results = logger.query_events(query_all()).await?;
    assert_eq!(
        results.len(),
        3,
        "all three logged events should be queryable"
    );

    // A logged event's payload survives round-trip.
    let login = results
        .iter()
        .find(|e| e.action == "login")
        .expect("login event should be present");
    assert_eq!(login.actor.name, "alice");
    assert_eq!(login.resource.name, "auth_service");
    // log_event fills an empty id with a UUID.
    assert!(!login.id.is_empty(), "logger should assign an id");

    logger.shutdown().await;
    Ok(())
}

/// Query filters (by event type, severity, and actor) select the right subset.
#[tokio::test]
async fn test_query_filters() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let logger = AuditLogger::new(make_config(&temp_dir, LogLevel::All)).await?;

    for event in [
        make_event(
            EventType::Authentication,
            Severity::Info,
            "alice",
            "auth",
            "login",
        ),
        make_event(
            EventType::Authentication,
            Severity::High,
            "bob",
            "auth",
            "failed_login",
        ),
        make_event(
            EventType::ModelManagement,
            Severity::Medium,
            "admin",
            "llama_7b",
            "load",
        ),
        make_event(
            EventType::SecurityEvent,
            Severity::Critical,
            "unknown",
            "firewall",
            "intrusion",
        ),
    ] {
        logger.log_event(event).await?;
    }

    // By event type.
    let auth = logger
        .query_events(AuditQuery {
            event_types: Some(vec![EventType::Authentication]),
            ..query_all()
        })
        .await?;
    assert_eq!(auth.len(), 2, "two authentication events");

    // By severity.
    let critical = logger
        .query_events(AuditQuery {
            severities: Some(vec![Severity::Critical]),
            ..query_all()
        })
        .await?;
    assert_eq!(critical.len(), 1, "one critical event");
    assert_eq!(critical[0].action, "intrusion");

    // By actor (matches actor id or name).
    let by_alice = logger
        .query_events(AuditQuery {
            actors: Some(vec!["alice".to_string()]),
            ..query_all()
        })
        .await?;
    assert_eq!(by_alice.len(), 1, "one event by alice");
    assert_eq!(by_alice[0].action, "login");

    logger.shutdown().await;
    Ok(())
}

/// `log_level` filtering drops events below the configured threshold at write
/// time, so they never reach the queryable buffer.
#[tokio::test]
async fn test_log_level_filtering() -> Result<()> {
    let temp_dir = TempDir::new()?;
    // HighAndAbove keeps Critical + High, drops Medium/Low/Info.
    let logger = AuditLogger::new(make_config(&temp_dir, LogLevel::HighAndAbove)).await?;

    logger
        .log_event(make_event(
            EventType::ApiCall,
            Severity::Info,
            "u",
            "r",
            "low_prio",
        ))
        .await?;
    logger
        .log_event(make_event(
            EventType::ApiCall,
            Severity::Medium,
            "u",
            "r",
            "mid_prio",
        ))
        .await?;
    logger
        .log_event(make_event(
            EventType::SecurityEvent,
            Severity::High,
            "u",
            "r",
            "high_prio",
        ))
        .await?;

    let results = logger.query_events(query_all()).await?;
    assert_eq!(results.len(), 1, "only the High-severity event is retained");
    assert_eq!(results[0].action, "high_prio");

    logger.shutdown().await;
    Ok(())
}

/// Statistics are computed over the buffered events.
#[tokio::test]
async fn test_statistics() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let logger = AuditLogger::new(make_config(&temp_dir, LogLevel::All)).await?;

    // 3 of each of 5 (type, severity) pairs = 15 events; one pair is Critical.
    let pairs = [
        (EventType::Authentication, Severity::Info),
        (EventType::SecurityEvent, Severity::High),
        (EventType::ModelManagement, Severity::Medium),
        (EventType::ErrorEvent, Severity::Critical),
        (EventType::ApiCall, Severity::Info),
    ];
    for (event_type, severity) in pairs {
        for i in 0..3 {
            logger
                .log_event(make_event(
                    event_type.clone(),
                    severity.clone(),
                    &format!("user_{i}"),
                    &format!("resource_{i}"),
                    "action",
                ))
                .await?;
        }
    }

    let stats = logger.get_statistics().await?;
    assert_eq!(stats.total_events, 15);
    // Keys are the Debug rendering of the enum variant.
    assert_eq!(stats.events_by_type.get("Authentication"), Some(&3));
    assert_eq!(stats.events_by_severity.get("Critical"), Some(&3));
    assert_eq!(stats.critical_events_count, 3);
    // Only the Critical events were marked unsuccessful (12 of 15 succeed).
    assert_eq!(stats.error_events_count, 3);
    assert!(
        (stats.success_rate - (12.0 / 15.0 * 100.0)).abs() < 1e-6,
        "success_rate should be 80%, got {}",
        stats.success_rate
    );

    logger.shutdown().await;
    Ok(())
}

/// Export produces non-empty files in each supported format; unsupported
/// formats surface an error rather than a silent empty file.
#[tokio::test]
async fn test_export_formats() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let logger = AuditLogger::new(make_config(&temp_dir, LogLevel::All)).await?;

    for i in 0..5 {
        logger
            .log_event(make_event(
                EventType::ApiCall,
                Severity::Info,
                &format!("user_{i}"),
                &format!("resource_{i}"),
                "api_call",
            ))
            .await?;
    }

    let export_dir = temp_dir.path().join("exports");
    fs::create_dir_all(&export_dir).await?;

    // JSON
    let json_path: PathBuf = export_dir.join("events.json");
    logger
        .export_events(query_all(), &json_path, ExportFormat::Json)
        .await?;
    let json = fs::read_to_string(&json_path).await?;
    assert!(
        json.contains("api_call"),
        "JSON export should contain event data"
    );

    // JSON Lines: one JSON object per line.
    let jsonl_path = export_dir.join("events.jsonl");
    logger
        .export_events(query_all(), &jsonl_path, ExportFormat::JsonLines)
        .await?;
    let jsonl = fs::read_to_string(&jsonl_path).await?;
    assert_eq!(jsonl.lines().count(), 5, "one JSONL line per event");

    // CSV
    let csv_path = export_dir.join("events.csv");
    logger
        .export_events(query_all(), &csv_path, ExportFormat::Csv)
        .await?;
    let csv = fs::read_to_string(&csv_path).await?;
    assert!(!csv.is_empty(), "CSV export should not be empty");

    // Parquet is not supported by export_events and must error, not write junk.
    let parquet_path = export_dir.join("events.parquet");
    let err = logger
        .export_events(query_all(), &parquet_path, ExportFormat::Parquet)
        .await;
    assert!(err.is_err(), "Parquet export should be rejected");
    assert!(
        !parquet_path.exists(),
        "no file should be written for an unsupported format"
    );

    logger.shutdown().await;
    Ok(())
}

/// Concurrent writers all land in the buffer without loss or corruption.
#[tokio::test]
async fn test_concurrent_logging() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let logger = Arc::new(AuditLogger::new(make_config(&temp_dir, LogLevel::All)).await?);

    let mut tasks = Vec::new();
    for task_id in 0..5 {
        let logger = logger.clone();
        tasks.push(tokio::spawn(async move {
            for i in 0..20 {
                let event = make_event(
                    EventType::ApiCall,
                    Severity::Info,
                    &format!("task_{task_id}_user_{i}"),
                    &format!("task_{task_id}_resource_{i}"),
                    "concurrent_action",
                );
                logger
                    .log_event(event)
                    .await
                    .expect("log_event should succeed");
            }
        }));
    }
    for task in tasks {
        task.await?;
    }

    // 5 tasks * 20 events = 100, well under batch_size*2, so the count is exact.
    let events = logger
        .query_events(AuditQuery {
            event_types: Some(vec![EventType::ApiCall]),
            limit: Some(500),
            ..Default::default()
        })
        .await?;
    assert_eq!(
        events.len(),
        100,
        "all 100 concurrent events should be present"
    );

    for event in &events {
        assert!(event.actor.name.starts_with("task_"));
        assert!(event.resource.name.starts_with("task_"));
        assert_eq!(event.action, "concurrent_action");
    }

    logger.shutdown().await;
    Ok(())
}

/// Encryption key generation yields distinct, non-empty keys, and a logger
/// configured with encryption + compression still logs and queries.
#[tokio::test]
async fn test_encryption_and_compression_config() -> Result<()> {
    let key_a = AuditLogger::generate_encryption_key().await?;
    let key_b = AuditLogger::generate_encryption_key().await?;
    assert!(!key_a.is_empty(), "generated key should be non-empty");
    assert_ne!(key_a, key_b, "each generated key should be unique");

    let temp_dir = TempDir::new()?;
    let mut config = make_config(&temp_dir, LogLevel::All);
    config.encryption_enabled = true;
    config.encryption_key_env = "INFERNO_TEST_AUDIT_KEY".to_string();
    config.compression_enabled = true;

    let logger = AuditLogger::new(config).await?;
    logger
        .log_event(make_event(
            EventType::DataAccess,
            Severity::High,
            "reader",
            "secret_dataset",
            "read",
        ))
        .await?;

    let results = logger.query_events(query_all()).await?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].resource.name, "secret_dataset");

    logger.shutdown().await;
    Ok(())
}

/// The audit logger coexists with other subsystems (model manager + cache) and
/// records events that reference their resources.
#[tokio::test]
async fn test_integration_with_components() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let models_dir = temp_dir.path().join("models");
    fs::create_dir_all(&models_dir).await?;

    let logger = AuditLogger::new(make_config(&temp_dir, LogLevel::All)).await?;

    let model_manager = Arc::new(ModelManager::new(&models_dir));
    let cache = ModelCache::new(
        CacheConfig::default(),
        BackendConfig::default(),
        model_manager.clone(),
        None,
    )
    .await?;
    // Discovery over an empty dir simply returns an empty list, not an error.
    let discovered = model_manager.list_models().await?;
    assert!(discovered.is_empty(), "no models in an empty dir");
    drop(cache);

    for event in [
        make_event(
            EventType::ModelManagement,
            Severity::Info,
            "model_manager",
            "llama_7b",
            "discovered",
        ),
        make_event(
            EventType::SystemChange,
            Severity::Info,
            "cache_system",
            "model_cache",
            "cache_hit",
        ),
        make_event(
            EventType::SecurityEvent,
            Severity::Medium,
            "security",
            "auth_module",
            "access_granted",
        ),
    ] {
        logger.log_event(event).await?;
    }

    let events = logger.query_events(query_all()).await?;
    assert_eq!(
        events.len(),
        3,
        "events from all three components are recorded"
    );

    let distinct_types: std::collections::HashSet<_> = events
        .iter()
        .map(|e| std::mem::discriminant(&e.event_type))
        .collect();
    assert!(distinct_types.len() >= 2, "events span multiple types");

    logger.shutdown().await;
    Ok(())
}
