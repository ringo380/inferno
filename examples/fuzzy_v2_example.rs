//! Fuzzy Matching Command v2 Example
//!
//! Demonstrates fuzzy command matching and suggestion utilities.
//!
//! Run with: cargo run --example fuzzy_v2_example

use anyhow::Result;
use inferno::cli::fuzzy_v2::{FuzzyMatch, FuzzyMultiMatch};
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

    println!("🔥 Inferno Fuzzy Matching Command v2 Examples\n");

    // Create configuration
    let config = Config::default();

    // Build command pipeline with middleware
    let pipeline = CommandPipeline::builder()
        .with_middleware(Box::new(LoggingMiddleware::new()))
        .with_middleware(Box::new(MetricsMiddleware::new()))
        .build();

    // ========================================================================
    // Example 1: Find best match for typo
    // ========================================================================
    println!("Example 1: Find Best Match for Typo");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let match_cmd = FuzzyMatch::new(");
    println!("      config.clone(),");
    println!("      \"instal\".to_string(),");
    println!("  );");
    println!();
    println!("Output:");
    println!("  Best match for 'instal':");
    println!("    → install");

    println!("\n");

    // ========================================================================
    // Example 2: Find match for partial command
    // ========================================================================
    println!("Example 2: Find Match for Partial Command");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let match_cmd = FuzzyMatch::new(");
    println!("      config.clone(),");
    println!("      \"mod\".to_string(),");
    println!("  );");
    println!();
    println!("Output:");
    println!("  Best match for 'mod':");
    println!("    → models");

    println!("\n");

    // ========================================================================
    // Example 3: Get multiple suggestions
    // ========================================================================
    println!("Example 3: Get Multiple Suggestions");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let multi_match = FuzzyMultiMatch::new(");
    println!("      config.clone(),");
    println!("      \"bat\".to_string(),");
    println!("      5,");
    println!("  );");
    println!();
    println!("Output:");
    println!("  Top 5 matches for 'bat':");
    println!("    1. batch");
    println!("    2. batch-queue");
    println!("    3. validate");
    println!("    4. cache");

    println!("\n");

    // ========================================================================
    // Example 4: Get all suggestions for vague query
    // ========================================================================
    println!("Example 4: Get All Suggestions for Vague Query");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let multi_match = FuzzyMultiMatch::new(");
    println!("      config.clone(),");
    println!("      \"m\".to_string(),");
    println!("      10,");
    println!("  );");
    println!();
    println!("Output:");
    println!("  Top 10 matches for 'm':");
    println!("    1. models");
    println!("    2. metrics");
    println!("    3. models list");
    println!("    4. model-versioning");
    println!("    5. marketplace");
    println!("    6. multi-modal");
    println!("    7. monitoring");
    println!("    8. multi-tenancy");

    println!("\n");

    // ========================================================================
    // Example 5: Handle no matches
    // ========================================================================
    println!("Example 5: Handle No Matches");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let match_cmd = FuzzyMatch::new(");
    println!("      config.clone(),");
    println!("      \"xyzabc123\".to_string(),");
    println!("  );");
    println!();
    println!("Output:");
    println!("  No match found for 'xyzabc123'");

    println!("\n");

    // ========================================================================
    // Example 6: Validation tests
    // ========================================================================
    println!("Example 6: Input Validation");
    println!("{}", "─".repeat(80));

    let empty_input = FuzzyMatch::new(config.clone(), String::new());
    let ctx_invalid = CommandContext::new(config.clone());

    match pipeline
        .execute(Box::new(empty_input), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught empty input:");
            println!("  {}", e);
        }
    }

    println!();

    let zero_limit = FuzzyMultiMatch::new(config.clone(), "test".to_string(), 0);

    match pipeline
        .execute(Box::new(zero_limit), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught zero limit:");
            println!("  {}", e);
        }
    }

    println!();

    let excessive_limit = FuzzyMultiMatch::new(config.clone(), "test".to_string(), 100);

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
    println!("Summary: Fuzzy Matching Command v2 Features");
    println!("{}", "═".repeat(80));
    println!("✓ Single best match finding");
    println!("✓ Multiple match retrieval with configurable limit");
    println!("✓ Typo correction suggestions");
    println!("✓ Partial command completion");
    println!("✓ Comprehensive validation");
    println!("✓ Structured JSON output");
    println!("✓ Middleware support");
    println!();
    println!("Validation:");
    println!("  - Input string not empty");
    println!("  - Limit range: 1-50 for multi-match");
    println!();
    println!("Use Cases:");
    println!("  - Command-line typo correction");
    println!("  - Autocomplete systems");
    println!("  - User-friendly CLI navigation");
    println!("  - Discovery of similar commands");

    Ok(())
}
