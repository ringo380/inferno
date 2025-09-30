//! Package Command v2 Example
//!
//! Demonstrates model package management including install, remove, search,
//! and maintenance operations.
//!
//! Run with: cargo run --example package_v2_example

use anyhow::Result;
use inferno::cli::package_v2::*;
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

    println!("🔥 Inferno Package Command v2 Examples\n");

    // Create configuration
    let config = Config::default();

    // Build command pipeline with middleware
    let pipeline = CommandPipeline::builder()
        .with_middleware(Box::new(LoggingMiddleware::new()))
        .with_middleware(Box::new(MetricsMiddleware::new()))
        .build();

    // ========================================================================
    // Example 1: Install a package
    // ========================================================================
    println!("Example 1: Install a Package");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let install = PackageInstall::new(");
    println!("      config.clone(),");
    println!("      \"llama-2-7b\".to_string(),");
    println!("      false,    // resolve dependencies");
    println!("      None,     // default target");
    println!("      false,    // prompt for confirmation");
    println!("      false,    // no auto-update");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Installing Package ===");
    println!("  Package: llama-2-7b");
    println!("  Auto-update: false");
    println!("  Resolve Dependencies: true");
    println!("  ");
    println!("  ✓ Package installed successfully");
    println!("  ");
    println!("  Installed:");
    println!("    - llama-2-7b v1.0.0");
    println!("  ");
    println!("  Dependencies:");
    println!("    - tokenizer v0.5.0");
    println!("    - sentencepiece v0.3.2");

    println!("\n");

    // ========================================================================
    // Example 2: Install with auto-update
    // ========================================================================
    println!("Example 2: Install with Auto-Update");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let install = PackageInstall::new(");
    println!("      config.clone(),");
    println!("      \"mistral-7b\".to_string(),");
    println!("      false,");
    println!("      Some(PathBuf::from(\"/models\")),");
    println!("      true,     // auto-confirm");
    println!("      true,     // enable auto-update");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Installing Package ===");
    println!("  Package: mistral-7b");
    println!("  Target: \"/models\"");
    println!("  Auto-update: true");
    println!("  Resolve Dependencies: true");
    println!("  ");
    println!("  ✓ Package installed successfully");
    println!("  ");
    println!("  Installed:");
    println!("    - mistral-7b v1.0.0");
    println!("  ");
    println!("  Dependencies:");
    println!("    - tokenizer v0.5.0");
    println!("    - sentencepiece v0.3.2");

    println!("\n");

    // ========================================================================
    // Example 3: Install without dependencies
    // ========================================================================
    println!("Example 3: Install Without Dependencies");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let install = PackageInstall::new(");
    println!("      config.clone(),");
    println!("      \"gpt-neo-1.3b\".to_string(),");
    println!("      true,     // no dependencies");
    println!("      None,");
    println!("      true,");
    println!("      false,");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Installing Package ===");
    println!("  Package: gpt-neo-1.3b");
    println!("  Auto-update: false");
    println!("  Resolve Dependencies: false");
    println!("  ");
    println!("  ✓ Package installed successfully");
    println!("  ");
    println!("  Installed:");
    println!("    - gpt-neo-1.3b v1.0.0");

    println!("\n");

    // ========================================================================
    // Example 4: Search for packages
    // ========================================================================
    println!("Example 4: Search for Packages");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let search = PackageSearch::new(");
    println!("      config.clone(),");
    println!("      \"llama\".to_string(),");
    println!("      None,     // all repositories");
    println!("      20,       // limit");
    println!("      false,    // not detailed");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Searching Packages ===");
    println!("  Query: llama");
    println!("  Limit: 20");
    println!("  ");
    println!("  Package: llama-2-7b");
    println!("    Version: 1.0.0");
    println!("    Repository: huggingface");
    println!("  ");
    println!("  Package: mistral-7b");
    println!("    Version: 0.1.0");
    println!("    Repository: mistralai");
    println!("  ");
    println!("  Total Results: 2");

    println!("\n");

    // ========================================================================
    // Example 5: Detailed search
    // ========================================================================
    println!("Example 5: Detailed Search Results");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let search = PackageSearch::new(");
    println!("      config.clone(),");
    println!("      \"mistral\".to_string(),");
    println!("      Some(\"mistralai\".to_string()),");
    println!("      10,");
    println!("      true,     // detailed");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Searching Packages ===");
    println!("  Query: mistral");
    println!("  Repository: mistralai");
    println!("  Limit: 10");
    println!("  ");
    println!("  Package: llama-2-7b");
    println!("    Version: 1.0.0");
    println!("    Repository: huggingface");
    println!("    Description: LLaMA 2 7B parameter model");
    println!("    Size: 13.5 GB");
    println!("    License: Meta AI");
    println!("  ");
    println!("  Package: mistral-7b");
    println!("    Version: 0.1.0");
    println!("    Repository: mistralai");
    println!("    Description: Mistral 7B parameter model");
    println!("    Size: 14.2 GB");
    println!("    License: Apache 2.0");
    println!("  ");
    println!("  Total Results: 2");

    println!("\n");

    // ========================================================================
    // Example 6: Show package info
    // ========================================================================
    println!("Example 6: Show Package Information");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let info = PackageInfo::new(");
    println!("      config.clone(),");
    println!("      \"llama-2-7b\".to_string(),");
    println!("      false,    // no dependencies");
    println!("      false,    // not detailed");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Package Information ===");
    println!("  Package: llama-2-7b");
    println!("  Version: 1.0.0");
    println!("  Repository: huggingface");
    println!("  Status: Installed");

    println!("\n");

    // ========================================================================
    // Example 7: Detailed package info
    // ========================================================================
    println!("Example 7: Detailed Package Information");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let info = PackageInfo::new(");
    println!("      config.clone(),");
    println!("      \"llama-2-7b\".to_string(),");
    println!("      true,     // show dependencies");
    println!("      true,     // detailed");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Package Information ===");
    println!("  Package: llama-2-7b");
    println!("  Version: 1.0.0");
    println!("  Repository: huggingface");
    println!("  Status: Installed");
    println!("  ");
    println!("  Detailed Information:");
    println!("    Description: LLaMA 2 7B parameter model");
    println!("    Size: 13.5 GB");
    println!("    License: Meta AI");
    println!("    Install Date: 2025-09-29");
    println!("    Auto-update: Enabled");
    println!("  ");
    println!("  Dependencies:");
    println!("    - tokenizer v0.5.0");
    println!("    - sentencepiece v0.3.2");

    println!("\n");

    // ========================================================================
    // Example 8: List installed packages
    // ========================================================================
    println!("Example 8: List Installed Packages");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let list = PackageList::new(");
    println!("      config.clone(),");
    println!("      None,     // no filter");
    println!("      false,    // not detailed");
    println!("      false,    // all packages");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Installed Packages ===");
    println!("  ");
    println!("  Package: llama-2-7b");
    println!("    Version: 1.0.0");
    println!("    Status: Installed");
    println!("  ");
    println!("  Package: tokenizer");
    println!("    Version: 0.5.0");
    println!("    Status: Installed");
    println!("  ");
    println!("  Total Packages: 2");

    println!("\n");

    // ========================================================================
    // Example 9: List with filter
    // ========================================================================
    println!("Example 9: List Packages with Filter");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let list = PackageList::new(");
    println!("      config.clone(),");
    println!("      Some(\"llama\".to_string()),");
    println!("      true,     // detailed");
    println!("      false,");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Installed Packages ===");
    println!("  Filter: llama");
    println!("  ");
    println!("  Package: llama-2-7b");
    println!("    Version: 1.0.0");
    println!("    Status: Installed");
    println!("    Size: 13.5 GB");
    println!("    Install Date: 2025-09-29");
    println!("    Auto-installed: No");
    println!("  ");
    println!("  Package: tokenizer");
    println!("    Version: 0.5.0");
    println!("    Status: Installed");
    println!("    Size: 125 MB");
    println!("    Install Date: 2025-09-29");
    println!("    Auto-installed: Yes");
    println!("  ");
    println!("  Total Packages: 2");

    println!("\n");

    // ========================================================================
    // Example 10: List auto-installed only
    // ========================================================================
    println!("Example 10: List Auto-Installed Packages");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let list = PackageList::new(");
    println!("      config.clone(),");
    println!("      None,");
    println!("      false,");
    println!("      true,     // auto-installed only");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Installed Packages ===");
    println!("  Showing: Auto-installed packages only");
    println!("  ");
    println!("  Package: tokenizer");
    println!("    Version: 0.5.0");
    println!("    Status: Installed");
    println!("  ");
    println!("  Total Packages: 1");

    println!("\n");

    // ========================================================================
    // Example 11: Check for updates
    // ========================================================================
    println!("Example 11: Check for Updates");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let update = PackageUpdate::new(");
    println!("      config.clone(),");
    println!("      None,     // all packages");
    println!("      false,");
    println!("      true,     // check only");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Updating Packages ===");
    println!("  Updating: All packages");
    println!("  Check Only: true");
    println!("  ");
    println!("  Available Updates:");
    println!("    - llama-2-7b: 1.0.0 → 1.1.0");
    println!("    - tokenizer: 0.5.0 → 0.5.1");
    println!("  ");
    println!("  Total Updates Available: 2");

    println!("\n");

    // ========================================================================
    // Example 12: Update all packages
    // ========================================================================
    println!("Example 12: Update All Packages");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let update = PackageUpdate::new(");
    println!("      config.clone(),");
    println!("      None,");
    println!("      true,     // auto-confirm");
    println!("      false,    // perform update");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Updating Packages ===");
    println!("  Updating: All packages");
    println!("  Check Only: false");
    println!("  ");
    println!("  Updating packages...");
    println!("  ");
    println!("  ✓ llama-2-7b updated: 1.0.0 → 1.1.0");
    println!("  ✓ tokenizer updated: 0.5.0 → 0.5.1");
    println!("  ");
    println!("  ✓ All packages updated successfully");

    println!("\n");

    // ========================================================================
    // Example 13: Update specific package
    // ========================================================================
    println!("Example 13: Update Specific Package");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let update = PackageUpdate::new(");
    println!("      config.clone(),");
    println!("      Some(\"llama-2-7b\".to_string()),");
    println!("      true,");
    println!("      false,");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Updating Packages ===");
    println!("  Package: llama-2-7b");
    println!("  Check Only: false");
    println!("  ");
    println!("  Updating packages...");
    println!("  ");
    println!("  ✓ llama-2-7b updated: 1.0.0 → 1.1.0");
    println!("  ✓ tokenizer updated: 0.5.0 → 0.5.1");
    println!("  ");
    println!("  ✓ All packages updated successfully");

    println!("\n");

    // ========================================================================
    // Example 14: Remove package
    // ========================================================================
    println!("Example 14: Remove Package");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let remove = PackageRemove::new(");
    println!("      config.clone(),");
    println!("      \"mistral-7b\".to_string(),");
    println!("      false,    // remove dependencies");
    println!("      true,     // auto-confirm");
    println!("      false,    // don't keep config");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Removing Package ===");
    println!("  Package: mistral-7b");
    println!("  Remove Dependencies: true");
    println!("  Keep Configuration: false");
    println!("  ");
    println!("  ✓ Package removed successfully");
    println!("  ");
    println!("  Removed:");
    println!("    - mistral-7b");
    println!("  ");
    println!("  Dependencies also removed:");
    println!("    - tokenizer v0.5.0");

    println!("\n");

    // ========================================================================
    // Example 15: Remove with config preservation
    // ========================================================================
    println!("Example 15: Remove with Config Preservation");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let remove = PackageRemove::new(");
    println!("      config.clone(),");
    println!("      \"gpt-neo-1.3b\".to_string(),");
    println!("      true,     // keep dependencies");
    println!("      true,");
    println!("      true,     // keep config");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Removing Package ===");
    println!("  Package: gpt-neo-1.3b");
    println!("  Remove Dependencies: false");
    println!("  Keep Configuration: true");
    println!("  ");
    println!("  ✓ Package removed successfully");
    println!("  ");
    println!("  Removed:");
    println!("    - gpt-neo-1.3b");

    println!("\n");

    // ========================================================================
    // Example 16: Clean cache (dry run)
    // ========================================================================
    println!("Example 16: Clean Cache (Dry Run)");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let clean = PackageClean::new(");
    println!("      config.clone(),");
    println!("      false,    // standard clean");
    println!("      true,     // dry run");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Cleaning Package Cache ===");
    println!("  Mode: Standard");
    println!("  Dry Run: true");
    println!("  ");
    println!("  Would clean:");
    println!("    - Download cache: 2.5 GB");
    println!("    - Temporary files: 145 MB");
    println!("  ");
    println!("  Total space to be freed: 2.6 GB");

    println!("\n");

    // ========================================================================
    // Example 17: Full cache clean
    // ========================================================================
    println!("Example 17: Full Cache Clean");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let clean = PackageClean::new(");
    println!("      config.clone(),");
    println!("      true,     // full clean");
    println!("      false,    // execute");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Cleaning Package Cache ===");
    println!("  Mode: Full clean");
    println!("  Dry Run: false");
    println!("  ");
    println!("  Cleaning...");
    println!("  ✓ Download cache cleaned: 2.5 GB");
    println!("  ✓ Temporary files removed: 145 MB");
    println!("  ✓ Old versions removed: 8.2 GB");
    println!("  ");
    println!("  ✓ Total space freed: 10.8 GB");

    println!("\n");

    // ========================================================================
    // Example 18: Validation tests
    // ========================================================================
    println!("Example 18: Input Validation");
    println!("{}", "─".repeat(80));

    let empty_package = PackageInstall::new(
        config.clone(),
        "".to_string(),
        false,
        None,
        false,
        false,
    );
    let ctx_invalid = CommandContext::new(config.clone());

    match pipeline
        .execute(Box::new(empty_package), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught empty package name:");
            println!("  {}", e);
        }
    }

    println!();

    let invalid_limit = PackageSearch::new(
        config.clone(),
        "llama".to_string(),
        None,
        150,
        false,
    );

    match pipeline
        .execute(Box::new(invalid_limit), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught excessive limit:");
            println!("  {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Summary
    // ========================================================================
    println!("{}", "═".repeat(80));
    println!("Summary: Package Command v2 Features");
    println!("{}", "═".repeat(80));
    println!("✓ Package installation with dependency resolution");
    println!("✓ Package removal with cleanup options");
    println!("✓ Package search with filtering");
    println!("✓ Detailed package information");
    println!("✓ Installed package listing");
    println!("✓ Update checking and execution");
    println!("✓ Cache cleaning (dry run and full)");
    println!("✓ Auto-update support");
    println!("✓ Configuration preservation");
    println!("✓ Comprehensive validation");
    println!("✓ Structured JSON output");
    println!("✓ Middleware support");
    println!();
    println!("Validation:");
    println!("  - Package name cannot be empty");
    println!("  - Search limit: 1-100");
    println!("  - Target directory must exist");
    println!();
    println!("Use Cases:");
    println!("  - Model package management");
    println!("  - Dependency resolution");
    println!("  - Repository searches");
    println!("  - Version updates and maintenance");

    Ok(())
}