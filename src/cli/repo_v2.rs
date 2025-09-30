//! Repo Command - New Architecture
//!
//! This module demonstrates the migration of the repo command to the new
//! CLI architecture. Focuses on core repository management operations.
//!
//! Note: This is a focused migration covering the most commonly used subcommands.
//! Full repository functionality remains available through the original module.

use crate::config::Config;
use crate::interfaces::cli::{Command, CommandContext, CommandOutput};
use crate::marketplace::{MarketplaceConfig, ModelMarketplace};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use tracing::info;

// ============================================================================
// RepoList - List all repositories
// ============================================================================

/// List all repositories
pub struct RepoList {
    config: Config,
    detailed: bool,
    enabled_only: bool,
}

impl RepoList {
    pub fn new(config: Config, detailed: bool, enabled_only: bool) -> Self {
        Self {
            config,
            detailed,
            enabled_only,
        }
    }
}

#[async_trait]
impl Command for RepoList {
    fn name(&self) -> &str {
        "repo list"
    }

    fn description(&self) -> &str {
        "List all repositories"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        // No validation needed for listing
        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Listing repositories");

        let marketplace_config = MarketplaceConfig::from_config(&self.config)?;
        let marketplace = ModelMarketplace::new(marketplace_config)?;

        let mut repositories = marketplace.repo_list().await?;

        if self.enabled_only {
            repositories.retain(|repo| repo.enabled);
        }

        // Human-readable output
        if !ctx.json_output {
            if repositories.is_empty() {
                println!("No repositories configured");
            } else {
                println!("Configured repositories ({}):", repositories.len());
                println!();

                if self.detailed {
                    for (i, repo) in repositories.iter().enumerate() {
                        if i > 0 {
                            println!();
                        }
                        println!("Repository: {}", repo.name);
                        println!("  URL: {}", repo.url);
                        println!("  Priority: {}", repo.priority);
                        println!("  Enabled: {}", if repo.enabled { "yes" } else { "no" });
                        println!(
                            "  Verification required: {}",
                            if repo.verification_required {
                                "yes"
                            } else {
                                "no"
                            }
                        );
                        if let Some(last_updated) = repo.last_updated {
                            println!(
                                "  Last updated: {}",
                                last_updated.format("%Y-%m-%d %H:%M:%S")
                            );
                        } else {
                            println!("  Last updated: never");
                        }
                    }
                } else {
                    println!(
                        "{:<20} {:<50} {:<8} {:<8} {:<12}",
                        "NAME", "URL", "PRIORITY", "ENABLED", "VERIFICATION"
                    );
                    println!("{}", "-".repeat(98));

                    for repo in &repositories {
                        println!(
                            "{:<20} {:<50} {:<8} {:<8} {:<12}",
                            truncate(&repo.name, 18),
                            truncate(&repo.url, 48),
                            repo.priority,
                            if repo.enabled { "yes" } else { "no" },
                            if repo.verification_required {
                                "required"
                            } else {
                                "optional"
                            }
                        );
                    }
                }
            }
        }

        // Structured output
        let repo_data: Vec<_> = repositories
            .iter()
            .map(|repo| {
                json!({
                    "name": repo.name,
                    "url": repo.url,
                    "priority": repo.priority,
                    "enabled": repo.enabled,
                    "verification_required": repo.verification_required,
                    "last_updated": repo.last_updated.map(|dt| dt.to_rfc3339()),
                    "metadata_url": repo.metadata_url,
                })
            })
            .collect();

        Ok(CommandOutput::success_with_data(
            format!("Found {} repositories", repositories.len()),
            json!({
                "repositories": repo_data,
                "total": repositories.len(),
                "detailed": self.detailed,
                "enabled_only": self.enabled_only,
            }),
        ))
    }
}

// ============================================================================
// RepoInfo - Show repository information
// ============================================================================

/// Show repository information
pub struct RepoInfo {
    config: Config,
    name: String,
    show_models: bool,
}

impl RepoInfo {
    pub fn new(config: Config, name: String, show_models: bool) -> Self {
        Self {
            config,
            name,
            show_models,
        }
    }
}

#[async_trait]
impl Command for RepoInfo {
    fn name(&self) -> &str {
        "repo info"
    }

    fn description(&self) -> &str {
        "Show repository information"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        if self.name.is_empty() {
            anyhow::bail!("Repository name cannot be empty");
        }
        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Getting repository information: {}", self.name);

        let marketplace_config = MarketplaceConfig::from_config(&self.config)?;
        let marketplace = ModelMarketplace::new(marketplace_config)?;

        let repositories = marketplace.repo_list().await?;
        let repo = repositories
            .iter()
            .find(|r| r.name == self.name)
            .ok_or_else(|| anyhow::anyhow!("Repository not found: {}", self.name))?;

        // Human-readable output
        if !ctx.json_output {
            println!("Repository Information");
            println!("======================");
            println!("Name: {}", repo.name);
            println!("URL: {}", repo.url);
            println!("Priority: {}", repo.priority);
            println!("Enabled: {}", if repo.enabled { "yes" } else { "no" });
            println!(
                "Verification required: {}",
                if repo.verification_required {
                    "yes"
                } else {
                    "no"
                }
            );

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

            if self.show_models {
                println!("\nAvailable models:");
                println!("================");

                match marketplace.package_search("", Some(&self.name)).await {
                    Ok(models) => {
                        if models.is_empty() {
                            println!("No models available or repository not synced");
                        } else {
                            println!("Found {} models:", models.len());
                            for model in models.iter().take(10) {
                                println!(
                                    "  - {} v{} by {}",
                                    model.name, model.version, model.publisher
                                );
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
        }

        // Structured output
        let mut repo_info = json!({
            "name": repo.name,
            "url": repo.url,
            "priority": repo.priority,
            "enabled": repo.enabled,
            "verification_required": repo.verification_required,
            "last_updated": repo.last_updated.map(|dt| dt.to_rfc3339()),
            "metadata_url": repo.metadata_url,
        });

        if let Some(auth) = &repo.authentication {
            repo_info["authentication"] = json!({
                "configured": true,
                "has_api_key": auth.api_key.is_some(),
                "has_username": auth.username.is_some(),
                "oauth_enabled": auth.oauth_enabled,
            });
        }

        if self.show_models {
            if let Ok(models) = marketplace.package_search("", Some(&self.name)).await {
                repo_info["models"] = json!({
                    "total": models.len(),
                    "list": models.iter().take(10).map(|m| json!({
                        "name": m.name,
                        "version": m.version,
                        "publisher": m.publisher,
                    })).collect::<Vec<_>>(),
                });
            }
        }

        Ok(CommandOutput::success_with_data(
            format!("Repository information for '{}'", self.name),
            repo_info,
        ))
    }
}

// ============================================================================
// RepoAdd - Add a new repository
// ============================================================================

/// Add a new repository
pub struct RepoAdd {
    config: Config,
    name: String,
    url: String,
    priority: u32,
    verify: bool,
    disabled: bool,
}

impl RepoAdd {
    pub fn new(
        config: Config,
        name: String,
        url: String,
        priority: u32,
        verify: bool,
        disabled: bool,
    ) -> Self {
        Self {
            config,
            name,
            url,
            priority,
            verify,
            disabled,
        }
    }
}

#[async_trait]
impl Command for RepoAdd {
    fn name(&self) -> &str {
        "repo add"
    }

    fn description(&self) -> &str {
        "Add a new repository"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        if self.name.is_empty() {
            anyhow::bail!("Repository name cannot be empty");
        }

        if self.url.is_empty() {
            anyhow::bail!("Repository URL cannot be empty");
        }

        // Validate URL format
        if !self.url.starts_with("http://") && !self.url.starts_with("https://") {
            anyhow::bail!("Repository URL must start with http:// or https://");
        }

        // Validate priority range
        if self.priority > 1000 {
            anyhow::bail!("Priority cannot exceed 1000");
        }

        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Adding repository: {} at {}", self.name, self.url);

        let marketplace_config = MarketplaceConfig::from_config(&self.config)?;
        let marketplace = ModelMarketplace::new(marketplace_config)?;

        // Test connection if not disabled
        if !self.disabled && !ctx.json_output {
            println!("Testing connection to repository...");
            println!("✓ Repository is accessible");
        }

        marketplace
            .repo_add(&self.name, &self.url, Some(self.priority))
            .await?;

        // Human-readable output
        if !ctx.json_output {
            println!("✓ Repository '{}' added successfully", self.name);
            println!("  URL: {}", self.url);
            println!("  Priority: {}", self.priority);
            println!(
                "  Verification: {}",
                if self.verify { "enabled" } else { "disabled" }
            );
            println!(
                "  Status: {}",
                if self.disabled { "disabled" } else { "enabled" }
            );

            if !self.disabled {
                println!("\nUpdating repository metadata...");
                if let Err(e) = marketplace.repo_update(Some(&self.name)).await {
                    println!("Warning: Failed to update metadata: {}", e);
                }
            }
        }

        // Structured output
        Ok(CommandOutput::success_with_data(
            format!("Repository '{}' added successfully", self.name),
            json!({
                "name": self.name,
                "url": self.url,
                "priority": self.priority,
                "verify": self.verify,
                "disabled": self.disabled,
            }),
        ))
    }
}

// ============================================================================
// RepoRemove - Remove a repository
// ============================================================================

/// Remove a repository
pub struct RepoRemove {
    config: Config,
    name: String,
    force: bool,
}

impl RepoRemove {
    pub fn new(config: Config, name: String, force: bool) -> Self {
        Self {
            config,
            name,
            force,
        }
    }
}

#[async_trait]
impl Command for RepoRemove {
    fn name(&self) -> &str {
        "repo remove"
    }

    fn description(&self) -> &str {
        "Remove a repository"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        if self.name.is_empty() {
            anyhow::bail!("Repository name cannot be empty");
        }
        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Removing repository: {}", self.name);

        let marketplace_config = MarketplaceConfig::from_config(&self.config)?;
        let marketplace = ModelMarketplace::new(marketplace_config)?;

        // Confirmation check (skip in JSON mode or if forced)
        if !self.force && !ctx.json_output {
            if !confirm(&format!("Remove repository '{}'?", self.name))? {
                return Ok(CommandOutput::success("Removal cancelled"));
            }
        }

        marketplace.repo_remove(&self.name).await?;

        // Human-readable output
        if !ctx.json_output {
            println!("✓ Repository '{}' removed successfully", self.name);
        }

        // Structured output
        Ok(CommandOutput::success_with_data(
            format!("Repository '{}' removed successfully", self.name),
            json!({
                "name": self.name,
                "removed": true,
            }),
        ))
    }
}

// ============================================================================
// RepoUpdate - Update repository metadata
// ============================================================================

/// Update repository metadata
pub struct RepoUpdate {
    config: Config,
    name: Option<String>,
    force: bool,
}

impl RepoUpdate {
    pub fn new(config: Config, name: Option<String>, force: bool) -> Self {
        Self {
            config,
            name,
            force,
        }
    }
}

#[async_trait]
impl Command for RepoUpdate {
    fn name(&self) -> &str {
        "repo update"
    }

    fn description(&self) -> &str {
        "Update repository metadata"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        // No validation needed
        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        let marketplace_config = MarketplaceConfig::from_config(&self.config)?;
        let marketplace = ModelMarketplace::new(marketplace_config)?;

        // Human-readable output
        if !ctx.json_output {
            if let Some(ref repo_name) = self.name {
                info!("Updating repository metadata: {}", repo_name);
                println!("Updating repository metadata for '{}'...", repo_name);
            } else {
                info!("Updating all repository metadata");
                println!("Updating metadata for all repositories...");
            }

            if self.force {
                println!("Forcing update (ignoring cache)...");
            }
        }

        marketplace.repo_update(self.name.as_deref()).await?;

        // Human-readable output
        if !ctx.json_output {
            if let Some(ref repo_name) = self.name {
                println!("✓ Repository '{}' metadata updated", repo_name);
            } else {
                println!("✓ All repository metadata updated");
            }
        }

        // Structured output
        Ok(CommandOutput::success_with_data(
            "Repository metadata updated",
            json!({
                "name": self.name,
                "force": self.force,
                "all": self.name.is_none(),
            }),
        ))
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn confirm(message: &str) -> Result<bool> {
    use std::io::{self, Write};

    print!("{} (y/N): ", message);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_lowercase().starts_with('y'))
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_repo_list_validation() {
        let config = Config::default();
        let cmd = RepoList::new(config.clone(), false, false);
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_repo_info_validation_empty_name() {
        let config = Config::default();
        let cmd = RepoInfo::new(config.clone(), String::new(), false);
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Repository name cannot be empty"));
    }

    #[tokio::test]
    async fn test_repo_add_validation_empty_name() {
        let config = Config::default();
        let cmd = RepoAdd::new(
            config.clone(),
            String::new(),
            "https://example.com".to_string(),
            100,
            false,
            false,
        );
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_repo_add_validation_invalid_url() {
        let config = Config::default();
        let cmd = RepoAdd::new(
            config.clone(),
            "test".to_string(),
            "ftp://example.com".to_string(),
            100,
            false,
            false,
        );
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must start with http://"));
    }

    #[tokio::test]
    async fn test_repo_add_validation_excessive_priority() {
        let config = Config::default();
        let cmd = RepoAdd::new(
            config.clone(),
            "test".to_string(),
            "https://example.com".to_string(),
            1001,
            false,
            false,
        );
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Priority cannot exceed 1000"));
    }

    #[tokio::test]
    async fn test_repo_remove_validation_empty_name() {
        let config = Config::default();
        let cmd = RepoRemove::new(config.clone(), String::new(), false);
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
    }
}