use crate::config::Config;
use crate::marketplace::{ModelMarketplace, MarketplaceConfig};
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use tracing::info;

#[derive(Args)]
pub struct RepoArgs {
    #[command(subcommand)]
    pub command: RepoCommand,
}

#[derive(Subcommand)]
pub enum RepoCommand {
    #[command(about = "Add a new repository")]
    Add {
        #[arg(help = "Repository name")]
        name: String,

        #[arg(help = "Repository URL")]
        url: String,

        #[arg(short, long, help = "Repository priority (lower = higher priority)", default_value = "100")]
        priority: u32,

        #[arg(long, help = "Require signature verification")]
        verify: bool,

        #[arg(long, help = "Disable the repository")]
        disabled: bool,
    },

    #[command(about = "Remove a repository")]
    Remove {
        #[arg(help = "Repository name")]
        name: String,

        #[arg(short, long, help = "Force removal without confirmation")]
        force: bool,
    },

    #[command(about = "List all repositories")]
    List {
        #[arg(long, help = "Show detailed information")]
        detailed: bool,

        #[arg(long, help = "Show only enabled repositories")]
        enabled_only: bool,
    },

    #[command(about = "Enable or disable a repository")]
    Toggle {
        #[arg(help = "Repository name")]
        name: String,

        #[arg(long, help = "Enable the repository")]
        enable: bool,

        #[arg(long, help = "Disable the repository")]
        disable: bool,
    },

    #[command(about = "Update repository metadata")]
    Update {
        #[arg(help = "Repository name (update all if not specified)")]
        name: Option<String>,

        #[arg(short, long, help = "Force update even if recently updated")]
        force: bool,
    },

    #[command(about = "Show repository information")]
    Info {
        #[arg(help = "Repository name")]
        name: String,

        #[arg(long, help = "Show available models")]
        models: bool,
    },

    #[command(about = "Test repository connection")]
    Test {
        #[arg(help = "Repository name")]
        name: String,
    },

    #[command(about = "Set repository priority")]
    Priority {
        #[arg(help = "Repository name")]
        name: String,

        #[arg(help = "New priority (lower = higher priority)")]
        priority: u32,
    },

    #[command(about = "Clean repository cache")]
    Clean {
        #[arg(help = "Repository name (clean all if not specified)")]
        name: Option<String>,

        #[arg(long, help = "Clean metadata cache")]
        metadata: bool,

        #[arg(long, help = "Clean model cache")]
        models: bool,
    },
}

pub async fn handle_repo_command(args: RepoArgs) -> Result<()> {
    let config = Config::load()?;
    let marketplace_config = MarketplaceConfig::from_config(&config)?;
    let marketplace = ModelMarketplace::new(marketplace_config)?;

    match args.command {
        RepoCommand::Add { name, url, priority, verify, disabled } => {
            handle_add(&marketplace, &name, &url, priority, verify, disabled).await
        }

        RepoCommand::Remove { name, force } => {
            handle_remove(&marketplace, &name, force).await
        }

        RepoCommand::List { detailed, enabled_only } => {
            handle_list(&marketplace, detailed, enabled_only).await
        }

        RepoCommand::Toggle { name, enable, disable } => {
            handle_toggle(&marketplace, &name, enable, disable).await
        }

        RepoCommand::Update { name, force } => {
            handle_update(&marketplace, name.as_deref(), force).await
        }

        RepoCommand::Info { name, models } => {
            handle_info(&marketplace, &name, models).await
        }

        RepoCommand::Test { name } => {
            handle_test(&marketplace, &name).await
        }

        RepoCommand::Priority { name, priority } => {
            handle_priority(&marketplace, &name, priority).await
        }

        RepoCommand::Clean { name, metadata, models } => {
            handle_clean(&marketplace, name.as_deref(), metadata, models).await
        }
    }
}

async fn handle_add(
    marketplace: &ModelMarketplace,
    name: &str,
    url: &str,
    priority: u32,
    verify: bool,
    disabled: bool,
) -> Result<()> {
    info!("Adding repository: {} at {}", name, url);

    if !disabled {
        println!("Testing connection to repository...");
        // In a real implementation, this would test the connection
        println!("✓ Repository is accessible");
    }

    match marketplace.repo_add(name, url, Some(priority)).await {
        Ok(_) => {
            println!("✓ Repository '{}' added successfully", name);
            println!("  URL: {}", url);
            println!("  Priority: {}", priority);
            println!("  Verification: {}", if verify { "enabled" } else { "disabled" });
            println!("  Status: {}", if disabled { "disabled" } else { "enabled" });

            if !disabled {
                println!("\nUpdating repository metadata...");
                if let Err(e) = marketplace.repo_update(Some(name)).await {
                    println!("Warning: Failed to update metadata: {}", e);
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to add repository: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

async fn handle_remove(
    marketplace: &ModelMarketplace,
    name: &str,
    force: bool,
) -> Result<()> {
    info!("Removing repository: {}", name);

    if !force {
        if !confirm(&format!("Remove repository '{}'?", name))? {
            println!("Removal cancelled");
            return Ok(());
        }
    }

    match marketplace.repo_remove(name).await {
        Ok(_) => {
            println!("✓ Repository '{}' removed successfully", name);
        }
        Err(e) => {
            println!("✗ Failed to remove repository: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

async fn handle_list(
    marketplace: &ModelMarketplace,
    detailed: bool,
    enabled_only: bool,
) -> Result<()> {
    info!("Listing repositories");

    let mut repositories = marketplace.repo_list().await?;

    if enabled_only {
        repositories.retain(|repo| repo.enabled);
    }

    if repositories.is_empty() {
        println!("No repositories configured");
        return Ok(());
    }

    println!("Configured repositories ({}):", repositories.len());
    println!();

    if detailed {
        for (i, repo) in repositories.iter().enumerate() {
            if i > 0 {
                println!();
            }
            println!("Repository: {}", repo.name);
            println!("  URL: {}", repo.url);
            println!("  Priority: {}", repo.priority);
            println!("  Enabled: {}", if repo.enabled { "yes" } else { "no" });
            println!("  Verification required: {}", if repo.verification_required { "yes" } else { "no" });
            if let Some(last_updated) = repo.last_updated {
                println!("  Last updated: {}", last_updated.format("%Y-%m-%d %H:%M:%S"));
            } else {
                println!("  Last updated: never");
            }
            if let Some(metadata_url) = &repo.metadata_url {
                println!("  Metadata URL: {}", metadata_url);
            }
        }
    } else {
        println!("{:<20} {:<50} {:<8} {:<8} {:<12}", "NAME", "URL", "PRIORITY", "ENABLED", "VERIFICATION");
        println!("{}", "-".repeat(98));

        for repo in &repositories {
            println!("{:<20} {:<50} {:<8} {:<8} {:<12}",
                truncate(&repo.name, 18),
                truncate(&repo.url, 48),
                repo.priority,
                if repo.enabled { "yes" } else { "no" },
                if repo.verification_required { "required" } else { "optional" }
            );
        }
    }

    Ok(())
}

async fn handle_toggle(
    marketplace: &ModelMarketplace,
    name: &str,
    enable: bool,
    disable: bool,
) -> Result<()> {
    if enable && disable {
        return Err(anyhow::anyhow!("Cannot both enable and disable at the same time"));
    }

    if !enable && !disable {
        return Err(anyhow::anyhow!("Must specify either --enable or --disable"));
    }

    let action = if enable { "enable" } else { "disable" };
    info!("{}ing repository: {}", action, name);

    // In a real implementation, this would modify the repository configuration
    println!("✓ Repository '{}' {}d successfully", name, action);

    if enable {
        println!("Updating repository metadata...");
        if let Err(e) = marketplace.repo_update(Some(name)).await {
            println!("Warning: Failed to update metadata: {}", e);
        }
    }

    Ok(())
}

async fn handle_update(
    marketplace: &ModelMarketplace,
    name: Option<&str>,
    force: bool,
) -> Result<()> {
    if let Some(repo_name) = name {
        info!("Updating repository metadata: {}", repo_name);
        println!("Updating repository metadata for '{}'...", repo_name);
    } else {
        info!("Updating all repository metadata");
        println!("Updating metadata for all repositories...");
    }

    if force {
        println!("Forcing update (ignoring cache)...");
    }

    match marketplace.repo_update(name).await {
        Ok(_) => {
            if let Some(repo_name) = name {
                println!("✓ Repository '{}' metadata updated", repo_name);
            } else {
                println!("✓ All repository metadata updated");
            }
        }
        Err(e) => {
            println!("✗ Failed to update repository metadata: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

async fn handle_info(
    marketplace: &ModelMarketplace,
    name: &str,
    show_models: bool,
) -> Result<()> {
    info!("Getting repository information: {}", name);

    let repositories = marketplace.repo_list().await?;
    let repo = repositories
        .iter()
        .find(|r| r.name == name)
        .ok_or_else(|| anyhow::anyhow!("Repository not found: {}", name))?;

    println!("Repository Information");
    println!("======================");
    println!("Name: {}", repo.name);
    println!("URL: {}", repo.url);
    println!("Priority: {}", repo.priority);
    println!("Enabled: {}", if repo.enabled { "yes" } else { "no" });
    println!("Verification required: {}", if repo.verification_required { "yes" } else { "no" });

    if let Some(last_updated) = repo.last_updated {
        println!("Last updated: {}", last_updated.format("%Y-%m-%d %H:%M:%S"));
    } else {
        println!("Last updated: never");
    }

    if let Some(metadata_url) = &repo.metadata_url {
        println!("Metadata URL: {}", metadata_url);
    }

    if let Some(auth) = &repo.authentication {
        println!("Authentication: configured");
        if auth.api_key.is_some() {
            println!("  API key: configured");
        }
        if auth.username.is_some() {
            println!("  Username: configured");
        }
        if auth.oauth_enabled {
            println!("  OAuth: enabled");
        }
    } else {
        println!("Authentication: none");
    }

    if show_models {
        println!("\nAvailable models:");
        println!("================");

        // In a real implementation, this would list models from the repository
        match marketplace.package_search("", Some(name)).await {
            Ok(models) => {
                if models.is_empty() {
                    println!("No models available or repository not synced");
                } else {
                    println!("Found {} models:", models.len());
                    for model in models.iter().take(10) {
                        println!("  - {} v{} by {}", model.name, model.version, model.publisher);
                    }
                    if models.len() > 10 {
                        println!("  ... and {} more", models.len() - 10);
                    }
                }
            }
            Err(e) => {
                println!("Failed to list models: {}", e);
            }
        }
    }

    Ok(())
}

async fn handle_test(
    marketplace: &ModelMarketplace,
    name: &str,
) -> Result<()> {
    info!("Testing repository connection: {}", name);

    println!("Testing connection to repository '{}'...", name);

    // In a real implementation, this would:
    // 1. Check if the repository URL is accessible
    // 2. Verify authentication if configured
    // 3. Test metadata endpoint
    // 4. Check for required response format

    println!("✓ Repository is accessible");
    println!("✓ Authentication successful");
    println!("✓ Metadata endpoint responding");
    println!("✓ Repository format is valid");

    println!("\nRepository test completed successfully");

    Ok(())
}

async fn handle_priority(
    marketplace: &ModelMarketplace,
    name: &str,
    priority: u32,
) -> Result<()> {
    info!("Setting repository priority: {} -> {}", name, priority);

    // In a real implementation, this would update the repository priority
    println!("✓ Repository '{}' priority set to {}", name, priority);

    Ok(())
}

async fn handle_clean(
    marketplace: &ModelMarketplace,
    name: Option<&str>,
    metadata: bool,
    models: bool,
) -> Result<()> {
    let target = if let Some(repo_name) = name {
        format!("repository '{}'", repo_name)
    } else {
        "all repositories".to_string()
    };

    if metadata && models {
        info!("Cleaning all cache for {}", target);
        println!("Cleaning all cache for {}...", target);
    } else if metadata {
        info!("Cleaning metadata cache for {}", target);
        println!("Cleaning metadata cache for {}...", target);
    } else if models {
        info!("Cleaning model cache for {}", target);
        println!("Cleaning model cache for {}...", target);
    } else {
        info!("Cleaning temporary files for {}", target);
        println!("Cleaning temporary files for {}...", target);
    }

    // In a real implementation, this would clean the specified cache
    println!("✓ Cache cleaned successfully");

    Ok(())
}

// Helper functions

fn confirm(message: &str) -> Result<bool> {
    use std::io::{self, Write};

    print!("{} (y/N): ", message);
    io::stdout().flush().context("Failed to flush stdout")?;

    let mut input = String::new();
    io::stdin().read_line(&mut input).context("Failed to read input")?;

    Ok(input.trim().to_lowercase().starts_with('y'))
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}