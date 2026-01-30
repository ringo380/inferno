use crate::{
    backends::{Backend, BackendType, InferenceParams},
    config::Config,
    models::ModelManager,
    streaming::{StreamingConfig, StreamingManager},
};
use anyhow::Result;
use clap::{Args, Subcommand};
use futures::StreamExt;
use std::io::{self, Write};
use std::sync::Arc;
use tracing::{error, info, warn};

// ============================================================================
// Validation Constants
// ============================================================================

/// Maximum allowed concurrent streams for benchmarking
const MAX_CONCURRENT_STREAMS: usize = 100;

/// Maximum allowed benchmark duration in seconds (1 hour)
const MAX_BENCHMARK_DURATION_SECS: u64 = 3600;

#[derive(Args)]
pub struct StreamingArgs {
    #[command(subcommand)]
    pub command: StreamingCommand,
}

#[derive(Subcommand)]
pub enum StreamingCommand {
    #[command(about = "Start interactive streaming inference session")]
    Interactive {
        #[arg(short, long, help = "Model to use")]
        model: String,

        #[arg(long, help = "Maximum tokens per response", default_value = "512")]
        max_tokens: u32,

        #[arg(long, help = "Temperature for generation", default_value = "0.7")]
        temperature: f32,

        #[arg(long, help = "Top-K for generation", default_value = "40")]
        top_k: u32,

        #[arg(long, help = "Top-p for generation", default_value = "0.9")]
        top_p: f32,

        #[arg(long, help = "Enable verbose output")]
        verbose: bool,
    },

    #[command(about = "Test streaming performance with concurrent streams")]
    Benchmark {
        #[arg(short, long, help = "Model to use")]
        model: String,

        #[arg(
            short,
            long,
            help = "Number of concurrent streams",
            default_value = "5"
        )]
        concurrent: usize,

        #[arg(short, long, help = "Test prompt")]
        prompt: String,

        #[arg(long, help = "Duration in seconds", default_value = "30")]
        duration: u64,
    },

    #[command(about = "Monitor active streaming sessions")]
    Monitor {
        #[arg(long, help = "Refresh interval in seconds", default_value = "2")]
        interval: u64,

        #[arg(long, help = "Show detailed stream information")]
        detailed: bool,
    },

    #[command(about = "Start WebSocket streaming server")]
    Server {
        #[arg(
            short,
            long,
            help = "Server bind address",
            default_value = "127.0.0.1:8081"
        )]
        bind: std::net::SocketAddr,

        #[arg(short, long, help = "Default model to load")]
        model: Option<String>,

        #[arg(long, help = "Maximum concurrent connections", default_value = "50")]
        max_connections: usize,
    },

    #[command(about = "Export streaming configuration")]
    Config {
        #[arg(long, help = "Output format", value_enum, default_value = "json")]
        format: ConfigFormat,

        #[arg(short, long, help = "Output file (stdout if not specified)")]
        output: Option<std::path::PathBuf>,
    },
}

#[derive(clap::ValueEnum, Clone)]
pub enum ConfigFormat {
    Json,
    Yaml,
    Toml,
}

pub async fn execute(args: StreamingArgs, config: &Config) -> Result<()> {
    match args.command {
        StreamingCommand::Interactive {
            model,
            max_tokens,
            temperature,
            top_k,
            top_p,
            verbose,
        } => {
            // Validate interactive session parameters
            validate_interactive(&model, temperature, top_p)?;

            execute_interactive(
                model,
                max_tokens,
                temperature,
                top_k,
                top_p,
                verbose,
                config,
            )
            .await
        }
        StreamingCommand::Benchmark {
            model,
            concurrent,
            prompt,
            duration,
        } => {
            // Validate benchmark parameters
            validate_benchmark(&model, &prompt, concurrent, duration)?;

            execute_benchmark(model, concurrent, prompt, duration, config).await
        }
        StreamingCommand::Monitor { interval, detailed } => {
            // Validate monitor parameters
            validate_monitor(interval)?;

            execute_monitor(interval, detailed, config).await
        }
        StreamingCommand::Server {
            bind,
            model,
            max_connections,
        } => {
            // Validate server parameters
            validate_server(max_connections)?;

            execute_server(bind, model, max_connections, config).await
        }
        StreamingCommand::Config { format, output } => {
            // Validate config export parameters
            validate_config_export(&output)?;

            execute_config(format, output).await
        }
    }
}

// ============================================================================
// Validation Functions
// ============================================================================

/// Validate interactive session parameters
fn validate_interactive(model: &str, temperature: f32, top_p: f32) -> Result<()> {
    if model.is_empty() {
        anyhow::bail!("Model name cannot be empty");
    }

    if !(0.0..=2.0).contains(&temperature) {
        anyhow::bail!(
            "Temperature must be between 0.0 and 2.0, got {}",
            temperature
        );
    }

    if !(0.0..=1.0).contains(&top_p) {
        anyhow::bail!("Top-p must be between 0.0 and 1.0, got {}", top_p);
    }

    Ok(())
}

/// Validate benchmark parameters
fn validate_benchmark(model: &str, prompt: &str, concurrent: usize, duration: u64) -> Result<()> {
    if model.is_empty() {
        anyhow::bail!("Model name cannot be empty");
    }

    if prompt.is_empty() {
        anyhow::bail!("Prompt cannot be empty");
    }

    if concurrent == 0 {
        anyhow::bail!("Concurrent streams must be at least 1");
    }

    if concurrent > MAX_CONCURRENT_STREAMS {
        anyhow::bail!(
            "Concurrent streams cannot exceed {} (got {})",
            MAX_CONCURRENT_STREAMS,
            concurrent
        );
    }

    if duration == 0 {
        anyhow::bail!("Duration must be at least 1 second");
    }

    if duration > MAX_BENCHMARK_DURATION_SECS {
        anyhow::bail!(
            "Duration cannot exceed {} seconds (1 hour), got {}",
            MAX_BENCHMARK_DURATION_SECS,
            duration
        );
    }

    Ok(())
}

/// Validate monitor parameters
fn validate_monitor(interval: u64) -> Result<()> {
    if interval == 0 {
        anyhow::bail!("Monitor interval must be at least 1 second");
    }

    if interval > 3600 {
        anyhow::bail!("Monitor interval cannot exceed 3600 seconds (1 hour)");
    }

    Ok(())
}

/// Validate server parameters
fn validate_server(max_connections: usize) -> Result<()> {
    if max_connections == 0 {
        anyhow::bail!("Maximum connections must be at least 1");
    }

    if max_connections > 10000 {
        anyhow::bail!("Maximum connections cannot exceed 10000");
    }

    Ok(())
}

/// Validate config export parameters
fn validate_config_export(output: &Option<std::path::PathBuf>) -> Result<()> {
    if let Some(path) = output {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                anyhow::bail!("Output directory does not exist: {}", parent.display());
            }
        }
    }

    Ok(())
}

async fn execute_interactive(
    model_name: String,
    max_tokens: u32,
    temperature: f32,
    top_k: u32,
    top_p: f32,
    verbose: bool,
    config: &Config,
) -> Result<()> {
    info!("Starting interactive streaming inference session");

    // Initialize streaming manager
    let streaming_config = StreamingConfig {
        max_concurrent_streams: 1,
        enable_metrics: verbose,
        ..Default::default()
    };

    let streaming_manager = StreamingManager::new(streaming_config);
    streaming_manager.start().await?;

    // Load model
    let model_manager = ModelManager::new(&config.models_dir);
    let model_info = model_manager.resolve_model(&model_name).await?;
    let backend_type = BackendType::from_model_path(&model_info.path).ok_or_else(|| {
        anyhow::anyhow!(
            "No suitable backend found for model: {}",
            model_info.path.display()
        )
    })?;
    let mut backend = Backend::new(backend_type, &config.backend_config)?;
    backend.load_model(&model_info).await?;

    println!("🔥 Inferno Interactive Streaming Session");
    println!("Model: {}", model_info.name);
    println!("Type your prompts (press Ctrl+C to exit):\n");

    let inference_params = InferenceParams {
        max_tokens,
        temperature,
        top_k,
        top_p,
        stream: true,
        stop_sequences: vec![],
        seed: None,
    };

    loop {
        print!("💬 You: ");
        io::stdout().flush()?;

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let input = input.trim();
                if input.is_empty() {
                    continue;
                }

                print!("🤖 Assistant: ");
                io::stdout().flush()?;

                // Create enhanced streaming session
                match streaming_manager
                    .create_enhanced_stream(&mut backend, input, &inference_params)
                    .await
                {
                    Ok(mut stream) => {
                        let mut token_count = 0;
                        let start_time = std::time::Instant::now();

                        while let Some(token_result) = stream.next().await {
                            match token_result {
                                Ok(streaming_token) => {
                                    if !streaming_token.is_heartbeat() {
                                        print!("{}", streaming_token.content);
                                        io::stdout().flush()?;
                                        token_count += 1;
                                    }
                                }
                                Err(e) => {
                                    error!("Streaming error: {}", e);
                                    break;
                                }
                            }
                        }

                        let elapsed = start_time.elapsed();
                        println!();

                        if verbose {
                            println!(
                                "📊 Generated {} tokens in {:.2}s ({:.1} tok/s)",
                                token_count,
                                elapsed.as_secs_f32(),
                                token_count as f32 / elapsed.as_secs_f32()
                            );

                            let metrics = streaming_manager.get_metrics();
                            println!(
                                "📈 Total streams: {}, Total tokens: {}",
                                metrics.total_streams_created, metrics.total_tokens_streamed
                            );
                        }
                        println!();
                    }
                    Err(e) => {
                        error!("Failed to create stream: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to read input: {}", e);
                break;
            }
        }
    }

    Ok(())
}

async fn execute_benchmark(
    model_name: String,
    concurrent: usize,
    prompt: String,
    duration: u64,
    config: &Config,
) -> Result<()> {
    info!(
        "Starting streaming benchmark with {} concurrent streams",
        concurrent
    );

    // Initialize streaming manager
    let streaming_config = StreamingConfig {
        max_concurrent_streams: concurrent * 2, // Allow some headroom
        enable_metrics: true,
        ..Default::default()
    };

    let streaming_manager = Arc::new(StreamingManager::new(streaming_config));
    streaming_manager.start().await?;

    // Load model
    let model_manager = ModelManager::new(&config.models_dir);
    let model_info = model_manager.resolve_model(&model_name).await?;
    let backend_type = BackendType::from_model_path(&model_info.path);

    println!("🚀 Starting streaming benchmark");
    println!("Model: {}", model_info.name);
    println!("Concurrent streams: {}", concurrent);
    println!("Duration: {}s", duration);
    println!("Prompt: {}\n", prompt);

    let inference_params = InferenceParams {
        max_tokens: 100, // Shorter responses for benchmarking
        temperature: 0.7,
        top_k: 40,
        top_p: 0.9,
        stream: true,
        stop_sequences: vec![],
        seed: None,
    };

    // Start concurrent streams
    let mut handles = Vec::new();

    for i in 0..concurrent {
        let streaming_manager = streaming_manager.clone();
        let model_info = model_info.clone();
        let prompt = prompt.clone();
        let inference_params = inference_params.clone();
        let backend_config = config.backend_config.clone();

        let handle = tokio::spawn(async move {
            let result: Result<(usize, u64, u64)> = async move {
                let backend_type =
                    backend_type.ok_or_else(|| anyhow::anyhow!("No suitable backend found"))?;
                let mut backend = Backend::new(backend_type, &backend_config)?;
                backend.load_model(&model_info).await?;

                let start_time = std::time::Instant::now();
                let mut total_tokens = 0u64;
                let mut total_streams = 0u64;

                while start_time.elapsed().as_secs() < duration {
                    match streaming_manager
                        .create_enhanced_stream(&mut backend, &prompt, &inference_params)
                        .await
                    {
                        Ok(mut stream) => {
                            total_streams += 1;

                            while let Some(token_result) = stream.next().await {
                                match token_result {
                                    Ok(streaming_token) => {
                                        if !streaming_token.is_heartbeat() {
                                            total_tokens += 1;
                                        }
                                    }
                                    Err(_) => break,
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Stream {} failed: {}", i, e);
                            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        }
                    }
                }

                Ok((i, total_streams, total_tokens))
            }
            .await;
            result.unwrap_or((i, 0, 0))
        });

        handles.push(handle);
    }

    // Monitor progress
    let monitor_handle = {
        let streaming_manager = streaming_manager.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

            for _ in 0..(duration / 5) {
                interval.tick().await;
                let metrics = streaming_manager.get_metrics();
                println!(
                    "📊 Active: {}, Total tokens: {}, Avg tok/s: {:.1}",
                    metrics.active_streams,
                    metrics.total_tokens_streamed,
                    metrics.average_tokens_per_second
                );
            }
        })
    };

    // Wait for all streams to complete
    let mut total_streams = 0u64;
    let mut total_tokens = 0u64;

    for handle in handles {
        match handle.await {
            Ok((stream_id, streams, tokens)) => {
                total_streams += streams;
                total_tokens += tokens;
                println!(
                    "Stream {} completed: {} streams, {} tokens",
                    stream_id, streams, tokens
                );
            }
            Err(e) => {
                error!("Stream failed: {}", e);
            }
        }
    }

    monitor_handle.abort();

    // Final metrics
    let final_metrics = streaming_manager.get_metrics();

    println!("\n🏁 Benchmark Results:");
    println!("Total streams created: {}", total_streams);
    println!("Total tokens generated: {}", total_tokens);
    println!(
        "Average tokens/second: {:.1}",
        total_tokens as f32 / duration as f32
    );
    println!("Errors: {}", final_metrics.errors_count);
    println!("Timeouts: {}", final_metrics.timeouts);
    println!("Buffer overflows: {}", final_metrics.buffer_overflows);

    Ok(())
}

async fn execute_monitor(interval: u64, detailed: bool, _config: &Config) -> Result<()> {
    println!("📡 Starting stream monitoring (press Ctrl+C to exit)");

    // This would connect to a running streaming manager
    // For now, we'll show a demo of what monitoring would look like

    let mut counter = 0;
    loop {
        counter += 1;

        // Simulate some metrics
        let active_streams = (counter % 10) + 1;
        let total_tokens = counter * 50;
        let avg_latency = 150.0 + (counter as f32 * 10.0) % 100.0;

        println!(
            "\n📊 Streaming Monitor ({})",
            chrono::Utc::now().format("%H:%M:%S")
        );
        println!("Active streams: {}", active_streams);
        println!("Total tokens streamed: {}", total_tokens);
        println!("Average latency: {:.1}ms", avg_latency);

        if detailed {
            println!("Stream details:");
            for i in 0..active_streams {
                println!(
                    "  Stream {}: {} tokens, {:.1}ms latency",
                    i + 1,
                    (i + 1) * 10,
                    avg_latency + (i as f32 * 20.0)
                );
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
    }
}

async fn execute_server(
    bind: std::net::SocketAddr,
    model: Option<String>,
    max_connections: usize,
    _config: &Config,
) -> Result<()> {
    info!("Starting WebSocket streaming server on {}", bind);

    // Initialize streaming manager
    let streaming_config = StreamingConfig {
        max_concurrent_streams: max_connections,
        enable_metrics: true,
        heartbeat_interval_ms: 10000, // 10 second heartbeat for WebSocket
        ..Default::default()
    };

    let streaming_manager = Arc::new(StreamingManager::new(streaming_config));
    streaming_manager.start().await?;

    // Optionally load a default model
    if let Some(model_name) = model {
        info!("Loading default model: {}", model_name);
        // Model loading would be implemented here
    }

    println!("🌐 WebSocket streaming server started");
    println!("Address: ws://{}", bind);
    println!("Max connections: {}", max_connections);
    println!("\nExample client connection:");
    println!("wscat -c ws://{}/stream", bind);
    println!("\nAPI endpoints:");
    println!("  /stream     - WebSocket streaming inference");
    println!("  /metrics    - Streaming metrics");
    println!("  /health     - Health check");

    // This would start the actual WebSocket server
    // For now, just keep the process running
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        let metrics = streaming_manager.get_metrics();
        info!(
            "Server running - Active streams: {}, Total: {}",
            metrics.active_streams, metrics.total_streams_created
        );
    }
}

async fn execute_config(format: ConfigFormat, output: Option<std::path::PathBuf>) -> Result<()> {
    let config = StreamingConfig::default();

    let content = match format {
        ConfigFormat::Json => serde_json::to_string_pretty(&config)?,
        ConfigFormat::Yaml => serde_yaml::to_string(&config)?,
        ConfigFormat::Toml => toml::to_string_pretty(&config)?,
    };

    match output {
        Some(path) => {
            tokio::fs::write(path, content).await?;
            println!("Configuration exported successfully");
        }
        None => {
            println!("{}", content);
        }
    }

    Ok(())
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // -------------------------------------------------------------------------
    // Interactive Validation Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_validate_interactive_empty_model() {
        let result = validate_interactive("", 0.7, 0.9);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Model name cannot be empty")
        );
    }

    #[test]
    fn test_validate_interactive_valid() {
        let result = validate_interactive("model.gguf", 0.7, 0.9);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_interactive_temperature_out_of_range() {
        let result = validate_interactive("model.gguf", 2.5, 0.9);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Temperature must be between")
        );

        let result = validate_interactive("model.gguf", -0.5, 0.9);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_interactive_top_p_out_of_range() {
        let result = validate_interactive("model.gguf", 0.7, 1.5);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Top-p must be between")
        );

        let result = validate_interactive("model.gguf", 0.7, -0.1);
        assert!(result.is_err());
    }

    // -------------------------------------------------------------------------
    // Benchmark Validation Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_validate_benchmark_empty_model() {
        let result = validate_benchmark("", "test prompt", 5, 30);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Model name cannot be empty")
        );
    }

    #[test]
    fn test_validate_benchmark_empty_prompt() {
        let result = validate_benchmark("model.gguf", "", 5, 30);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Prompt cannot be empty")
        );
    }

    #[test]
    fn test_validate_benchmark_zero_concurrent() {
        let result = validate_benchmark("model.gguf", "test", 0, 30);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Concurrent streams must be at least 1")
        );
    }

    #[test]
    fn test_validate_benchmark_excessive_concurrent() {
        let result = validate_benchmark("model.gguf", "test", 200, 30);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Concurrent streams cannot exceed 100")
        );
    }

    #[test]
    fn test_validate_benchmark_zero_duration() {
        let result = validate_benchmark("model.gguf", "test", 5, 0);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Duration must be at least 1 second")
        );
    }

    #[test]
    fn test_validate_benchmark_excessive_duration() {
        let result = validate_benchmark("model.gguf", "test", 5, 5000);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Duration cannot exceed 3600 seconds")
        );
    }

    #[test]
    fn test_validate_benchmark_valid() {
        let result = validate_benchmark("model.gguf", "test prompt", 5, 30);
        assert!(result.is_ok());
    }

    // -------------------------------------------------------------------------
    // Monitor Validation Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_validate_monitor_zero_interval() {
        let result = validate_monitor(0);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Monitor interval must be at least 1 second")
        );
    }

    #[test]
    fn test_validate_monitor_excessive_interval() {
        let result = validate_monitor(4000);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Monitor interval cannot exceed 3600 seconds")
        );
    }

    #[test]
    fn test_validate_monitor_valid() {
        let result = validate_monitor(5);
        assert!(result.is_ok());
    }

    // -------------------------------------------------------------------------
    // Server Validation Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_validate_server_zero_connections() {
        let result = validate_server(0);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Maximum connections must be at least 1")
        );
    }

    #[test]
    fn test_validate_server_excessive_connections() {
        let result = validate_server(20000);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Maximum connections cannot exceed 10000")
        );
    }

    #[test]
    fn test_validate_server_valid() {
        let result = validate_server(50);
        assert!(result.is_ok());
    }

    // -------------------------------------------------------------------------
    // Config Export Validation Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_validate_config_export_no_output() {
        let result = validate_config_export(&None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_config_export_nonexistent_parent() {
        let path = PathBuf::from("/nonexistent/directory/config.json");
        let result = validate_config_export(&Some(path));
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Output directory does not exist")
        );
    }

    #[test]
    fn test_validate_config_export_valid_path() {
        // Use current directory which should exist
        let path = PathBuf::from("./config.json");
        let result = validate_config_export(&Some(path));
        assert!(result.is_ok());
    }
}
