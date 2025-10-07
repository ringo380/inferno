#![allow(dead_code, unused_imports, unused_variables)]
//! GPU Command - New Architecture
//!
//! This module demonstrates the migration of the gpu command to the new
//! CLI architecture. Focuses on core GPU management operations.
//!
//! Note: This is a focused migration covering the most commonly used subcommands.
//! Full GPU management functionality remains available through the original module.

use crate::config::Config;
use crate::gpu::{GpuConfiguration, GpuManager, GpuVendor};
use crate::interfaces::cli::{Command, CommandContext, CommandOutput};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use tracing::info;

// ============================================================================
// GpuList - List available GPUs
// ============================================================================

/// List available GPUs with optional filtering
pub struct GpuList {
    config: Config,
    detailed: bool,
    vendor: Option<GpuVendor>,
}

impl GpuList {
    pub fn new(config: Config, detailed: bool, vendor: Option<GpuVendor>) -> Self {
        Self {
            config,
            detailed,
            vendor,
        }
    }
}

#[async_trait]
impl Command for GpuList {
    fn name(&self) -> &str {
        "gpu list"
    }

    fn description(&self) -> &str {
        "List available GPUs"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        // No validation needed for list
        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Listing available GPUs");

        let gpu_config = GpuConfiguration::default();
        let manager = GpuManager::new(gpu_config);
        manager.initialize().await?;

        let mut available_gpus = manager.get_available_gpus().await;

        // Filter by vendor if specified
        if let Some(ref vendor) = self.vendor {
            available_gpus.retain(|gpu| {
                std::mem::discriminant(&gpu.vendor) == std::mem::discriminant(vendor)
            });
        }

        // Human-readable output
        if !ctx.json_output {
            if available_gpus.is_empty() {
                println!("No GPUs found matching the criteria");
            } else if self.detailed {
                for gpu in &available_gpus {
                    println!("GPU {}: {}", gpu.id, gpu.name);
                    println!("  Vendor: {:?}", gpu.vendor);
                    println!("  Driver: {}", gpu.driver_version);
                    println!(
                        "  Memory: {} MB total, {} MB free",
                        gpu.memory_total_mb, gpu.memory_free_mb
                    );
                    println!("  Utilization: {:.1}%", gpu.utilization_percent);
                    if let Some(temp) = gpu.temperature_celsius {
                        println!("  Temperature: {:.1}°C", temp);
                    }
                    println!("  Status: {:?}", gpu.status);
                    println!();
                }
            } else {
                println!(
                    "{:<4} {:<20} {:<12} {:<12} {:<10} {:<12}",
                    "ID", "Name", "Vendor", "Memory", "Util%", "Status"
                );
                println!("{:-<80}", "");
                for gpu in &available_gpus {
                    println!(
                        "{:<4} {:<20} {:<12} {:<12} {:<10} {:<12}",
                        gpu.id,
                        &gpu.name[..gpu.name.len().min(20)],
                        format!("{:?}", gpu.vendor),
                        format!("{}MB", gpu.memory_total_mb),
                        format!("{:.1}%", gpu.utilization_percent),
                        format!("{:?}", gpu.status)
                    );
                }
            }
        }

        // Structured output
        let gpus_json = serde_json::to_value(&available_gpus)?;

        Ok(CommandOutput::success_with_data(
            format!("Found {} GPUs", available_gpus.len()),
            json!({
                "count": available_gpus.len(),
                "gpus": gpus_json,
            }),
        ))
    }
}

// ============================================================================
// GpuInfo - Show detailed GPU information
// ============================================================================

/// Show detailed information about a specific GPU
pub struct GpuInfo {
    config: Config,
    gpu_id: u32,
    include_metrics: bool,
}

impl GpuInfo {
    pub fn new(config: Config, gpu_id: u32, include_metrics: bool) -> Self {
        Self {
            config,
            gpu_id,
            include_metrics,
        }
    }
}

#[async_trait]
impl Command for GpuInfo {
    fn name(&self) -> &str {
        "gpu info"
    }

    fn description(&self) -> &str {
        "Show detailed GPU information"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        // GPU ID will be validated during execution
        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Getting GPU {} information", self.gpu_id);

        let gpu_config = GpuConfiguration::default();
        let manager = GpuManager::new(gpu_config);
        manager.initialize().await?;

        let gpu = manager
            .get_gpu_info(self.gpu_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("GPU {} not found", self.gpu_id))?;

        // Get metrics if requested
        let metrics = if self.include_metrics {
            Some(manager.get_gpu_metrics(Some(self.gpu_id)).await)
        } else {
            None
        };

        // Human-readable output
        if !ctx.json_output {
            println!("GPU {} Information:", gpu.id);
            println!("{:-<40}", "");
            println!("Name: {}", gpu.name);
            println!("Vendor: {:?}", gpu.vendor);
            println!("Architecture: {}", gpu.architecture);
            println!("Driver Version: {}", gpu.driver_version);

            if let Some(ref cuda_version) = gpu.cuda_version {
                println!("CUDA Version: {}", cuda_version);
            }

            println!(
                "Memory: {} MB total, {} MB free, {} MB used",
                gpu.memory_total_mb, gpu.memory_free_mb, gpu.memory_used_mb
            );
            println!("Utilization: {:.1}%", gpu.utilization_percent);

            if let Some(temp) = gpu.temperature_celsius {
                println!("Temperature: {:.1}°C", temp);
            }

            if let Some(power) = gpu.power_usage_watts {
                println!("Power Usage: {:.1}W", power);
            }

            println!("Status: {:?}", gpu.status);

            if let Some(ref metrics_list) = metrics {
                if !metrics_list.is_empty() {
                    println!("\nRecent Metrics:");
                    println!("  Samples: {}", metrics_list.len());
                }
            }
        }

        // Structured output
        Ok(CommandOutput::success_with_data(
            format!("GPU {} information retrieved", self.gpu_id),
            json!({
                "gpu": serde_json::to_value(&gpu)?,
                "metrics": metrics.as_ref().and_then(|m| serde_json::to_value(m).ok()),
            }),
        ))
    }
}

// ============================================================================
// GpuAllocate - Allocate GPU memory for a model
// ============================================================================

/// Allocate GPU memory for a model
pub struct GpuAllocate {
    config: Config,
    memory_mb: u64,
    model_name: String,
    gpu_id: Option<u32>,
    vendor: Option<GpuVendor>,
}

impl GpuAllocate {
    pub fn new(
        config: Config,
        memory_mb: u64,
        model_name: String,
        gpu_id: Option<u32>,
        vendor: Option<GpuVendor>,
    ) -> Self {
        Self {
            config,
            memory_mb,
            model_name,
            gpu_id,
            vendor,
        }
    }
}

#[async_trait]
impl Command for GpuAllocate {
    fn name(&self) -> &str {
        "gpu allocate"
    }

    fn description(&self) -> &str {
        "Allocate GPU memory for a model"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        if self.memory_mb == 0 {
            anyhow::bail!("Memory allocation must be greater than 0 MB");
        }

        if self.memory_mb > 100_000 {
            anyhow::bail!("Memory allocation cannot exceed 100,000 MB (100 GB)");
        }

        if self.model_name.is_empty() {
            anyhow::bail!("Model name cannot be empty");
        }

        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!(
            "Allocating {} MB for model '{}'",
            self.memory_mb, self.model_name
        );

        let gpu_config = GpuConfiguration::default();
        let manager = GpuManager::new(gpu_config);
        manager.initialize().await?;

        let allocated_gpu = if let Some(id) = self.gpu_id {
            // Allocate specific GPU
            if !ctx.json_output {
                println!(
                    "Allocating {}MB on GPU {} for model '{}'...",
                    self.memory_mb, id, self.model_name
                );
            }

            if manager
                .allocate_specific_gpu(id, self.memory_mb, self.model_name.clone())
                .await?
            {
                if !ctx.json_output {
                    println!("✓ Successfully allocated GPU {}", id);
                }
                Some(id)
            } else {
                anyhow::bail!(
                    "Failed to allocate GPU {} (insufficient memory or unavailable)",
                    id
                );
            }
        } else {
            // Auto-select best GPU
            if !ctx.json_output {
                println!(
                    "Allocating {}MB for model '{}'...",
                    self.memory_mb, self.model_name
                );
            }

            manager
                .allocate_gpu(self.memory_mb, self.model_name.clone(), self.vendor.clone())
                .await?
        };

        let allocated_gpu_id =
            allocated_gpu.ok_or_else(|| anyhow::anyhow!("No suitable GPU found for allocation"))?;

        // Structured output
        Ok(CommandOutput::success_with_data(
            format!("Successfully allocated GPU {}", allocated_gpu_id),
            json!({
                "gpu_id": allocated_gpu_id,
                "memory_mb": self.memory_mb,
                "model_name": self.model_name,
            }),
        ))
    }
}

// ============================================================================
// GpuHealth - Check GPU health status
// ============================================================================

/// Check GPU health status
pub struct GpuHealth {
    config: Config,
    gpu_id: Option<u32>,
}

impl GpuHealth {
    pub fn new(config: Config, gpu_id: Option<u32>) -> Self {
        Self { config, gpu_id }
    }
}

#[async_trait]
impl Command for GpuHealth {
    fn name(&self) -> &str {
        "gpu health"
    }

    fn description(&self) -> &str {
        "Check GPU health status"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Checking GPU health");

        let gpu_config = GpuConfiguration::default();
        let manager = GpuManager::new(gpu_config);
        manager.initialize().await?;

        let health_status = manager.check_gpu_health().await?;

        let filtered_status: Vec<_> = if let Some(id) = self.gpu_id {
            health_status
                .into_iter()
                .filter(|(gpu_id, _)| *gpu_id == id)
                .collect()
        } else {
            health_status.into_iter().collect()
        };

        // Human-readable output
        if !ctx.json_output {
            println!("{:<4} {:<15}", "GPU", "Health Status");
            println!("{:-<25}", "");
            for (gpu_id, status) in &filtered_status {
                println!("{:<4} {:<15}", gpu_id, format!("{:?}", status));
            }
        }

        // Structured output
        let status_map: std::collections::HashMap<u32, String> = filtered_status
            .iter()
            .map(|(id, status)| (*id, format!("{:?}", status)))
            .collect();

        Ok(CommandOutput::success_with_data(
            format!("Health check complete for {} GPUs", filtered_status.len()),
            json!({
                "count": filtered_status.len(),
                "health_status": status_map,
            }),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gpu_allocate_validation() {
        let config = Config::default();
        let cmd = GpuAllocate::new(config, 0, "test".to_string(), None, None);
        let ctx = CommandContext::new(Config::default());

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("greater than 0"));
    }

    #[tokio::test]
    async fn test_gpu_allocate_empty_model() {
        let config = Config::default();
        let cmd = GpuAllocate::new(config, 1000, "".to_string(), None, None);
        let ctx = CommandContext::new(Config::default());

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }
}
