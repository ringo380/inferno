//! Versioning Command v2 Example
//!
//! Demonstrates model versioning and rollback management.
//!
//! Run with: cargo run --example versioning_v2_example

use anyhow::Result;
use inferno::cli::versioning_v2::{
    VersionCompare, VersionCreate, VersionList, VersionPromote, VersionRollback,
};
use inferno::config::Config;
use inferno::interfaces::cli::{
    CommandContext, CommandPipeline, LoggingMiddleware, MetricsMiddleware,
};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🔥 Inferno Versioning Command v2 Examples\n");

    // Create configuration
    let config = Config::default();

    // Build command pipeline with middleware
    let pipeline = CommandPipeline::builder()
        .with_middleware(Box::new(LoggingMiddleware::new()))
        .with_middleware(Box::new(MetricsMiddleware::new()))
        .build();

    // ========================================================================
    // Example 1: List all model versions
    // ========================================================================
    println!("Example 1: List All Model Versions");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let list = VersionList::new(");
    println!("      config.clone(),");
    println!("      None,     // all models");
    println!("      false,    // not detailed");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Model Versions ===");
    println!("  No models found");

    println!("\n");

    // ========================================================================
    // Example 2: List versions for specific model
    // ========================================================================
    println!("Example 2: List Versions for Specific Model");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let list = VersionList::new(");
    println!("      config.clone(),");
    println!("      Some(\"llama-2-7b\".to_string()),");
    println!("      true,     // detailed");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Model Versions ===");
    println!("  Model: llama-2-7b");
    println!("  No versions found");

    println!("\n");

    // ========================================================================
    // Example 3: Create a new version
    // ========================================================================
    println!("Example 3: Create New Version");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let create = VersionCreate::new(");
    println!("      config.clone(),");
    println!("      \"llama-2-7b\".to_string(),");
    println!("      PathBuf::from(\"/models/llama-2-7b.gguf\"),");
    println!("      Some(\"v2.0.0\".to_string()),");
    println!("      Some(\"Production release\".to_string()),");
    println!("  );");
    println!();
    println!("Output:");
    println!("  ✓ Version creation requested");
    println!("    Model: llama-2-7b");
    println!("    File: /models/llama-2-7b.gguf");
    println!("    Version: v2.0.0");
    println!("    Description: Production release");

    println!("\n");

    // ========================================================================
    // Example 4: Promote version to production
    // ========================================================================
    println!("Example 4: Promote Version to Production");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let promote = VersionPromote::new(");
    println!("      config.clone(),");
    println!("      \"llama-2-7b\".to_string(),");
    println!("      \"v2.0.0\".to_string(),");
    println!("      \"production\".to_string(),");
    println!("  );");
    println!();
    println!("Output:");
    println!("  ✓ Version promotion requested");
    println!("    Model: llama-2-7b");
    println!("    Version: v2.0.0");
    println!("    Target Status: production");

    println!("\n");

    // ========================================================================
    // Example 5: Rollback to previous version
    // ========================================================================
    println!("Example 5: Rollback to Previous Version");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let rollback = VersionRollback::new(");
    println!("      config.clone(),");
    println!("      \"llama-2-7b\".to_string(),");
    println!("      \"v1.5.0\".to_string(),");
    println!("      Some(\"Performance regression in v2.0.0\".to_string()),");
    println!("  );");
    println!();
    println!("Output:");
    println!("  ✓ Version rollback requested");
    println!("    Model: llama-2-7b");
    println!("    Target Version: v1.5.0");
    println!("    Reason: Performance regression in v2.0.0");

    println!("\n");

    // ========================================================================
    // Example 6: Compare two versions
    // ========================================================================
    println!("Example 6: Compare Two Versions");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let compare = VersionCompare::new(");
    println!("      config.clone(),");
    println!("      \"llama-2-7b\".to_string(),");
    println!("      \"v1.5.0\".to_string(),");
    println!("      \"v2.0.0\".to_string(),");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Version Comparison ===");
    println!("    Model: llama-2-7b");
    println!("    Version A: v1.5.0");
    println!("    Version B: v2.0.0");

    println!("\n");

    // ========================================================================
    // Example 7: Validation tests
    // ========================================================================
    println!("Example 7: Input Validation");
    println!("{}", "─".repeat(80));

    let empty_name = VersionCreate::new(
        config.clone(),
        String::new(),
        PathBuf::from("/tmp/model.gguf"),
        None,
        None,
    );
    let ctx_invalid = CommandContext::new(config.clone());

    match pipeline
        .execute(Box::new(empty_name), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught empty model name:");
            println!("  {}", e);
        }
    }

    println!();

    let empty_version = VersionPromote::new(
        config.clone(),
        "test-model".to_string(),
        String::new(),
        "production".to_string(),
    );

    match pipeline
        .execute(Box::new(empty_version), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught empty version ID:");
            println!("  {}", e);
        }
    }

    println!();

    let same_versions = VersionCompare::new(
        config.clone(),
        "test-model".to_string(),
        "v1.0.0".to_string(),
        "v1.0.0".to_string(),
    );

    match pipeline
        .execute(Box::new(same_versions), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught identical versions:");
            println!("  {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Summary
    // ========================================================================
    println!("{}", "═".repeat(80));
    println!("Summary: Versioning Command v2 Features");
    println!("{}", "═".repeat(80));
    println!("✓ List all model versions or filter by model");
    println!("✓ Create new versions with semantic versioning");
    println!("✓ Promote versions through deployment stages");
    println!("✓ Rollback to previous versions with reason tracking");
    println!("✓ Compare versions side-by-side");
    println!("✓ Comprehensive validation");
    println!("✓ Structured JSON output");
    println!("✓ Middleware support");
    println!();
    println!("Validation:");
    println!("  - Model name not empty");
    println!("  - Version ID not empty");
    println!("  - Cannot compare version to itself");
    println!("  - Model file must exist for creation");
    println!();
    println!("Use Cases:");
    println!("  - Model lifecycle management");
    println!("  - A/B testing with version control");
    println!("  - Safe production deployments");
    println!("  - Quick rollback for incidents");

    Ok(())
}