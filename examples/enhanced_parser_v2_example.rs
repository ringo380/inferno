//! Enhanced Parser Command v2 Example
//!
//! Demonstrates CLI parser validation and suggestion utilities.
//!
//! Run with: cargo run --example enhanced_parser_v2_example

use anyhow::Result;
use inferno::cli::enhanced_parser_v2::{CheckPrerequisites, GetSuggestions, ValidateCommand};
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

    println!("🔥 Inferno Enhanced Parser Command v2 Examples\n");

    // Create configuration
    let config = Config::default();

    // Build command pipeline with middleware
    let pipeline = CommandPipeline::builder()
        .with_middleware(Box::new(LoggingMiddleware::new()))
        .with_middleware(Box::new(MetricsMiddleware::new()))
        .build();

    // ========================================================================
    // Example 1: Validate valid command
    // ========================================================================
    println!("Example 1: Validate Valid Command");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let validate = ValidateCommand::new(");
    println!("      config.clone(),");
    println!("      \"run\".to_string(),");
    println!("  );");
    println!();
    println!("Output:");
    println!("  ✓ 'run' is a valid command");

    println!("\n");

    // ========================================================================
    // Example 2: Validate command alias
    // ========================================================================
    println!("Example 2: Validate Command Alias");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let validate = ValidateCommand::new(");
    println!("      config.clone(),");
    println!("      \"ls\".to_string(),");
    println!("  );");
    println!();
    println!("Output:");
    println!("  ✓ 'ls' is an alias for 'list'");
    println!("    Canonical form: list");

    println!("\n");

    // ========================================================================
    // Example 3: Get suggestions for typo
    // ========================================================================
    println!("Example 3: Get Suggestions for Typo");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let suggestions = GetSuggestions::new(");
    println!("      config.clone(),");
    println!("      \"isntall\".to_string(),");
    println!("      5,");
    println!("  );");
    println!();
    println!("Output:");
    println!("  Suggestions for 'isntall':");
    println!("    1. install");
    println!("    2. list");
    println!("    3. uninstall");

    println!("\n");

    // ========================================================================
    // Example 4: Get multiple suggestions
    // ========================================================================
    println!("Example 4: Get Multiple Suggestions");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let suggestions = GetSuggestions::new(");
    println!("      config.clone(),");
    println!("      \"mod\".to_string(),");
    println!("      10,");
    println!("  );");
    println!();
    println!("Output:");
    println!("  Suggestions for 'mod':");
    println!("    1. models");
    println!("    2. models list");
    println!("    3. models download");
    println!("    4. model-versioning");

    println!("\n");

    // ========================================================================
    // Example 5: Check prerequisites
    // ========================================================================
    println!("Example 5: Check Command Prerequisites");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let prereqs = CheckPrerequisites::new(");
    println!("      config.clone(),");
    println!("      \"install\".to_string(),");
    println!("  );");
    println!();
    println!("Output:");
    println!("  ⚠️  Prerequisites for 'install':");
    println!("  [prerequisite check message if any]");

    println!("\n");

    // ========================================================================
    // Example 6: Validation tests
    // ========================================================================
    println!("Example 6: Input Validation");
    println!("{}", "─".repeat(80));

    let empty_command = ValidateCommand::new(config.clone(), String::new());
    let ctx_invalid = CommandContext::new(config.clone());

    match pipeline
        .execute(Box::new(empty_command), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught empty command:");
            println!("  {}", e);
        }
    }

    println!();

    let excessive_limit = GetSuggestions::new(config.clone(), "test".to_string(), 25);

    match pipeline
        .execute(Box::new(excessive_limit), &mut ctx_invalid.clone())
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
    println!("Summary: Enhanced Parser Command v2 Features");
    println!("{}", "═".repeat(80));
    println!("✓ Command name validation");
    println!("✓ Fuzzy command suggestions");
    println!("✓ Alias detection");
    println!("✓ Multiple suggestion retrieval");
    println!("✓ Prerequisite checking");
    println!("✓ Comprehensive validation");
    println!("✓ Structured JSON output");
    println!("✓ Middleware support");
    println!();
    println!("Validation:");
    println!("  - Command name not empty");
    println!("  - Query not empty");
    println!("  - Suggestion limit (1-20)");
    println!();
    println!("Use Cases:");
    println!("  - CLI autocomplete systems");
    println!("  - User guidance for typos");
    println!("  - Command discovery");
    println!("  - Prerequisite validation");

    Ok(())
}
