/// Test fixtures for creating consistent test data across integration tests

use anyhow::Result;
use std::{collections::HashMap, fs, path::PathBuf, time::SystemTime};
use uuid::Uuid;

/// Model file fixtures for testing
pub struct ModelFixtures;

impl ModelFixtures {
    /// Create a mock GGUF model file with realistic structure
    pub fn create_gguf_file(path: &PathBuf, model_name: &str, size_kb: Option<usize>) -> Result<()> {
        let mut content = Vec::new();

        // GGUF magic number and version
        content.extend_from_slice(b"GGUF");
        content.extend_from_slice(&3u32.to_le_bytes()); // Version 3
        content.extend_from_slice(&2u64.to_le_bytes()); // Tensor count
        content.extend_from_slice(&5u64.to_le_bytes()); // Metadata count

        // Standard metadata entries
        Self::write_metadata_string(&mut content, "general.name", model_name)?;
        Self::write_metadata_string(&mut content, "general.architecture", "llama")?;
        Self::write_metadata_u32(&mut content, "llama.context_length", 2048)?;
        Self::write_metadata_u32(&mut content, "llama.embedding_length", 4096)?;
        Self::write_metadata_u32(&mut content, "llama.feed_forward_length", 11008)?;

        // Mock tensor info
        Self::write_tensor_info(&mut content, "token_embd.weight", &[32000, 4096], 1)?; // F16
        Self::write_tensor_info(&mut content, "output.weight", &[32000, 4096], 1)?; // F16

        // Pad to desired size
        let target_size = size_kb.unwrap_or(4) * 1024;
        if content.len() < target_size {
            content.resize(target_size, 0);
        }

        fs::write(path, content)?;
        Ok(())
    }

    /// Create a mock ONNX model file
    pub fn create_onnx_file(path: &PathBuf, model_name: &str, size_kb: Option<usize>) -> Result<()> {
        let mut content = Vec::new();

        // ONNX protobuf-like header
        content.extend_from_slice(&[0x08, 0x07]); // ir_version = 7
        content.extend_from_slice(&[0x12]); // producer_name field
        content.push(model_name.len() as u8);
        content.extend_from_slice(model_name.as_bytes());

        // Model version
        content.extend_from_slice(&[0x18, 0x01]); // model_version = 1

        // Graph definition
        content.extend_from_slice(&[0x22]); // graph field
        content.extend_from_slice(&[0x20]); // length placeholder

        // Mock graph content
        content.extend_from_slice(b"mock_onnx_graph_content");

        // Pad to desired size
        let target_size = size_kb.unwrap_or(2) * 1024;
        if content.len() < target_size {
            content.resize(target_size, 0);
        }

        fs::write(path, content)?;
        Ok(())
    }

    /// Create a mock PyTorch model file
    pub fn create_pytorch_file(path: &PathBuf, model_name: &str, size_kb: Option<usize>) -> Result<()> {
        let mut content = Vec::new();

        // Python pickle protocol header
        content.extend_from_slice(&[0x80, 0x02]); // PROTO 2
        content.extend_from_slice(&[0x63]); // GLOBAL
        content.extend_from_slice(b"torch._utils\n");
        content.extend_from_slice(b"_rebuild_tensor_v2\n");

        // Mock tensor data structure
        content.extend_from_slice(&[0x71, 0x00]); // BINPUT 0
        content.extend_from_slice(&[0x29]); // EMPTY_TUPLE

        // Model metadata
        content.extend_from_slice(model_name.as_bytes());

        // Pad to desired size
        let target_size = size_kb.unwrap_or(3) * 1024;
        if content.len() < target_size {
            content.resize(target_size, 0);
        }

        fs::write(path, content)?;
        Ok(())
    }

    /// Create a mock SafeTensors file
    pub fn create_safetensors_file(path: &PathBuf, model_name: &str, _size_kb: Option<usize>) -> Result<()> {
        use safetensors::{serialize, Dtype};
        use std::collections::HashMap;

        let mut tensors = HashMap::new();

        // Create mock tensor data
        let embedding_data: Vec<f32> = (0..1000).map(|i| (i as f32) * 0.001).collect();
        let output_data: Vec<f32> = (0..1000).map(|i| (i as f32) * 0.002).collect();

        tensors.insert(
            "embeddings.weight".to_string(),
            (Dtype::F32, vec![100, 10], embedding_data.as_slice())
        );
        tensors.insert(
            "output.weight".to_string(),
            (Dtype::F32, vec![10, 100], output_data.as_slice())
        );

        // Add model metadata
        let metadata = HashMap::from([
            ("model_name".to_string(), model_name.to_string()),
            ("created_by".to_string(), "inferno_test".to_string()),
        ]);

        let serialized = serialize(&tensors, &Some(metadata))?;
        fs::write(path, serialized)?;
        Ok(())
    }

    /// Write string metadata to GGUF format
    fn write_metadata_string(content: &mut Vec<u8>, key: &str, value: &str) -> Result<()> {
        content.extend_from_slice(&(key.len() as u64).to_le_bytes());
        content.extend_from_slice(key.as_bytes());
        content.extend_from_slice(&8u32.to_le_bytes()); // String type
        content.extend_from_slice(&(value.len() as u64).to_le_bytes());
        content.extend_from_slice(value.as_bytes());
        Ok(())
    }

    /// Write u32 metadata to GGUF format
    fn write_metadata_u32(content: &mut Vec<u8>, key: &str, value: u32) -> Result<()> {
        content.extend_from_slice(&(key.len() as u64).to_le_bytes());
        content.extend_from_slice(key.as_bytes());
        content.extend_from_slice(&4u32.to_le_bytes()); // U32 type
        content.extend_from_slice(&value.to_le_bytes());
        Ok(())
    }

    /// Write tensor info to GGUF format
    fn write_tensor_info(content: &mut Vec<u8>, name: &str, dims: &[u64], ggml_type: u32) -> Result<()> {
        content.extend_from_slice(&(name.len() as u64).to_le_bytes());
        content.extend_from_slice(name.as_bytes());
        content.extend_from_slice(&(dims.len() as u32).to_le_bytes());
        for &dim in dims {
            content.extend_from_slice(&dim.to_le_bytes());
        }
        content.extend_from_slice(&ggml_type.to_le_bytes());
        content.extend_from_slice(&0u64.to_le_bytes()); // Offset placeholder
        Ok(())
    }
}

/// Configuration fixtures for various components
pub struct ConfigFixtures;

impl ConfigFixtures {
    /// Create a test backend configuration
    pub fn backend_config() -> inferno::backends::BackendConfig {
        inferno::backends::BackendConfig {
            gpu_enabled: false,
            gpu_device: None,
            cpu_threads: Some(2),
            context_size: 512,
            batch_size: 8,
            memory_map: true,
        }
    }

    /// Create a test cache configuration
    pub fn cache_config(cache_dir: Option<PathBuf>) -> inferno::cache::CacheConfig {
        inferno::cache::CacheConfig {
            max_cached_models: 3,
            max_memory_mb: 1024,
            model_ttl_seconds: 300,
            enable_warmup: false,
            warmup_strategy: inferno::cache::WarmupStrategy::UsageBased,
            always_warm: vec![],
            predictive_loading: false,
            usage_window_seconds: 3600,
            min_usage_frequency: 0.1,
            memory_based_eviction: true,
            persist_cache: cache_dir.is_some(),
            cache_dir,
        }
    }

    /// Create a test audit configuration
    pub fn audit_config(audit_dir: PathBuf) -> inferno::audit::AuditConfig {
        inferno::audit::AuditConfig {
            enabled: true,
            log_directory: audit_dir,
            max_file_size_mb: 10,
            max_files: 5,
            compression: inferno::audit::CompressionType::Zstd,
            compression_level: 1,
            encryption: inferno::audit::EncryptionConfig {
                enabled: false,
                algorithm: "AES-256-GCM".to_string(),
                key_derivation: "PBKDF2".to_string(),
                key_rotation_days: 30,
                master_key_path: None,
            },
            retention: inferno::audit::RetentionPolicy {
                default_retention_days: 365,
                critical_retention_days: 2555,
                audit_log_retention_days: 2555,
                compliance_retention_days: 2555,
                max_storage_gb: 100,
                auto_archive: true,
                archive_compression: inferno::audit::CompressionType::Zstd,
            },
            compliance: inferno::audit::ComplianceConfig {
                enable_sox: false,
                enable_hipaa: false,
                enable_gdpr: false,
                enable_pci: false,
                custom_requirements: HashMap::new(),
            },
            alerting: inferno::audit::AlertingConfig {
                enabled: false,
                alert_directory: audit_dir.join("alerts"),
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
            flush_interval_seconds: 2,
            async_processing: true,
            enable_metrics: true,
            debug_mode: true,
        }
    }

    /// Create a test batch queue configuration
    pub fn batch_queue_config(storage_dir: Option<PathBuf>) -> inferno::batch::queue::JobQueueConfig {
        inferno::batch::queue::JobQueueConfig {
            max_queues: 10,
            max_jobs_per_queue: 100,
            default_timeout_minutes: 30,
            max_retries: 3,
            cleanup_interval_seconds: 60,
            metrics_retention_hours: 24,
            persistent_storage: storage_dir.is_some(),
            storage_path: storage_dir,
            enable_metrics: true,
            enable_deadletter_queue: true,
            max_concurrent_jobs: 3,
            job_timeout_seconds: 300,
            retry_delay_seconds: 5,
            max_retry_delay_seconds: 300,
            exponential_backoff: true,
        }
    }

    /// Create a test response cache configuration
    pub fn response_cache_config(cache_dir: Option<PathBuf>) -> inferno::response_cache::ResponseCacheConfig {
        inferno::response_cache::ResponseCacheConfig {
            enabled: true,
            max_entries: 1000,
            ttl_seconds: 3600,
            max_memory_mb: 100,
            compression_enabled: true,
            compression_algorithm: "zstd".to_string(),
            compression_level: 3,
            persistence_enabled: cache_dir.is_some(),
            persistence_path: cache_dir,
            enable_metrics: true,
        }
    }

    /// Create a test dashboard configuration
    pub fn dashboard_config(data_dir: PathBuf, models_dir: PathBuf) -> inferno::dashboard::DashboardConfig {
        inferno::dashboard::DashboardConfig {
            bind_address: "127.0.0.1".to_string(),
            port: 0, // Random port for testing
            data_dir: Some(data_dir),
            models_dir: Some(models_dir),
            cache_dir: None,
            enable_auth: false,
            cors_enabled: true,
            max_connections: 100,
            request_timeout_seconds: 30,
            static_files_dir: None,
            ssl_cert_path: None,
            ssl_key_path: None,
            api_keys: Vec::new(),
            rate_limit_requests_per_minute: 1000,
            backup_enabled: false,
            backup_interval_hours: 24,
            backup_retention_days: 7,
        }
    }

    /// Create a test conversion configuration
    pub fn conversion_config() -> inferno::conversion::ConversionConfig {
        inferno::conversion::ConversionConfig {
            optimization: inferno::conversion::OptimizationOptions {
                level: inferno::conversion::OptimizationLevel::Balanced,
                quantization: Some(inferno::conversion::QuantizationType::Q4_0),
                precision: Some(inferno::conversion::Precision::F16),
                context_length: Some(2048),
                batch_size: Some(32),
                preserve_metadata: true,
            },
            input_format: None,
            output_format: inferno::conversion::ModelFormat::Gguf,
            validate_conversion: true,
            backup_original: false,
        }
    }
}

/// Data fixtures for creating test data
pub struct DataFixtures;

impl DataFixtures {
    /// Create a test model info structure
    pub fn model_info(id: &str, name: &str, path: PathBuf, format: &str) -> inferno::models::ModelInfo {
        inferno::models::ModelInfo {
            id: id.to_string(),
            name: name.to_string(),
            path,
            format: format.to_string(),
            size: 4096, // 4KB default
            metadata: HashMap::from([
                ("test_model".to_string(), "true".to_string()),
                ("created_by".to_string(), "inferno_test".to_string()),
            ]),
            backend_type: None,
            created_at: SystemTime::now(),
            modified_at: SystemTime::now(),
            checksum: None,
        }
    }

    /// Create test inference parameters
    pub fn inference_params() -> inferno::backends::InferenceParams {
        inferno::backends::InferenceParams {
            max_tokens: 100,
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            stream: false,
            stop_sequences: vec![],
            seed: None,
        }
    }

    /// Create test batch input
    pub fn batch_input(id: &str, content: &str) -> inferno::batch::BatchInput {
        inferno::batch::BatchInput {
            id: id.to_string(),
            content: content.to_string(),
            metadata: Some(HashMap::from([
                ("test_input".to_string(), "true".to_string()),
            ])),
        }
    }

    /// Create test batch job
    pub fn batch_job(id: &str, model_name: &str, inputs: Vec<inferno::batch::BatchInput>) -> inferno::batch::queue::BatchJob {
        inferno::batch::queue::BatchJob {
            id: id.to_string(),
            name: format!("Test Job {}", id),
            description: Some("Integration test batch job".to_string()),
            priority: inferno::batch::queue::JobPriority::Normal,
            inputs,
            inference_params: Self::inference_params(),
            model_name: model_name.to_string(),
            batch_config: inferno::batch::BatchConfig {
                batch_size: 10,
                timeout_seconds: 300,
                parallel_processing: true,
                max_parallel_batches: 2,
                enable_streaming: false,
                output_format: "json".to_string(),
                compression_enabled: false,
                checkpointing_enabled: false,
                checkpoint_interval_seconds: 60,
            },
            schedule: None,
            dependencies: vec![],
            resource_requirements: inferno::batch::queue::ResourceRequirements {
                min_memory_mb: 256,
                min_cpu_cores: 1,
                min_gpu_memory_mb: None,
                required_gpu: false,
                estimated_duration_seconds: Some(60),
                max_memory_mb: Some(1024),
                max_cpu_cores: Some(2),
            },
            timeout_minutes: Some(10),
            retry_count: 0,
            max_retries: 2,
            created_at: SystemTime::now(),
            scheduled_at: None,
            tags: HashMap::from([
                ("test".to_string(), "true".to_string()),
                ("environment".to_string(), "integration".to_string()),
            ]),
            metadata: HashMap::from([
                ("test_run_id".to_string(), Uuid::new_v4().to_string()),
            ]),
        }
    }

    /// Create test audit event
    pub fn audit_event(event_type: inferno::audit::EventType, severity: inferno::audit::Severity) -> inferno::audit::AuditEvent {
        inferno::audit::AuditEvent {
            id: Uuid::new_v4().to_string(),
            timestamp: SystemTime::now(),
            event_type,
            severity,
            actor: inferno::audit::Actor {
                actor_type: inferno::audit::ActorType::User,
                id: "test_user".to_string(),
                name: "Test User".to_string(),
                ip_address: Some("127.0.0.1".to_string()),
                user_agent: Some("test-agent/1.0".to_string()),
                session_id: Some(Uuid::new_v4().to_string()),
            },
            resource: inferno::audit::Resource {
                resource_type: inferno::audit::ResourceType::Model,
                id: "test_resource".to_string(),
                name: "Test Resource".to_string(),
                path: Some("/test/resource".to_string()),
                attributes: HashMap::new(),
            },
            action: "test_action".to_string(),
            details: inferno::audit::EventDetails {
                description: "Test audit event for integration testing".to_string(),
                request_id: Some(Uuid::new_v4().to_string()),
                trace_id: None,
                span_id: None,
                parameters: HashMap::new(),
                response_data: None,
                error_details: None,
            },
            context: inferno::audit::EventContext {
                source_component: "integration_test".to_string(),
                environment: "test".to_string(),
                version: "1.0.0".to_string(),
                region: None,
                availability_zone: None,
                cluster: None,
                node: None,
                tenant_id: None,
                correlation_id: None,
            },
            outcome: inferno::audit::EventOutcome {
                success: true,
                status_code: Some(200),
                duration_ms: Some(100),
                bytes_processed: Some(1024),
                records_affected: Some(1),
                resource_usage: HashMap::new(),
            },
            metadata: HashMap::from([
                ("test_event".to_string(), serde_json::Value::Bool(true)),
            ]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_model_fixtures() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Test GGUF file creation
        let gguf_path = temp_dir.path().join("test.gguf");
        ModelFixtures::create_gguf_file(&gguf_path, "test_model", Some(4))?;
        assert!(gguf_path.exists());
        assert!(gguf_path.metadata()?.len() >= 4096);

        // Test ONNX file creation
        let onnx_path = temp_dir.path().join("test.onnx");
        ModelFixtures::create_onnx_file(&onnx_path, "test_model", Some(2))?;
        assert!(onnx_path.exists());
        assert!(onnx_path.metadata()?.len() >= 2048);

        Ok(())
    }

    #[test]
    fn test_config_fixtures() {
        let backend_config = ConfigFixtures::backend_config();
        assert_eq!(backend_config.context_size, 512);
        assert!(!backend_config.gpu_enabled);

        let cache_config = ConfigFixtures::cache_config(None);
        assert_eq!(cache_config.max_cached_models, 3);
        assert!(!cache_config.persist_cache);
    }

    #[test]
    fn test_data_fixtures() {
        let temp_dir = TempDir::new().unwrap();
        let model_path = temp_dir.path().join("test.gguf");

        let model_info = DataFixtures::model_info("test_id", "Test Model", model_path, "gguf");
        assert_eq!(model_info.id, "test_id");
        assert_eq!(model_info.name, "Test Model");
        assert_eq!(model_info.format, "gguf");

        let inference_params = DataFixtures::inference_params();
        assert_eq!(inference_params.max_tokens, 100);
        assert_eq!(inference_params.temperature, 0.7);

        let batch_input = DataFixtures::batch_input("input_1", "test content");
        assert_eq!(batch_input.id, "input_1");
        assert_eq!(batch_input.content, "test content");
    }
}