#![allow(
    clippy::single_component_path_imports,
    clippy::println_empty_string,
    clippy::redundant_pattern_matching,
    clippy::useless_vec,
    clippy::needless_borrows_for_generic_args,
    dead_code,
    unused_variables
)]

//! Dashboard API Usage Examples
//!
//! This example demonstrates how to interact with the Inferno Dashboard API endpoints
//! that manage models, deployments, nodes, and metrics.

use reqwest;
use serde_json::json;
use std::error::Error;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::new();
    let base_url = "http://localhost:8080/api/v1";

    println!("🚀 Inferno Dashboard API Examples");
    println!("=================================\n");

    // 1. Model Management Examples
    println!("📋 Model Management:");

    // Create a new model
    let create_model_request = json!({
        "name": "GPT-3.5-Turbo-Fine-Tuned",
        "version": "v1.2.0",
        "format": "GGUF",
        "description": "Fine-tuned GPT-3.5 model for specific domain tasks",
        "tags": ["gpt", "fine-tuned", "domain-specific"]
    });

    let response = client
        .post(&format!("{}/models", base_url))
        .json(&create_model_request)
        .send()
        .await?;

    println!("✓ Create Model Response: {}", response.status());
    if response.status().is_success() {
        let model: serde_json::Value = response.json().await?;
        println!(
            "  Created model ID: {}",
            model["data"]["id"].as_str().unwrap_or("N/A")
        );
    }

    // List all models
    let response = client.get(&format!("{}/models", base_url)).send().await?;

    println!("✓ List Models Response: {}", response.status());

    // Get specific model metrics
    let response = client
        .get(&format!("{}/models/llama-7b/metrics", base_url))
        .send()
        .await?;

    println!("✓ Model Metrics Response: {}", response.status());
    if response.status().is_success() {
        let metrics: serde_json::Value = response.json().await?;
        println!(
            "  Model requests/sec: {}",
            metrics["inference_metrics"]["requests_per_second"]
                .as_f64()
                .unwrap_or(0.0)
        );
    }

    // 2. Deployment Management Examples
    println!("\n🚀 Deployment Management:");

    // Create a new deployment
    let create_deployment_request = json!({
        "model_id": "llama-7b",
        "environment": "staging",
        "replicas": 3
    });

    let response = client
        .post(&format!("{}/deployments", base_url))
        .json(&create_deployment_request)
        .send()
        .await?;

    println!("✓ Create Deployment Response: {}", response.status());
    if response.status().is_success() {
        let deployment: serde_json::Value = response.json().await?;
        let deployment_id = deployment["data"]["id"].as_str().unwrap_or("");
        println!("  Created deployment ID: {}", deployment_id);

        // Scale the deployment
        let scale_request = json!({
            "replicas": 5
        });

        let response = client
            .post(&format!("{}/deployments/{}/scale", base_url, deployment_id))
            .json(&scale_request)
            .send()
            .await?;

        println!("✓ Scale Deployment Response: {}", response.status());
    }

    // List all deployments
    let response = client
        .get(&format!("{}/deployments", base_url))
        .send()
        .await?;

    println!("✓ List Deployments Response: {}", response.status());

    // 3. Node Management Examples
    println!("\n🖥️ Node Management:");

    // Get node information
    let response = client
        .get(&format!("{}/nodes/node-001", base_url))
        .send()
        .await?;

    println!("✓ Get Node Info Response: {}", response.status());
    if response.status().is_success() {
        let node: serde_json::Value = response.json().await?;
        println!(
            "  Node load: {}%",
            node["node"]["current_load"].as_f64().unwrap_or(0.0)
        );
        println!(
            "  Node status: {}",
            node["node"]["status"].as_str().unwrap_or("unknown")
        );
    }

    // Get node status
    let response = client
        .get(&format!("{}/nodes/node-001/status", base_url))
        .send()
        .await?;

    println!("✓ Get Node Status Response: {}", response.status());

    // 4. Metrics Management Examples
    println!("\n📊 Metrics Management:");

    // Get current system metrics
    let response = client.get(&format!("{}/metrics", base_url)).send().await?;

    println!("✓ System Metrics Response: {}", response.status());
    if response.status().is_success() {
        let metrics: serde_json::Value = response.json().await?;
        println!(
            "  CPU Usage: {}%",
            metrics["cpu_usage"].as_f64().unwrap_or(0.0)
        );
        println!(
            "  Memory Usage: {}%",
            metrics["memory_usage"].as_f64().unwrap_or(0.0)
        );
    }

    // Get metrics history
    let response = client
        .get(&format!("{}/metrics/history?interval=1h", base_url))
        .send()
        .await?;

    println!("✓ Metrics History Response: {}", response.status());
    if response.status().is_success() {
        let history: serde_json::Value = response.json().await?;
        println!(
            "  Data points: {}",
            history["data_points"].as_u64().unwrap_or(0)
        );
    }

    // Export metrics in different formats
    for format in &["json", "csv", "prometheus"] {
        let response = client
            .get(&format!("{}/metrics/export?format={}", base_url, format))
            .send()
            .await?;

        println!(
            "✓ Export Metrics ({}) Response: {}",
            format,
            response.status()
        );
    }

    // 5. System Health Check
    println!("\n🏥 System Health:");

    let response = client
        .get(&format!("{}/system/health", base_url))
        .send()
        .await?;

    println!("✓ Health Check Response: {}", response.status());
    if response.status().is_success() {
        let health: serde_json::Value = response.json().await?;
        println!(
            "  Status: {}",
            health["status"].as_str().unwrap_or("unknown")
        );
        println!(
            "  Version: {}",
            health["version"].as_str().unwrap_or("unknown")
        );
    }

    println!("\n🎉 All API examples completed!");
    Ok(())
}
