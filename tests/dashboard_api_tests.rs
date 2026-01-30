#[cfg(test)]
mod dashboard_api_tests {
    use super::*;
    use axum::{
        Router,
        http::{Method, Request, StatusCode},
    };
    use hyper::Body;
    use inferno::dashboard::{
        CreateDeploymentRequest, CreateModelRequest, DashboardConfig, DashboardServer,
        DeployModelRequest, ScaleDeploymentRequest, UpdateDeploymentRequest, UpdateModelRequest,
    };
    use serde_json::json;
    use tower::ServiceExt;

    async fn create_test_server() -> Router {
        let config = DashboardConfig::default();
        let server = DashboardServer::new(config).unwrap();
        server.load_initial_data().await.unwrap();
        server.create_router().await.unwrap()
    }

    #[tokio::test]
    async fn test_create_model_success() {
        let app = create_test_server().await;

        let request_body = CreateModelRequest {
            name: "test-model".to_string(),
            version: "v1.0".to_string(),
            format: "GGUF".to_string(),
            description: "Test model for unit testing".to_string(),
            tags: vec!["test".to_string(), "unit-test".to_string()],
        };

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/models")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_create_model_duplicate_name() {
        let app = create_test_server().await;

        let request_body = CreateModelRequest {
            name: "LLaMA 7B".to_string(), // This name already exists in test data
            version: "v2.0".to_string(),
            format: "GGUF".to_string(),
            description: "Duplicate model name test".to_string(),
            tags: vec!["test".to_string()],
        };

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/models")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn test_create_deployment_success() {
        let app = create_test_server().await;

        let request_body = CreateDeploymentRequest {
            model_id: "llama-7b".to_string(), // This model exists in test data
            environment: "testing".to_string(),
            replicas: 2,
        };

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/deployments")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_create_deployment_invalid_model() {
        let app = create_test_server().await;

        let request_body = CreateDeploymentRequest {
            model_id: "nonexistent-model".to_string(),
            environment: "testing".to_string(),
            replicas: 2,
        };

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/deployments")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_scale_deployment_invalid_replicas() {
        let app = create_test_server().await;

        let request_body = ScaleDeploymentRequest {
            replicas: 0, // Invalid replica count
        };

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/deployments/deploy-001/scale") // This deployment exists in test data
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_get_node_success() {
        let app = create_test_server().await;

        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/nodes/node-001") // This node exists in test data
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_node_not_found() {
        let app = create_test_server().await;

        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/nodes/nonexistent-node")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_metrics_history_success() {
        let app = create_test_server().await;

        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/metrics/history?interval=1h")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_metrics_history_invalid_time_range() {
        let app = create_test_server().await;

        // Create a request with start_time after end_time
        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/metrics/history?start_time=2024-01-02T00:00:00Z&end_time=2024-01-01T00:00:00Z")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_export_metrics_json() {
        let app = create_test_server().await;

        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/metrics/export?format=json")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_export_metrics_unsupported_format() {
        let app = create_test_server().await;

        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/metrics/export?format=xml")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
