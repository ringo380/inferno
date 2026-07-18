pub mod ab_testing;
pub mod audit;
pub mod batch;
pub mod batch_queue;
pub mod bench;
pub mod cache;
pub mod config;
pub mod convert;
pub mod deployment;
pub mod distributed;
pub mod enhanced_parser;
pub mod fuzzy;
pub mod gpu;
pub mod help;
pub mod metrics;
pub mod model_versioning;
pub mod models;
pub mod monitoring;
pub mod observability;
pub mod optimization;
pub mod performance_benchmark;
pub mod resilience;
pub mod response_cache;
pub mod run;
pub mod security;
pub mod serve;
pub mod streaming;
pub mod upgrade;
pub mod validate;
pub mod versioning;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "inferno",
    about = "An offline AI/ML model runner for GGUF and ONNX models",
    version,
    author
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Run inference on text, image, or audio input")]
    Run(run::RunArgs),

    #[command(about = "Process multiple inputs in batch mode")]
    Batch(batch::BatchArgs),

    #[command(about = "Start local HTTP API server")]
    Serve(serve::ServeArgs),

    #[command(about = "Manage and list available models")]
    Models(models::ModelsArgs),

    #[command(about = "Metrics collection and export")]
    Metrics(metrics::MetricsArgs),

    #[command(about = "Benchmark model performance")]
    Bench(bench::BenchArgs),

    #[command(about = "Validate model files and configurations")]
    Validate(validate::ValidateArgs),

    #[command(about = "Manage configuration settings")]
    Config(config::ConfigArgs),

    #[command(about = "Model caching and warm-up management")]
    Cache(cache::CacheArgs),

    #[command(about = "Convert and optimize models between formats")]
    Convert(convert::ConvertArgs),

    #[command(about = "Response caching and deduplication management")]
    ResponseCache(response_cache::ResponseCacheArgs),

    #[command(about = "Real-time performance monitoring and alerting")]
    Monitor(monitoring::MonitoringArgs),

    #[command(about = "Distributed inference with worker pools")]
    Distributed(distributed::DistributedArgs),

    #[command(about = "A/B testing and canary deployment management")]
    ABTest(ab_testing::ABTestingArgs),

    #[command(about = "Comprehensive audit logging and compliance tracking")]
    Audit(audit::AuditArgs),

    #[command(about = "Advanced batch processing with job queues and scheduling")]
    Queue(batch_queue::BatchQueueArgs),

    #[command(about = "Model versioning and rollback management")]
    Version(versioning::VersioningArgs),

    #[command(about = "GPU acceleration support and management")]
    Gpu(gpu::GpuArgs),

    #[command(about = "Production resilience patterns and error recovery")]
    Resilience(resilience::ResilienceArgs),

    #[command(about = "Real-time streaming inference and monitoring")]
    Streaming(streaming::StreamingArgs),

    #[command(about = "Security and access control management")]
    Security(security::SecurityArgs),

    #[command(about = "Observability stack for metrics, tracing, and dashboards")]
    Observability(observability::ObservabilityArgs),

    #[command(about = "Model optimization with quantization, pruning, and distillation")]
    Optimization(optimization::OptimizationArgs),

    #[command(about = "Generate Kubernetes manifests and Helm charts")]
    Deployment(deployment::DeploymentCliArgs),

    #[command(about = "Model versioning and A/B testing framework")]
    ModelVersioning(model_versioning::ModelVersioningArgs),

    #[command(about = "Performance benchmarking and baseline establishment")]
    PerformanceBenchmark(performance_benchmark::PerformanceBenchmarkArgs),

    #[command(about = "Application upgrade and update management")]
    Upgrade(upgrade::UpgradeArgs),

    #[command(about = "Launch terminal user interface")]
    Tui,
}
