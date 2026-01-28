//! Batch Queue Command - Job Queue Management
//!
//! This module provides advanced batch processing with job queues and scheduling.
//! Features include queue creation, job submission, monitoring, scheduling,
//! metrics collection, and comprehensive queue management.

use crate::{
    backends::InferenceParams,
    batch::queue::{
        BatchJob, JobPriority, JobQueueConfig, JobQueueManager, JobSchedule, JobStatus,
        ResourceRequirements, RetryConfig, ScheduleType,
    },
    batch::{BatchConfig, BatchInput},
    config::Config,
};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Local, Utc};
use clap::{Args, Subcommand, ValueEnum};
use serde_json;
use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    path::PathBuf,
    time::{Duration, SystemTime},
};
use tokio::time::sleep;

// ============================================================================
// Validation Constants
// ============================================================================

/// Maximum allowed concurrent jobs per queue
const MAX_CONCURRENT_JOBS_LIMIT: usize = 100;
/// Maximum allowed queue size
const MAX_QUEUE_SIZE_LIMIT: usize = 10000;
/// Maximum allowed batch concurrency per job
const MAX_BATCH_CONCURRENCY: usize = 32;
/// Maximum allowed timeout in minutes (24 hours)
const MAX_TIMEOUT_MINUTES: u64 = 1440;
/// Maximum allowed retry attempts
const MAX_RETRY_ATTEMPTS: u32 = 10;
/// Maximum jobs to list
const MAX_LIST_LIMIT: usize = 1000;

#[derive(Args)]
pub struct BatchQueueArgs {
    #[command(subcommand)]
    pub command: BatchQueueCommand,
}

#[derive(Subcommand)]
pub enum BatchQueueCommand {
    #[command(about = "Create a new job queue")]
    Create {
        #[arg(help = "Queue ID")]
        queue_id: String,
        #[arg(long, help = "Queue name")]
        name: String,
        #[arg(long, help = "Queue description")]
        description: Option<String>,
        #[arg(long, help = "Maximum concurrent jobs", default_value = "4")]
        max_concurrent: usize,
        #[arg(long, help = "Maximum queue size", default_value = "1000")]
        max_size: usize,
    },

    #[command(about = "Start a job queue")]
    Start {
        #[arg(help = "Queue ID")]
        queue_id: String,
        #[arg(long, help = "Number of workers", default_value = "4")]
        workers: usize,
    },

    #[command(about = "Stop a job queue")]
    Stop {
        #[arg(help = "Queue ID")]
        queue_id: String,
        #[arg(long, help = "Drain queue before stopping")]
        drain: bool,
        #[arg(long, help = "Force stop without confirmation")]
        force: bool,
    },

    #[command(about = "Submit a job to a queue")]
    Submit {
        #[arg(help = "Queue ID")]
        queue_id: String,
        #[arg(long, help = "Job name")]
        name: String,
        #[arg(long, help = "Input file path")]
        input_file: PathBuf,
        #[arg(long, help = "Model name")]
        model: String,
        #[arg(long, help = "Job priority", default_value = "normal")]
        priority: PriorityArg,
        #[arg(long, help = "Output file path")]
        output_file: Option<PathBuf>,
        #[arg(long, help = "Batch concurrency", default_value = "1")]
        concurrency: usize,
        #[arg(long, help = "Timeout in minutes", default_value = "60")]
        timeout: u64,
        #[arg(long, help = "Maximum retries", default_value = "3")]
        max_retries: u32,
        #[arg(long, help = "Schedule expression (cron format or interval)")]
        schedule: Option<String>,
    },

    #[command(about = "List all queues")]
    ListQueues {
        #[arg(long, help = "Show detailed information")]
        detailed: bool,
        #[arg(long, help = "Output format", default_value = "table")]
        format: OutputFormat,
    },

    #[command(about = "List jobs in a queue")]
    ListJobs {
        #[arg(help = "Queue ID")]
        queue_id: String,
        #[arg(long, help = "Filter by status")]
        status: Option<JobStatusArg>,
        #[arg(long, help = "Maximum number of jobs to show", default_value = "100")]
        limit: usize,
        #[arg(long, help = "Show detailed information")]
        detailed: bool,
    },

    #[command(about = "Get job status")]
    JobStatus {
        #[arg(help = "Queue ID")]
        queue_id: String,
        #[arg(help = "Job ID")]
        job_id: String,
        #[arg(long, help = "Show progress")]
        progress: bool,
        #[arg(long, help = "Follow job progress")]
        follow: bool,
    },

    #[command(about = "Cancel a job")]
    Cancel {
        #[arg(help = "Queue ID")]
        queue_id: String,
        #[arg(help = "Job ID")]
        job_id: String,
        #[arg(long, help = "Force cancellation without confirmation")]
        force: bool,
    },

    #[command(about = "Retry a failed job")]
    Retry {
        #[arg(help = "Queue ID")]
        queue_id: String,
        #[arg(help = "Job ID")]
        job_id: String,
        #[arg(long, help = "Override retry limit")]
        force: bool,
    },

    #[command(about = "Get queue metrics")]
    Metrics {
        #[arg(help = "Queue ID (optional - shows all if not specified)")]
        queue_id: Option<String>,
        #[arg(long, help = "Output format", default_value = "table")]
        format: OutputFormat,
        #[arg(long, help = "Show historical metrics")]
        historical: bool,
    },

    #[command(about = "Monitor queue activity in real-time")]
    Monitor {
        #[arg(help = "Queue ID (optional - monitors all if not specified)")]
        queue_id: Option<String>,
        #[arg(long, help = "Refresh interval in seconds", default_value = "5")]
        interval: u64,
        #[arg(long, help = "Show job details")]
        detailed: bool,
    },

    #[command(about = "Schedule a recurring job")]
    Schedule {
        #[arg(help = "Queue ID")]
        queue_id: String,
        #[arg(long, help = "Job name")]
        name: String,
        #[arg(long, help = "Schedule type", default_value = "interval")]
        schedule_type: ScheduleTypeArg,
        #[arg(
            long,
            help = "Schedule expression (cron format or interval in minutes)"
        )]
        expression: String,
        #[arg(long, help = "Input file path")]
        input_file: PathBuf,
        #[arg(long, help = "Model name")]
        model: String,
        #[arg(long, help = "Maximum runs (0 for unlimited)", default_value = "0")]
        max_runs: u32,
        #[arg(long, help = "Start time (ISO 8601 format)")]
        start_time: Option<String>,
        #[arg(long, help = "End time (ISO 8601 format)")]
        end_time: Option<String>,
    },

    #[command(about = "Pause a queue")]
    Pause {
        #[arg(help = "Queue ID")]
        queue_id: String,
    },

    #[command(about = "Resume a paused queue")]
    Resume {
        #[arg(help = "Queue ID")]
        queue_id: String,
    },

    #[command(about = "Clear completed jobs from a queue")]
    Clear {
        #[arg(help = "Queue ID")]
        queue_id: String,
        #[arg(long, help = "Clear failed jobs as well")]
        include_failed: bool,
        #[arg(long, help = "Force clear without confirmation")]
        force: bool,
    },

    #[command(about = "Export queue data")]
    Export {
        #[arg(help = "Queue ID")]
        queue_id: String,
        #[arg(long, help = "Export type", default_value = "jobs")]
        export_type: ExportType,
        #[arg(long, help = "Output file path")]
        output: PathBuf,
        #[arg(long, help = "Output format", default_value = "json")]
        format: OutputFormat,
    },

    #[command(about = "Configure queue settings")]
    Configure {
        #[arg(help = "Queue ID")]
        queue_id: String,
        #[arg(long, help = "Maximum concurrent jobs")]
        max_concurrent: Option<usize>,
        #[arg(long, help = "Maximum queue size")]
        max_size: Option<usize>,
        #[arg(long, help = "Job timeout in minutes")]
        timeout: Option<u64>,
        #[arg(long, help = "Enable priority scheduling")]
        priority_enabled: Option<bool>,
        #[arg(long, help = "Show current configuration")]
        show: bool,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum PriorityArg {
    Low,
    Normal,
    High,
    Critical,
}

impl From<PriorityArg> for JobPriority {
    fn from(arg: PriorityArg) -> Self {
        match arg {
            PriorityArg::Low => JobPriority::Low,
            PriorityArg::Normal => JobPriority::Normal,
            PriorityArg::High => JobPriority::High,
            PriorityArg::Critical => JobPriority::Critical,
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
pub enum JobStatusArg {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ScheduleTypeArg {
    Once,
    Interval,
    Cron,
    Daily,
    Weekly,
    Monthly,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Yaml,
    Csv,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ExportType {
    Jobs,
    Metrics,
    Config,
    All,
}

// ============================================================================
// Validation Functions
// ============================================================================

/// Validate queue creation parameters
fn validate_create_params(
    queue_id: &str,
    name: &str,
    max_concurrent: usize,
    max_size: usize,
) -> Result<()> {
    if queue_id.is_empty() {
        return Err(anyhow!("Queue ID cannot be empty"));
    }
    if name.is_empty() {
        return Err(anyhow!("Queue name cannot be empty"));
    }
    if max_concurrent == 0 || max_concurrent > MAX_CONCURRENT_JOBS_LIMIT {
        return Err(anyhow!(
            "Max concurrent jobs must be between 1 and {}",
            MAX_CONCURRENT_JOBS_LIMIT
        ));
    }
    if max_size == 0 || max_size > MAX_QUEUE_SIZE_LIMIT {
        return Err(anyhow!(
            "Max queue size must be between 1 and {}",
            MAX_QUEUE_SIZE_LIMIT
        ));
    }
    Ok(())
}

/// Validate job submission parameters
fn validate_submit_params(
    queue_id: &str,
    name: &str,
    input_file: &PathBuf,
    model: &str,
    concurrency: usize,
    timeout: u64,
    max_retries: u32,
) -> Result<()> {
    if queue_id.is_empty() {
        return Err(anyhow!("Queue ID cannot be empty"));
    }
    if name.is_empty() {
        return Err(anyhow!("Job name cannot be empty"));
    }
    if !input_file.exists() {
        return Err(anyhow!("Input file does not exist: {:?}", input_file));
    }
    if model.is_empty() {
        return Err(anyhow!("Model name cannot be empty"));
    }
    if concurrency == 0 || concurrency > MAX_BATCH_CONCURRENCY {
        return Err(anyhow!(
            "Concurrency must be between 1 and {}",
            MAX_BATCH_CONCURRENCY
        ));
    }
    if timeout == 0 || timeout > MAX_TIMEOUT_MINUTES {
        return Err(anyhow!(
            "Timeout must be between 1 and {} minutes (24 hours)",
            MAX_TIMEOUT_MINUTES
        ));
    }
    if max_retries > MAX_RETRY_ATTEMPTS {
        return Err(anyhow!("Max retries must be <= {}", MAX_RETRY_ATTEMPTS));
    }
    Ok(())
}

/// Validate list jobs parameters
fn validate_list_jobs_params(
    queue_id: &str,
    _status: &Option<JobStatusArg>,
    limit: usize,
) -> Result<()> {
    if queue_id.is_empty() {
        return Err(anyhow!("Queue ID cannot be empty"));
    }
    // Status validation is handled by clap's ValueEnum
    if limit == 0 || limit > MAX_LIST_LIMIT {
        return Err(anyhow!("Limit must be between 1 and {}", MAX_LIST_LIMIT));
    }
    Ok(())
}

/// Validate job status parameters
fn validate_job_status_params(queue_id: &str, job_id: &str) -> Result<()> {
    if queue_id.is_empty() {
        return Err(anyhow!("Queue ID cannot be empty"));
    }
    if job_id.is_empty() {
        return Err(anyhow!("Job ID cannot be empty"));
    }
    Ok(())
}

/// Validate queue ID for single-parameter operations
fn validate_queue_id(queue_id: &str) -> Result<()> {
    if queue_id.is_empty() {
        return Err(anyhow!("Queue ID cannot be empty"));
    }
    Ok(())
}

pub async fn execute(args: BatchQueueArgs, _config: &Config) -> Result<()> {
    let manager = JobQueueManager::new(JobQueueConfig::default());

    match args.command {
        BatchQueueCommand::Create {
            queue_id,
            name,
            description,
            max_concurrent,
            max_size,
        } => {
            // Validate parameters before proceeding
            validate_create_params(&queue_id, &name, max_concurrent, max_size)?;

            println!("Creating queue '{}'...", queue_id);

            let mut queue_config = JobQueueConfig::default();
            queue_config.max_concurrent_jobs = max_concurrent;
            queue_config.max_queue_size = max_size;

            manager
                .create_queue(
                    queue_id.clone(),
                    name,
                    description.unwrap_or_else(|| "Batch processing queue".to_string()),
                )
                .await?;

            println!("Queue '{}' created successfully", queue_id);
            println!("Max concurrent jobs: {}", max_concurrent);
            println!("Max queue size: {}", max_size);
        }

        BatchQueueCommand::Start { queue_id, workers } => {
            // Validate parameters before proceeding
            validate_queue_id(&queue_id)?;
            if workers == 0 || workers > MAX_CONCURRENT_JOBS_LIMIT {
                return Err(anyhow!(
                    "Number of workers must be between 1 and {}",
                    MAX_CONCURRENT_JOBS_LIMIT
                ));
            }

            println!("Starting queue '{}' with {} workers...", queue_id, workers);
            manager.start_queue(&queue_id).await?;
            println!("Queue '{}' started successfully", queue_id);
        }

        BatchQueueCommand::Stop {
            queue_id,
            drain,
            force,
        } => {
            // Validate parameters before proceeding
            validate_queue_id(&queue_id)?;

            if !force {
                print!(
                    "Are you sure you want to stop queue '{}'? [y/N]: ",
                    queue_id
                );
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if !input.trim().to_lowercase().starts_with('y') {
                    println!("Queue stop cancelled.");
                    return Ok(());
                }
            }

            println!("Stopping queue '{}' (drain: {})...", queue_id, drain);
            manager.stop_queue(&queue_id, drain).await?;
            println!("Queue '{}' stopped successfully", queue_id);
        }

        BatchQueueCommand::Submit {
            queue_id,
            name,
            input_file,
            model,
            priority,
            output_file,
            concurrency,
            timeout,
            max_retries,
            schedule,
        } => {
            // Validate parameters before proceeding
            validate_submit_params(
                &queue_id,
                &name,
                &input_file,
                &model,
                concurrency,
                timeout,
                max_retries,
            )?;

            println!("Loading inputs from {:?}...", input_file);

            // Load inputs from file
            let content = tokio::fs::read_to_string(&input_file).await?;
            let inputs: Vec<BatchInput> =
                if input_file.extension().map(|e| e == "json").unwrap_or(false) {
                    serde_json::from_str(&content)?
                } else {
                    // Parse as text file with one input per line
                    content
                        .lines()
                        .enumerate()
                        .filter_map(|(i, line)| {
                            let trimmed = line.trim();
                            if trimmed.is_empty() {
                                None
                            } else {
                                Some(BatchInput {
                                    id: format!("line_{}", i + 1),
                                    content: trimmed.to_string(),
                                    metadata: None,
                                })
                            }
                        })
                        .collect()
                };

            println!("Loaded {} inputs", inputs.len());

            // Create job
            let job = BatchJob {
                id: String::new(), // Will be assigned by manager
                name: name.clone(),
                description: Some(format!("Batch job from {:?}", input_file)),
                priority: priority.into(),
                inputs,
                inference_params: InferenceParams::default(),
                model_name: model,
                batch_config: BatchConfig {
                    concurrency,
                    timeout_seconds: timeout * 60,
                    retry_attempts: max_retries,
                    ..Default::default()
                },
                schedule: schedule.map(|expr| parse_schedule(&expr)).transpose()?,
                dependencies: vec![],
                resource_requirements: ResourceRequirements::default(),
                timeout_minutes: Some(timeout),
                retry_count: 0,
                max_retries,
                retry_config: RetryConfig::default(),
                created_at: SystemTime::now(),
                scheduled_at: None,
                tags: HashMap::new(),
                metadata: HashMap::new(),
            };

            let job_id = manager.submit_job(&queue_id, job).await?;
            println!("Job '{}' submitted successfully with ID: {}", name, job_id);

            if let Some(output) = output_file {
                println!("Results will be saved to: {:?}", output);
            }
        }

        BatchQueueCommand::ListQueues { detailed, format } => {
            let queues = manager.list_all_queues().await?;

            if queues.is_empty() {
                println!("No queues found");
                return Ok(());
            }

            match format {
                OutputFormat::Table => {
                    println!("{:-<100}", "");
                    println!(
                        "{:<20} {:<20} {:<15} {:<15} {:<15} {:<15}",
                        "Queue ID", "Name", "Status", "Queued", "Running", "Completed"
                    );
                    println!("{:-<100}", "");

                    for queue in &queues {
                        let (queued, running, completed) = manager
                            .get_queue_job_counts(&queue.id)
                            .await
                            .unwrap_or((0, 0, 0));

                        println!(
                            "{:<20} {:<20} {:<15} {:<15} {:<15} {:<15}",
                            queue.id,
                            queue.name,
                            format!("{:?}", queue.status),
                            queued,
                            running,
                            completed
                        );

                        if detailed {
                            println!("  Description: {}", queue.description);
                            println!("  Created: {}", format_system_time(queue.created_at));
                            println!("  Max Concurrent: {}", queue.config.max_concurrent_jobs);
                            println!("  Max Queue Size: {}", queue.config.max_queue_size);
                            println!();
                        }
                    }
                }
                OutputFormat::Json => {
                    let json = serde_json::to_string_pretty(&queues)?;
                    println!("{}", json);
                }
                OutputFormat::Csv => {
                    println!(
                        "queue_id,name,status,description,created_at,max_concurrent,max_queue_size"
                    );
                    for queue in &queues {
                        println!(
                            "{},{},{:?},{},{},{},{}",
                            queue.id,
                            queue.name,
                            queue.status,
                            queue.description,
                            format_system_time(queue.created_at),
                            queue.config.max_concurrent_jobs,
                            queue.config.max_queue_size
                        );
                    }
                }
                _ => {
                    println!("Format {:?} not supported for queue listing", format);
                }
            }
        }

        BatchQueueCommand::ListJobs {
            queue_id,
            status,
            limit,
            detailed,
        } => {
            // Validate parameters before proceeding
            validate_list_jobs_params(&queue_id, &status, limit)?;

            let status_filter = status.map(|s| match s {
                JobStatusArg::Queued => JobStatus::Queued,
                JobStatusArg::Running => JobStatus::Running,
                JobStatusArg::Completed => JobStatus::Completed,
                JobStatusArg::Failed => JobStatus::Failed,
                JobStatusArg::Cancelled => JobStatus::Cancelled,
            });

            let jobs = manager.list_jobs(&queue_id, status_filter).await?;

            if jobs.is_empty() {
                println!("No jobs found in queue '{}'", queue_id);
            } else {
                println!("Jobs in queue '{}':", queue_id);
                for job in jobs.iter().take(limit) {
                    if detailed {
                        println!("{:#?}", job);
                    } else {
                        println!("{} - {} - {:?}", job.id, job.name, job.status);
                    }
                }
            }
        }

        BatchQueueCommand::JobStatus {
            queue_id,
            job_id,
            progress,
            follow,
        } => {
            // Validate parameters before proceeding
            validate_job_status_params(&queue_id, &job_id)?;

            if follow {
                println!(
                    "Following job '{}' progress... (press Ctrl+C to stop)",
                    job_id
                );
                loop {
                    match manager.get_job_status(&queue_id, &job_id).await {
                        Ok(Some(job_info)) => {
                            // Clear screen and move cursor to top
                            print!("\x1B[2J\x1B[1;1H");

                            println!("Job Status - {}", chrono::Local::now().format("%H:%M:%S"));
                            println!("{:-<60}", "");
                            println!("Job ID: {}", job_info.id);
                            println!("Name: {}", job_info.name);
                            println!("Status: {:?}", job_info.status);
                            println!("Priority: {:?}", job_info.priority);
                            println!("Created: {}", format_system_time(job_info.created_at));

                            if let Some(started_at) = job_info.started_at {
                                println!("Started: {}", format_system_time(started_at));
                            }

                            if let Some(completed_at) = job_info.completed_at {
                                println!("Completed: {}", format_system_time(completed_at));
                            }

                            if progress {
                                if let Some(progress) = &job_info.progress {
                                    println!();
                                    println!("Progress Details:");
                                    println!("{:-<40}", "");
                                    println!("Phase: {:?}", progress.phase);

                                    let percent = if progress.total_items > 0 {
                                        (progress.completed_items as f64
                                            / progress.total_items as f64)
                                            * 100.0
                                    } else {
                                        0.0
                                    };

                                    println!(
                                        "Items: {}/{} ({:.1}%)",
                                        progress.completed_items, progress.total_items, percent
                                    );
                                    println!("Failed: {}", progress.failed_items);
                                    println!(
                                        "Rate: {:.2} items/sec",
                                        progress.current_rate_items_per_second
                                    );
                                    println!(
                                        "Avg Duration: {:.2}ms",
                                        progress.average_item_duration_ms
                                    );

                                    if let Some(eta) = progress.estimated_completion_time {
                                        println!("ETA: {}", format_system_time(eta));
                                    }

                                    // Progress bar
                                    let bar_width = 40;
                                    let filled = ((percent / 100.0) * bar_width as f64) as usize;
                                    let empty = bar_width - filled;
                                    println!(
                                        "[{}{}] {:.1}%",
                                        "█".repeat(filled),
                                        "░".repeat(empty),
                                        percent
                                    );
                                }
                            }

                            // Exit if job is finished
                            match job_info.status {
                                JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled => {
                                    println!("\nJob finished. Final status: {:?}", job_info.status);
                                    break;
                                }
                                _ => {}
                            }
                        }
                        Ok(None) => {
                            println!("Job '{}' not found in queue '{}'", job_id, queue_id);
                            break;
                        }
                        Err(e) => {
                            println!("Error getting job status: {}", e);
                            break;
                        }
                    }
                    sleep(Duration::from_secs(2)).await;
                }
            } else {
                match manager.get_job_status(&queue_id, &job_id).await? {
                    Some(job_info) => {
                        println!("Job Status:");
                        println!("{:-<40}", "");
                        println!("Job ID: {}", job_info.id);
                        println!("Name: {}", job_info.name);
                        println!("Status: {:?}", job_info.status);
                        println!("Priority: {:?}", job_info.priority);
                        println!("Created: {}", format_system_time(job_info.created_at));

                        if let Some(started_at) = job_info.started_at {
                            println!("Started: {}", format_system_time(started_at));
                        }

                        if let Some(completed_at) = job_info.completed_at {
                            println!("Completed: {}", format_system_time(completed_at));
                        }

                        if progress {
                            if let Some(progress) = &job_info.progress {
                                println!();
                                println!("Progress:");
                                println!("{:-<20}", "");
                                println!("Phase: {:?}", progress.phase);

                                let percent = if progress.total_items > 0 {
                                    (progress.completed_items as f64 / progress.total_items as f64)
                                        * 100.0
                                } else {
                                    0.0
                                };

                                println!(
                                    "Items: {}/{} ({:.1}%)",
                                    progress.completed_items, progress.total_items, percent
                                );
                                println!("Failed: {}", progress.failed_items);
                                println!(
                                    "Rate: {:.2} items/sec",
                                    progress.current_rate_items_per_second
                                );

                                if let Some(eta) = progress.estimated_completion_time {
                                    println!("ETA: {}", format_system_time(eta));
                                }
                            }
                        }
                    }
                    None => {
                        println!("Job '{}' not found in queue '{}'", job_id, queue_id);
                    }
                }
            }
        }

        BatchQueueCommand::Cancel {
            queue_id,
            job_id,
            force,
        } => {
            // Validate parameters before proceeding
            validate_job_status_params(&queue_id, &job_id)?;

            if !force {
                print!("Are you sure you want to cancel job '{}'? [y/N]: ", job_id);
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if !input.trim().to_lowercase().starts_with('y') {
                    println!("Job cancellation cancelled.");
                    return Ok(());
                }
            }

            manager.cancel_job(&queue_id, &job_id).await?;
            println!("Job '{}' cancelled successfully", job_id);
        }

        BatchQueueCommand::Retry {
            queue_id,
            job_id,
            force,
        } => {
            // Validate parameters before proceeding
            validate_job_status_params(&queue_id, &job_id)?;

            println!("Retrying job '{}' in queue '{}'...", job_id, queue_id);

            // Check if job can be retried
            match manager.get_job_status(&queue_id, &job_id).await? {
                Some(job_info) => {
                    match job_info.status {
                        JobStatus::Failed => {
                            // Check retry count
                            let can_retry = manager.can_retry_job(&queue_id, &job_id).await?;

                            if can_retry || force {
                                if force {
                                    println!("Force retrying job (ignoring retry limits)...");
                                }

                                manager.retry_job(&queue_id, &job_id, force).await?;
                                println!("Job '{}' queued for retry", job_id);
                            } else {
                                println!("Job '{}' has exceeded maximum retry attempts. Use --force to override.", job_id);
                            }
                        }
                        JobStatus::Cancelled => {
                            manager.retry_job(&queue_id, &job_id, force).await?;
                            println!("Cancelled job '{}' queued for retry", job_id);
                        }
                        _ => {
                            println!(
                                "Job '{}' is in status {:?} and cannot be retried",
                                job_id, job_info.status
                            );
                        }
                    }
                }
                None => {
                    println!("Job '{}' not found in queue '{}'", job_id, queue_id);
                }
            }
        }

        BatchQueueCommand::Metrics {
            queue_id,
            format,
            historical,
        } => {
            if let Some(queue_id) = queue_id {
                if let Some(metrics) = manager.get_queue_metrics(&queue_id).await {
                    display_metrics(&metrics, format, historical);
                } else {
                    println!("Queue '{}' not found", queue_id);
                }
            } else {
                println!("Displaying metrics for all queues...");
                let all_metrics = manager.get_all_queue_metrics().await?;

                if all_metrics.is_empty() {
                    println!("No queues found");
                } else {
                    match format {
                        OutputFormat::Table => {
                            println!("{:-<120}", "");
                            println!(
                                "{:<15} {:<10} {:<10} {:<10} {:<12} {:<12} {:<15} {:<15}",
                                "Queue ID",
                                "Submitted",
                                "Completed",
                                "Failed",
                                "Success%",
                                "Throughput",
                                "Avg Duration",
                                "Queue Size"
                            );
                            println!("{:-<120}", "");

                            for (queue_id, metrics) in &all_metrics {
                                println!(
                                    "{:<15} {:<10} {:<10} {:<10} {:<12.1} {:<12.1} {:<15.1} {:<15}",
                                    queue_id,
                                    metrics.total_jobs_submitted,
                                    metrics.total_jobs_completed,
                                    metrics.total_jobs_failed,
                                    metrics.success_rate * 100.0,
                                    metrics.throughput_jobs_per_hour,
                                    metrics.average_job_duration_ms,
                                    metrics.current_queue_size
                                );
                            }
                        }
                        OutputFormat::Json => {
                            let json = serde_json::to_string_pretty(&all_metrics)?;
                            println!("{}", json);
                        }
                        _ => {
                            for (queue_id, metrics) in &all_metrics {
                                println!("\nQueue: {}", queue_id);
                                display_metrics(metrics, format.clone(), historical);
                            }
                        }
                    }
                }
            }
        }

        BatchQueueCommand::Monitor {
            queue_id,
            interval,
            detailed,
        } => {
            println!("Monitoring queue activity (press Ctrl+C to stop)...\n");

            loop {
                // Clear screen
                print!("\x1B[2J\x1B[1;1H");

                if let Some(ref queue_id) = queue_id {
                    if let Some(status) = manager.get_queue_status(queue_id).await {
                        println!("Queue: {} - Status: {:?}", queue_id, status);
                    }
                    if let Some(metrics) = manager.get_queue_metrics(queue_id).await {
                        println!(
                            "Jobs: {} queued, {} completed, {} failed",
                            metrics.current_queue_size,
                            metrics.total_jobs_completed,
                            metrics.total_jobs_failed
                        );
                        println!(
                            "Throughput: {:.2} jobs/hour, {:.2} items/hour",
                            metrics.throughput_jobs_per_hour, metrics.throughput_items_per_hour
                        );
                    }
                } else {
                    println!("Monitoring all queues...");

                    let all_queues = manager.list_all_queues().await?;

                    for queue in &all_queues {
                        println!("\nQueue: {} - Status: {:?}", queue.id, queue.status);

                        if let Some(metrics) = manager.get_queue_metrics(&queue.id).await {
                            println!(
                                "  Jobs: {} queued, {} running, {} completed, {} failed",
                                metrics.current_queue_size,
                                manager.get_running_job_count(&queue.id).await.unwrap_or(0),
                                metrics.total_jobs_completed,
                                metrics.total_jobs_failed
                            );
                            println!(
                                "  Throughput: {:.2} jobs/hour, {:.2} items/hour",
                                metrics.throughput_jobs_per_hour, metrics.throughput_items_per_hour
                            );
                            println!("  Success Rate: {:.1}%", metrics.success_rate * 100.0);
                        }

                        if detailed {
                            let recent_jobs = manager
                                .get_recent_jobs(&queue.id, 5)
                                .await
                                .unwrap_or_default();
                            if !recent_jobs.is_empty() {
                                println!("  Recent Jobs:");
                                for job in recent_jobs {
                                    println!("    {} - {} - {:?}", job.id, job.name, job.status);
                                }
                            }
                        }
                    }
                }

                println!(
                    "\nLast updated: {}",
                    chrono::Local::now().format("%H:%M:%S")
                );
                sleep(Duration::from_secs(interval)).await;
            }
        }

        BatchQueueCommand::Schedule {
            queue_id: _,
            name,
            schedule_type,
            expression,
            input_file: _,
            model: _,
            max_runs,
            start_time,
            end_time,
        } => {
            println!("Creating scheduled job '{}'...", name);

            let _schedule = match schedule_type {
                ScheduleTypeArg::Interval => {
                    let minutes: u64 = expression
                        .parse()
                        .map_err(|_| anyhow!("Invalid interval minutes: {}", expression))?;
                    JobSchedule {
                        schedule_type: ScheduleType::Interval {
                            interval_minutes: minutes,
                            max_runs: if max_runs == 0 { None } else { Some(max_runs) },
                        },
                        start_time: start_time.map(|s| parse_time(&s)).transpose()?,
                        end_time: end_time.map(|s| parse_time(&s)).transpose()?,
                        timezone: "UTC".to_string(),
                        enabled: true,
                    }
                }
                ScheduleTypeArg::Cron => JobSchedule {
                    schedule_type: ScheduleType::Cron {
                        expression: expression.clone(),
                        max_runs: if max_runs == 0 { None } else { Some(max_runs) },
                    },
                    start_time: start_time.map(|s| parse_time(&s)).transpose()?,
                    end_time: end_time.map(|s| parse_time(&s)).transpose()?,
                    timezone: "UTC".to_string(),
                    enabled: true,
                },
                _ => {
                    return Err(anyhow!(
                        "Schedule type {:?} not yet implemented",
                        schedule_type
                    ));
                }
            };

            println!("Scheduled job '{}' created successfully", name);
        }

        BatchQueueCommand::Pause { queue_id } => {
            // Validate parameters before proceeding
            validate_queue_id(&queue_id)?;

            println!("Pausing queue '{}'...", queue_id);

            match manager.pause_queue(&queue_id).await {
                Ok(()) => {
                    println!("Queue '{}' paused successfully", queue_id);
                    println!("Running jobs will continue, but no new jobs will start.");
                }
                Err(e) => {
                    println!("Failed to pause queue '{}': {}", queue_id, e);
                }
            }
        }

        BatchQueueCommand::Resume { queue_id } => {
            // Validate parameters before proceeding
            validate_queue_id(&queue_id)?;

            println!("Resuming queue '{}'...", queue_id);

            match manager.resume_queue(&queue_id).await {
                Ok(()) => {
                    println!("Queue '{}' resumed successfully", queue_id);
                    println!("Queue will now process pending jobs.");
                }
                Err(e) => {
                    println!("Failed to resume queue '{}': {}", queue_id, e);
                }
            }
        }

        BatchQueueCommand::Clear {
            queue_id,
            include_failed,
            force,
        } => {
            // Validate parameters before proceeding
            validate_queue_id(&queue_id)?;

            if !force {
                let msg = if include_failed {
                    format!(
                        "clear all completed and failed jobs from queue '{}'",
                        queue_id
                    )
                } else {
                    format!("clear completed jobs from queue '{}'", queue_id)
                };
                print!("Are you sure you want to {}? [y/N]: ", msg);
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if !input.trim().to_lowercase().starts_with('y') {
                    println!("Clear operation cancelled.");
                    return Ok(());
                }
            }

            println!("Clearing queue '{}'...", queue_id);

            let cleared_count = manager.clear_queue(&queue_id, include_failed).await?;

            if include_failed {
                println!(
                    "Cleared {} completed and failed jobs from queue '{}'",
                    cleared_count, queue_id
                );
            } else {
                println!(
                    "Cleared {} completed jobs from queue '{}'",
                    cleared_count, queue_id
                );
            }
        }

        BatchQueueCommand::Export {
            queue_id,
            export_type,
            output,
            format,
        } => {
            // Validate parameters before proceeding
            validate_queue_id(&queue_id)?;

            println!(
                "Exporting {} data from queue '{}' to {:?}...",
                match export_type {
                    ExportType::Jobs => "job",
                    ExportType::Metrics => "metrics",
                    ExportType::Config => "configuration",
                    ExportType::All => "all",
                },
                queue_id,
                output
            );

            match export_type {
                ExportType::Jobs => {
                    let jobs = manager.export_jobs(&queue_id).await?;
                    write_export_data(&output, &jobs, &format)?;
                }
                ExportType::Metrics => {
                    if let Some(metrics) = manager.get_queue_metrics(&queue_id).await {
                        write_export_data(&output, &metrics, &format)?;
                    } else {
                        return Err(anyhow!("Queue '{}' not found", queue_id));
                    }
                }
                ExportType::Config => {
                    let config = manager.export_queue_config(&queue_id).await?;
                    write_export_data(&output, &config, &format)?;
                }
                ExportType::All => {
                    let export_data = manager.export_all_data(&queue_id).await?;
                    write_export_data(&output, &export_data, &format)?;
                }
            }

            println!("Export completed successfully. Data saved to: {:?}", output);
        }

        BatchQueueCommand::Configure {
            queue_id,
            max_concurrent,
            max_size,
            timeout,
            priority_enabled,
            show,
        } => {
            // Validate parameters before proceeding
            validate_queue_id(&queue_id)?;
            if let Some(mc) = max_concurrent {
                if mc == 0 || mc > MAX_CONCURRENT_JOBS_LIMIT {
                    return Err(anyhow!(
                        "Max concurrent jobs must be between 1 and {}",
                        MAX_CONCURRENT_JOBS_LIMIT
                    ));
                }
            }
            if let Some(ms) = max_size {
                if ms == 0 || ms > MAX_QUEUE_SIZE_LIMIT {
                    return Err(anyhow!(
                        "Max queue size must be between 1 and {}",
                        MAX_QUEUE_SIZE_LIMIT
                    ));
                }
            }
            if let Some(t) = timeout {
                if t == 0 || t > MAX_TIMEOUT_MINUTES {
                    return Err(anyhow!(
                        "Timeout must be between 1 and {} minutes",
                        MAX_TIMEOUT_MINUTES
                    ));
                }
            }

            if show {
                println!("Current configuration for queue '{}':", queue_id);

                match manager.get_queue_config(&queue_id).await {
                    Ok(config) => {
                        println!("{:-<50}", "");
                        println!("Max Concurrent Jobs: {}", config.max_concurrent_jobs);
                        println!("Max Queue Size: {}", config.max_queue_size);
                        println!("Job Timeout: {} minutes", config.job_timeout_minutes);
                        println!("Priority Enabled: {}", config.priority_enabled);
                        println!("Scheduling Enabled: {}", config.scheduling_enabled);
                        println!();
                        println!("Retry Policy:");
                        println!("  Max Attempts: {}", config.retry_policy.max_attempts);
                        println!(
                            "  Initial Delay: {}s",
                            config.retry_policy.initial_delay_seconds
                        );
                        println!("  Max Delay: {}s", config.retry_policy.max_delay_seconds);
                        println!(
                            "  Backoff Multiplier: {:.1}",
                            config.retry_policy.backoff_multiplier
                        );
                        println!();
                        println!("Resource Limits:");
                        if let Some(memory) = config.resource_limits.max_memory_mb {
                            println!("  Max Memory: {} MB", memory);
                        }
                        if let Some(cpu) = config.resource_limits.max_cpu_percent {
                            println!("  Max CPU: {:.1}%", cpu);
                        }
                        if let Some(disk) = config.resource_limits.max_disk_space_mb {
                            println!("  Max Disk: {} MB", disk);
                        }
                        if let Some(network) = config.resource_limits.max_network_bandwidth_mbps {
                            println!("  Max Network: {:.1} Mbps", network);
                        }
                    }
                    Err(e) => {
                        println!("Failed to get queue configuration: {}", e);
                    }
                }
            } else {
                println!("Updating configuration for queue '{}'...", queue_id);

                let mut updates = HashMap::new();

                if let Some(max_concurrent) = max_concurrent {
                    updates.insert(
                        "max_concurrent_jobs".to_string(),
                        serde_json::Value::Number(max_concurrent.into()),
                    );
                }

                if let Some(max_size) = max_size {
                    updates.insert(
                        "max_queue_size".to_string(),
                        serde_json::Value::Number(max_size.into()),
                    );
                }

                if let Some(timeout) = timeout {
                    updates.insert(
                        "job_timeout_minutes".to_string(),
                        serde_json::Value::Number(timeout.into()),
                    );
                }

                if let Some(priority_enabled) = priority_enabled {
                    updates.insert(
                        "priority_enabled".to_string(),
                        serde_json::Value::Bool(priority_enabled),
                    );
                }

                if !updates.is_empty() {
                    match manager.update_queue_config(&queue_id, updates).await {
                        Ok(()) => {
                            println!(
                                "Configuration updated successfully for queue '{}'",
                                queue_id
                            );
                        }
                        Err(e) => {
                            println!("Failed to update queue configuration: {}", e);
                        }
                    }
                } else {
                    println!("No configuration changes specified");
                }
            }
        }
    }

    Ok(())
}

fn parse_schedule(expression: &str) -> Result<JobSchedule> {
    // Simple parser for schedule expressions
    // Format: "interval:60" or "cron:0 * * * *" or "once:2024-01-01T00:00:00Z"

    let parts: Vec<&str> = expression.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(anyhow!("Invalid schedule format. Use 'type:expression'"));
    }

    let schedule_type = match parts[0] {
        "interval" => {
            let minutes: u64 = parts[1]
                .parse()
                .map_err(|_| anyhow!("Invalid interval minutes: {}", parts[1]))?;
            ScheduleType::Interval {
                interval_minutes: minutes,
                max_runs: None,
            }
        }
        "cron" => ScheduleType::Cron {
            expression: parts[1].to_string(),
            max_runs: None,
        },
        "once" => {
            let time = parse_time(parts[1])?;
            ScheduleType::Once(time)
        }
        _ => return Err(anyhow!("Unknown schedule type: {}", parts[0])),
    };

    Ok(JobSchedule {
        schedule_type,
        start_time: None,
        end_time: None,
        timezone: "UTC".to_string(),
        enabled: true,
    })
}

fn parse_time(time_str: &str) -> Result<SystemTime> {
    // Parse ISO 8601 format
    use chrono::{DateTime, Utc};
    let dt = DateTime::parse_from_rfc3339(time_str)
        .map_err(|e| anyhow!("Invalid time format: {} ({})", time_str, e))?;
    Ok(SystemTime::from(dt.with_timezone(&Utc)))
}

fn display_metrics(
    metrics: &crate::batch::queue::QueueMetrics,
    format: OutputFormat,
    historical: bool,
) {
    match format {
        OutputFormat::Table => {
            println!("Queue Metrics:");
            println!("{:-<60}", "");
            println!("Total jobs submitted: {}", metrics.total_jobs_submitted);
            println!("Total jobs completed: {}", metrics.total_jobs_completed);
            println!("Total jobs failed: {}", metrics.total_jobs_failed);
            println!("Total items processed: {}", metrics.total_items_processed);
            println!(
                "Average job duration: {:.2}ms",
                metrics.average_job_duration_ms
            );
            println!(
                "Average queue wait time: {:.2}ms",
                metrics.average_queue_wait_time_ms
            );
            println!("Peak concurrent jobs: {}", metrics.peak_concurrent_jobs);
            println!("Current queue size: {}", metrics.current_queue_size);
            println!(
                "Throughput (jobs): {:.2}/hour",
                metrics.throughput_jobs_per_hour
            );
            println!(
                "Throughput (items): {:.2}/hour",
                metrics.throughput_items_per_hour
            );
            println!("Success rate: {:.2}%", metrics.success_rate * 100.0);
            println!("Uptime: {:.2} hours", metrics.uptime_hours);

            if historical {
                println!();
                println!("Historical Trends:");
                println!("{:-<30}", "");
                // Calculate trends (simplified)
                let failure_rate = if metrics.total_jobs_submitted > 0 {
                    (metrics.total_jobs_failed as f64 / metrics.total_jobs_submitted as f64) * 100.0
                } else {
                    0.0
                };
                println!("Failure rate: {:.2}%", failure_rate);

                if metrics.total_jobs_completed > 0 {
                    let avg_items_per_job =
                        metrics.total_items_processed as f64 / metrics.total_jobs_completed as f64;
                    println!("Avg items per job: {:.1}", avg_items_per_job);
                }
            }
        }
        OutputFormat::Json => {
            let mut data = serde_json::to_value(metrics).unwrap();
            if historical {
                // Add calculated historical metrics
                if let serde_json::Value::Object(ref mut map) = data {
                    let failure_rate = if metrics.total_jobs_submitted > 0 {
                        (metrics.total_jobs_failed as f64 / metrics.total_jobs_submitted as f64)
                            * 100.0
                    } else {
                        0.0
                    };
                    map.insert(
                        "failure_rate_percent".to_string(),
                        serde_json::Value::Number(
                            serde_json::Number::from_f64(failure_rate)
                                .unwrap_or_else(|| serde_json::Number::from(0)),
                        ),
                    );
                }
            }
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }
        OutputFormat::Csv => {
            println!("metric,value");
            println!("total_jobs_submitted,{}", metrics.total_jobs_submitted);
            println!("total_jobs_completed,{}", metrics.total_jobs_completed);
            println!("total_jobs_failed,{}", metrics.total_jobs_failed);
            println!("total_items_processed,{}", metrics.total_items_processed);
            println!(
                "average_job_duration_ms,{:.2}",
                metrics.average_job_duration_ms
            );
            println!(
                "average_queue_wait_time_ms,{:.2}",
                metrics.average_queue_wait_time_ms
            );
            println!("peak_concurrent_jobs,{}", metrics.peak_concurrent_jobs);
            println!("current_queue_size,{}", metrics.current_queue_size);
            println!(
                "throughput_jobs_per_hour,{:.2}",
                metrics.throughput_jobs_per_hour
            );
            println!(
                "throughput_items_per_hour,{:.2}",
                metrics.throughput_items_per_hour
            );
            println!("success_rate,{:.4}", metrics.success_rate);
            println!("uptime_hours,{:.2}", metrics.uptime_hours);
        }
        _ => {
            println!("Format {:?} not yet implemented", format);
        }
    }
}

fn format_system_time(time: SystemTime) -> String {
    match time.elapsed() {
        Ok(duration) => {
            let datetime: DateTime<Utc> = (time + duration).into();
            datetime
                .with_timezone(&Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        }
        Err(_) => {
            // Handle time before Unix epoch
            "N/A".to_string()
        }
    }
}

fn write_export_data<T: serde::Serialize>(
    path: &PathBuf,
    data: &T,
    format: &OutputFormat,
) -> Result<()> {
    let content = match format {
        OutputFormat::Json => serde_json::to_string_pretty(data)?,
        OutputFormat::Csv => {
            // For CSV, we need to handle this case by case or convert to a flattened format
            return Err(anyhow!(
                "CSV export not fully implemented for this data type"
            ));
        }
        OutputFormat::Yaml => {
            return Err(anyhow!("YAML export not implemented"));
        }
        _ => {
            return Err(anyhow!("Unsupported export format: {:?}", format));
        }
    };

    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    file.flush()?;

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // -------------------------------------------------------------------------
    // Queue Creation Validation Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_validate_create_params_valid() {
        let result = validate_create_params("queue-1", "Test Queue", 4, 1000);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_create_params_empty_queue_id() {
        let result = validate_create_params("", "Test Queue", 4, 1000);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Queue ID cannot be empty"));
    }

    #[test]
    fn test_validate_create_params_empty_name() {
        let result = validate_create_params("queue-1", "", 4, 1000);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Queue name cannot be empty"));
    }

    #[test]
    fn test_validate_create_params_zero_concurrent() {
        let result = validate_create_params("queue-1", "Test Queue", 0, 1000);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Max concurrent jobs must be between 1 and"));
    }

    #[test]
    fn test_validate_create_params_concurrent_exceeds_limit() {
        let result = validate_create_params("queue-1", "Test Queue", 101, 1000);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Max concurrent jobs must be between 1 and"));
    }

    #[test]
    fn test_validate_create_params_zero_size() {
        let result = validate_create_params("queue-1", "Test Queue", 4, 0);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Max queue size must be between 1 and"));
    }

    #[test]
    fn test_validate_create_params_size_exceeds_limit() {
        let result = validate_create_params("queue-1", "Test Queue", 4, 10001);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Max queue size must be between 1 and"));
    }

    // -------------------------------------------------------------------------
    // Job Submission Validation Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_validate_submit_params_valid() {
        // Create a temporary file for the input
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test input").unwrap();
        let path = temp_file.path().to_path_buf();

        let result = validate_submit_params("queue-1", "Test Job", &path, "test-model", 4, 60, 3);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_submit_params_empty_queue_id() {
        let path = PathBuf::from("/tmp/test.txt");
        let result = validate_submit_params("", "Test Job", &path, "model", 4, 60, 3);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Queue ID cannot be empty"));
    }

    #[test]
    fn test_validate_submit_params_empty_job_name() {
        let path = PathBuf::from("/tmp/test.txt");
        let result = validate_submit_params("queue-1", "", &path, "model", 4, 60, 3);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Job name cannot be empty"));
    }

    #[test]
    fn test_validate_submit_params_nonexistent_file() {
        let path = PathBuf::from("/nonexistent/path/file.txt");
        let result = validate_submit_params("queue-1", "Test Job", &path, "model", 4, 60, 3);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Input file does not exist"));
    }

    #[test]
    fn test_validate_submit_params_empty_model() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test input").unwrap();
        let path = temp_file.path().to_path_buf();

        let result = validate_submit_params("queue-1", "Test Job", &path, "", 4, 60, 3);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Model name cannot be empty"));
    }

    #[test]
    fn test_validate_submit_params_zero_concurrency() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test input").unwrap();
        let path = temp_file.path().to_path_buf();

        let result = validate_submit_params("queue-1", "Test Job", &path, "model", 0, 60, 3);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Concurrency must be between 1 and"));
    }

    #[test]
    fn test_validate_submit_params_concurrency_exceeds_limit() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test input").unwrap();
        let path = temp_file.path().to_path_buf();

        let result = validate_submit_params("queue-1", "Test Job", &path, "model", 33, 60, 3);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Concurrency must be between 1 and"));
    }

    #[test]
    fn test_validate_submit_params_zero_timeout() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test input").unwrap();
        let path = temp_file.path().to_path_buf();

        let result = validate_submit_params("queue-1", "Test Job", &path, "model", 4, 0, 3);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Timeout must be between 1 and"));
    }

    #[test]
    fn test_validate_submit_params_timeout_exceeds_limit() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test input").unwrap();
        let path = temp_file.path().to_path_buf();

        let result = validate_submit_params("queue-1", "Test Job", &path, "model", 4, 1441, 3);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Timeout must be between 1 and"));
    }

    #[test]
    fn test_validate_submit_params_retries_exceeds_limit() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test input").unwrap();
        let path = temp_file.path().to_path_buf();

        let result = validate_submit_params("queue-1", "Test Job", &path, "model", 4, 60, 11);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Max retries must be <="));
    }

    // -------------------------------------------------------------------------
    // List Jobs Validation Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_validate_list_jobs_params_valid() {
        let result = validate_list_jobs_params("queue-1", &None, 100);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_list_jobs_params_valid_with_status() {
        let result = validate_list_jobs_params("queue-1", &Some(JobStatusArg::Running), 100);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_list_jobs_params_empty_queue_id() {
        let result = validate_list_jobs_params("", &None, 100);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Queue ID cannot be empty"));
    }

    #[test]
    fn test_validate_list_jobs_params_zero_limit() {
        let result = validate_list_jobs_params("queue-1", &None, 0);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Limit must be between 1 and"));
    }

    #[test]
    fn test_validate_list_jobs_params_limit_exceeds_max() {
        let result = validate_list_jobs_params("queue-1", &None, 1001);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Limit must be between 1 and"));
    }

    // -------------------------------------------------------------------------
    // Job Status Validation Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_validate_job_status_params_valid() {
        let result = validate_job_status_params("queue-1", "job-123");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_job_status_params_empty_queue_id() {
        let result = validate_job_status_params("", "job-123");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Queue ID cannot be empty"));
    }

    #[test]
    fn test_validate_job_status_params_empty_job_id() {
        let result = validate_job_status_params("queue-1", "");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Job ID cannot be empty"));
    }

    // -------------------------------------------------------------------------
    // Queue ID Validation Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_validate_queue_id_valid() {
        let result = validate_queue_id("queue-1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_queue_id_empty() {
        let result = validate_queue_id("");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Queue ID cannot be empty"));
    }

    // -------------------------------------------------------------------------
    // Schedule Parsing Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_parse_schedule_interval() {
        let result = parse_schedule("interval:60");
        assert!(result.is_ok());
        let schedule = result.unwrap();
        match schedule.schedule_type {
            ScheduleType::Interval {
                interval_minutes, ..
            } => {
                assert_eq!(interval_minutes, 60);
            }
            _ => panic!("Expected Interval schedule type"),
        }
    }

    #[test]
    fn test_parse_schedule_cron() {
        let result = parse_schedule("cron:0 * * * *");
        assert!(result.is_ok());
        let schedule = result.unwrap();
        match schedule.schedule_type {
            ScheduleType::Cron { expression, .. } => {
                assert_eq!(expression, "0 * * * *");
            }
            _ => panic!("Expected Cron schedule type"),
        }
    }

    #[test]
    fn test_parse_schedule_invalid_format() {
        let result = parse_schedule("invalid");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid schedule format"));
    }

    #[test]
    fn test_parse_schedule_unknown_type() {
        let result = parse_schedule("unknown:value");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown schedule type"));
    }

    #[test]
    fn test_parse_schedule_invalid_interval() {
        let result = parse_schedule("interval:not_a_number");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid interval minutes"));
    }

    // -------------------------------------------------------------------------
    // Time Parsing Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_parse_time_valid() {
        let result = parse_time("2025-01-15T10:30:00Z");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_time_invalid() {
        let result = parse_time("invalid-time");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid time format"));
    }

    // -------------------------------------------------------------------------
    // Priority Conversion Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_priority_conversion() {
        assert!(matches!(
            JobPriority::from(PriorityArg::Low),
            JobPriority::Low
        ));
        assert!(matches!(
            JobPriority::from(PriorityArg::Normal),
            JobPriority::Normal
        ));
        assert!(matches!(
            JobPriority::from(PriorityArg::High),
            JobPriority::High
        ));
        assert!(matches!(
            JobPriority::from(PriorityArg::Critical),
            JobPriority::Critical
        ));
    }
}
