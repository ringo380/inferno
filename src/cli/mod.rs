pub mod ab_testing;
pub mod ab_testing_v2; // New architecture implementation
pub mod advanced_cache;
pub mod advanced_cache_v2; // New architecture implementation
pub mod advanced_monitoring;
pub mod advanced_monitoring_v2; // New architecture implementation
pub mod api_gateway;
pub mod api_gateway_v2; // New architecture implementation
pub mod audit;
pub mod audit_v2; // New architecture implementation
pub mod backup_recovery;
pub mod backup_recovery_v2; // New architecture implementation
pub mod batch;
pub mod batch_queue;
pub mod batch_queue_v2; // New architecture implementation
pub mod batch_v2; // New architecture implementation
pub mod bench;
pub mod bench_v2; // New architecture implementation
pub mod cache;
pub mod cache_v2; // New architecture implementation
pub mod config;
pub mod config_v2; // New architecture implementation
pub mod convert;
pub mod convert_v2; // New architecture implementation
pub mod dashboard;
pub mod dashboard_v2; // New architecture implementation
pub mod data_pipeline;
pub mod data_pipeline_v2; // New architecture implementation
pub mod deployment;
pub mod deployment_v2; // New architecture implementation
pub mod distributed;
pub mod distributed_v2; // New architecture implementation
pub mod enhanced_parser;
pub mod enhanced_parser_v2; // New architecture implementation
pub mod federated;
pub mod federated_v2; // New architecture implementation
pub mod fuzzy;
pub mod fuzzy_v2; // New architecture implementation
pub mod gpu;
pub mod gpu_v2; // New architecture implementation
pub mod help;
pub mod help_v2; // New architecture implementation
pub mod logging_audit;
pub mod logging_audit_v2; // New architecture implementation
pub mod marketplace;
pub mod marketplace_v2; // New architecture implementation
pub mod metrics;
pub mod metrics_v2; // New architecture implementation
pub mod model_versioning;
pub mod model_versioning_v2; // New architecture implementation
pub mod models;
pub mod models_v2; // New architecture implementation
pub mod monitoring;
pub mod monitoring_v2; // New architecture implementation
pub mod multi_tenancy;
pub mod multi_tenancy_v2; // New architecture implementation
pub mod multimodal;
pub mod multimodal_v2; // New architecture implementation
pub mod observability;
pub mod observability_v2; // New architecture implementation
pub mod optimization;
pub mod optimization_v2; // New architecture implementation
pub mod package;
pub mod package_v2; // New architecture implementation
pub mod performance_benchmark;
pub mod performance_benchmark_v2; // New architecture implementation
pub mod performance_optimization;
pub mod performance_optimization_v2; // New architecture implementation
pub mod qa_framework;
pub mod qa_framework_v2; // New architecture implementation
pub mod repo;
pub mod repo_v2; // New architecture implementation
pub mod resilience;
pub mod resilience_v2; // New architecture implementation
pub mod response_cache;
pub mod response_cache_v2; // New architecture implementation
pub mod run;
pub mod run_v2; // New architecture implementation
pub mod security;
pub mod security_v2; // New architecture implementation
pub mod serve;
pub mod serve_v2; // New architecture implementation
pub mod streaming;
pub mod streaming_v2; // New architecture implementation
pub mod upgrade;
pub mod upgrade_v2; // New architecture implementation
pub mod validate;
pub mod validate_v2; // New architecture implementation
pub mod versioning;
pub mod versioning_v2; // New architecture implementation

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

    #[command(about = "Multi-modal inference with vision, audio, and mixed media")]
    MultiModal(multimodal::MultiModalArgs),

    #[command(about = "Advanced deployment automation with Kubernetes and Helm")]
    Deployment(deployment::DeploymentCliArgs),

    #[command(about = "Model marketplace and registry management")]
    Marketplace(marketplace::MarketplaceArgs),

    #[command(about = "Package manager for models (install, remove, search, etc.)")]
    Package(package::PackageArgs),

    // Simplified package manager aliases
    #[command(about = "Install a model package")]
    Install(package::InstallArgs),

    #[command(about = "Remove a model package")]
    Remove(package::RemoveArgs),

    #[command(about = "Search for model packages")]
    Search(package::SearchArgs),

    #[command(about = "List installed model packages")]
    List(package::ListArgs),

    #[command(about = "Manage model repositories")]
    Repo(repo::RepoArgs),

    #[command(about = "Federated learning and edge deployment")]
    Federated(federated::FederatedArgs),

    #[command(about = "Web-based admin dashboard")]
    Dashboard(dashboard::DashboardArgs),

    #[command(about = "Advanced monitoring and alerting with Prometheus integration")]
    AdvancedMonitoring(advanced_monitoring::AdvancedMonitoringArgs),

    #[command(about = "API gateway with rate limiting and load balancing")]
    ApiGateway(api_gateway::ApiGatewayArgs),

    #[command(about = "Model versioning and A/B testing framework")]
    ModelVersioning(model_versioning::ModelVersioningArgs),

    #[command(about = "Data pipeline and ETL system for model training")]
    DataPipeline(data_pipeline::DataPipelineArgs),

    #[command(about = "Enterprise-grade backup and disaster recovery")]
    BackupRecovery(backup_recovery::BackupRecoveryArgs),

    #[command(about = "Comprehensive logging and audit trail system")]
    LoggingAudit(logging_audit::LoggingAuditArgs),

    #[command(about = "Enterprise performance optimization and auto-tuning")]
    PerformanceOptimization(performance_optimization::PerformanceOptimizationArgs),

    #[command(about = "Multi-tenant resource isolation and management")]
    MultiTenancy(multi_tenancy::MultiTenancyArgs),

    #[command(about = "Advanced caching and memory management")]
    AdvancedCache(advanced_cache::AdvancedCacheArgs),

    #[command(about = "Comprehensive testing and quality assurance framework")]
    QAFramework(qa_framework::QAFrameworkArgs),

    #[command(about = "Performance benchmarking and baseline establishment")]
    PerformanceBenchmark(performance_benchmark::PerformanceBenchmarkArgs),

    #[command(about = "Application upgrade and update management")]
    Upgrade(upgrade::UpgradeArgs),

    #[command(about = "Launch terminal user interface")]
    Tui,
}
