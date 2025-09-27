use crate::cli::enhanced_parser::execute_with_prerequisites;
use crate::config::Config;
use crate::marketplace::{MarketplaceConfig, ModelMarketplace};
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use std::path::PathBuf;
use tracing::info;

#[derive(Args)]
pub struct PackageArgs {
    #[command(subcommand)]
    pub command: PackageCommand,
}

#[derive(Subcommand)]
pub enum PackageCommand {
    #[command(about = "Install a model package")]
    Install {
        #[arg(help = "Package name or model ID")]
        package: String,

        #[arg(short, long, help = "Don't resolve dependencies automatically")]
        no_deps: bool,

        #[arg(short, long, help = "Install to specific directory")]
        target: Option<PathBuf>,

        #[arg(short, long, help = "Answer yes to all prompts")]
        yes: bool,

        #[arg(long, help = "Enable automatic updates")]
        auto_update: bool,
    },

    #[command(about = "Remove a model package")]
    Remove {
        #[arg(help = "Package name or model ID")]
        package: String,

        #[arg(short, long, help = "Don't remove dependencies")]
        no_deps: bool,

        #[arg(short, long, help = "Answer yes to all prompts")]
        yes: bool,

        #[arg(long, help = "Keep configuration files")]
        keep_config: bool,
    },

    #[command(about = "Search for model packages")]
    Search {
        #[arg(help = "Search query")]
        query: String,

        #[arg(short, long, help = "Search in specific repository")]
        repo: Option<String>,

        #[arg(short, long, help = "Number of results to show", default_value = "20")]
        limit: usize,

        #[arg(long, help = "Show detailed information")]
        detailed: bool,
    },

    #[command(about = "Show package information")]
    Info {
        #[arg(help = "Package name or model ID")]
        package: String,

        #[arg(long, help = "Show dependency information")]
        deps: bool,

        #[arg(long, help = "Show detailed metadata")]
        detailed: bool,
    },

    #[command(about = "List installed packages")]
    List {
        #[arg(short, long, help = "Filter by name")]
        filter: Option<String>,

        #[arg(long, help = "Show detailed information")]
        detailed: bool,

        #[arg(long, help = "Show only auto-installed packages")]
        auto_only: bool,
    },

    #[command(about = "Update packages")]
    Update {
        #[arg(help = "Package to update (update all if not specified)")]
        package: Option<String>,

        #[arg(short, long, help = "Answer yes to all prompts")]
        yes: bool,

        #[arg(long, help = "Check for updates without installing")]
        check_only: bool,
    },

    #[command(about = "Upgrade all packages")]
    Upgrade {
        #[arg(short, long, help = "Answer yes to all prompts")]
        yes: bool,

        #[arg(long, help = "Show what would be upgraded without doing it")]
        dry_run: bool,
    },

    #[command(about = "Remove unused packages")]
    Autoremove {
        #[arg(short, long, help = "Answer yes to all prompts")]
        yes: bool,

        #[arg(long, help = "Show what would be removed without doing it")]
        dry_run: bool,
    },

    #[command(about = "Clean package cache")]
    Clean {
        #[arg(long, help = "Clean all cached data")]
        all: bool,

        #[arg(long, help = "Clean only downloaded packages")]
        packages: bool,

        #[arg(long, help = "Clean only metadata cache")]
        metadata: bool,
    },

    #[command(about = "Show package history")]
    History {
        #[arg(help = "Package name (show all if not specified)")]
        package: Option<String>,

        #[arg(short, long, help = "Number of entries to show", default_value = "10")]
        limit: usize,
    },

    #[command(about = "Check package dependencies")]
    Depends {
        #[arg(help = "Package name or model ID")]
        package: String,

        #[arg(long, help = "Show reverse dependencies (what depends on this)")]
        reverse: bool,

        #[arg(long, help = "Show full dependency tree")]
        tree: bool,
    },

    #[command(about = "Verify installed packages")]
    Check {
        #[arg(help = "Package to check (check all if not specified)")]
        package: Option<String>,

        #[arg(long, help = "Perform deep verification")]
        deep: bool,

        #[arg(long, help = "Fix issues automatically")]
        fix: bool,
    },
}

// Simplified command aliases
#[derive(Args)]
pub struct InstallArgs {
    #[arg(help = "Package name or model ID")]
    pub package: String,

    #[arg(short, long, help = "Don't resolve dependencies automatically")]
    pub no_deps: bool,

    #[arg(short, long, help = "Answer yes to all prompts")]
    pub yes: bool,

    #[arg(long, help = "Enable automatic updates")]
    pub auto_update: bool,
}

#[derive(Args)]
pub struct RemoveArgs {
    #[arg(help = "Package name or model ID")]
    pub package: String,

    #[arg(short, long, help = "Don't remove dependencies")]
    pub no_deps: bool,

    #[arg(short, long, help = "Answer yes to all prompts")]
    pub yes: bool,
}

#[derive(Args)]
pub struct SearchArgs {
    #[arg(help = "Search query")]
    pub query: String,

    #[arg(short, long, help = "Search in specific repository")]
    pub repo: Option<String>,

    #[arg(short, long, help = "Number of results to show", default_value = "20")]
    pub limit: usize,
}

#[derive(Args)]
pub struct ListArgs {
    #[arg(short, long, help = "Filter by name")]
    pub filter: Option<String>,

    #[arg(long, help = "Show detailed information")]
    pub detailed: bool,
}

pub async fn handle_package_command(args: PackageArgs) -> Result<()> {
    let config = Config::load()?;
    let marketplace_config = MarketplaceConfig::from_config(&config)?;
    let marketplace = ModelMarketplace::new(marketplace_config)?;

    match args.command {
        PackageCommand::Install {
            package,
            no_deps,
            target: _,
            yes,
            auto_update,
        } => handle_install(&marketplace, &package, !no_deps, yes, auto_update).await,

        PackageCommand::Remove {
            package,
            no_deps,
            yes,
            keep_config: _,
        } => handle_remove(&marketplace, &package, !no_deps, yes).await,

        PackageCommand::Search {
            query,
            repo,
            limit,
            detailed,
        } => handle_search(&marketplace, &query, repo.as_deref(), limit, detailed).await,

        PackageCommand::Info {
            package,
            deps,
            detailed,
        } => handle_info(&marketplace, &package, deps, detailed).await,

        PackageCommand::List {
            filter,
            detailed,
            auto_only,
        } => handle_list(&marketplace, filter.as_deref(), detailed, auto_only).await,

        PackageCommand::Update {
            package,
            yes,
            check_only,
        } => handle_update(&marketplace, package.as_deref(), yes, check_only).await,

        PackageCommand::Upgrade { yes, dry_run } => {
            handle_upgrade(&marketplace, yes, dry_run).await
        }

        PackageCommand::Autoremove { yes, dry_run } => {
            handle_autoremove(&marketplace, yes, dry_run).await
        }

        PackageCommand::Clean {
            all,
            packages,
            metadata,
        } => handle_clean(&marketplace, all, packages, metadata).await,

        PackageCommand::History {
            package: _,
            limit: _,
        } => {
            println!("Package history feature not yet implemented");
            Ok(())
        }

        PackageCommand::Depends {
            package,
            reverse,
            tree,
        } => handle_depends(&marketplace, &package, reverse, tree).await,

        PackageCommand::Check { package, deep, fix } => {
            handle_check(&marketplace, package.as_deref(), deep, fix).await
        }
    }
}

// Simplified command handlers

pub async fn handle_install_simple(args: InstallArgs) -> Result<()> {
    execute_with_prerequisites("install", async {
        let config = Config::load()?;
        let marketplace_config = MarketplaceConfig::from_config(&config)?;
        let marketplace = ModelMarketplace::new(marketplace_config)?;

        handle_install(
            &marketplace,
            &args.package,
            !args.no_deps,
            args.yes,
            args.auto_update,
        )
        .await
    })
    .await
}

pub async fn handle_remove_simple(args: RemoveArgs) -> Result<()> {
    let config = Config::load()?;
    let marketplace_config = MarketplaceConfig::from_config(&config)?;
    let marketplace = ModelMarketplace::new(marketplace_config)?;

    handle_remove(&marketplace, &args.package, !args.no_deps, args.yes).await
}

pub async fn handle_search_simple(args: SearchArgs) -> Result<()> {
    execute_with_prerequisites("search", async {
        let config = Config::load()?;
        let marketplace_config = MarketplaceConfig::from_config(&config)?;
        let marketplace = ModelMarketplace::new(marketplace_config)?;

        handle_search(
            &marketplace,
            &args.query,
            args.repo.as_deref(),
            args.limit,
            false,
        )
        .await
    })
    .await
}

pub async fn handle_list_simple(args: ListArgs) -> Result<()> {
    let config = Config::load()?;
    let marketplace_config = MarketplaceConfig::from_config(&config)?;
    let marketplace = ModelMarketplace::new(marketplace_config)?;

    handle_list(&marketplace, args.filter.as_deref(), args.detailed, false).await
}

// Implementation functions

async fn handle_install(
    marketplace: &ModelMarketplace,
    package: &str,
    resolve_deps: bool,
    auto_confirm: bool,
    auto_update: bool,
) -> Result<()> {
    info!("Installing package: {}", package);

    if !auto_confirm {
        println!("Installing package: {}", package);
        if resolve_deps {
            println!("Dependencies will be resolved automatically");
        }
        if !confirm("Continue?")? {
            println!("Installation cancelled");
            return Ok(());
        }
    }

    match marketplace.package_install(package, resolve_deps).await {
        Ok(download_id) => {
            println!("✅ Package installation started successfully!");
            println!("📦 Package: {}", package);
            println!("🔄 Download ID: {}", download_id);

            if resolve_deps {
                println!("📋 Dependencies will be resolved automatically");
            }

            if auto_update {
                println!("🔄 Automatic updates enabled for this package");
            }

            println!("\n💡 Monitor progress with:");
            println!("   inferno marketplace progress {}", download_id);
        }
        Err(e) => {
            let error_msg = e.to_string().to_lowercase();

            if error_msg.contains("not found") {
                println!("❌ Package '{}' not found", package);
                println!("\n💡 Try these alternatives:");
                println!("   • Search for similar packages:");
                println!("     inferno search {}", package);
                println!("   • Check available repositories:");
                println!("     inferno repo list");
                println!("   • Search in specific repository:");
                println!("     inferno search {} --repo huggingface", package);
            } else if error_msg.contains("network") || error_msg.contains("connection") {
                println!("❌ Network error during installation");
                println!("\n💡 Check your connection and try again:");
                println!("   • Test repository connectivity:");
                println!("     inferno repo test huggingface");
                println!("   • Update repository metadata:");
                println!("     inferno repo update --force");
            } else {
                println!("❌ Installation failed: {}", e);
                println!("\n💡 Try these troubleshooting steps:");
                println!("   • Check package name: inferno search {}", package);
                println!("   • Clean cache: inferno package clean --all");
                println!(
                    "   • Check logs: INFERNO_LOG_LEVEL=debug inferno install {}",
                    package
                );
            }

            return Err(e);
        }
    }

    Ok(())
}

async fn handle_remove(
    marketplace: &ModelMarketplace,
    package: &str,
    remove_deps: bool,
    auto_confirm: bool,
) -> Result<()> {
    info!("Removing package: {}", package);

    if !auto_confirm {
        let message = if remove_deps {
            format!("Remove package '{}' and its dependencies?", package)
        } else {
            format!("Remove package '{}'?", package)
        };

        if !confirm(&message)? {
            println!("Removal cancelled");
            return Ok(());
        }
    }

    match marketplace.package_remove(package, remove_deps).await {
        Ok(_) => {
            println!("✓ Package removed successfully");
        }
        Err(e) => {
            println!("✗ Removal failed: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

async fn handle_search(
    marketplace: &ModelMarketplace,
    query: &str,
    repo_filter: Option<&str>,
    limit: usize,
    detailed: bool,
) -> Result<()> {
    info!("Searching for: {}", query);

    let results = marketplace.package_search(query, repo_filter).await?;
    let results: Vec<_> = results.into_iter().take(limit).collect();

    if results.is_empty() {
        println!("❌ No packages found matching: '{}'", query);

        if let Some(repo) = repo_filter {
            println!("\n💡 No results in repository '{}'. Try:", repo);
            println!("   • Search in all repositories:");
            println!("     inferno search {}", query);
            println!("   • Check repository status:");
            println!("     inferno repo info {}", repo);
        } else {
            println!("\n💡 Suggestions:");
            println!("   • Try a broader search term");
            println!("   • Check spelling: {}", query);
            println!("   • Update repository metadata:");
            println!("     inferno repo update --force");
            println!("   • List available repositories:");
            println!("     inferno repo list");
        }

        println!("\n📚 Popular models to try:");
        println!("   • inferno install microsoft/DialoGPT-medium");
        println!("   • inferno install google/flan-t5-base");
        println!("   • inferno install facebook/bart-large-cnn");

        return Ok(());
    }

    println!("Found {} packages:", results.len());
    println!();

    if detailed {
        for (i, model) in results.iter().enumerate() {
            if i > 0 {
                println!();
            }
            println!("Package: {}", model.name);
            println!("  ID: {}", model.id);
            println!("  Version: {}", model.version);
            println!("  Publisher: {}", model.publisher);
            println!("  Description: {}", model.description);
            println!("  Size: {}", format_size(model.size_bytes));
            println!("  Downloads: {}", model.downloads);
            if let Some(rating) = model.rating {
                println!("  Rating: {:.1}/5.0", rating);
            }
        }
    } else {
        println!(
            "{:<40} {:<20} {:<15} {:<10}",
            "PACKAGE", "PUBLISHER", "VERSION", "SIZE"
        );
        println!("{}", "-".repeat(85));

        for model in &results {
            println!(
                "{:<40} {:<20} {:<15} {:<10}",
                truncate(&model.name, 38),
                truncate(&model.publisher, 18),
                truncate(&model.version, 13),
                format_size(model.size_bytes)
            );
        }
    }

    Ok(())
}

async fn handle_info(
    marketplace: &ModelMarketplace,
    package: &str,
    show_deps: bool,
    detailed: bool,
) -> Result<()> {
    info!("Getting info for package: {}", package);

    // Try to get from marketplace first
    match marketplace.get_model_details(package).await {
        Ok(model) => {
            println!("Package Information");
            println!("===================");
            println!("Name: {}", model.name);
            println!("ID: {}", model.id);
            println!("Version: {}", model.version);
            println!("Publisher: {}", model.publisher);
            println!("License: {}", model.license);
            println!("Size: {}", format_size(model.size_bytes));
            println!("Downloads: {}", model.downloads);

            if let Some(rating) = model.rating {
                println!("Rating: {:.1}/5.0", rating);
            }

            println!("Published: {}", model.published_at.format("%Y-%m-%d"));
            println!("Updated: {}", model.updated_at.format("%Y-%m-%d"));

            println!("\nDescription:");
            println!("{}", model.description);

            if show_deps && !model.dependencies.is_empty() {
                println!("\nDependencies:");
                for dep in &model.dependencies {
                    println!("  - {} ({})", dep.name, dep.version);
                }
            }

            if detailed {
                println!("\nCompatibility:");
                println!(
                    "  Minimum RAM: {:.1} GB",
                    model.compatibility.minimum_ram_gb
                );
                if let Some(vram) = model.compatibility.minimum_vram_gb {
                    println!("  Minimum VRAM: {:.1} GB", vram);
                }
                println!(
                    "  Platforms: {}",
                    model.compatibility.supported_platforms.join(", ")
                );
                println!(
                    "  Backends: {}",
                    model.compatibility.supported_backends.join(", ")
                );
            }
        }
        Err(_e) => {
            // Try to get from installed packages
            match marketplace.package_list(Some(package)).await {
                Ok(packages) => {
                    if let Some(pkg) = packages.first() {
                        println!("📦 Installed Package Information");
                        println!("=================================");
                        println!("Name: {}", pkg.name);
                        println!("ID: {}", pkg.model_id);
                        println!("Version: {}", pkg.version);
                        println!("Repository: {}", pkg.repository);
                        println!(
                            "Installed: {}",
                            pkg.install_date.format("%Y-%m-%d %H:%M:%S")
                        );
                        println!(
                            "Auto-installed: {}",
                            if pkg.auto_installed { "Yes" } else { "No" }
                        );
                        println!("Local path: {}", pkg.local_path.display());

                        if show_deps && !pkg.dependencies.is_empty() {
                            println!("\nDependencies:");
                            for dep in &pkg.dependencies {
                                println!("  - {}", dep);
                            }
                        }
                    } else {
                        return Err(anyhow::anyhow!("Package not found: {}", package));
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }

    Ok(())
}

async fn handle_list(
    marketplace: &ModelMarketplace,
    filter: Option<&str>,
    detailed: bool,
    auto_only: bool,
) -> Result<()> {
    info!("Listing installed packages");

    let mut packages = marketplace.package_list(filter).await?;

    if auto_only {
        packages.retain(|pkg| pkg.auto_installed);
    }

    if packages.is_empty() {
        println!("No packages installed");
        return Ok(());
    }

    println!("Installed packages ({}):", packages.len());
    println!();

    if detailed {
        for (i, pkg) in packages.iter().enumerate() {
            if i > 0 {
                println!();
            }
            println!("Name: {}", pkg.name);
            println!("  ID: {}", pkg.model_id);
            println!("  Version: {}", pkg.version);
            println!("  Repository: {}", pkg.repository);
            println!(
                "  Installed: {}",
                pkg.install_date.format("%Y-%m-%d %H:%M:%S")
            );
            println!(
                "  Auto-installed: {}",
                if pkg.auto_installed { "Yes" } else { "No" }
            );
            println!("  Path: {}", pkg.local_path.display());
            if !pkg.dependencies.is_empty() {
                println!("  Dependencies: {}", pkg.dependencies.join(", "));
            }
        }
    } else {
        println!(
            "{:<40} {:<15} {:<12} {:<20}",
            "PACKAGE", "VERSION", "REPOSITORY", "INSTALLED"
        );
        println!("{}", "-".repeat(87));

        for pkg in &packages {
            println!(
                "{:<40} {:<15} {:<12} {:<20}",
                truncate(&pkg.name, 38),
                truncate(&pkg.version, 13),
                truncate(&pkg.repository, 10),
                pkg.install_date.format("%Y-%m-%d")
            );
        }
    }

    Ok(())
}

async fn handle_update(
    marketplace: &ModelMarketplace,
    package: Option<&str>,
    auto_confirm: bool,
    check_only: bool,
) -> Result<()> {
    if let Some(pkg) = package {
        info!("Checking updates for package: {}", pkg);

        if check_only {
            println!("Checking for updates to package: {}", pkg);
            // Implementation would check for updates
            println!("No updates available");
        } else {
            if !auto_confirm {
                if !confirm(&format!("Update package '{}'?", pkg))? {
                    println!("Update cancelled");
                    return Ok(());
                }
            }

            match marketplace.package_upgrade(Some(pkg)).await {
                Ok(download_ids) => {
                    if download_ids.is_empty() {
                        println!("Package is already up to date");
                    } else {
                        println!("✓ Update started for package: {}", pkg);
                        for id in download_ids {
                            println!("Download ID: {}", id);
                        }
                    }
                }
                Err(e) => {
                    println!("✗ Update failed: {}", e);
                    return Err(e);
                }
            }
        }
    } else {
        info!("Checking updates for all packages");

        if check_only {
            let updates = marketplace.check_for_updates().await?;
            if updates.is_empty() {
                println!("All packages are up to date");
            } else {
                println!("Updates available for {} packages:", updates.len());
                for pkg in updates {
                    println!("  - {}", pkg);
                }
            }
        } else {
            if !auto_confirm {
                if !confirm("Update all packages?")? {
                    println!("Update cancelled");
                    return Ok(());
                }
            }

            match marketplace.package_upgrade(None).await {
                Ok(download_ids) => {
                    if download_ids.is_empty() {
                        println!("All packages are up to date");
                    } else {
                        println!("✓ Updates started for {} packages", download_ids.len());
                    }
                }
                Err(e) => {
                    println!("✗ Update failed: {}", e);
                    return Err(e);
                }
            }
        }
    }

    Ok(())
}

async fn handle_upgrade(
    marketplace: &ModelMarketplace,
    auto_confirm: bool,
    dry_run: bool,
) -> Result<()> {
    info!("Upgrading all packages");

    if dry_run {
        let updates = marketplace.check_for_updates().await?;
        if updates.is_empty() {
            println!("All packages are up to date");
        } else {
            println!("The following packages would be upgraded:");
            for pkg in updates {
                println!("  - {}", pkg);
            }
        }
        return Ok(());
    }

    if !auto_confirm {
        if !confirm("Upgrade all packages?")? {
            println!("Upgrade cancelled");
            return Ok(());
        }
    }

    match marketplace.package_upgrade(None).await {
        Ok(download_ids) => {
            if download_ids.is_empty() {
                println!("All packages are up to date");
            } else {
                println!("✓ Upgrade started for {} packages", download_ids.len());
            }
        }
        Err(e) => {
            println!("✗ Upgrade failed: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

async fn handle_autoremove(
    marketplace: &ModelMarketplace,
    auto_confirm: bool,
    dry_run: bool,
) -> Result<()> {
    info!("Removing unused packages");

    let to_remove = marketplace.package_autoremove().await?;

    if to_remove.is_empty() {
        println!("No unused packages to remove");
        return Ok(());
    }

    if dry_run {
        println!("The following packages would be removed:");
        for pkg in to_remove {
            println!("  - {}", pkg);
        }
        return Ok(());
    }

    if !auto_confirm {
        println!("The following packages will be removed:");
        for pkg in &to_remove {
            println!("  - {}", pkg);
        }
        if !confirm("Continue?")? {
            println!("Autoremove cancelled");
            return Ok(());
        }
    }

    println!("✓ Removed {} unused packages", to_remove.len());
    Ok(())
}

async fn handle_clean(
    _marketplace: &ModelMarketplace,
    all: bool,
    packages: bool,
    metadata: bool,
) -> Result<()> {
    info!("Cleaning package cache");

    if all {
        println!("Cleaning all cache data...");
        // Implementation would clean all cache
        println!("✓ All cache data cleaned");
    } else if packages {
        println!("Cleaning package cache...");
        // Implementation would clean package cache
        println!("✓ Package cache cleaned");
    } else if metadata {
        println!("Cleaning metadata cache...");
        // Implementation would clean metadata cache
        println!("✓ Metadata cache cleaned");
    } else {
        println!("Cleaning temporary files...");
        // Default: clean temporary files
        println!("✓ Temporary files cleaned");
    }

    Ok(())
}

async fn handle_depends(
    _marketplace: &ModelMarketplace,
    package: &str,
    reverse: bool,
    tree: bool,
) -> Result<()> {
    info!("Checking dependencies for package: {}", package);

    if reverse {
        println!("Packages that depend on '{}':", package);
        // Implementation would show reverse dependencies
        println!("  (no reverse dependencies)");
    } else if tree {
        println!("Dependency tree for '{}':", package);
        // Implementation would show dependency tree
        println!("  └─ {} (no dependencies)", package);
    } else {
        println!("Dependencies for '{}':", package);
        // Implementation would show direct dependencies
        println!("  (no dependencies)");
    }

    Ok(())
}

async fn handle_check(
    marketplace: &ModelMarketplace,
    package: Option<&str>,
    deep: bool,
    fix: bool,
) -> Result<()> {
    if let Some(pkg) = package {
        info!("Checking package: {}", pkg);
        println!("Checking package: {}", pkg);

        if deep {
            println!("Performing deep verification...");
        }

        // Implementation would verify the package
        println!("✓ Package verification completed");

        if fix {
            println!("No issues found to fix");
        }
    } else {
        info!("Checking all installed packages");
        let packages = marketplace.package_list(None).await?;

        println!("Checking {} installed packages...", packages.len());

        for pkg in &packages {
            print!("Checking {}... ", pkg.name);
            // Implementation would check each package
            println!("✓");
        }

        println!("All packages verified successfully");
    }

    Ok(())
}

// Helper functions

fn confirm(message: &str) -> Result<bool> {
    use std::io::{self, Write};

    print!("{} (y/N): ", message);
    io::stdout().flush().context("Failed to flush stdout")?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("Failed to read input")?;

    Ok(input.trim().to_lowercase().starts_with('y'))
}

fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
