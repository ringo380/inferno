//! GPU Command v2 Example
//!
//! Demonstrates the new CLI architecture for the gpu command.
//! Shows GPU management operations including listing, info, allocation, and health checks.
//!
//! Run with: cargo run --example gpu_v2_example

use anyhow::Result;
use inferno::cli::gpu_v2::{GpuAllocate, GpuHealth, GpuInfo, GpuList};
use inferno::config::Config;
use inferno::core::config::ConfigBuilder;
use inferno::gpu::GpuVendor;
use inferno::interfaces::cli::{
    CommandContext, CommandPipeline, LoggingMiddleware, MetricsMiddleware,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🔥 Inferno GPU Command v2 Examples\n");

    // Create configuration
    let config = Config::default();
    let _core_config = ConfigBuilder::new().build_unchecked();

    // Build command pipeline with middleware
    let pipeline = CommandPipeline::builder()
        .with_middleware(Box::new(LoggingMiddleware::new()))
        .with_middleware(Box::new(MetricsMiddleware::new()))
        .build();

    // ========================================================================
    // Example 1: List all GPUs
    // ========================================================================
    println!("Example 1: List All GPUs");
    println!("{}", "─".repeat(80));
    println!("Note: This example requires GPU hardware to show actual results.");
    println!("Usage example:");
    println!("  let list_cmd = GpuList::new(");
    println!("      config.clone(),");
    println!("      false,   // detailed mode");
    println!("      None,    // no vendor filter");
    println!("  );");
    println!();
    println!("Expected output format (table view):");
    println!("  ID   Name                 Vendor       Memory       Util%      Status");
    println!("  ------------------------------------------------------------------------");
    println!("  0    NVIDIA RTX 4090      Nvidia       24576MB      15.2%      Available");
    println!("  1    AMD RX 7900 XTX      Amd          24576MB      0.0%       Available");

    println!("\n");

    // ========================================================================
    // Example 2: List GPUs with detailed information
    // ========================================================================
    println!("Example 2: Detailed GPU Listing");
    println!("{}", "─".repeat(80));

    let list_detailed = GpuList::new(config.clone(), true, None);
    let mut ctx_list = CommandContext::new(config.clone());

    println!("Running: GpuList with detailed=true...");
    match pipeline
        .execute(Box::new(list_detailed), &mut ctx_list)
        .await
    {
        Ok(output) => {
            println!("✓ {}", output.message);
            if let Some(data) = output.data {
                if let Some(count) = data["count"].as_u64() {
                    println!("  Found {} GPUs", count);
                }
            }
        }
        Err(e) => {
            println!("Note: {}", e);
            println!("This is expected if no GPUs are available on this system.");
        }
    }

    println!("\n");

    // ========================================================================
    // Example 3: Filter GPUs by vendor
    // ========================================================================
    println!("Example 3: Filter GPUs by Vendor");
    println!("{}", "─".repeat(80));
    println!("Usage examples:");
    println!();
    println!("Filter for NVIDIA GPUs:");
    println!("  GpuList::new(config, false, Some(GpuVendor::Nvidia))");
    println!();
    println!("Filter for AMD GPUs:");
    println!("  GpuList::new(config, false, Some(GpuVendor::Amd))");
    println!();
    println!("Filter for Intel GPUs:");
    println!("  GpuList::new(config, false, Some(GpuVendor::Intel))");

    println!("\n");

    // ========================================================================
    // Example 4: Get detailed GPU information
    // ========================================================================
    println!("Example 4: Get GPU Information");
    println!("{}", "─".repeat(80));
    println!("Usage example:");
    println!("  let info_cmd = GpuInfo::new(");
    println!("      config,");
    println!("      0,       // GPU ID");
    println!("      true,    // include metrics");
    println!("  );");
    println!();
    println!("Expected output:");
    println!("  GPU 0 Information:");
    println!("  ----------------------------------------");
    println!("  Name: NVIDIA GeForce RTX 4090");
    println!("  Vendor: Nvidia");
    println!("  Architecture: Ada Lovelace");
    println!("  Driver Version: 535.154.05");
    println!("  CUDA Version: 12.2");
    println!("  Memory: 24576 MB total, 23552 MB free, 1024 MB used");
    println!("  Utilization: 15.2%");
    println!("  Temperature: 45.0°C");
    println!("  Power Usage: 120.5W");
    println!("  Status: Available");

    println!("\n");

    // ========================================================================
    // Example 5: JSON output mode
    // ========================================================================
    println!("Example 5: JSON Output Mode");
    println!("{}", "─".repeat(80));
    println!("When run with json_output = true, returns structured data:");
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "count": 2,
            "gpus": [
                {
                    "id": 0,
                    "name": "NVIDIA GeForce RTX 4090",
                    "vendor": "Nvidia",
                    "architecture": "Ada Lovelace",
                    "driver_version": "535.154.05",
                    "cuda_version": "12.2",
                    "memory_total_mb": 24576,
                    "memory_free_mb": 23552,
                    "memory_used_mb": 1024,
                    "utilization_percent": 15.2,
                    "temperature_celsius": 45.0,
                    "power_usage_watts": 120.5,
                    "status": "Available"
                }
            ]
        }))?
    );

    println!("\n");

    // ========================================================================
    // Example 6: Allocate GPU memory
    // ========================================================================
    println!("Example 6: Allocate GPU Memory");
    println!("{}", "─".repeat(80));
    println!("Usage examples:");
    println!();
    println!("Auto-select best GPU:");
    println!("  GpuAllocate::new(");
    println!("      config,");
    println!("      8192,                    // 8GB memory");
    println!("      \"llama-2-7b\".to_string(), // model name");
    println!("      None,                    // auto-select GPU");
    println!("      None,                    // any vendor");
    println!("  )");
    println!();
    println!("Allocate specific GPU:");
    println!("  GpuAllocate::new(");
    println!("      config,");
    println!("      8192,                    // 8GB memory");
    println!("      \"llama-2-7b\".to_string(),");
    println!("      Some(0),                 // GPU 0");
    println!("      None,");
    println!("  )");
    println!();
    println!("Prefer specific vendor:");
    println!("  GpuAllocate::new(");
    println!("      config,");
    println!("      8192,");
    println!("      \"llama-2-7b\".to_string(),");
    println!("      None,");
    println!("      Some(GpuVendor::Nvidia), // prefer NVIDIA");
    println!("  )");

    println!("\n");

    // ========================================================================
    // Example 7: Validation examples
    // ========================================================================
    println!("Example 7: Input Validation");
    println!("{}", "─".repeat(80));

    // Test with zero memory
    let zero_memory = GpuAllocate::new(
        config.clone(),
        0, // Invalid - should fail
        "test-model".to_string(),
        None,
        None,
    );
    let ctx_zero = CommandContext::new(config.clone());

    match pipeline
        .execute(Box::new(zero_memory), &mut ctx_zero.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation correctly caught zero memory allocation:");
            println!("  {}", e);
        }
    }

    println!();

    // Test with empty model name
    let empty_model = GpuAllocate::new(
        config.clone(),
        1000,
        "".to_string(), // Invalid - should fail
        None,
        None,
    );

    match pipeline
        .execute(Box::new(empty_model), &mut ctx_zero.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation correctly caught empty model name:");
            println!("  {}", e);
        }
    }

    println!();

    // Test with excessive memory
    let too_much_memory = GpuAllocate::new(
        config.clone(),
        200_000, // Invalid - exceeds 100GB limit
        "test-model".to_string(),
        None,
        None,
    );

    match pipeline
        .execute(Box::new(too_much_memory), &mut ctx_zero.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation correctly caught excessive memory:");
            println!("  {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Example 8: Check GPU health
    // ========================================================================
    println!("Example 8: Check GPU Health");
    println!("{}", "─".repeat(80));

    let health_all = GpuHealth::new(config.clone(), None);
    let mut ctx_health = CommandContext::new(config.clone());

    println!("Running: GpuHealth for all GPUs...");
    match pipeline
        .execute(Box::new(health_all), &mut ctx_health)
        .await
    {
        Ok(output) => {
            println!("✓ {}", output.message);
            if let Some(data) = output.data {
                if let Some(count) = data["count"].as_u64() {
                    println!("  Checked {} GPUs", count);
                }
            }
        }
        Err(e) => {
            println!("Note: {}", e);
            println!("This is expected if no GPUs are available on this system.");
        }
    }

    println!("\n");

    // ========================================================================
    // Example 9: Check specific GPU health
    // ========================================================================
    println!("Example 9: Check Specific GPU Health");
    println!("{}", "─".repeat(80));
    println!("Usage example:");
    println!("  GpuHealth::new(config, Some(0))  // Check GPU 0 only");
    println!();
    println!("Expected output:");
    println!("  GPU  Health Status");
    println!("  -------------------------");
    println!("  0    Healthy");

    println!("\n");

    // ========================================================================
    // Example 10: Real allocation attempt (if GPU available)
    // ========================================================================
    println!("Example 10: Actual GPU Allocation Attempt");
    println!("{}", "─".repeat(80));

    let allocate_cmd = GpuAllocate::new(
        config.clone(),
        1024, // 1GB
        "example-model".to_string(),
        None,
        None,
    );
    let mut ctx_allocate = CommandContext::new(config.clone());

    println!("Attempting to allocate 1GB for 'example-model'...");
    match pipeline
        .execute(Box::new(allocate_cmd), &mut ctx_allocate)
        .await
    {
        Ok(output) => {
            println!("✓ {}", output.message);
            if let Some(data) = output.data {
                if let Some(gpu_id) = data["gpu_id"].as_u64() {
                    println!("  Allocated GPU: {}", gpu_id);
                }
            }
        }
        Err(e) => {
            println!("Note: Could not allocate GPU");
            println!("  {}", e);
            println!("This is expected if no GPUs are available or insufficient free memory.");
        }
    }

    println!("\n");

    // ========================================================================
    // Summary
    // ========================================================================
    println!("{}", "═".repeat(80));
    println!("Summary: GPU Command v2 Features");
    println!("{}", "═".repeat(80));
    println!("✓ List available GPUs (basic and detailed views)");
    println!("✓ Filter GPUs by vendor (NVIDIA, AMD, Intel)");
    println!("✓ Get detailed GPU information with metrics");
    println!("✓ Allocate GPU memory for models");
    println!("✓ Auto-select best available GPU");
    println!("✓ Allocate specific GPU by ID");
    println!("✓ Vendor preference for allocation");
    println!("✓ Check GPU health status");
    println!("✓ Comprehensive input validation");
    println!("✓ Structured JSON output");
    println!("✓ Human-readable table output");
    println!("✓ Middleware support (logging, metrics)");
    println!();
    println!("Validation Checks:");
    println!("  - Memory allocation must be > 0 MB");
    println!("  - Memory allocation must be ≤ 100,000 MB (100 GB)");
    println!("  - Model name cannot be empty");
    println!("  - GPU ID validated during execution");
    println!();
    println!("Use Cases:");
    println!("  - Discover available GPU hardware");
    println!("  - Monitor GPU utilization and health");
    println!("  - Allocate GPU resources for models");
    println!("  - Select optimal GPU for workload");
    println!("  - Filter by GPU vendor preference");
    println!("  - Get detailed GPU specifications");
    println!();
    println!("Note: This is a focused migration covering core GPU operations.");
    println!("Full GPU management functionality remains available in the original module.");

    Ok(())
}
