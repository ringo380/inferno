use crate::{
    backends::InferenceParams,
    config::Config,
    distributed::DistributedInference,
    metrics::MetricsCollector,
    models::ModelManager,
};
use anyhow::Result;
use clap::{Args, Subcommand};
use futures::StreamExt;
use std::{sync::Arc, time::Instant};
use tracing::{info, warn};

#[derive(Args)]
pub struct DistributedArgs {
    #[command(subcommand)]
    pub command: DistributedCommand,
}

#[derive(Subcommand)]
pub enum DistributedCommand {
    #[command(about = "Start distributed inference server")]
    Start {
        #[arg(short, long, help = "Number of worker processes", default_value = "0")]
        workers: usize,

        #[arg(short, long, help = "Model to preload")]
        preload_model: Option<String>,

        #[arg(long, help = "Enable load balancing", default_value = "true")]
        load_balancing: bool,

        #[arg(long, help = "Maximum concurrent requests per worker", default_value = "4")]
        max_concurrent: usize,
    },

    #[command(about = "Test distributed inference performance")]
    Benchmark {
        #[arg(short, long, help = "Model name to benchmark")]
        model: String,

        #[arg(short, long, help = "Number of concurrent requests", default_value = "10")]
        concurrent: usize,

        #[arg(short, long, help = "Number of requests per client", default_value = "5")]
        requests: usize,

        #[arg(short, long, help = "Test prompt", default_value = "Hello, world!")]
        prompt: String,
    },

    #[command(about = "Show worker statistics")]
    Stats,

    #[command(about = "Test single inference request")]
    Test {
        #[arg(short, long, help = "Model name")]
        model: String,

        #[arg(short, long, help = "Input text", default_value = "Hello, world!")]
        input: String,

        #[arg(long, help = "Enable streaming output")]
        stream: bool,

        #[arg(long, help = "Maximum tokens", default_value = "100")]
        max_tokens: u32,

        #[arg(long, help = "Temperature", default_value = "0.7")]
        temperature: f32,
    },
}

pub async fn execute(args: DistributedArgs, config: &Config) -> Result<()> {
    match args.command {
        DistributedCommand::Start {
            workers,
            preload_model,
            load_balancing,
            max_concurrent,
        } => {
            start_distributed_server(
                config,
                workers,
                preload_model,
                load_balancing,
                max_concurrent,
            ).await
        }
        DistributedCommand::Benchmark {
            model,
            concurrent,
            requests,
            prompt,
        } => {
            benchmark_distributed_inference(config, &model, concurrent, requests, &prompt).await
        }
        DistributedCommand::Stats => {
            show_worker_stats(config).await
        }
        DistributedCommand::Test {
            model,
            input,
            stream,
            max_tokens,
            temperature,
        } => {
            test_inference(config, &model, &input, stream, max_tokens, temperature).await
        }
    }
}

async fn start_distributed_server(
    config: &Config,
    workers: usize,
    preload_model: Option<String>,
    load_balancing: bool,
    max_concurrent: usize,
) -> Result<()> {
    info!("Starting distributed inference server");

    let model_manager = Arc::new(ModelManager::new(&config.models_dir));
    let metrics = Some(Arc::new({
        let mut collector = MetricsCollector::new();
        collector.start_event_processing().await?;
        collector
    }));

    // Override config with command-line arguments
    let mut distributed_config = config.distributed.clone();
    if workers > 0 {
        distributed_config.worker_count = workers;
    }
    distributed_config.load_balancing = load_balancing;
    distributed_config.max_concurrent_per_worker = max_concurrent;
    if preload_model.is_some() {
        distributed_config.preload_models = true;
    }

    info!("Initializing {} workers with max {} concurrent requests each",
          distributed_config.worker_count, distributed_config.max_concurrent_per_worker);

    let mut distributed = DistributedInference::new(
        distributed_config,
        config.backend_config.clone(),
        model_manager,
        metrics,
    ).await?;

    info!("Distributed inference system started successfully");

    // If a specific model was requested for preloading, wait a moment for it to load
    if let Some(model_name) = preload_model {
        info!("Preloading model: {}", model_name);
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    // Keep the server running
    info!("Server is running. Press Ctrl+C to stop.");
    tokio::signal::ctrl_c().await?;

    info!("Shutting down distributed inference system");
    distributed.shutdown().await?;

    Ok(())
}

async fn benchmark_distributed_inference(
    config: &Config,
    model_name: &str,
    concurrent: usize,
    requests_per_client: usize,
    prompt: &str,
) -> Result<()> {
    info!("Starting distributed inference benchmark");
    info!("Model: {}", model_name);
    info!("Concurrent clients: {}", concurrent);
    info!("Requests per client: {}", requests_per_client);
    info!("Prompt: \"{}\"", prompt);

    let model_manager = Arc::new(ModelManager::new(&config.models_dir));
    let metrics = Some(Arc::new({
        let mut collector = MetricsCollector::new();
        collector.start_event_processing().await?;
        collector
    }));

    let distributed = Arc::new(DistributedInference::new(
        config.distributed.clone(),
        config.backend_config.clone(),
        model_manager,
        metrics,
    ).await?);

    info!("Warming up workers...");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    let total_requests = concurrent * requests_per_client;
    let start_time = Instant::now();

    info!("Starting benchmark with {} total requests", total_requests);

    let mut handles = Vec::new();

    for client_id in 0..concurrent {
        let distributed_clone = distributed.clone();
        let model_name = model_name.to_string();
        let prompt = prompt.to_string();

        let handle = tokio::spawn(async move {
            let mut client_stats = ClientStats::new(client_id);

            for request_id in 0..requests_per_client {
                let request_start = Instant::now();

                let params = InferenceParams {
                    max_tokens: 50,
                    temperature: 0.7,
                    top_p: 0.9,
                    stream: false,
                };

                match distributed_clone.infer(&model_name, &prompt, &params).await {
                    Ok(response) => {
                        let duration = request_start.elapsed();
                        client_stats.record_success(duration, response.tokens_generated);

                        if request_id % 10 == 0 {
                            println!("Client {}: Request {}/{} completed in {:?}",
                                   client_id, request_id + 1, requests_per_client, duration);
                        }
                    }
                    Err(e) => {
                        client_stats.record_failure();
                        warn!("Request failed for client {}: {}", client_id, e);
                    }
                }
            }

            client_stats
        });

        handles.push(handle);
    }

    // Wait for all clients to complete
    let mut all_stats = Vec::new();
    for handle in handles {
        match handle.await {
            Ok(stats) => all_stats.push(stats),
            Err(e) => warn!("Client task failed: {}", e),
        }
    }

    let total_duration = start_time.elapsed();

    // Aggregate statistics
    let total_successful = all_stats.iter().map(|s| s.successful_requests).sum::<u64>();
    let total_failed = all_stats.iter().map(|s| s.failed_requests).sum::<u64>();
    let total_tokens = all_stats.iter().map(|s| s.total_tokens).sum::<u32>();

    let avg_response_time = if total_successful > 0 {
        all_stats.iter()
            .map(|s| s.total_response_time.as_millis() as u64)
            .sum::<u64>() / total_successful
    } else {
        0
    };

    println!("\n=== Benchmark Results ===");
    println!("Total Duration: {:?}", total_duration);
    println!("Total Requests: {}", total_requests);
    println!("Successful Requests: {}", total_successful);
    println!("Failed Requests: {}", total_failed);
    println!("Success Rate: {:.2}%", (total_successful as f64 / total_requests as f64) * 100.0);
    println!("Average Response Time: {}ms", avg_response_time);
    println!("Requests per Second: {:.2}", total_successful as f64 / total_duration.as_secs_f64());
    println!("Total Tokens Generated: {}", total_tokens);
    println!("Tokens per Second: {:.2}", total_tokens as f64 / total_duration.as_secs_f64());

    // Get worker statistics
    let worker_stats = distributed.get_detailed_stats().await?;
    println!("\n=== Worker Statistics ===");
    for (worker_id, stats) in worker_stats {
        println!("Worker {}: {} requests, {} successful, {} failed, avg: {:?}",
                 worker_id, stats.total_requests, stats.successful_requests,
                 stats.failed_requests, stats.average_response_time);
    }

    Ok(())
}

async fn show_worker_stats(config: &Config) -> Result<()> {
    info!("Showing worker statistics (this would connect to a running distributed server)");

    // In a real implementation, this would connect to a running distributed server
    // For now, just show the configuration
    println!("Distributed Configuration:");
    println!("{}", serde_json::to_string_pretty(&config.distributed)?);

    Ok(())
}

async fn test_inference(
    config: &Config,
    model_name: &str,
    input: &str,
    stream: bool,
    max_tokens: u32,
    temperature: f32,
) -> Result<()> {
    info!("Testing distributed inference");
    info!("Model: {}", model_name);
    info!("Input: \"{}\"", input);
    info!("Streaming: {}", stream);

    let model_manager = Arc::new(ModelManager::new(&config.models_dir));
    let metrics = Some(Arc::new({
        let mut collector = MetricsCollector::new();
        collector.start_event_processing().await?;
        collector
    }));

    let distributed = DistributedInference::new(
        config.distributed.clone(),
        config.backend_config.clone(),
        model_manager,
        metrics,
    ).await?;

    let params = InferenceParams {
        max_tokens,
        temperature,
        top_p: 0.9,
        stream,
    };

    let start_time = Instant::now();

    if stream {
        info!("Starting streaming inference...");
        let mut stream = distributed.infer_stream(model_name, input, &params).await?;

        print!("Response: ");
        while let Some(token_result) = stream.next().await {
            match token_result {
                Ok(token) => print!("{}", token),
                Err(e) => {
                    eprintln!("\nStreaming error: {}", e);
                    break;
                }
            }
        }
        println!();
    } else {
        info!("Starting non-streaming inference...");
        match distributed.infer(model_name, input, &params).await {
            Ok(response) => {
                println!("Response: {}", response.output);
                println!("Tokens generated: {}", response.tokens_generated);
                println!("Worker ID: {}", response.worker_id);
                println!("Duration: {:?}", response.duration);
            }
            Err(e) => {
                eprintln!("Inference failed: {}", e);
                return Err(e);
            }
        }
    }

    let total_time = start_time.elapsed();
    println!("Total time: {:?}", total_time);

    Ok(())
}

#[derive(Debug, Clone)]
struct ClientStats {
    client_id: usize,
    successful_requests: u64,
    failed_requests: u64,
    total_response_time: std::time::Duration,
    total_tokens: u32,
}

impl ClientStats {
    fn new(client_id: usize) -> Self {
        Self {
            client_id,
            successful_requests: 0,
            failed_requests: 0,
            total_response_time: std::time::Duration::ZERO,
            total_tokens: 0,
        }
    }

    fn record_success(&mut self, duration: std::time::Duration, tokens: u32) {
        self.successful_requests += 1;
        self.total_response_time += duration;
        self.total_tokens += tokens;
    }

    fn record_failure(&mut self) {
        self.failed_requests += 1;
    }
}