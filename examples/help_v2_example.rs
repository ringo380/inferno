//! Help Command v2 Example
//!
//! Demonstrates user-friendly help and guidance utilities.
//!
//! Run with: cargo run --example help_v2_example

use anyhow::Result;
use inferno::cli::help_v2::{CheckPrerequisites, GetExamples, HandleError};
use inferno::config::Config;
use inferno::interfaces::cli::{
    CommandContext, CommandPipeline, LoggingMiddleware, MetricsMiddleware,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🔥 Inferno Help Command v2 Examples\n");

    // Create configuration
    let config = Config::default();

    // Build command pipeline with middleware
    let pipeline = CommandPipeline::builder()
        .with_middleware(Box::new(LoggingMiddleware::new()))
        .with_middleware(Box::new(MetricsMiddleware::new()))
        .build();

    // ========================================================================
    // Example 1: Get error guidance for file not found
    // ========================================================================
    println!("Example 1: Get Error Guidance (File Not Found)");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let error_help = HandleError::new(");
    println!("      config.clone(),");
    println!("      \"no such file or directory: model not found\".to_string(),");
    println!("  );");
    println!();
    println!("Output:");
    println!("  ❌ File or directory not found.");
    println!();
    println!("  💡 This usually means:");
    println!("     • No models directory has been configured");
    println!("     • The specified model file doesn't exist");
    println!();
    println!("  🔧 Try these solutions:");
    println!("     1. Check your models directory:");
    println!("        inferno models list");

    println!("\n");

    // ========================================================================
    // Example 2: Get error guidance for network error
    // ========================================================================
    println!("Example 2: Get Error Guidance (Network Error)");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let error_help = HandleError::new(");
    println!("      config.clone(),");
    println!("      \"network connection error\".to_string(),");
    println!("  );");
    println!();
    println!("Output:");
    println!("  ❌ Network connection error.");
    println!();
    println!("  💡 This usually means:");
    println!("     • No internet connection");
    println!("     • Repository server is down");
    println!("     • Firewall is blocking the connection");

    println!("\n");

    // ========================================================================
    // Example 3: Check prerequisites for install command
    // ========================================================================
    println!("Example 3: Check Prerequisites (Install Command)");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let prereqs = CheckPrerequisites::new(");
    println!("      config.clone(),");
    println!("      \"install\".to_string(),");
    println!("  );");
    println!();
    println!("Output:");
    println!("  ✓ No prerequisites for 'install'");
    println!("     (or prerequisite message if network/config needed)");

    println!("\n");

    // ========================================================================
    // Example 4: Check prerequisites for serve command
    // ========================================================================
    println!("Example 4: Check Prerequisites (Serve Command)");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let prereqs = CheckPrerequisites::new(");
    println!("      config.clone(),");
    println!("      \"serve\".to_string(),");
    println!("  );");
    println!();
    println!("Output:");
    println!("  ✓ No prerequisites for 'serve'");
    println!("     (or warning if no models available)");

    println!("\n");

    // ========================================================================
    // Example 5: Get usage examples for install command
    // ========================================================================
    println!("Example 5: Get Usage Examples (Install)");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let examples = GetExamples::new(");
    println!("      config.clone(),");
    println!("      \"install\".to_string(),");
    println!("  );");
    println!();
    println!("Output:");
    println!("  📚 Installation Examples:");
    println!();
    println!("  # Install popular models");
    println!("  inferno install microsoft/DialoGPT-medium");
    println!("  inferno install google/flan-t5-base");

    println!("\n");

    // ========================================================================
    // Example 6: Get usage examples for search command
    // ========================================================================
    println!("Example 6: Get Usage Examples (Search)");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let examples = GetExamples::new(");
    println!("      config.clone(),");
    println!("      \"search\".to_string(),");
    println!("  );");
    println!();
    println!("Output:");
    println!("  📚 Search Examples:");
    println!();
    println!("  # Basic search");
    println!("  inferno search \"language model\"");
    println!("  inferno search \"code generation\"");

    println!("\n");

    // ========================================================================
    // Example 7: Validation tests
    // ========================================================================
    println!("Example 7: Input Validation");
    println!("{}", "─".repeat(80));

    let empty_error = HandleError::new(config.clone(), String::new());
    let ctx_invalid = CommandContext::new(config.clone());

    match pipeline
        .execute(Box::new(empty_error), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught empty error message:");
            println!("  {}", e);
        }
    }

    println!();

    let empty_command = CheckPrerequisites::new(config.clone(), String::new());

    match pipeline
        .execute(Box::new(empty_command), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught empty command name:");
            println!("  {}", e);
        }
    }

    println!();

    let empty_examples = GetExamples::new(config.clone(), String::new());

    match pipeline
        .execute(Box::new(empty_examples), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught empty command name:");
            println!("  {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Summary
    // ========================================================================
    println!("{}", "═".repeat(80));
    println!("Summary: Help Command v2 Features");
    println!("{}", "═".repeat(80));
    println!("✓ Contextual error guidance with solutions");
    println!("✓ Command prerequisite checking");
    println!("✓ Usage examples for commands");
    println!("✓ Smart error pattern detection");
    println!("✓ Comprehensive validation");
    println!("✓ Structured JSON output");
    println!("✓ Middleware support");
    println!();
    println!("Error Types Handled:");
    println!("  - File not found");
    println!("  - Permission denied");
    println!("  - Network connectivity");
    println!("  - Configuration errors");
    println!("  - Repository errors");
    println!("  - Model not found");
    println!("  - Authentication");
    println!("  - Disk space");
    println!("  - Dependency errors");
    println!();
    println!("Use Cases:");
    println!("  - User-friendly error messages");
    println!("  - Setup guidance for new users");
    println!("  - Command discovery");
    println!("  - Troubleshooting assistance");

    Ok(())
}
