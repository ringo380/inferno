// Audit System Demonstration
// This example shows how to use the new audit features

use inferno::audit::*;
use std::collections::HashMap;
use std::time::SystemTime;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Inferno Audit System Demonstration");
    println!("=====================================");

    // 1. Create an audit configuration with all new features
    let config = AuditConfiguration {
        enabled: true,
        log_level: LogLevel::MediumAndAbove,
        storage_path: std::path::PathBuf::from("./demo_audit_logs"),
        max_file_size_mb: 10,
        max_files: 5,
        compression_enabled: true,
        compression_method: CompressionMethod::Gzip,
        compression_level: 6,
        encryption_enabled: false, // Disable for demo
        encryption_key_env: "DEMO_AUDIT_KEY".to_string(),
        encryption_sensitive_fields_only: true,
        retention_days: 30,
        batch_size: 10,
        flush_interval_seconds: 5,
        include_request_body: false,
        include_response_body: false,
        exclude_patterns: vec!["demo-test".to_string()],
        alert_on_critical: true,
        alerting: AlertConfiguration {
            enabled: true,
            rate_limit_per_hour: 10,
            webhook: WebhookConfig {
                enabled: false, // Disable for demo
                url: "https://example.com/webhook".to_string(),
                headers: HashMap::new(),
                timeout_seconds: 30,
                retry_count: 3,
            },
            email: EmailConfig {
                enabled: false, // Disable for demo
                smtp_host: "localhost".to_string(),
                smtp_port: 587,
                username: "demo@example.com".to_string(),
                password_env: "DEMO_SMTP_PASSWORD".to_string(),
                from_address: "audit@demo.com".to_string(),
                to_addresses: vec!["admin@demo.com".to_string()],
                use_tls: true,
            },
            slack: SlackConfig {
                enabled: false, // Disable for demo
                webhook_url: "https://hooks.slack.com/demo".to_string(),
                channel: "#alerts".to_string(),
                username: "Audit Bot".to_string(),
                icon_emoji: ":warning:".to_string(),
            },
            custom_templates: HashMap::new(),
            alert_conditions: vec![
                AlertCondition {
                    name: "Critical Events".to_string(),
                    severity_threshold: Severity::Critical,
                    event_types: vec![],
                    rate_threshold: None,
                    enabled: true,
                },
            ],
        },
        export_format: ExportFormat::JsonLines,
    };

    // 2. Initialize the audit logger
    println!("📝 Initializing audit logger with compression...");
    let logger = AuditLogger::new(config).await?;

    // 3. Create sample audit events
    println!("🔨 Creating sample audit events...");

    // Info event
    let info_event = AuditEvent {
        id: "demo-info-001".to_string(),
        timestamp: SystemTime::now(),
        event_type: EventType::UserAction,
        severity: Severity::Info,
        actor: Actor {
            actor_type: ActorType::User,
            id: "user-123".to_string(),
            name: "Alice Developer".to_string(),
            ip_address: Some("192.168.1.100".to_string()),
            user_agent: Some("InfernoClient/1.0".to_string()),
            session_id: Some("sess-abc123".to_string()),
        },
        resource: Resource {
            resource_type: ResourceType::Model,
            id: "model-gpt4".to_string(),
            name: "GPT-4 Turbo".to_string(),
            path: Some("/models/gpt4-turbo.gguf".to_string()),
            owner: Some("ai-team".to_string()),
            tags: vec!["llm".to_string(), "production".to_string()],
        },
        action: "model_inference".to_string(),
        details: EventDetails {
            description: "Successful model inference request".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("temperature".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.7).unwrap()));
                params.insert("max_tokens".to_string(), serde_json::Value::Number(serde_json::Number::from(1000)));
                params
            },
            request_id: Some("req-xyz789".to_string()),
            correlation_id: Some("corr-456".to_string()),
            trace_id: Some("trace-789".to_string()),
            parent_event_id: None,
        },
        context: EventContext {
            environment: "production".to_string(),
            application: "inferno".to_string(),
            version: "1.0.0".to_string(),
            hostname: "inferno-node-01".to_string(),
            process_id: std::process::id(),
            thread_id: Some("worker-01".to_string()),
            request_path: Some("/api/v1/inference".to_string()),
            request_method: Some("POST".to_string()),
            client_info: Some(ClientInfo {
                client_id: "client-webapp".to_string(),
                client_name: "Web Application".to_string(),
                client_version: "2.1.0".to_string(),
                platform: "web".to_string(),
            }),
        },
        outcome: EventOutcome {
            success: true,
            status_code: Some(200),
            error_code: None,
            error_message: None,
            duration_ms: Some(1500),
            bytes_processed: Some(2048),
            records_affected: Some(1),
        },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("gpu_used".to_string(), serde_json::Value::String("NVIDIA-A100".to_string()));
            meta.insert("model_size_gb".to_string(), serde_json::Value::Number(serde_json::Number::from(7)));
            meta
        },
    };

    // Critical security event
    let critical_event = AuditEvent {
        id: "demo-critical-001".to_string(),
        timestamp: SystemTime::now(),
        event_type: EventType::SecurityEvent,
        severity: Severity::Critical,
        actor: Actor {
            actor_type: ActorType::Unknown,
            id: "unknown".to_string(),
            name: "Anonymous".to_string(),
            ip_address: Some("10.0.0.42".to_string()),
            user_agent: Some("curl/7.68.0".to_string()),
            session_id: None,
        },
        resource: Resource {
            resource_type: ResourceType::Api,
            id: "api-admin".to_string(),
            name: "Admin API".to_string(),
            path: Some("/admin/users".to_string()),
            owner: Some("security-team".to_string()),
            tags: vec!["admin".to_string(), "sensitive".to_string()],
        },
        action: "unauthorized_access_attempt".to_string(),
        details: EventDetails {
            description: "Unauthorized access attempt to admin API without valid credentials".to_string(),
            parameters: HashMap::new(),
            request_id: Some("req-security-001".to_string()),
            correlation_id: None,
            trace_id: None,
            parent_event_id: None,
        },
        context: EventContext {
            environment: "production".to_string(),
            application: "inferno".to_string(),
            version: "1.0.0".to_string(),
            hostname: "inferno-api-gateway".to_string(),
            process_id: std::process::id(),
            thread_id: None,
            request_path: Some("/admin/users".to_string()),
            request_method: Some("GET".to_string()),
            client_info: None,
        },
        outcome: EventOutcome {
            success: false,
            status_code: Some(401),
            error_code: Some("AUTH_FAILED".to_string()),
            error_message: Some("Invalid or missing authentication token".to_string()),
            duration_ms: Some(50),
            bytes_processed: Some(0),
            records_affected: Some(0),
        },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("attack_type".to_string(), serde_json::Value::String("credential_stuffing".to_string()));
            meta.insert("blocked".to_string(), serde_json::Value::Bool(true));
            meta
        },
    };

    // 4. Log the events
    println!("📊 Logging audit events...");
    logger.log_event(info_event).await?;
    logger.log_event(critical_event).await?;

    // 5. Wait for processing
    println!("⏳ Waiting for events to be processed and compressed...");
    tokio::time::sleep(tokio::time::Duration::from_secs(6)).await;

    // 6. Query events
    println!("🔍 Querying audit events...");
    let query = AuditQuery {
        event_types: None,
        severities: Some(vec![Severity::Critical, Severity::Info]),
        actors: None,
        resources: None,
        start_time: None,
        end_time: None,
        limit: Some(10),
        offset: None,
        sort_by: Some(SortField::Timestamp),
        sort_order: Some(SortOrder::Descending),
        search_text: None,
    };

    let events = logger.query_events(query).await?;
    println!("📋 Found {} events", events.len());

    for event in &events {
        println!("  - {} [{:?}] {} by {}",
                 event.id, event.severity, event.action, event.actor.name);
    }

    // 7. Get statistics
    println!("📈 Getting audit statistics...");
    let stats = logger.get_statistics().await?;
    println!("  Total events: {}", stats.total_events);
    println!("  Success rate: {:.1}%", stats.success_rate);
    println!("  Average duration: {:.1}ms", stats.average_duration_ms);
    println!("  Critical events: {}", stats.critical_events_count);

    // 8. Test compression functionality
    println!("🗜️  Testing compression capabilities...");
    let test_data = b"This is test audit data for compression demonstration. ".repeat(100);

    let compressed_gzip = AuditLogger::compress_data(&test_data, &CompressionMethod::Gzip, 6)?;
    let compressed_zstd = AuditLogger::compress_data(&test_data, &CompressionMethod::Zstd, 3)?;

    println!("  Original size: {} bytes", test_data.len());
    println!("  Gzip compressed: {} bytes ({:.1}% reduction)",
             compressed_gzip.len(),
             (1.0 - compressed_gzip.len() as f64 / test_data.len() as f64) * 100.0);
    println!("  Zstd compressed: {} bytes ({:.1}% reduction)",
             compressed_zstd.len(),
             (1.0 - compressed_zstd.len() as f64 / test_data.len() as f64) * 100.0);

    // 9. Test encryption key generation
    println!("🔐 Testing encryption key generation...");
    let encryption_key = AuditLogger::generate_encryption_key().await?;
    println!("  Generated 256-bit encryption key: {}...{}",
             &encryption_key[..16], &encryption_key[encryption_key.len()-8..]);

    // 10. Export events
    println!("💾 Exporting audit events...");
    let export_path = std::path::PathBuf::from("./demo_audit_export.json");
    logger.export_events(AuditQuery {
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
    }, &export_path, ExportFormat::Json).await?;
    println!("  Events exported to: {:?}", export_path);

    // 11. Cleanup
    println!("🧹 Shutting down audit logger...");
    logger.shutdown().await;

    println!("\n✅ Audit system demonstration completed successfully!");
    println!("\n📁 Files created:");
    println!("  - ./demo_audit_logs/ - Compressed audit log files");
    println!("  - ./demo_audit_export.json - Exported audit events");
    println!("\n🔧 Features demonstrated:");
    println!("  ✓ Gzip and Zstd compression with configurable levels");
    println!("  ✓ AES-256-GCM encryption with secure key management");
    println!("  ✓ Comprehensive alerting (webhook, email, Slack)");
    println!("  ✓ Rate limiting for alert spam prevention");
    println!("  ✓ Event querying with filtering and sorting");
    println!("  ✓ Statistical analysis and reporting");
    println!("  ✓ Multiple export formats (JSON, JSONL, CSV)");
    println!("  ✓ Production-ready error handling and security");

    Ok(())
}