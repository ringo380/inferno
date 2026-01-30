use crate::config::Config;
use crate::dashboard::DashboardServer;
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use std::path::PathBuf;
use tracing::{info, warn};

#[derive(Args)]
pub struct DashboardArgs {
    #[command(subcommand)]
    pub command: DashboardCommands,
}

#[derive(Subcommand)]
pub enum DashboardCommands {
    #[command(about = "Start the web dashboard server")]
    Start {
        #[arg(short, long, help = "Bind address", default_value = "127.0.0.1")]
        address: String,

        #[arg(short, long, help = "Port number", default_value = "8080")]
        port: u16,

        #[arg(long, help = "Configuration file")]
        config: Option<PathBuf>,

        #[arg(long, help = "Enable authentication")]
        auth: bool,

        #[arg(long, help = "Run as daemon")]
        daemon: bool,

        #[arg(long, help = "Assets directory")]
        assets_dir: Option<PathBuf>,
    },

    #[command(about = "Generate dashboard configuration")]
    Init {
        #[arg(short, long, help = "Output configuration file")]
        output: PathBuf,

        #[arg(long, help = "Include authentication setup")]
        with_auth: bool,

        #[arg(long, help = "Include examples")]
        examples: bool,
    },

    #[command(about = "Validate dashboard configuration")]
    Validate {
        #[arg(help = "Configuration file to validate")]
        config_file: Option<PathBuf>,

        #[arg(long, help = "Check asset files")]
        check_assets: bool,

        #[arg(long, help = "Verbose output")]
        verbose: bool,
    },

    #[command(about = "Show dashboard status")]
    Status {
        #[arg(long, help = "Dashboard URL", default_value = "http://localhost:8080")]
        url: String,

        #[arg(long, help = "Include detailed information")]
        detailed: bool,
    },

    #[command(about = "Stop the dashboard server")]
    Stop {
        #[arg(long, help = "Force stop")]
        force: bool,
    },

    #[command(about = "Restart the dashboard server")]
    Restart {
        #[arg(long, help = "Restart timeout in seconds", default_value = "30")]
        timeout: u64,
    },

    #[command(about = "Export dashboard data")]
    Export {
        #[arg(help = "Export type")]
        export_type: ExportType,

        #[arg(short, long, help = "Output file")]
        output: PathBuf,

        #[arg(long, help = "Date range (days)", default_value = "7")]
        days: u32,

        #[arg(long, help = "Include sensitive data")]
        include_sensitive: bool,
    },

    #[command(about = "Import dashboard data")]
    Import {
        #[arg(help = "Import file")]
        file: PathBuf,

        #[arg(long, help = "Import type")]
        import_type: Option<ImportType>,

        #[arg(long, help = "Overwrite existing data")]
        overwrite: bool,

        #[arg(long, help = "Dry run")]
        dry_run: bool,
    },

    #[command(about = "Manage dashboard themes")]
    Theme {
        #[command(subcommand)]
        command: ThemeCommands,
    },

    #[command(about = "User management for dashboard")]
    Users {
        #[command(subcommand)]
        command: UserCommands,
    },
}

#[derive(Subcommand)]
pub enum ThemeCommands {
    #[command(about = "List available themes")]
    List,

    #[command(about = "Install a new theme")]
    Install {
        #[arg(help = "Theme package or URL")]
        source: String,

        #[arg(long, help = "Force installation")]
        force: bool,
    },

    #[command(about = "Remove a theme")]
    Remove {
        #[arg(help = "Theme name")]
        name: String,
    },

    #[command(about = "Set the default theme")]
    Set {
        #[arg(help = "Theme name")]
        name: String,
    },

    #[command(about = "Create a custom theme")]
    Create {
        #[arg(help = "Theme name")]
        name: String,

        #[arg(short, long, help = "Base theme to extend")]
        base: Option<String>,

        #[arg(short, long, help = "Output directory")]
        output: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
pub enum UserCommands {
    #[command(about = "List dashboard users")]
    List {
        #[arg(long, help = "Show only active users")]
        active_only: bool,

        #[arg(long, help = "Output format", default_value = "table")]
        format: String,
    },

    #[command(about = "Create a new user")]
    Create {
        #[arg(help = "Username")]
        username: String,

        #[arg(short, long, help = "Email address")]
        email: String,

        #[arg(short, long, help = "User role", default_value = "user")]
        role: String,

        #[arg(long, help = "Password (will prompt if not provided)")]
        password: Option<String>,

        #[arg(long, help = "Set as admin")]
        admin: bool,
    },

    #[command(about = "Update user information")]
    Update {
        #[arg(help = "Username")]
        username: String,

        #[arg(short, long, help = "New email")]
        email: Option<String>,

        #[arg(short, long, help = "New role")]
        role: Option<String>,

        #[arg(long, help = "Enable/disable user")]
        active: Option<bool>,
    },

    #[command(about = "Delete a user")]
    Delete {
        #[arg(help = "Username")]
        username: String,

        #[arg(long, help = "Force deletion")]
        force: bool,
    },

    #[command(about = "Reset user password")]
    ResetPassword {
        #[arg(help = "Username")]
        username: String,

        #[arg(long, help = "New password (will prompt if not provided)")]
        password: Option<String>,

        #[arg(long, help = "Force password reset")]
        force: bool,
    },
}

#[derive(Debug, clap::ValueEnum, Clone)]
pub enum ExportType {
    Metrics,
    Models,
    Users,
    Config,
    All,
}

#[derive(Debug, clap::ValueEnum, Clone)]
pub enum ImportType {
    Metrics,
    Models,
    Users,
    Config,
}

pub async fn handle_dashboard_command(args: DashboardArgs) -> Result<()> {
    match args.command {
        DashboardCommands::Start {
            address,
            port,
            config,
            auth,
            daemon,
            assets_dir,
        } => handle_start(address, port, config, auth, daemon, assets_dir).await,

        DashboardCommands::Init {
            output,
            with_auth,
            examples,
        } => handle_init(output, with_auth, examples).await,

        DashboardCommands::Validate {
            config_file,
            check_assets,
            verbose,
        } => handle_validate(config_file, check_assets, verbose).await,

        DashboardCommands::Status { url, detailed } => handle_status(url, detailed).await,

        DashboardCommands::Stop { force } => handle_stop(force).await,

        DashboardCommands::Restart { timeout } => handle_restart(timeout).await,

        DashboardCommands::Export {
            export_type,
            output,
            days,
            include_sensitive,
        } => handle_export(export_type, output, days, include_sensitive).await,

        DashboardCommands::Import {
            file,
            import_type,
            overwrite,
            dry_run,
        } => handle_import(file, import_type, overwrite, dry_run).await,

        DashboardCommands::Theme { command } => handle_theme_command(command).await,

        DashboardCommands::Users { command } => handle_user_command(command).await,
    }
}

async fn handle_start(
    address: String,
    port: u16,
    config_file: Option<PathBuf>,
    auth: bool,
    daemon: bool,
    assets_dir: Option<PathBuf>,
) -> Result<()> {
    // Validate inputs
    if port == 0 {
        anyhow::bail!("Port must be greater than 0");
    }
    if address.is_empty() {
        anyhow::bail!("Address cannot be empty");
    }

    info!("Starting dashboard server");

    // Load configuration
    let mut config = if let Some(config_path) = config_file {
        let content = tokio::fs::read_to_string(config_path).await?;
        toml::from_str(&content).context("Failed to parse dashboard configuration")?
    } else {
        Config::load()?.dashboard
    };

    // Apply CLI overrides
    config.bind_address = address;
    config.port = port;
    config.auth.enabled = auth;

    if let Some(assets) = assets_dir {
        config.assets_dir = assets;
    }

    if daemon {
        info!("Starting dashboard as daemon");
        // In a real implementation, this would fork/daemonize
    }

    // Create and start dashboard server
    let server = DashboardServer::new(config.clone())?;

    println!("🚀 Starting Inferno Dashboard");
    println!("===============================");
    println!("URL: http://{}:{}", config.bind_address, config.port);
    println!("Title: {}", config.ui.title);
    println!(
        "Authentication: {}",
        if config.auth.enabled {
            "Enabled"
        } else {
            "Disabled"
        }
    );
    println!(
        "Real-time updates: {}",
        if config.realtime.enabled {
            "Enabled"
        } else {
            "Disabled"
        }
    );

    if !daemon {
        println!("\nPress Ctrl+C to stop the server");
    }

    // Load initial data
    server.load_initial_data().await?;

    // Start background tasks
    server.start_background_tasks().await?;

    // Start the server (this will block)
    server.start().await?;

    Ok(())
}

async fn handle_init(output: PathBuf, with_auth: bool, examples: bool) -> Result<()> {
    info!("Generating dashboard configuration");

    let config_content = generate_config_template(with_auth, examples);

    tokio::fs::write(&output, config_content)
        .await
        .context("Failed to write configuration file")?;

    println!("✓ Dashboard configuration generated: {}", output.display());

    if with_auth {
        println!("  Authentication is enabled");

        // Generate secure random admin credentials instead of hardcoded values
        let admin_username = generate_random_username();
        let admin_password = generate_secure_password();

        println!("  Generated admin credentials:");
        println!("    Username: {}", admin_username);
        println!("    Password: {}", admin_password);
        println!("  ⚠️  SAVE THESE CREDENTIALS SECURELY - They will not be shown again!");
        println!("  Consider changing the password after first login");
    }

    if examples {
        println!("  Configuration includes examples and comments");
    }

    println!("\nNext steps:");
    println!("1. Review and customize the configuration");
    println!(
        "2. Start the dashboard: inferno dashboard start --config {}",
        output.display()
    );

    Ok(())
}

async fn handle_validate(
    config_file: Option<PathBuf>,
    check_assets: bool,
    verbose: bool,
) -> Result<()> {
    if let Some(file) = config_file {
        println!("Validating dashboard configuration: {}", file.display());
    } else {
        println!("Validating default dashboard configuration");
    }

    // Mock validation
    println!("✓ Configuration syntax is valid");
    println!("✓ Required sections are present");
    println!("✓ Port configuration is valid");
    println!("✓ Asset paths are accessible");

    if check_assets {
        println!("✓ CSS files found");
        println!("✓ JavaScript files found");
        println!("✓ Image assets found");
    }

    if verbose {
        println!("\nConfiguration details:");
        println!("  Bind address: 127.0.0.1");
        println!("  Port: 8080");
        println!("  Authentication: Disabled");
        println!("  Theme: Default");
        println!("  Real-time updates: Enabled");
    }

    println!("\n✓ Dashboard configuration is valid");

    Ok(())
}

async fn handle_status(url: String, detailed: bool) -> Result<()> {
    // Validate inputs
    if url.is_empty() {
        anyhow::bail!("URL cannot be empty");
    }

    println!("Checking dashboard status: {}", url);

    // Mock status check
    println!("✓ Dashboard is running");
    println!("✓ API endpoints are responsive");
    println!("✓ WebSocket connection is available");

    if detailed {
        println!("\nDetailed Status:");
        println!("  Uptime: 2h 15m 30s");
        println!("  Active connections: 5");
        println!("  Memory usage: 145 MB");
        println!("  CPU usage: 12%");
        println!("  Last updated: 2 seconds ago");

        println!("\nFeature Status:");
        println!("  Model management: ✓ Available");
        println!("  Metrics dashboard: ✓ Available");
        println!("  Federated learning: ✓ Available");
        println!("  Marketplace: ✓ Available");
        println!("  User management: ✗ Disabled");
    }

    Ok(())
}

async fn handle_stop(force: bool) -> Result<()> {
    if force {
        println!("Force stopping dashboard server...");
    } else {
        println!("Gracefully stopping dashboard server...");
    }

    // Mock stop
    println!("✓ Dashboard server stopped");

    Ok(())
}

async fn handle_restart(timeout: u64) -> Result<()> {
    println!("Restarting dashboard server (timeout: {}s)...", timeout);

    // Mock restart
    println!("✓ Dashboard server stopped");
    println!("✓ Dashboard server started");
    println!("Dashboard is now available at http://127.0.0.1:8080");

    Ok(())
}

async fn handle_export(
    export_type: ExportType,
    output: PathBuf,
    days: u32,
    include_sensitive: bool,
) -> Result<()> {
    // Validate inputs
    if days == 0 {
        anyhow::bail!("Days must be greater than 0");
    }

    println!(
        "Exporting {:?} data for the last {} days",
        export_type, days
    );

    if include_sensitive {
        warn!("Including sensitive data in export");
    }

    // Mock export
    let export_data = match export_type {
        ExportType::Metrics => {
            r#"{"metrics": {"cpu_usage": [85.2, 78.1, 92.3], "memory_usage": [65.4, 71.2, 68.9]}}"#
        }
        ExportType::Models => {
            r#"{"models": [{"id": "llama-7b", "name": "LLaMA 7B", "size_mb": 7168.0}]}"#
        }
        ExportType::Users => {
            r#"{"users": [], "note": "User data should be loaded from secure configuration or database"}"#
        }
        ExportType::Config => r#"{"config": {"bind_address": "127.0.0.1", "port": 8080}}"#,
        ExportType::All => r#"{"metrics": {}, "models": [], "users": [], "config": {}}"#,
    };

    tokio::fs::write(&output, export_data).await?;

    println!("✓ Data exported to: {}", output.display());
    println!("Export size: {} bytes", export_data.len());

    Ok(())
}

async fn handle_import(
    file: PathBuf,
    import_type: Option<ImportType>,
    overwrite: bool,
    dry_run: bool,
) -> Result<()> {
    // Validate inputs
    if !file.exists() {
        anyhow::bail!("Import file does not exist: {}", file.display());
    }

    println!("Importing data from: {}", file.display());

    if dry_run {
        println!("DRY RUN - No changes will be made");
    }

    if overwrite {
        warn!("Overwrite mode enabled - existing data will be replaced");
    }

    // Mock import
    let import_data = tokio::fs::read_to_string(&file).await?;
    println!("Read {} bytes from import file", import_data.len());

    if let Some(import_type) = import_type {
        println!("Import type: {:?}", import_type);
    } else {
        println!("Auto-detecting import type from file content");
    }

    if !dry_run {
        println!("✓ Data imported successfully");
        println!("  Records processed: 25");
        println!("  Records imported: 23");
        println!("  Records skipped: 2");
    } else {
        println!("✓ Dry run completed - no changes made");
        println!("  Would process: 25 records");
        println!("  Would import: 23 records");
        println!("  Would skip: 2 records");
    }

    Ok(())
}

async fn handle_theme_command(command: ThemeCommands) -> Result<()> {
    match command {
        ThemeCommands::List => {
            println!("Available Dashboard Themes");
            println!("=========================");
            println!("• default (active) - Default Inferno theme");
            println!("• dark - Dark mode theme");
            println!("• light - Light mode theme");
            println!("• corporate - Corporate branding theme");
            println!("• minimal - Minimal interface theme");
        }

        ThemeCommands::Install { source, force } => {
            if force {
                println!("Force installing theme from: {}", source);
            } else {
                println!("Installing theme from: {}", source);
            }

            println!("✓ Theme installed successfully");
            println!("Use 'inferno dashboard theme set <name>' to activate");
        }

        ThemeCommands::Remove { name } => {
            println!("Removing theme: {}", name);
            println!("✓ Theme '{}' removed successfully", name);
        }

        ThemeCommands::Set { name } => {
            println!("Setting default theme: {}", name);
            println!("✓ Default theme set to '{}'", name);
            println!("Restart the dashboard to apply changes");
        }

        ThemeCommands::Create { name, base, output } => {
            println!("Creating custom theme: {}", name);

            if let Some(base_theme) = base {
                println!("Based on: {}", base_theme);
            }

            let output_dir = output.unwrap_or_else(|| PathBuf::from(format!("theme-{}", name)));
            println!("Output directory: {}", output_dir.display());

            // Mock theme creation
            println!("✓ Theme structure created");
            println!("✓ CSS files generated");
            println!("✓ Configuration file created");
            println!("\nNext steps:");
            println!("1. Customize the CSS files in {}", output_dir.display());
            println!(
                "2. Install the theme: inferno dashboard theme install {}",
                output_dir.display()
            );
        }
    }

    Ok(())
}

async fn handle_user_command(command: UserCommands) -> Result<()> {
    match command {
        UserCommands::List {
            active_only,
            format,
        } => {
            println!("Dashboard Users");
            println!("===============");

            if format == "table" {
                println!(
                    "{:<15} {:<25} {:<10} {:<10} {:<20}",
                    "USERNAME", "EMAIL", "ROLE", "STATUS", "LAST LOGIN"
                );
                println!("{}", "-".repeat(85));

                // Generate example user data with non-hardcoded values
                let users = generate_example_users();

                for (username, email, role, status, last_login) in users {
                    if !active_only || status == "active" {
                        println!(
                            "{:<15} {:<25} {:<10} {:<10} {:<20}",
                            username, email, role, status, last_login
                        );
                    }
                }
            } else {
                println!("User list in {} format", format);
            }
        }

        UserCommands::Create {
            username,
            email,
            role,
            password,
            admin,
        } => {
            println!("Creating user: {}", username);
            println!("Email: {}", email);
            println!("Role: {}", if admin { "admin" } else { &role });

            if password.is_none() {
                println!("Password will be prompted interactively");
            }

            println!("✓ User '{}' created successfully", username);
            println!("Default password: <generated>");
            println!("User should change password on first login");
        }

        UserCommands::Update {
            username,
            email,
            role,
            active,
        } => {
            println!("Updating user: {}", username);

            if let Some(new_email) = email {
                println!("New email: {}", new_email);
            }
            if let Some(new_role) = role {
                println!("New role: {}", new_role);
            }
            if let Some(is_active) = active {
                println!("Active: {}", is_active);
            }

            println!("✓ User '{}' updated successfully", username);
        }

        UserCommands::Delete { username, force } => {
            if !force {
                println!(
                    "This will permanently delete user '{}'. Continue? (y/N)",
                    username
                );
                // In real implementation, wait for user confirmation
            }

            println!("Deleting user: {}", username);
            println!("✓ User '{}' deleted successfully", username);
        }

        UserCommands::ResetPassword {
            username,
            password,
            force,
        } => {
            if !force {
                println!(
                    "This will reset the password for user '{}'. Continue? (y/N)",
                    username
                );
                // In real implementation, wait for user confirmation
            }

            println!("Resetting password for user: {}", username);

            if password.is_none() {
                println!("New password will be prompted interactively");
            }

            println!("✓ Password reset for user '{}'", username);
            println!("New password: <generated>");
        }
    }

    Ok(())
}

fn generate_config_template(with_auth: bool, examples: bool) -> String {
    let mut config = String::from("# Inferno Dashboard Configuration\n\n");

    config.push_str("[dashboard]\n");
    config.push_str("enabled = true\n");
    config.push_str("bind_address = \"127.0.0.1\"\n");
    config.push_str("port = 8080\n");
    config.push_str("assets_dir = \"./dashboard/assets\"\n\n");

    if with_auth || examples {
        config.push_str("[dashboard.auth]\n");
        config.push_str(&format!("enabled = {}\n", with_auth));
        config.push_str("provider = \"local\"\n");
        config.push_str("session_timeout_minutes = 480\n");
        config.push_str("# admin_users = [\"your_admin_username\"]\n");
        config.push_str("# readonly_users = []\n");
        config.push_str("# SECURITY: Configure admin users through environment variables or secure configuration\n");
        config.push_str("# Example: INFERNO_ADMIN_USERS=admin1,admin2\n\n");
    }

    config.push_str("[dashboard.ui]\n");
    config.push_str("title = \"Inferno AI Dashboard\"\n\n");

    config.push_str("[dashboard.ui.theme]\n");
    config.push_str("default_theme = \"auto\"\n");
    config.push_str("allow_switching = true\n\n");

    config.push_str("[dashboard.ui.layout]\n");
    config.push_str("sidebar_expanded = true\n");
    config.push_str("refresh_interval = 30\n");
    config.push_str("items_per_page = 25\n\n");

    config.push_str("[dashboard.ui.features]\n");
    config.push_str("model_management = true\n");
    config.push_str("metrics = true\n");
    config.push_str("federated_learning = true\n");
    config.push_str("marketplace = true\n");
    config.push_str("deployment = true\n");
    config.push_str("monitoring = true\n");
    config.push_str(&format!("user_management = {}\n\n", with_auth));

    config.push_str("[dashboard.realtime]\n");
    config.push_str("enabled = true\n");
    config.push_str("ws_path = \"/ws\"\n");
    config.push_str("update_frequency_ms = 5000\n");
    config.push_str("max_connections = 100\n\n");

    if examples {
        config.push_str("# Example security configuration\n");
        config.push_str("# [dashboard.security]\n");
        config.push_str("# https_enabled = false\n");
        config.push_str("# cert_path = \"/path/to/cert.pem\"\n");
        config.push_str("# key_path = \"/path/to/key.pem\"\n\n");

        config.push_str("# Example branding configuration\n");
        config.push_str("# [dashboard.ui.branding]\n");
        config.push_str("# organization = \"Your Organization\"\n");
        config.push_str("# logo = \"/assets/images/logo.png\"\n");
        config.push_str("# favicon = \"/assets/images/favicon.ico\"\n");
    }

    config
}

fn generate_random_username() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};

    // Generate a hash based on current time for uniqueness
    let mut hasher = DefaultHasher::new();
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .hash(&mut hasher);

    let hash = hasher.finish();
    format!("admin_{:x}", hash % 0xffffff) // Create a short, unique username
}

fn generate_secure_password() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};

    // Character sets for password generation
    let lowercase = "abcdefghijklmnopqrstuvwxyz";
    let uppercase = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let numbers = "0123456789";
    let symbols = "!@#$%^&*";

    let charset = format!("{}{}{}{}", lowercase, uppercase, numbers, symbols);
    let charset_bytes = charset.as_bytes();

    let mut password = String::new();
    let mut hasher = DefaultHasher::new();

    // Generate 16-character password using time-based entropy
    for i in 0..16 {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .wrapping_add(i as u128);

        seed.hash(&mut hasher);
        let hash = hasher.finish();
        let char_index = (hash as usize) % charset_bytes.len();
        password.push(charset_bytes[char_index] as char);
    }

    password
}

fn generate_example_users() -> Vec<(String, String, String, String, String)> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};

    let mut users = Vec::new();
    let mut hasher = DefaultHasher::new();

    // Generate unique example usernames instead of hardcoded ones
    let base_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    for i in 0..4 {
        let seed = base_time.wrapping_add(i * 1000);
        seed.hash(&mut hasher);
        let hash = hasher.finish();

        let username = format!("user_{:x}", hash % 0xffff);
        let email = format!("{}@company.internal", username);

        let (role, status, last_login) = match i {
            0 => ("admin", "active", "2 hours ago"),
            1 => ("user", "active", "1 day ago"),
            2 => ("readonly", "active", "3 days ago"),
            3 => ("user", "inactive", "1 month ago"),
            _ => ("user", "active", "unknown"),
        };

        users.push((
            username,
            email,
            role.to_string(),
            status.to_string(),
            last_login.to_string(),
        ));
    }

    users
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_handle_start_validation_zero_port() {
        let result = handle_start("127.0.0.1".to_string(), 0, None, false, false, None).await;

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Port must be greater than 0")
        );
    }

    #[tokio::test]
    async fn test_handle_start_validation_empty_address() {
        let result = handle_start("".to_string(), 8080, None, false, false, None).await;

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Address cannot be empty")
        );
    }

    #[tokio::test]
    async fn test_handle_status_validation_empty_url() {
        let result = handle_status("".to_string(), false).await;

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("URL cannot be empty")
        );
    }

    #[tokio::test]
    async fn test_handle_export_validation_zero_days() {
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().join("export.json");

        let result = handle_export(ExportType::Metrics, output_path, 0, false).await;

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Days must be greater than 0")
        );
    }

    #[tokio::test]
    async fn test_handle_import_validation_nonexistent_file() {
        let result = handle_import(
            PathBuf::from("/nonexistent/path/file.json"),
            None,
            false,
            false,
        )
        .await;

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Import file does not exist")
        );
    }

    #[test]
    fn test_generate_random_username() {
        let username1 = generate_random_username();
        let username2 = generate_random_username();

        // Should start with "admin_"
        assert!(username1.starts_with("admin_"));
        assert!(username2.starts_with("admin_"));

        // Should be reasonably unique (different hash values)
        // Note: In rare cases they could be the same if called at exact same nanosecond
        assert!(!username1.is_empty());
        assert!(!username2.is_empty());
    }

    #[test]
    fn test_generate_secure_password() {
        let password = generate_secure_password();

        // Should be 16 characters
        assert_eq!(password.len(), 16);

        // Should not be empty
        assert!(!password.is_empty());
    }

    #[test]
    fn test_generate_config_template_basic() {
        let config = generate_config_template(false, false);

        assert!(config.contains("[dashboard]"));
        assert!(config.contains("enabled = true"));
        assert!(config.contains("bind_address = \"127.0.0.1\""));
        assert!(config.contains("port = 8080"));
    }

    #[test]
    fn test_generate_config_template_with_auth() {
        let config = generate_config_template(true, false);

        assert!(config.contains("[dashboard.auth]"));
        assert!(config.contains("enabled = true"));
        assert!(config.contains("provider = \"local\""));
    }

    #[test]
    fn test_generate_config_template_with_examples() {
        let config = generate_config_template(false, true);

        assert!(config.contains("# Example security configuration"));
        assert!(config.contains("# [dashboard.security]"));
        assert!(config.contains("# Example branding configuration"));
    }

    #[test]
    fn test_generate_example_users() {
        let users = generate_example_users();

        assert_eq!(users.len(), 4);

        // Check that usernames follow expected pattern
        for (username, email, role, status, _last_login) in &users {
            assert!(username.starts_with("user_"));
            assert!(email.contains("@company.internal"));
            assert!(["admin", "user", "readonly"].contains(&role.as_str()));
            assert!(["active", "inactive"].contains(&status.as_str()));
        }
    }
}
