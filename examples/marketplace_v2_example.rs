//! Marketplace Command v2 Example
//!
//! Demonstrates model marketplace operations including search, download,
//! publish, and management.
//!
//! Run with: cargo run --example marketplace_v2_example

use anyhow::Result;
use inferno::cli::marketplace_v2::*;
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

    println!("🔥 Inferno Marketplace Command v2 Examples\n");

    // Create configuration
    let config = Config::default();

    // Build command pipeline with middleware
    let pipeline = CommandPipeline::builder()
        .with_middleware(Box::new(LoggingMiddleware::new()))
        .with_middleware(Box::new(MetricsMiddleware::new()))
        .build();

    // ========================================================================
    // Example 1: Search for models
    // ========================================================================
    println!("Example 1: Search for Models");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let search = MarketplaceSearch::new(");
    println!("      config.clone(),");
    println!("      \"llama\".to_string(),");
    println!("      None,     // all categories");
    println!("      false,    // not verified only");
    println!("      false,    // not free only");
    println!("      20,       // limit");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Searching Marketplace ===");
    println!("  Query: llama");
    println!("  Limit: 20");
    println!("  ");
    println!("  Model: llama-2-7b");
    println!("    Publisher: Meta AI");
    println!("    Category: language");
    println!("    Size: 13.5 GB");
    println!("    Rating: 4.8/5.0");
    println!("    Verified: ✓");
    println!("  ");
    println!("  Model: whisper-base");
    println!("    Publisher: OpenAI");
    println!("    Category: audio");
    println!("    Size: 145 MB");
    println!("    Rating: 4.6/5.0");
    println!("    Verified: ✓");
    println!("  ");
    println!("  Total Results: 2");

    println!("\n");

    // ========================================================================
    // Example 2: Search with category filter
    // ========================================================================
    println!("Example 2: Search with Category Filter");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let search = MarketplaceSearch::new(");
    println!("      config.clone(),");
    println!("      \"model\".to_string(),");
    println!("      Some(\"language\".to_string()),");
    println!("      false,");
    println!("      false,");
    println!("      10,");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Searching Marketplace ===");
    println!("  Query: model");
    println!("  Category: language");
    println!("  Limit: 10");
    println!("  ");
    println!("  Model: llama-2-7b");
    println!("    Publisher: Meta AI");
    println!("    Category: language");
    println!("    Size: 13.5 GB");
    println!("    Rating: 4.8/5.0");
    println!("    Verified: ✓");
    println!("  ");
    println!("  Total Results: 1");

    println!("\n");

    // ========================================================================
    // Example 3: Search verified only
    // ========================================================================
    println!("Example 3: Search Verified Models Only");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let search = MarketplaceSearch::new(");
    println!("      config.clone(),");
    println!("      \"ai\".to_string(),");
    println!("      None,");
    println!("      true,     // verified only");
    println!("      false,");
    println!("      20,");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Searching Marketplace ===");
    println!("  Query: ai");
    println!("  Filter: Verified models only");
    println!("  Limit: 20");
    println!("  ");
    println!("  [Only verified models shown...]");

    println!("\n");

    // ========================================================================
    // Example 4: Search free models
    // ========================================================================
    println!("Example 4: Search Free Models");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let search = MarketplaceSearch::new(");
    println!("      config.clone(),");
    println!("      \"model\".to_string(),");
    println!("      None,");
    println!("      false,");
    println!("      true,     // free only");
    println!("      20,");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Searching Marketplace ===");
    println!("  Query: model");
    println!("  Filter: Free models only");
    println!("  Limit: 20");
    println!("  ");
    println!("  [Only free models shown...]");

    println!("\n");

    // ========================================================================
    // Example 5: Get model information
    // ========================================================================
    println!("Example 5: Get Model Information");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let info = MarketplaceInfo::new(");
    println!("      config.clone(),");
    println!("      \"llama-2-7b\".to_string(),");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Model Information ===");
    println!("  Model ID: llama-2-7b");
    println!("  Name: LLaMA 2 7B");
    println!("  Publisher: Meta AI");
    println!("  Category: language");
    println!("  ");
    println!("  Description:");
    println!("    LLaMA 2 7B parameter language model");
    println!("  ");
    println!("  Details:");
    println!("    Size: 13.5 GB");
    println!("    License: Meta AI Community License");
    println!("    Rating: 4.8/5.0 (1,234 reviews)");
    println!("    Downloads: 45,678");
    println!("    Last Updated: 2025-09-15");
    println!("    Verified: ✓");

    println!("\n");

    // ========================================================================
    // Example 6: Download a model
    // ========================================================================
    println!("Example 6: Download a Model");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let download = MarketplaceDownload::new(");
    println!("      config.clone(),");
    println!("      \"llama-2-7b\".to_string(),");
    println!("      None,     // default output");
    println!("      false,    // run compatibility checks");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Downloading Model ===");
    println!("  Model ID: llama-2-7b");
    println!("  Skip Compatibility Checks: false");
    println!("  ");
    println!("  Running compatibility checks...");
    println!("  ✓ System requirements met");
    println!("  ✓ Storage space available");
    println!("  ");
    println!("  Downloading...");
    println!("  Progress: [████████████████████] 100%");
    println!("  ");
    println!("  ✓ Model downloaded successfully");
    println!("  Download ID: dl-abc123");

    println!("\n");

    // ========================================================================
    // Example 7: Download to specific directory
    // ========================================================================
    println!("Example 7: Download to Specific Directory");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let download = MarketplaceDownload::new(");
    println!("      config.clone(),");
    println!("      \"whisper-base\".to_string(),");
    println!("      Some(PathBuf::from(\"/models\")),");
    println!("      false,");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Downloading Model ===");
    println!("  Model ID: whisper-base");
    println!("  Output Directory: \"/models\"");
    println!("  Skip Compatibility Checks: false");
    println!("  ");
    println!("  Running compatibility checks...");
    println!("  ✓ System requirements met");
    println!("  ✓ Storage space available");
    println!("  ");
    println!("  Downloading...");
    println!("  Progress: [████████████████████] 100%");
    println!("  ");
    println!("  ✓ Model downloaded successfully");
    println!("  Download ID: dl-xyz789");

    println!("\n");

    // ========================================================================
    // Example 8: Download skipping checks
    // ========================================================================
    println!("Example 8: Download Skipping Compatibility Checks");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let download = MarketplaceDownload::new(");
    println!("      config.clone(),");
    println!("      \"gpt-neo-1.3b\".to_string(),");
    println!("      None,");
    println!("      true,     // skip checks");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Downloading Model ===");
    println!("  Model ID: gpt-neo-1.3b");
    println!("  Skip Compatibility Checks: true");
    println!("  ");
    println!("  Downloading...");
    println!("  Progress: [████████████████████] 100%");
    println!("  ");
    println!("  ✓ Model downloaded successfully");
    println!("  Download ID: dl-def456");

    println!("\n");

    // ========================================================================
    // Example 9: Publish a free model
    // ========================================================================
    println!("Example 9: Publish a Free Model");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let publish = MarketplacePublish::new(");
    println!("      config.clone(),");
    println!("      PathBuf::from(\"/models/my-model.gguf\"),");
    println!("      \"My Custom Model\".to_string(),");
    println!("      \"A fine-tuned language model\".to_string(),");
    println!("      \"language\".to_string(),");
    println!("      \"public\".to_string(),");
    println!("      None,     // free");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Publishing Model ===");
    println!("  Model: \"/models/my-model.gguf\"");
    println!("  Name: My Custom Model");
    println!("  Description: A fine-tuned language model");
    println!("  Category: language");
    println!("  Visibility: public");
    println!("  Price: Free");
    println!("  ");
    println!("  Validating model...");
    println!("  ✓ Model format valid");
    println!("  ");
    println!("  Uploading...");
    println!("  Progress: [████████████████████] 100%");
    println!("  ");
    println!("  ✓ Model published successfully");
    println!("  Model ID: my-model-123");

    println!("\n");

    // ========================================================================
    // Example 10: Publish a paid model
    // ========================================================================
    println!("Example 10: Publish a Paid Model");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let publish = MarketplacePublish::new(");
    println!("      config.clone(),");
    println!("      PathBuf::from(\"/models/premium-model.gguf\"),");
    println!("      \"Premium Model\".to_string(),");
    println!("      \"Enterprise-grade model\".to_string(),");
    println!("      \"language\".to_string(),");
    println!("      \"public\".to_string(),");
    println!("      Some(49.99),");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Publishing Model ===");
    println!("  Model: \"/models/premium-model.gguf\"");
    println!("  Name: Premium Model");
    println!("  Description: Enterprise-grade model");
    println!("  Category: language");
    println!("  Visibility: public");
    println!("  Price: $49.99");
    println!("  ");
    println!("  Validating model...");
    println!("  ✓ Model format valid");
    println!("  ");
    println!("  Uploading...");
    println!("  Progress: [████████████████████] 100%");
    println!("  ");
    println!("  ✓ Model published successfully");
    println!("  Model ID: premium-123");

    println!("\n");

    // ========================================================================
    // Example 11: Publish private model
    // ========================================================================
    println!("Example 11: Publish Private Model");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let publish = MarketplacePublish::new(");
    println!("      config.clone(),");
    println!("      PathBuf::from(\"/models/internal.gguf\"),");
    println!("      \"Internal Model\".to_string(),");
    println!("      \"For internal use only\".to_string(),");
    println!("      \"language\".to_string(),");
    println!("      \"private\".to_string(),");
    println!("      None,");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Publishing Model ===");
    println!("  Model: \"/models/internal.gguf\"");
    println!("  Name: Internal Model");
    println!("  Description: For internal use only");
    println!("  Category: language");
    println!("  Visibility: private");
    println!("  Price: Free");
    println!("  ");
    println!("  [Publishing process...]");

    println!("\n");

    // ========================================================================
    // Example 12: List my models
    // ========================================================================
    println!("Example 12: List My Published Models");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let list = MarketplaceList::new(");
    println!("      config.clone(),");
    println!("      true,     // my models only");
    println!("      None,     // all statuses");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Marketplace Models ===");
    println!("  Filter: My models only");
    println!("  ");
    println!("  Model: my-model-123");
    println!("    Name: My Custom Model");
    println!("    Status: published");
    println!("    Downloads: 234");
    println!("  ");
    println!("  Model: my-model-456");
    println!("    Name: Another Model");
    println!("    Status: pending");
    println!("    Downloads: 0");
    println!("  ");
    println!("  Total Models: 2");

    println!("\n");

    // ========================================================================
    // Example 13: List with status filter
    // ========================================================================
    println!("Example 13: List Published Models");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let list = MarketplaceList::new(");
    println!("      config.clone(),");
    println!("      true,");
    println!("      Some(\"published\".to_string()),");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Marketplace Models ===");
    println!("  Filter: My models only");
    println!("  Status: published");
    println!("  ");
    println!("  Model: my-model-123");
    println!("    Name: My Custom Model");
    println!("    Status: published");
    println!("    Downloads: 234");
    println!("  ");
    println!("  Total Models: 1");

    println!("\n");

    // ========================================================================
    // Example 14: Update model metadata
    // ========================================================================
    println!("Example 14: Update Model Metadata");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let update = MarketplaceUpdate::new(");
    println!("      config.clone(),");
    println!("      \"my-model-123\".to_string(),");
    println!("      Some(\"Updated Model Name\".to_string()),");
    println!("      Some(\"Updated description\".to_string()),");
    println!("      None,     // keep visibility");
    println!("      None,     // keep price");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Updating Model Metadata ===");
    println!("  Model ID: my-model-123");
    println!("  ");
    println!("  New Name: Updated Model Name");
    println!("  New Description: Updated description");
    println!("  ");
    println!("  ✓ Model metadata updated successfully");

    println!("\n");

    // ========================================================================
    // Example 15: Update model price
    // ========================================================================
    println!("Example 15: Update Model Price");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let update = MarketplaceUpdate::new(");
    println!("      config.clone(),");
    println!("      \"premium-123\".to_string(),");
    println!("      None,");
    println!("      None,");
    println!("      None,");
    println!("      Some(39.99),");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Updating Model Metadata ===");
    println!("  Model ID: premium-123");
    println!("  ");
    println!("  New Price: $39.99");
    println!("  ");
    println!("  ✓ Model metadata updated successfully");

    println!("\n");

    // ========================================================================
    // Example 16: Unpublish a model
    // ========================================================================
    println!("Example 16: Unpublish a Model");
    println!("{}", "─".repeat(80));
    println!("Usage:");
    println!("  let unpublish = MarketplaceUnpublish::new(");
    println!("      config.clone(),");
    println!("      \"my-model-456\".to_string(),");
    println!("      true,     // confirm");
    println!("  );");
    println!();
    println!("Output:");
    println!("  === Unpublishing Model ===");
    println!("  Model ID: my-model-456");
    println!("  ");
    println!("  ⚠️  This will remove the model from the marketplace");
    println!("  ✓ Model unpublished successfully");

    println!("\n");

    // ========================================================================
    // Example 17: Validation tests
    // ========================================================================
    println!("Example 17: Input Validation");
    println!("{}", "─".repeat(80));

    let empty_query = MarketplaceSearch::new(
        config.clone(),
        "".to_string(),
        None,
        false,
        false,
        20,
    );
    let ctx_invalid = CommandContext::new(config.clone());

    match pipeline
        .execute(Box::new(empty_query), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught empty query:");
            println!("  {}", e);
        }
    }

    println!();

    let invalid_limit = MarketplaceSearch::new(
        config.clone(),
        "llama".to_string(),
        None,
        false,
        false,
        150,
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

    println!();

    let no_confirm = MarketplaceUnpublish::new(
        config.clone(),
        "model-123".to_string(),
        false,
    );

    match pipeline
        .execute(Box::new(no_confirm), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught missing confirmation:");
            println!("  {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Summary
    // ========================================================================
    println!("{}", "═".repeat(80));
    println!("Summary: Marketplace Command v2 Features");
    println!("{}", "═".repeat(80));
    println!("✓ Model search with filtering");
    println!("✓ Detailed model information");
    println!("✓ Model download with compatibility checks");
    println!("✓ Model publishing (free and paid)");
    println!("✓ Model unpublishing");
    println!("✓ Published model listing");
    println!("✓ Metadata updates");
    println!("✓ Visibility control (public, private, unlisted)");
    println!("✓ Category filtering (language, vision, audio, multimodal)");
    println!("✓ Verified/free model filters");
    println!("✓ Comprehensive validation");
    println!("✓ Structured JSON output");
    println!("✓ Middleware support");
    println!();
    println!("Validation:");
    println!("  - Query cannot be empty");
    println!("  - Search limit: 1-100");
    println!("  - Category: language, vision, audio, multimodal");
    println!("  - Visibility: public, private, unlisted");
    println!("  - Price cannot be negative");
    println!("  - Unpublish requires confirmation");
    println!();
    println!("Use Cases:");
    println!("  - Model discovery and search");
    println!("  - Model distribution and monetization");
    println!("  - Model lifecycle management");
    println!("  - Marketplace participation");

    Ok(())
}