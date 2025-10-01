//! Distributed Command - New Architecture
//!
//! This module provides distributed inference and worker pool management.

use crate::{
    backends::InferenceParams,
    config::Config,
    distributed::DistributedInference,
    interfaces::cli::{Command, CommandContext, CommandOutput},
    metrics::MetricsCollector,
    models::ModelManager,
};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use std::{sync::Arc, time::Instant};
use tracing::info;

// ============================================================================
// DistributedStart - Start distributed server
// ============================================================================

/// Start distributed inference server
pub struct DistributedStart {
    config: Config,
    workers: usize,
    preload_model: Option<String>,
    load_balancing: bool,
    max_concurrent: usize,
}

impl DistributedStart {
    pub fn new(
        config: Config,
        workers: usize,
        preload_model: Option<String>,
        load_balancing: bool,
        max_concurrent: usize,
    ) -> Self {
        Self {
            config,
            workers,
            preload_model,
            load_balancing,
            max_concurrent,
        }
    }
}

#[async_trait]
impl Command for DistributedStart {
    fn name(&self) -> &str {
        "distributed start"
    }

    fn description(&self) -> &str {
        "Start distributed inference server"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        if self.workers > 0 && self.workers > 32 {
            anyhow::bail!("Worker count cannot exceed 32");
        }

        if self.max_concurrent == 0 {
            anyhow::bail!("Max concurrent must be greater than 0");
        }

        if self.max_concurrent > 100 {
            anyhow::bail!("Max concurrent cannot exceed 100");
        }

        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Starting distributed inference server");

        let model_manager = Arc::new(ModelManager::new(&self.config.models_dir));
        let metrics = Some(Arc::new({
            let (collector, processor) = MetricsCollector::new();
            processor.start();
            collector
        }));

        let mut distributed_config = self.config.distributed.clone();
        if self.workers > 0 {
            distributed_config.worker_count = self.workers;
        }
        distributed_config.load_balancing = self.load_balancing;
        distributed_config.max_concurrent_per_worker = self.max_concurrent;
        if self.preload_model.is_some() {
            distributed_config.preload_models = true;
        }

        let mut distributed = DistributedInference::new(
            distributed_config.clone(),
            self.config.backend_config.clone(),
            model_manager,
            metrics,
        )
        .await?;

        // Human-readable output
        if !ctx.json_output {
            println!("=== Distributed Inference Server Started ===");
            println!("Workers: {}", distributed_config.worker_count);
            println!(
                "Max Concurrent per Worker: {}",
                distributed_config.max_concurrent_per_worker
            );
            println!("Load Balancing: {}", self.load_balancing);
            if let Some(ref model) = self.preload_model {
                println!("Preloading Model: {}", model);
            }
            println!("\nServer is running. Press Ctrl+C to stop.");
        }

        // Wait for Ctrl+C
        tokio::signal::ctrl_c().await?;

        info!("Shutting down distributed inference system");
        distributed.shutdown().await?;

        // Structured output
        Ok(CommandOutput::success_with_data(
            "Distributed server stopped",
            json!({
                "workers": distributed_config.worker_count,
                "max_concurrent": distributed_config.max_concurrent_per_worker,
                "load_balancing": self.load_balancing,
                "preload_model": self.preload_model,
            }),
        ))
    }
}

// ============================================================================
// DistributedBenchmark - Benchmark distributed performance
// ============================================================================

/// Benchmark distributed inference performance
pub struct DistributedBenchmark {
    config: Config,
    model: String,
    concurrent: usize,
    requests: usize,
    prompt: String,
}

impl DistributedBenchmark {
    pub fn new(
        config: Config,
        model: String,
        concurrent: usize,
        requests: usize,
        prompt: String,
    ) -> Self {
        Self {
            config,
            model,
            concurrent,
            requests,
            prompt,
        }
    }
}

#[async_trait]
impl Command for DistributedBenchmark {
    fn name(&self) -> &str {
        "distributed benchmark"
    }

    fn description(&self) -> &str {
        "Benchmark distributed inference performance"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        if self.model.is_empty() {
            anyhow::bail!("Model name cannot be empty");
        }

        if self.concurrent == 0 {
            anyhow::bail!("Concurrent must be greater than 0");
        }

        if self.concurrent > 100 {
            anyhow::bail!("Concurrent cannot exceed 100");
        }

        if self.requests == 0 {
            anyhow::bail!("Requests must be greater than 0");
        }

        if self.prompt.is_empty() {
            anyhow::bail!("Prompt cannot be empty");
        }

        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Starting distributed inference benchmark");

        let model_manager = Arc::new(ModelManager::new(&self.config.models_dir));
        let metrics = Some(Arc::new({
            let (collector, processor) = MetricsCollector::new();
            processor.start();
            collector
        }));

        let distributed = Arc::new(
            DistributedInference::new(
                self.config.distributed.clone(),
                self.config.backend_config.clone(),
                model_manager,
                metrics,
            )
            .await?,
        );

        // Warm up
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        let total_requests = self.concurrent * self.requests;
        let start_time = Instant::now();

        // Run concurrent requests
        let mut handles = Vec::new();

        for client_id in 0..self.concurrent {
            let distributed_clone = distributed.clone();
            let model = self.model.clone();
            let prompt = self.prompt.clone();
            let requests = self.requests;

            let handle = tokio::spawn(async move {
                let mut successes = 0;
                let mut failures = 0;
                let mut total_tokens = 0;

                for _ in 0..requests {
                    let params = InferenceParams {
                        max_tokens: 50,
                        temperature: 0.7,
                        top_p: 0.9,
                        stream: false,
                        stop_sequences: vec![],
                        seed: None,
                    };

                    match distributed_clone.infer(&model, &prompt, &params).await {
                        Ok(response) => {
                            successes += 1;
                            total_tokens += response.tokens_generated;
                        }
                        Err(_) => {
                            failures += 1;
                        }
                    }
                }

                (successes, failures, total_tokens)
            });

            handles.push(handle);
        }

        // Collect results
        let mut total_successes = 0;
        let mut total_failures = 0;
        let mut total_tokens = 0;

        for handle in handles {
            if let Ok((successes, failures, tokens)) = handle.await {
                total_successes += successes;
                total_failures += failures;
                total_tokens += tokens;
            }
        }

        let total_duration = start_time.elapsed();

        let success_rate = if total_requests > 0 {
            (total_successes as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };

        let throughput = if total_duration.as_secs() > 0 {
            total_successes as f64 / total_duration.as_secs_f64()
        } else {
            0.0
        };

        // Human-readable output
        if !ctx.json_output {
            println!("\n=== Benchmark Results ===");
            println!("Model: {}", self.model);
            println!("Total Duration: {:?}", total_duration);
            println!("Total Requests: {}", total_requests);
            println!("Successful: {}", total_successes);
            println!("Failed: {}", total_failures);
            println!("Success Rate: {:.2}%", success_rate);
            println!("Throughput: {:.2} req/s", throughput);
            println!("Total Tokens: {}", total_tokens);
        }

        // Structured output
        Ok(CommandOutput::success_with_data(
            "Benchmark completed",
            json!({
                "model": self.model,
                "duration_secs": total_duration.as_secs_f64(),
                "total_requests": total_requests,
                "successful": total_successes,
                "failed": total_failures,
                "success_rate": success_rate,
                "throughput": throughput,
                "total_tokens": total_tokens,
            }),
        ))
    }
}

// ============================================================================
// DistributedStats - Show worker statistics
// ============================================================================

/// Show worker statistics
pub struct DistributedStats {
    config: Config,
}

impl DistributedStats {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Command for DistributedStats {
    fn name(&self) -> &str {
        "distributed stats"
    }

    fn description(&self) -> &str {
        "Show worker statistics"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Retrieving worker statistics");

        let model_manager = Arc::new(ModelManager::new(&self.config.models_dir));
        let metrics = Some(Arc::new({
            let (collector, processor) = MetricsCollector::new();
            processor.start();
            collector
        }));

        let distributed = DistributedInference::new(
            self.config.distributed.clone(),
            self.config.backend_config.clone(),
            model_manager,
            metrics,
        )
        .await?;

        let stats = distributed.get_stats().await;

        // Aggregate statistics from all workers
        let worker_count = stats.len();
        let active_workers = stats.values().filter(|s| s.active_requests > 0).count();
        let total_requests: u64 = stats.values().map(|s| s.total_requests).sum();
        let successful_requests: u64 = stats.values().map(|s| s.successful_requests).sum();
        let failed_requests: u64 = stats.values().map(|s| s.failed_requests).sum();

        let avg_response_time = if worker_count > 0 {
            let total_time: u128 = stats
                .values()
                .map(|s| s.average_response_time.as_millis())
                .sum();
            std::time::Duration::from_millis((total_time / worker_count as u128) as u64)
        } else {
            std::time::Duration::from_secs(0)
        };

        // Human-readable output
        if !ctx.json_output {
            println!("=== Distributed Inference Statistics ===");
            println!("Total Workers: {}", worker_count);
            println!("Active Workers: {}", active_workers);
            println!("Total Requests: {}", total_requests);
            println!("Successful Requests: {}", successful_requests);
            println!("Failed Requests: {}", failed_requests);
            println!(
                "Success Rate: {:.2}%",
                if total_requests > 0 {
                    (successful_requests as f64 / total_requests as f64) * 100.0
                } else {
                    0.0
                }
            );
            println!("Average Response Time: {:?}", avg_response_time);
        }

        // Structured output
        Ok(CommandOutput::success_with_data(
            "Worker statistics retrieved",
            json!({
                "worker_count": worker_count,
                "active_workers": active_workers,
                "total_requests": total_requests,
                "successful_requests": successful_requests,
                "failed_requests": failed_requests,
                "avg_response_time_ms": avg_response_time.as_millis(),
            }),
        ))
    }
}

// ============================================================================
// DistributedTest - Test single inference
// ============================================================================

/// Test single inference request
pub struct DistributedTest {
    config: Config,
    model: String,
    input: String,
    stream: bool,
    max_tokens: u32,
    temperature: f32,
}

impl DistributedTest {
    pub fn new(
        config: Config,
        model: String,
        input: String,
        stream: bool,
        max_tokens: u32,
        temperature: f32,
    ) -> Self {
        Self {
            config,
            model,
            input,
            stream,
            max_tokens,
            temperature,
        }
    }
}

#[async_trait]
impl Command for DistributedTest {
    fn name(&self) -> &str {
        "distributed test"
    }

    fn description(&self) -> &str {
        "Test single inference request"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        if self.model.is_empty() {
            anyhow::bail!("Model name cannot be empty");
        }

        if self.input.is_empty() {
            anyhow::bail!("Input cannot be empty");
        }

        if self.max_tokens == 0 {
            anyhow::bail!("Max tokens must be greater than 0");
        }

        if !(0.0..=2.0).contains(&self.temperature) {
            anyhow::bail!("Temperature must be between 0.0 and 2.0");
        }

        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Testing distributed inference");

        let model_manager = Arc::new(ModelManager::new(&self.config.models_dir));
        let metrics = Some(Arc::new({
            let (collector, processor) = MetricsCollector::new();
            processor.start();
            collector
        }));

        let distributed = DistributedInference::new(
            self.config.distributed.clone(),
            self.config.backend_config.clone(),
            model_manager,
            metrics,
        )
        .await?;

        let params = InferenceParams {
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            top_p: 0.9,
            stream: self.stream,
            stop_sequences: vec![],
            seed: None,
        };

        let start_time = Instant::now();
        let response = distributed.infer(&self.model, &self.input, &params).await?;
        let duration = start_time.elapsed();

        // Human-readable output
        if !ctx.json_output {
            println!("=== Inference Test Results ===");
            println!("Model: {}", self.model);
            println!("Input: {}", self.input);
            println!("Response: {}", response.output);
            println!("Tokens Generated: {}", response.tokens_generated);
            println!("Duration: {:?}", duration);
        }

        // Structured output
        Ok(CommandOutput::success_with_data(
            "Inference test completed",
            json!({
                "model": self.model,
                "input": self.input,
                "output": response.output,
                "tokens_generated": response.tokens_generated,
                "duration_ms": duration.as_millis(),
            }),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_distributed_start_validation() {
        let config = Config::default();
        let cmd = DistributedStart::new(config.clone(), 4, None, true, 8);
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_distributed_start_validation_too_many_workers() {
        let config = Config::default();
        let cmd = DistributedStart::new(config.clone(), 100, None, true, 8);
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Worker count cannot exceed 32"));
    }

    #[tokio::test]
    async fn test_distributed_benchmark_validation() {
        let config = Config::default();
        let cmd = DistributedBenchmark::new(
            config.clone(),
            "test-model".to_string(),
            10,
            5,
            "Hello".to_string(),
        );
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_distributed_benchmark_validation_empty_model() {
        let config = Config::default();
        let cmd =
            DistributedBenchmark::new(config.clone(), String::new(), 10, 5, "Hello".to_string());
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Model name cannot be empty"));
    }

    #[tokio::test]
    async fn test_distributed_stats_validation() {
        let config = Config::default();
        let cmd = DistributedStats::new(config.clone());
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_distributed_test_validation() {
        let config = Config::default();
        let cmd = DistributedTest::new(
            config.clone(),
            "test-model".to_string(),
            "Hello".to_string(),
            false,
            100,
            0.7,
        );
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_distributed_test_validation_invalid_temperature() {
        let config = Config::default();
        let cmd = DistributedTest::new(
            config.clone(),
            "test-model".to_string(),
            "Hello".to_string(),
            false,
            100,
            3.0,
        );
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Temperature must be between 0.0 and 2.0"));
    }
}
