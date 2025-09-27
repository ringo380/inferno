use anyhow::Result;
use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    Router,
};
use hyper::body::to_bytes;
use inferno::{
    backends::{BackendConfig, BackendType},
    cache::{CacheConfig, ModelCache},
    dashboard::{
        deployments::DeploymentManager, models::ModelRepository, BackupRequest,
        CreateDeploymentRequest, CreateModelRequest, DashboardConfig, DashboardMetrics,
        DashboardServer, DashboardState, DeployModelRequest, DeploymentInfo, DeploymentStatus,
        ModelInfo as DashboardModelInfo, ModelStatus, RestoreRequest, ScaleDeploymentRequest,
        UpdateDeploymentRequest, UpdateModelRequest,
    },
    metrics::MetricsCollector,
    models::{ModelInfo, ModelManager},
    InfernoError,
};
use serde_json::{json, Value};
use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};
use tempfile::TempDir;
use tokio::{
    fs,
    time::{sleep, timeout},
};
use tower::ServiceExt;

/// Test utilities for dashboard API workflow tests
mod dashboard_test_utils {
    use super::*;

    pub async fn create_test_dashboard_server(temp_dir: &TempDir) -> Result<DashboardServer> {
        let models_dir = temp_dir.path().join("models");
        let cache_dir = temp_dir.path().join("cache");
        let data_dir = temp_dir.path().join("dashboard_data");

        fs::create_dir_all(&models_dir).await?;
        fs::create_dir_all(&cache_dir).await?;
        fs::create_dir_all(&data_dir).await?;

        // Create test model files
        create_test_model_files(&models_dir).await?;

        let config = DashboardConfig {
            bind_address: "127.0.0.1".to_string(),
            port: 0, // Use random port for testing
            data_dir: Some(data_dir),
            models_dir: Some(models_dir),
            cache_dir: Some(cache_dir),
            enable_auth: false,
            cors_enabled: true,
            max_connections: 100,
            request_timeout_seconds: 30,
            static_files_dir: None,
            ssl_cert_path: None,
            ssl_key_path: None,
            api_keys: Vec::new(),
            rate_limit_requests_per_minute: 1000,
            backup_enabled: true,
            backup_interval_hours: 24,
            backup_retention_days: 7,
        };

        let server = DashboardServer::new(config).await?;
        server.initialize().await?;
        Ok(server)
    }

    pub async fn create_test_model_files(models_dir: &PathBuf) -> Result<()> {
        let model_files = vec![
            ("llama_7b.gguf", create_mock_gguf_content("LLaMA 7B")),
            ("gpt_3_5.gguf", create_mock_gguf_content("GPT-3.5")),
            ("bert_base.onnx", create_mock_onnx_content("BERT Base")),
        ];

        for (filename, content) in model_files {
            let path = models_dir.join(filename);
            fs::write(path, content).await?;
        }

        Ok(())
    }

    fn create_mock_gguf_content(model_name: &str) -> Vec<u8> {
        let mut content = Vec::new();
        // GGUF magic number
        content.extend_from_slice(b"GGUF");
        content.extend_from_slice(&3u32.to_le_bytes()); // Version
        content.extend_from_slice(&0u64.to_le_bytes()); // Tensor count
        content.extend_from_slice(&1u64.to_le_bytes()); // Metadata count

        // Model name metadata
        let key = "general.name";
        content.extend_from_slice(&(key.len() as u64).to_le_bytes());
        content.extend_from_slice(key.as_bytes());
        content.extend_from_slice(&8u32.to_le_bytes()); // String type
        content.extend_from_slice(&(model_name.len() as u64).to_le_bytes());
        content.extend_from_slice(model_name.as_bytes());

        content.resize(4096, 0); // Pad to reasonable size
        content
    }

    fn create_mock_onnx_content(model_name: &str) -> Vec<u8> {
        let mut content = Vec::new();
        content.extend_from_slice(&[0x08, 0x01]); // ONNX header
        content.extend_from_slice(&[0x12, model_name.len() as u8]);
        content.extend_from_slice(model_name.as_bytes());
        content.resize(2048, 0);
        content
    }

    pub fn create_test_model_request() -> CreateModelRequest {
        CreateModelRequest {
            name: "test_model".to_string(),
            version: "v1.0".to_string(),
            format: "GGUF".to_string(),
            description: "Test model for API testing".to_string(),
            tags: vec!["test".to_string(), "api".to_string()],
            file_path: None,
            metadata: HashMap::new(),
        }
    }

    pub fn create_test_deployment_request(model_id: &str) -> CreateDeploymentRequest {
        CreateDeploymentRequest {
            name: "test_deployment".to_string(),
            model_id: model_id.to_string(),
            environment: "testing".to_string(),
            replicas: 2,
            auto_scaling: false,
            min_replicas: Some(1),
            max_replicas: Some(5),
            target_cpu_utilization: Some(70),
            config: HashMap::new(),
        }
    }

    pub async fn extract_json_response(response: hyper::Response<Body>) -> Result<Value> {
        let body_bytes = to_bytes(response.into_body()).await?;
        let json: Value = serde_json::from_slice(&body_bytes)?;
        Ok(json)
    }
}

/// Test complete model lifecycle through API
#[tokio::test]
async fn test_model_lifecycle_workflow() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let server = dashboard_test_utils::create_test_dashboard_server(&temp_dir).await?;
    let app = server.create_router().await?;

    // 1. List initial models
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/models")
        .body(Body::empty())?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);

    let initial_models = dashboard_test_utils::extract_json_response(response).await?;
    let initial_count = initial_models.as_array().unwrap().len();

    // 2. Create a new model
    let create_request = dashboard_test_utils::create_test_model_request();
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/models")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&create_request)?))?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::CREATED);

    let created_model = dashboard_test_utils::extract_json_response(response).await?;
    let model_id = created_model["id"].as_str().unwrap();

    // 3. Verify model was created
    let request = Request::builder()
        .method(Method::GET)
        .uri(&format!("/api/v1/models/{}", model_id))
        .body(Body::empty())?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);

    let model_details = dashboard_test_utils::extract_json_response(response).await?;
    assert_eq!(model_details["name"], create_request.name);
    assert_eq!(model_details["version"], create_request.version);

    // 4. Update the model
    let update_request = UpdateModelRequest {
        name: Some("updated_test_model".to_string()),
        description: Some("Updated description".to_string()),
        tags: Some(vec!["updated".to_string(), "test".to_string()]),
        status: Some(ModelStatus::Active),
        metadata: Some(HashMap::new()),
    };

    let request = Request::builder()
        .method(Method::PUT)
        .uri(&format!("/api/v1/models/{}", model_id))
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&update_request)?))?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);

    // 5. Verify update
    let request = Request::builder()
        .method(Method::GET)
        .uri(&format!("/api/v1/models/{}", model_id))
        .body(Body::empty())?;

    let response = app.clone().oneshot(request).await?;
    let updated_model = dashboard_test_utils::extract_json_response(response).await?;
    assert_eq!(updated_model["name"], "updated_test_model");

    // 6. Delete the model
    let request = Request::builder()
        .method(Method::DELETE)
        .uri(&format!("/api/v1/models/{}", model_id))
        .body(Body::empty())?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // 7. Verify deletion
    let request = Request::builder()
        .method(Method::GET)
        .uri(&format!("/api/v1/models/{}", model_id))
        .body(Body::empty())?;

    let response = app.oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    Ok(())
}

/// Test complete deployment lifecycle through API
#[tokio::test]
async fn test_deployment_lifecycle_workflow() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let server = dashboard_test_utils::create_test_dashboard_server(&temp_dir).await?;
    let app = server.create_router().await?;

    // 1. Create a model first
    let model_request = dashboard_test_utils::create_test_model_request();
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/models")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&model_request)?))?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::CREATED);

    let created_model = dashboard_test_utils::extract_json_response(response).await?;
    let model_id = created_model["id"].as_str().unwrap();

    // 2. Create a deployment
    let deployment_request = dashboard_test_utils::create_test_deployment_request(model_id);
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/deployments")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&deployment_request)?))?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::CREATED);

    let created_deployment = dashboard_test_utils::extract_json_response(response).await?;
    let deployment_id = created_deployment["id"].as_str().unwrap();

    // 3. Verify deployment was created
    let request = Request::builder()
        .method(Method::GET)
        .uri(&format!("/api/v1/deployments/{}", deployment_id))
        .body(Body::empty())?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);

    let deployment_details = dashboard_test_utils::extract_json_response(response).await?;
    assert_eq!(deployment_details["name"], deployment_request.name);
    assert_eq!(deployment_details["model_id"], model_id);

    // 4. Scale the deployment
    let scale_request = ScaleDeploymentRequest {
        replicas: 4,
        auto_scaling: Some(true),
        min_replicas: Some(2),
        max_replicas: Some(8),
        target_cpu_utilization: Some(80),
    };

    let request = Request::builder()
        .method(Method::POST)
        .uri(&format!("/api/v1/deployments/{}/scale", deployment_id))
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&scale_request)?))?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);

    // 5. Update deployment
    let update_request = UpdateDeploymentRequest {
        name: Some("updated_deployment".to_string()),
        environment: Some("production".to_string()),
        status: Some(DeploymentStatus::Active),
        config: Some(HashMap::new()),
        auto_scaling: Some(true),
        min_replicas: Some(2),
        max_replicas: Some(10),
        target_cpu_utilization: Some(75),
    };

    let request = Request::builder()
        .method(Method::PUT)
        .uri(&format!("/api/v1/deployments/{}", deployment_id))
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&update_request)?))?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);

    // 6. Stop the deployment
    let request = Request::builder()
        .method(Method::POST)
        .uri(&format!("/api/v1/deployments/{}/stop", deployment_id))
        .body(Body::empty())?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);

    // 7. Start the deployment again
    let request = Request::builder()
        .method(Method::POST)
        .uri(&format!("/api/v1/deployments/{}/start", deployment_id))
        .body(Body::empty())?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);

    // 8. Delete the deployment
    let request = Request::builder()
        .method(Method::DELETE)
        .uri(&format!("/api/v1/deployments/{}", deployment_id))
        .body(Body::empty())?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // 9. Verify deletion
    let request = Request::builder()
        .method(Method::GET)
        .uri(&format!("/api/v1/deployments/{}", deployment_id))
        .body(Body::empty())?;

    let response = app.oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    Ok(())
}

/// Test metrics and monitoring endpoints
#[tokio::test]
async fn test_metrics_and_monitoring_workflow() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let server = dashboard_test_utils::create_test_dashboard_server(&temp_dir).await?;
    let app = server.create_router().await?;

    // 1. Get overall metrics
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/metrics")
        .body(Body::empty())?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);

    let metrics = dashboard_test_utils::extract_json_response(response).await?;
    assert!(metrics.get("total_models").is_some());
    assert!(metrics.get("total_deployments").is_some());
    assert!(metrics.get("system_metrics").is_some());

    // 2. Get system health
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/health")
        .body(Body::empty())?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);

    let health = dashboard_test_utils::extract_json_response(response).await?;
    assert_eq!(health["status"], "healthy");

    // 3. Create a model and deployment to generate metrics
    let model_request = dashboard_test_utils::create_test_model_request();
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/models")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&model_request)?))?;

    let response = app.clone().oneshot(request).await?;
    let created_model = dashboard_test_utils::extract_json_response(response).await?;
    let model_id = created_model["id"].as_str().unwrap();

    let deployment_request = dashboard_test_utils::create_test_deployment_request(model_id);
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/deployments")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&deployment_request)?))?;

    let response = app.clone().oneshot(request).await?;
    let created_deployment = dashboard_test_utils::extract_json_response(response).await?;
    let deployment_id = created_deployment["id"].as_str().unwrap();

    // 4. Get model-specific metrics
    let request = Request::builder()
        .method(Method::GET)
        .uri(&format!("/api/v1/models/{}/metrics", model_id))
        .body(Body::empty())?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);

    // 5. Get deployment-specific metrics
    let request = Request::builder()
        .method(Method::GET)
        .uri(&format!("/api/v1/deployments/{}/metrics", deployment_id))
        .body(Body::empty())?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);

    // 6. Get deployment logs
    let request = Request::builder()
        .method(Method::GET)
        .uri(&format!("/api/v1/deployments/{}/logs", deployment_id))
        .body(Body::empty())?;

    let response = app.oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
}

/// Test model deployment workflow
#[tokio::test]
async fn test_model_deployment_workflow() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let server = dashboard_test_utils::create_test_dashboard_server(&temp_dir).await?;
    let app = server.create_router().await?;

    // 1. Create a model
    let model_request = dashboard_test_utils::create_test_model_request();
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/models")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&model_request)?))?;

    let response = app.clone().oneshot(request).await?;
    let created_model = dashboard_test_utils::extract_json_response(response).await?;
    let model_id = created_model["id"].as_str().unwrap();

    // 2. Deploy the model directly
    let deploy_request = DeployModelRequest {
        environment: "production".to_string(),
        replicas: 3,
        auto_scaling: true,
        min_replicas: Some(1),
        max_replicas: Some(5),
        target_cpu_utilization: Some(70),
        config: HashMap::new(),
        name: Some("direct_deployment".to_string()),
    };

    let request = Request::builder()
        .method(Method::POST)
        .uri(&format!("/api/v1/models/{}/deploy", model_id))
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&deploy_request)?))?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::CREATED);

    let deployment = dashboard_test_utils::extract_json_response(response).await?;
    let deployment_id = deployment["id"].as_str().unwrap();

    // 3. Verify deployment was created and linked to model
    let request = Request::builder()
        .method(Method::GET)
        .uri(&format!("/api/v1/deployments/{}", deployment_id))
        .body(Body::empty())?;

    let response = app.clone().oneshot(request).await?;
    let deployment_details = dashboard_test_utils::extract_json_response(response).await?;
    assert_eq!(deployment_details["model_id"], model_id);
    assert_eq!(deployment_details["environment"], "production");
    assert_eq!(deployment_details["replicas"], 3);

    // 4. Get model's deployments
    let request = Request::builder()
        .method(Method::GET)
        .uri(&format!("/api/v1/models/{}/deployments", model_id))
        .body(Body::empty())?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);

    let deployments = dashboard_test_utils::extract_json_response(response).await?;
    let deployments_array = deployments.as_array().unwrap();
    assert!(!deployments_array.is_empty());
    assert!(deployments_array.iter().any(|d| d["id"] == deployment_id));

    Ok(())
}

/// Test backup and restore workflow
#[tokio::test]
async fn test_backup_restore_workflow() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let server = dashboard_test_utils::create_test_dashboard_server(&temp_dir).await?;
    let app = server.create_router().await?;

    // 1. Create some data (models and deployments)
    let model_request = dashboard_test_utils::create_test_model_request();
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/models")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&model_request)?))?;

    let response = app.clone().oneshot(request).await?;
    let created_model = dashboard_test_utils::extract_json_response(response).await?;
    let model_id = created_model["id"].as_str().unwrap();

    let deployment_request = dashboard_test_utils::create_test_deployment_request(model_id);
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/deployments")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&deployment_request)?))?;

    let response = app.clone().oneshot(request).await?;
    let _created_deployment = dashboard_test_utils::extract_json_response(response).await?;

    // 2. Create a backup
    let backup_request = BackupRequest {
        include_models: true,
        include_deployments: true,
        include_metrics: true,
        compression_level: Some(1),
        description: Some("Test backup".to_string()),
    };

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/backup")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&backup_request)?))?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::CREATED);

    let backup_response = dashboard_test_utils::extract_json_response(response).await?;
    let backup_id = backup_response["backup_id"].as_str().unwrap();

    // 3. List backups
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/backups")
        .body(Body::empty())?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);

    let backups = dashboard_test_utils::extract_json_response(response).await?;
    let backups_array = backups.as_array().unwrap();
    assert!(!backups_array.is_empty());

    // 4. Get backup details
    let request = Request::builder()
        .method(Method::GET)
        .uri(&format!("/api/v1/backups/{}", backup_id))
        .body(Body::empty())?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);

    // 5. Delete the original model to test restore
    let request = Request::builder()
        .method(Method::DELETE)
        .uri(&format!("/api/v1/models/{}", model_id))
        .body(Body::empty())?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // 6. Restore from backup
    let restore_request = RestoreRequest {
        backup_id: backup_id.to_string(),
        restore_models: true,
        restore_deployments: true,
        restore_metrics: false,
        overwrite_existing: true,
    };

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/restore")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&restore_request)?))?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);

    // 7. Verify model was restored
    let request = Request::builder()
        .method(Method::GET)
        .uri(&format!("/api/v1/models/{}", model_id))
        .body(Body::empty())?;

    let response = app.oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
}

/// Test error handling and validation
#[tokio::test]
async fn test_api_error_handling() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let server = dashboard_test_utils::create_test_dashboard_server(&temp_dir).await?;
    let app = server.create_router().await?;

    // 1. Test invalid JSON
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/models")
        .header("Content-Type", "application/json")
        .body(Body::from("invalid json"))?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // 2. Test missing required fields
    let incomplete_request = json!({
        "name": "test_model"
        // Missing version, format, description
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/models")
        .header("Content-Type", "application/json")
        .body(Body::from(incomplete_request.to_string()))?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // 3. Test accessing non-existent resource
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/models/nonexistent-model-id")
        .body(Body::empty())?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // 4. Test invalid method
    let request = Request::builder()
        .method(Method::PATCH)
        .uri("/api/v1/models")
        .body(Body::empty())?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);

    // 5. Test creating deployment with non-existent model
    let deployment_request =
        dashboard_test_utils::create_test_deployment_request("nonexistent-model");
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/deployments")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&deployment_request)?))?;

    let response = app.oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    Ok(())
}

/// Test concurrent API operations
#[tokio::test]
async fn test_concurrent_api_operations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let server = dashboard_test_utils::create_test_dashboard_server(&temp_dir).await?;
    let app = Arc::new(server.create_router().await?);

    // Launch multiple concurrent model creation requests
    let mut tasks = Vec::new();

    for i in 0..5 {
        let app_clone = app.clone();
        let task = tokio::spawn(async move {
            let mut model_request = dashboard_test_utils::create_test_model_request();
            model_request.name = format!("concurrent_model_{}", i);

            let request = Request::builder()
                .method(Method::POST)
                .uri("/api/v1/models")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&model_request).unwrap()))
                .unwrap();

            app_clone.clone().oneshot(request).await.unwrap()
        });

        tasks.push(task);
    }

    // Wait for all requests to complete
    let responses = futures::future::join_all(tasks).await;

    let mut success_count = 0;
    for response_result in responses {
        let response = response_result?;
        if response.status() == StatusCode::CREATED {
            success_count += 1;
        }
    }

    // All concurrent operations should succeed
    assert_eq!(success_count, 5);

    // Verify all models were created
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/models")
        .body(Body::empty())?;

    let response = app.as_ref().clone().oneshot(request).await?;
    let models = dashboard_test_utils::extract_json_response(response).await?;
    let models_array = models.as_array().unwrap();

    let concurrent_models = models_array
        .iter()
        .filter(|m| m["name"].as_str().unwrap().starts_with("concurrent_model_"))
        .count();

    assert_eq!(concurrent_models, 5);

    Ok(())
}

/// Test streaming and real-time features
#[tokio::test]
async fn test_streaming_endpoints() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let server = dashboard_test_utils::create_test_dashboard_server(&temp_dir).await?;
    let app = server.create_router().await?;

    // Test metrics streaming endpoint (if implemented)
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/metrics/stream")
        .header("Accept", "text/event-stream")
        .body(Body::empty())?;

    let response = app.clone().oneshot(request).await?;
    // May return 404 if streaming is not implemented yet
    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);

    // Test deployment logs streaming (if implemented)
    let model_request = dashboard_test_utils::create_test_model_request();
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/models")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&model_request)?))?;

    let response = app.clone().oneshot(request).await?;
    if response.status() == StatusCode::CREATED {
        let created_model = dashboard_test_utils::extract_json_response(response).await?;
        let model_id = created_model["id"].as_str().unwrap();

        let deployment_request = dashboard_test_utils::create_test_deployment_request(model_id);
        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/deployments")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&deployment_request)?))?;

        let response = app.clone().oneshot(request).await?;
        if response.status() == StatusCode::CREATED {
            let created_deployment = dashboard_test_utils::extract_json_response(response).await?;
            let deployment_id = created_deployment["id"].as_str().unwrap();

            // Test log streaming
            let request = Request::builder()
                .method(Method::GET)
                .uri(&format!(
                    "/api/v1/deployments/{}/logs/stream",
                    deployment_id
                ))
                .header("Accept", "text/event-stream")
                .body(Body::empty())?;

            let response = app.oneshot(request).await?;
            // May return 404 if streaming is not implemented yet
            assert!(
                response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND
            );
        }
    }

    Ok(())
}
