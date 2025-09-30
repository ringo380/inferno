//! Repo Command v2 Example
//!
//! Demonstrates the new CLI architecture for the repo command.
//! Shows repository management operations including list, info, add, remove, and update.
//!
//! Run with: cargo run --example repo_v2_example

use anyhow::Result;
use inferno::cli::repo_v2::{RepoAdd, RepoInfo, RepoList, RepoRemove, RepoUpdate};
use inferno::config::Config;
use inferno::core::config::ConfigBuilder;
use inferno::interfaces::cli::{
    CommandContext, CommandPipeline, LoggingMiddleware, MetricsMiddleware,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🔥 Inferno Repo Command v2 Examples\n");

    // Create configuration
    let config = Config::default();
    let _core_config = ConfigBuilder::new().build_unchecked();

    // Build command pipeline with middleware
    let pipeline = CommandPipeline::builder()
        .with_middleware(Box::new(LoggingMiddleware::new()))
        .with_middleware(Box::new(MetricsMiddleware::new()))
        .build();

    // ========================================================================
    // Example 1: List repositories
    // ========================================================================
    println!("Example 1: List All Repositories");
    println!("{}", "─".repeat(80));
    println!("Usage example:");
    println!("  let list_cmd = RepoList::new(config.clone(), false, false);");
    println!();
    println!("Expected output:");
    println!("  Configured repositories (3):");
    println!();
    println!("  NAME                 URL                                                PRIORITY ENABLED  VERIFICATION");
    println!("  {}", "-".repeat(98));
    println!("  huggingface          https://huggingface.co/api/models                  10       yes      optional");
    println!("  ollama               https://ollama.ai/library                          20       yes      optional");
    println!("  local                file:///var/inferno/models                         100      yes      required");

    println!("\n");

    // ========================================================================
    // Example 2: List repositories (detailed)
    // ========================================================================
    println!("Example 2: List Repositories (Detailed)");
    println!("{}", "─".repeat(80));
    println!("Usage example:");
    println!("  let detailed_cmd = RepoList::new(config.clone(), true, false);");
    println!();
    println!("Expected output:");
    println!("  Configured repositories (3):");
    println!();
    println!("  Repository: huggingface");
    println!("    URL: https://huggingface.co/api/models");
    println!("    Priority: 10");
    println!("    Enabled: yes");
    println!("    Verification required: no");
    println!("    Last updated: 2025-09-29 10:30:45");
    println!();
    println!("  Repository: ollama");
    println!("    URL: https://ollama.ai/library");
    println!("    Priority: 20");
    println!("    Enabled: yes");
    println!("    Verification required: no");
    println!("    Last updated: 2025-09-29 09:15:22");

    println!("\n");

    // ========================================================================
    // Example 3: List enabled repositories only
    // ========================================================================
    println!("Example 3: List Enabled Repositories Only");
    println!("{}", "─".repeat(80));
    println!("Usage example:");
    println!("  let enabled_cmd = RepoList::new(config.clone(), false, true);");
    println!();
    println!("Expected output:");
    println!("  Shows only repositories where enabled = true");
    println!("  Filters out any disabled repositories from the list");

    println!("\n");

    // ========================================================================
    // Example 4: Show repository information
    // ========================================================================
    println!("Example 4: Show Repository Information");
    println!("{}", "─".repeat(80));
    println!("Usage example:");
    println!("  let info_cmd = RepoInfo::new(");
    println!("      config.clone(),");
    println!("      \"huggingface\".to_string(),");
    println!("      false,  // don't show models");
    println!("  );");
    println!();
    println!("Expected output:");
    println!("  Repository Information");
    println!("  ======================");
    println!("  Name: huggingface");
    println!("  URL: https://huggingface.co/api/models");
    println!("  Priority: 10");
    println!("  Enabled: yes");
    println!("  Verification required: no");
    println!("  Last updated: 2025-09-29 10:30:45");
    println!("  Metadata URL: https://huggingface.co/api/models/metadata");
    println!("  Authentication: configured");
    println!("    API key: configured");

    println!("\n");

    // ========================================================================
    // Example 5: Show repository with models
    // ========================================================================
    println!("Example 5: Show Repository with Available Models");
    println!("{}", "─".repeat(80));
    println!("Usage example:");
    println!("  let info_models = RepoInfo::new(");
    println!("      config.clone(),");
    println!("      \"huggingface\".to_string(),");
    println!("      true,  // show models");
    println!("  );");
    println!();
    println!("Expected output:");
    println!("  Repository Information");
    println!("  ======================");
    println!("  [... repository details ...]");
    println!();
    println!("  Available models:");
    println!("  ================");
    println!("  Found 247 models:");
    println!("    - llama-2-7b-chat v1.0.0 by meta-llama");
    println!("    - mistral-7b-instruct v0.1.0 by mistralai");
    println!("    - codellama-13b v1.0.0 by meta-llama");
    println!("    - mixtral-8x7b v0.1.0 by mistralai");
    println!("    - phi-2 v2.0.0 by microsoft");
    println!("    ... and 242 more");

    println!("\n");

    // ========================================================================
    // Example 6: Add a new repository
    // ========================================================================
    println!("Example 6: Add a New Repository");
    println!("{}", "─".repeat(80));
    println!("Usage example:");
    println!("  let add_cmd = RepoAdd::new(");
    println!("      config.clone(),");
    println!("      \"custom-repo\".to_string(),");
    println!("      \"https://models.example.com/api\".to_string(),");
    println!("      50,     // priority");
    println!("      true,   // verify signatures");
    println!("      false,  // not disabled");
    println!("  );");
    println!();
    println!("Expected output:");
    println!("  Testing connection to repository...");
    println!("  ✓ Repository is accessible");
    println!("  ✓ Repository 'custom-repo' added successfully");
    println!("    URL: https://models.example.com/api");
    println!("    Priority: 50");
    println!("    Verification: enabled");
    println!("    Status: enabled");
    println!();
    println!("  Updating repository metadata...");

    println!("\n");

    // ========================================================================
    // Example 7: Add disabled repository
    // ========================================================================
    println!("Example 7: Add Disabled Repository");
    println!("{}", "─".repeat(80));
    println!("Usage example:");
    println!("  let add_disabled = RepoAdd::new(");
    println!("      config.clone(),");
    println!("      \"future-repo\".to_string(),");
    println!("      \"https://future.example.com/api\".to_string(),");
    println!("      200,   // low priority");
    println!("      false, // no verification");
    println!("      true,  // disabled");
    println!("  );");
    println!();
    println!("Expected behavior:");
    println!("  - Repository is added but not activated");
    println!("  - No connection test performed");
    println!("  - Metadata not downloaded");
    println!("  - Can be enabled later with 'repo toggle --enable'");

    println!("\n");

    // ========================================================================
    // Example 8: Remove a repository
    // ========================================================================
    println!("Example 8: Remove a Repository");
    println!("{}", "─".repeat(80));
    println!("Usage example:");
    println!("  let remove_cmd = RepoRemove::new(");
    println!("      config.clone(),");
    println!("      \"old-repo\".to_string(),");
    println!("      false,  // not forced - will prompt");
    println!("  );");
    println!();
    println!("Expected output:");
    println!("  Remove repository 'old-repo'? (y/N): y");
    println!("  ✓ Repository 'old-repo' removed successfully");

    println!("\n");

    // ========================================================================
    // Example 9: Force remove repository
    // ========================================================================
    println!("Example 9: Force Remove (No Confirmation)");
    println!("{}", "─".repeat(80));
    println!("Usage example:");
    println!("  let force_remove = RepoRemove::new(");
    println!("      config.clone(),");
    println!("      \"temp-repo\".to_string(),");
    println!("      true,  // force - no prompt");
    println!("  );");
    println!();
    println!("Expected output:");
    println!("  ✓ Repository 'temp-repo' removed successfully");
    println!("  (No confirmation prompt)");

    println!("\n");

    // ========================================================================
    // Example 10: Update repository metadata
    // ========================================================================
    println!("Example 10: Update Specific Repository");
    println!("{}", "─".repeat(80));
    println!("Usage example:");
    println!("  let update_cmd = RepoUpdate::new(");
    println!("      config.clone(),");
    println!("      Some(\"huggingface\".to_string()),");
    println!("      false,  // don't force");
    println!("  );");
    println!();
    println!("Expected output:");
    println!("  Updating repository metadata for 'huggingface'...");
    println!("  ✓ Repository 'huggingface' metadata updated");

    println!("\n");

    // ========================================================================
    // Example 11: Update all repositories
    // ========================================================================
    println!("Example 11: Update All Repositories");
    println!("{}", "─".repeat(80));
    println!("Usage example:");
    println!("  let update_all = RepoUpdate::new(");
    println!("      config.clone(),");
    println!("      None,   // None = all repositories");
    println!("      false,");
    println!("  );");
    println!();
    println!("Expected output:");
    println!("  Updating metadata for all repositories...");
    println!("  ✓ All repository metadata updated");

    println!("\n");

    // ========================================================================
    // Example 12: Force update (ignore cache)
    // ========================================================================
    println!("Example 12: Force Update (Ignore Cache)");
    println!("{}", "─".repeat(80));
    println!("Usage example:");
    println!("  let force_update = RepoUpdate::new(");
    println!("      config.clone(),");
    println!("      Some(\"ollama\".to_string()),");
    println!("      true,  // force - ignore cache");
    println!("  );");
    println!();
    println!("Expected output:");
    println!("  Updating repository metadata for 'ollama'...");
    println!("  Forcing update (ignoring cache)...");
    println!("  ✓ Repository 'ollama' metadata updated");

    println!("\n");

    // ========================================================================
    // Example 13: Validation examples
    // ========================================================================
    println!("Example 13: Input Validation");
    println!("{}", "─".repeat(80));

    // Test empty name
    let invalid_info = RepoInfo::new(config.clone(), String::new(), false);
    let ctx_invalid = CommandContext::new(config.clone());

    match pipeline
        .execute(Box::new(invalid_info), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught empty repository name:");
            println!("  {}", e);
        }
    }

    println!();

    // Test invalid URL
    let invalid_url = RepoAdd::new(
        config.clone(),
        "test".to_string(),
        "ftp://invalid.com".to_string(),
        100,
        false,
        false,
    );

    match pipeline
        .execute(Box::new(invalid_url), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught invalid URL protocol:");
            println!("  {}", e);
        }
    }

    println!();

    // Test excessive priority
    let excessive_priority = RepoAdd::new(
        config.clone(),
        "test".to_string(),
        "https://example.com".to_string(),
        1001,
        false,
        false,
    );

    match pipeline
        .execute(Box::new(excessive_priority), &mut ctx_invalid.clone())
        .await
    {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("✓ Validation caught excessive priority:");
            println!("  {}", e);
        }
    }

    println!("\n");

    // ========================================================================
    // Example 14: JSON output mode
    // ========================================================================
    println!("Example 14: JSON Output Mode");
    println!("{}", "─".repeat(80));
    println!("With json_output=true, structured data is returned:");
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "repositories": [
                {
                    "name": "huggingface",
                    "url": "https://huggingface.co/api/models",
                    "priority": 10,
                    "enabled": true,
                    "verification_required": false,
                    "last_updated": "2025-09-29T10:30:45Z",
                    "metadata_url": "https://huggingface.co/api/models/metadata"
                },
                {
                    "name": "ollama",
                    "url": "https://ollama.ai/library",
                    "priority": 20,
                    "enabled": true,
                    "verification_required": false,
                    "last_updated": "2025-09-29T09:15:22Z"
                }
            ],
            "total": 2
        }))?
    );

    println!("\n");

    // ========================================================================
    // Example 15: Integration use cases
    // ========================================================================
    println!("Example 15: Integration Use Cases");
    println!("{}", "─".repeat(80));
    println!("Common repository management scenarios:");
    println!();
    println!("1. Corporate Model Registry:");
    println!("   inferno repo add corporate https://models.corp.internal --priority 1 --verify");
    println!();
    println!("2. Public Model Hub:");
    println!("   inferno repo add huggingface https://huggingface.co/api/models --priority 10");
    println!();
    println!("3. Local Development:");
    println!("   inferno repo add local file:///home/user/models --priority 5");
    println!();
    println!("4. Repository Maintenance:");
    println!("   inferno repo update  # Update all repositories");
    println!("   inferno repo update huggingface --force  # Force update specific repo");
    println!();
    println!("5. Repository Discovery:");
    println!("   inferno repo list --detailed");
    println!("   inferno repo info huggingface --models");
    println!();
    println!("6. Repository Cleanup:");
    println!("   inferno repo remove old-repo --force");
    println!("   inferno repo clean --metadata  # Clean cached metadata");

    println!("\n");

    // ========================================================================
    // Summary
    // ========================================================================
    println!("{}", "═".repeat(80));
    println!("Summary: Repo Command v2 Features");
    println!("{}", "═".repeat(80));
    println!("✓ List all repositories (compact or detailed)");
    println!("✓ Filter by enabled status");
    println!("✓ Show repository information");
    println!("✓ Display available models from repository");
    println!("✓ Add new repositories with validation");
    println!("✓ Configure priority, verification, and status");
    println!("✓ Remove repositories with confirmation");
    println!("✓ Force operations without prompts");
    println!("✓ Update repository metadata");
    println!("✓ Bulk update all repositories");
    println!("✓ Force update (ignore cache)");
    println!("✓ Input validation (URLs, priorities)");
    println!("✓ Structured JSON output");
    println!("✓ Human-readable progress and results");
    println!("✓ Middleware support (logging, metrics)");
    println!();
    println!("Validation Checks:");
    println!("  - Repository name not empty");
    println!("  - URL format validation (http:// or https://)");
    println!("  - Priority range (0-1000)");
    println!("  - Repository existence checks");
    println!();
    println!("Repository Features:");
    println!("  - Multiple repository support");
    println!("  - Priority-based selection");
    println!("  - Optional signature verification");
    println!("  - Enable/disable repositories");
    println!("  - Metadata caching");
    println!("  - Authentication support");
    println!();
    println!("Use Cases:");
    println!("  - Corporate model registries");
    println!("  - Public model hubs (Hugging Face, Ollama)");
    println!("  - Local development repositories");
    println!("  - Multi-source model management");
    println!("  - Repository discovery and browsing");
    println!("  - Automated CI/CD integration");
    println!();
    println!("Note: This is a focused migration covering core repository management.");
    println!("Full repository functionality (toggle, test, priority, clean) remains");
    println!("available through the original repo module.");

    Ok(())
}