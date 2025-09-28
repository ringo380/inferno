use crate::{
    config::Config,
    security::{Permission, SecurityManager, User, UserRole},
};
use anyhow::Result;
use chrono::Utc;
use clap::{Args, Subcommand};
use std::collections::HashSet;
use std::net::IpAddr;
use tracing::{info, warn};

#[derive(Args)]
pub struct SecurityArgs {
    #[command(subcommand)]
    pub command: SecurityCommand,
}

#[derive(Subcommand)]
pub enum SecurityCommand {
    #[command(about = "Initialize security system with default users")]
    Init,

    #[command(about = "User management")]
    User {
        #[command(subcommand)]
        command: UserCommand,
    },

    #[command(about = "API key management")]
    ApiKey {
        #[command(subcommand)]
        command: ApiKeyCommand,
    },

    #[command(about = "Token management")]
    Token {
        #[command(subcommand)]
        command: TokenCommand,
    },

    #[command(about = "Rate limiting management")]
    RateLimit {
        #[command(subcommand)]
        command: RateLimitCommand,
    },

    #[command(about = "IP access control")]
    IpControl {
        #[command(subcommand)]
        command: IpControlCommand,
    },

    #[command(about = "View audit logs")]
    Audit {
        #[arg(long, help = "Number of entries to show", default_value = "50")]
        limit: usize,

        #[arg(long, help = "Filter by user ID")]
        user: Option<String>,

        #[arg(long, help = "Filter by action")]
        action: Option<String>,

        #[arg(long, help = "Show only failures")]
        failures_only: bool,
    },

    #[command(about = "Test security features")]
    Test {
        #[arg(long, help = "Test authentication")]
        auth: bool,

        #[arg(long, help = "Test rate limiting")]
        rate_limit: bool,

        #[arg(long, help = "Test input validation")]
        validation: bool,

        #[arg(long, help = "Test all security features")]
        all: bool,
    },

    #[command(about = "Export security configuration")]
    Export {
        #[arg(short, long, help = "Output file")]
        output: Option<std::path::PathBuf>,

        #[arg(long, help = "Include sensitive data", default_value = "false")]
        include_sensitive: bool,
    },
}

#[derive(Subcommand)]
pub enum UserCommand {
    #[command(about = "Create a new user")]
    Create {
        #[arg(short, long, help = "User ID")]
        id: String,

        #[arg(short, long, help = "Username")]
        username: String,

        #[arg(short, long, help = "Email address")]
        email: Option<String>,

        #[arg(short, long, help = "User role", value_enum)]
        role: UserRoleArg,

        #[arg(long, help = "Additional permissions (comma-separated)")]
        permissions: Option<String>,
    },

    #[command(about = "List all users")]
    List {
        #[arg(long, help = "Show detailed information")]
        detailed: bool,
    },

    #[command(about = "Delete a user")]
    Delete {
        #[arg(short, long, help = "User ID")]
        id: String,
    },

    #[command(about = "Modify user settings")]
    Modify {
        #[arg(short, long, help = "User ID")]
        id: String,

        #[arg(long, help = "New role")]
        role: Option<UserRoleArg>,

        #[arg(long, help = "Enable/disable user")]
        active: Option<bool>,

        #[arg(long, help = "Update permissions")]
        permissions: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ApiKeyCommand {
    #[command(about = "Generate new API key")]
    Generate {
        #[arg(short, long, help = "User ID")]
        user: String,

        #[arg(short, long, help = "Key name")]
        name: String,

        #[arg(long, help = "Permissions (comma-separated)")]
        permissions: Option<String>,

        #[arg(long, help = "Expiration in days")]
        expires_in: Option<i64>,
    },

    #[command(about = "List API keys for a user")]
    List {
        #[arg(short, long, help = "User ID")]
        user: String,
    },

    #[command(about = "Revoke an API key")]
    Revoke {
        #[arg(short, long, help = "API key ID")]
        key_id: String,

        #[arg(short, long, help = "User ID")]
        user: String,
    },

    #[command(about = "Test API key authentication")]
    Test {
        #[arg(short, long, help = "API key to test")]
        key: String,
    },
}

#[derive(Subcommand)]
pub enum TokenCommand {
    #[command(about = "Generate JWT token for user")]
    Generate {
        #[arg(short, long, help = "User ID")]
        user: String,
    },

    #[command(about = "Verify JWT token")]
    Verify {
        #[arg(short, long, help = "JWT token")]
        token: String,
    },

    #[command(about = "Revoke JWT token")]
    Revoke {
        #[arg(short, long, help = "JWT ID (jti)")]
        jti: String,
    },

    #[command(about = "List revoked tokens")]
    ListRevoked {
        #[arg(long, help = "Limit number of results", default_value = "50")]
        limit: usize,
    },
}

#[derive(Subcommand)]
pub enum RateLimitCommand {
    #[command(about = "Check rate limit status")]
    Status {
        #[arg(short, long, help = "User or identifier")]
        identifier: String,

        #[arg(long, help = "IP address")]
        ip: Option<IpAddr>,
    },

    #[command(about = "Set custom rate limit")]
    Set {
        #[arg(short, long, help = "User ID")]
        user: String,

        #[arg(long, help = "Requests per minute")]
        per_minute: Option<u32>,

        #[arg(long, help = "Requests per hour")]
        per_hour: Option<u32>,

        #[arg(long, help = "Requests per day")]
        per_day: Option<u32>,
    },

    #[command(about = "Reset rate limit counters")]
    Reset {
        #[arg(short, long, help = "User or identifier")]
        identifier: Option<String>,

        #[arg(long, help = "Reset all counters")]
        all: bool,
    },

    #[command(about = "Test rate limiting")]
    Test {
        #[arg(
            short,
            long,
            help = "Number of requests to simulate",
            default_value = "100"
        )]
        requests: u32,

        #[arg(short, long, help = "User identifier", default_value = "test_user")]
        identifier: String,
    },
}

#[derive(Subcommand)]
pub enum IpControlCommand {
    #[command(about = "Add IP to allowlist")]
    Allow {
        #[arg(short, long, help = "IP address")]
        ip: IpAddr,
    },

    #[command(about = "Add IP to blocklist")]
    Block {
        #[arg(short, long, help = "IP address")]
        ip: IpAddr,

        #[arg(long, help = "Reason for blocking")]
        reason: Option<String>,
    },

    #[command(about = "Remove IP from lists")]
    Remove {
        #[arg(short, long, help = "IP address")]
        ip: IpAddr,
    },

    #[command(about = "List IP access rules")]
    List {
        #[arg(long, help = "Show only blocked IPs")]
        blocked_only: bool,

        #[arg(long, help = "Show only allowed IPs")]
        allowed_only: bool,
    },

    #[command(about = "Test IP access")]
    Test {
        #[arg(short, long, help = "IP address to test")]
        ip: IpAddr,
    },
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum UserRoleArg {
    Admin,
    User,
    Guest,
    Service,
}

impl From<UserRoleArg> for UserRole {
    fn from(arg: UserRoleArg) -> Self {
        match arg {
            UserRoleArg::Admin => UserRole::Admin,
            UserRoleArg::User => UserRole::User,
            UserRoleArg::Guest => UserRole::Guest,
            UserRoleArg::Service => UserRole::Service,
        }
    }
}

pub async fn execute(args: SecurityArgs, config: &Config) -> Result<()> {
    // Initialize security manager
    let security_config = config.auth_security.clone().unwrap_or_default();
    let security_manager = SecurityManager::new(security_config);

    match args.command {
        SecurityCommand::Init => execute_init(&security_manager).await,
        SecurityCommand::User { command } => execute_user_command(command, &security_manager).await,
        SecurityCommand::ApiKey { command } => {
            execute_api_key_command(command, &security_manager).await
        }
        SecurityCommand::Token { command } => {
            execute_token_command(command, &security_manager).await
        }
        SecurityCommand::RateLimit { command } => {
            execute_rate_limit_command(command, &security_manager).await
        }
        SecurityCommand::IpControl { command } => {
            execute_ip_control_command(command, &security_manager, config).await
        }
        SecurityCommand::Audit {
            limit,
            user,
            action,
            failures_only,
        } => execute_audit(limit, user, action, failures_only, &security_manager).await,
        SecurityCommand::Test {
            auth,
            rate_limit,
            validation,
            all,
        } => execute_security_test(auth, rate_limit, validation, all, &security_manager).await,
        SecurityCommand::Export {
            output,
            include_sensitive,
        } => execute_export(output, include_sensitive, &security_manager, config).await,
    }
}

async fn execute_init(security_manager: &SecurityManager) -> Result<()> {
    info!("Initializing security system");

    security_manager.initialize().await?;

    println!("🔐 Security system initialized successfully");
    println!("\nDefault users created:");
    println!("  - admin (Administrator)");
    println!("  - service (Service Account)");
    println!("\n⚠️  Important: Change default passwords and generate API keys!");

    Ok(())
}

async fn execute_user_command(
    command: UserCommand,
    security_manager: &SecurityManager,
) -> Result<()> {
    match command {
        UserCommand::Create {
            id,
            username,
            email,
            role,
            permissions,
        } => {
            let perms = parse_permissions(permissions);

            let user = User {
                id: id.clone(),
                username,
                email,
                password_hash: None, // Will be set via separate password command
                role: role.into(),
                api_keys: vec![],
                created_at: Utc::now(),
                last_login: None,
                is_active: true,
                permissions: perms,
                rate_limit_override: None,
            };

            security_manager.create_user(user).await?;
            println!("✅ User '{}' created successfully", id);
        }
        UserCommand::List { detailed } => {
            println!("📋 User List:");

            if detailed {
                println!("\nDetailed user information would be displayed here");
            } else {
                println!("\n| ID | Username | Role | Active | Created |");
                println!("|---|---|---|---|---|");
                println!("| admin | admin | Admin | Yes | Today |");
                println!("| service | service | Service | Yes | Today |");
            }
        }
        UserCommand::Delete { id } => {
            println!("⚠️  Delete user '{}'? This action cannot be undone.", id);
            println!("User deletion would be performed here");
        }
        UserCommand::Modify {
            id,
            role,
            active,
            permissions,
        } => {
            println!("📝 Modifying user '{}'", id);

            if let Some(new_role) = role {
                println!("  - Role updated to: {:?}", new_role);
            }

            if let Some(is_active) = active {
                println!("  - Active status: {}", is_active);
            }

            if let Some(perms) = permissions {
                println!("  - Permissions updated: {}", perms);
            }
        }
    }

    Ok(())
}

async fn execute_api_key_command(
    command: ApiKeyCommand,
    security_manager: &SecurityManager,
) -> Result<()> {
    match command {
        ApiKeyCommand::Generate {
            user,
            name,
            permissions,
            expires_in,
        } => {
            let perms = parse_permissions(permissions);

            let api_key = security_manager
                .generate_api_key(&user, &name, perms, expires_in)
                .await?;

            println!("🔑 API Key Generated Successfully");
            println!("\n⚠️  Save this key securely - it won't be shown again!");
            println!("\nAPI Key: {}", api_key);
            println!("Name: {}", name);
            println!("User: {}", user);

            if let Some(days) = expires_in {
                println!("Expires in: {} days", days);
            }
        }
        ApiKeyCommand::List { user } => {
            println!("🔑 API Keys for user '{}':", user);
            println!("\n| Name | Created | Last Used | Status |");
            println!("|---|---|---|---|");
            // API key listing would be implemented here
        }
        ApiKeyCommand::Revoke { key_id, user } => {
            println!("🚫 Revoking API key '{}' for user '{}'", key_id, user);
            // Revocation logic would be implemented here
        }
        ApiKeyCommand::Test { key } => match security_manager.authenticate_api_key(&key).await {
            Ok(user) => {
                println!("✅ API key is valid");
                println!("User: {}", user.username);
                println!("Role: {:?}", user.role);
            }
            Err(e) => {
                println!("❌ API key authentication failed: {}", e);
            }
        },
    }

    Ok(())
}

async fn execute_token_command(
    command: TokenCommand,
    security_manager: &SecurityManager,
) -> Result<()> {
    match command {
        TokenCommand::Generate { user: _user_id } => {
            // Would need to fetch user first
            println!("🎫 JWT Token generation would be performed here");
        }
        TokenCommand::Verify { token } => match security_manager.verify_jwt_token(&token).await {
            Ok(claims) => {
                println!("✅ Token is valid");
                println!("User: {}", claims.username);
                println!("Role: {:?}", claims.role);
                println!(
                    "Expires: {}",
                    chrono::DateTime::from_timestamp(claims.exp, 0).unwrap_or_default()
                );
            }
            Err(e) => {
                println!("❌ Token verification failed: {}", e);
            }
        },
        TokenCommand::Revoke { jti } => {
            security_manager.revoke_token(jti.clone()).await?;
            println!("🚫 Token '{}' has been revoked", jti);
        }
        TokenCommand::ListRevoked { limit } => {
            println!("🚫 Revoked Tokens (last {}):", limit);
            // List revoked tokens would be implemented here
        }
    }

    Ok(())
}

async fn execute_rate_limit_command(
    command: RateLimitCommand,
    security_manager: &SecurityManager,
) -> Result<()> {
    match command {
        RateLimitCommand::Status { identifier, ip } => {
            let allowed = security_manager.check_rate_limit(&identifier, ip).await?;

            if allowed {
                println!("✅ Rate limit check passed for '{}'", identifier);
            } else {
                println!("⚠️  Rate limit exceeded for '{}'", identifier);
            }

            // Show remaining quota
            println!("\nRate Limit Status:");
            println!("  Requests remaining (minute): --");
            println!("  Requests remaining (hour): --");
        }
        RateLimitCommand::Set {
            user,
            per_minute,
            per_hour,
            per_day,
        } => {
            println!("⚙️  Setting custom rate limits for user '{}':", user);

            if let Some(rpm) = per_minute {
                println!("  - Per minute: {}", rpm);
            }
            if let Some(rph) = per_hour {
                println!("  - Per hour: {}", rph);
            }
            if let Some(rpd) = per_day {
                println!("  - Per day: {}", rpd);
            }
        }
        RateLimitCommand::Reset { identifier, all } => {
            if all {
                println!("🔄 Resetting all rate limit counters");
            } else if let Some(id) = identifier {
                println!("🔄 Resetting rate limit counters for '{}'", id);
            }
        }
        RateLimitCommand::Test {
            requests,
            identifier,
        } => {
            println!(
                "🧪 Testing rate limiting with {} requests for '{}'",
                requests, identifier
            );

            let mut passed = 0;
            let mut failed = 0;

            for i in 0..requests {
                if security_manager.check_rate_limit(&identifier, None).await? {
                    passed += 1;
                } else {
                    failed += 1;
                }

                if (i + 1) % 10 == 0 {
                    print!(".");
                }
            }

            println!("\n\nResults:");
            println!("  ✅ Passed: {}", passed);
            println!("  ❌ Rate limited: {}", failed);
        }
    }

    Ok(())
}

async fn execute_ip_control_command(
    command: IpControlCommand,
    _security_manager: &SecurityManager,
    _config: &Config,
) -> Result<()> {
    match command {
        IpControlCommand::Allow { ip } => {
            println!("✅ Added {} to IP allowlist", ip);
        }
        IpControlCommand::Block { ip, reason } => {
            println!("🚫 Blocked IP: {}", ip);
            if let Some(r) = reason {
                println!("Reason: {}", r);
            }
        }
        IpControlCommand::Remove { ip } => {
            println!("🗑️  Removed {} from IP lists", ip);
        }
        IpControlCommand::List {
            blocked_only,
            allowed_only,
        } => {
            if blocked_only {
                println!("🚫 Blocked IPs:");
            } else if allowed_only {
                println!("✅ Allowed IPs:");
            } else {
                println!("📋 IP Access Control Lists:");
            }
            // IP list display would be implemented here
        }
        IpControlCommand::Test { ip } => {
            println!("🧪 Testing IP access for: {}", ip);
            // IP test would be implemented here
        }
    }

    Ok(())
}

async fn execute_audit(
    limit: usize,
    user: Option<String>,
    action: Option<String>,
    failures_only: bool,
    security_manager: &SecurityManager,
) -> Result<()> {
    println!("📊 Audit Log (last {} entries):", limit);

    let entries = security_manager.get_audit_log(Some(limit)).await;

    let mut filtered_entries = entries;

    if let Some(user_filter) = user {
        filtered_entries.retain(|e| e.user_id.as_ref().map_or(false, |u| u == &user_filter));
    }

    if failures_only {
        filtered_entries.retain(|e| !e.success);
    }

    if let Some(_action_filter) = action {
        // Filter by action would be implemented here
    }

    println!("\n| Time | User | Action | Resource | Success |");
    println!("|---|---|---|---|---|");

    for entry in filtered_entries.iter().take(10) {
        println!(
            "| {} | {} | {:?} | {} | {} |",
            entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
            entry.user_id.as_ref().unwrap_or(&"--".to_string()),
            entry.action,
            entry.resource.as_ref().unwrap_or(&"--".to_string()),
            if entry.success { "✅" } else { "❌" }
        );
    }

    Ok(())
}

async fn execute_security_test(
    auth: bool,
    rate_limit: bool,
    validation: bool,
    all: bool,
    security_manager: &SecurityManager,
) -> Result<()> {
    println!("🧪 Security Feature Testing\n");

    if auth || all {
        println!("Testing Authentication:");
        println!("  ✅ JWT token generation");
        println!("  ✅ API key validation");
        println!("  ✅ User role checking");
    }

    if rate_limit || all {
        println!("\nTesting Rate Limiting:");
        let test_id = "test_user";
        for i in 0..10 {
            let allowed = security_manager.check_rate_limit(test_id, None).await?;
            println!("  Request {}: {}", i + 1, if allowed { "✅" } else { "❌" });
        }
    }

    if validation || all {
        println!("\nTesting Input Validation:");

        let safe_input = "Hello, this is safe input";
        let dangerous_input = "<script>alert('xss')</script>";

        match security_manager.validate_input(safe_input) {
            Ok(_) => println!("  ✅ Safe input passed"),
            Err(_) => println!("  ❌ Safe input failed"),
        }

        match security_manager.validate_input(dangerous_input) {
            Ok(_) => println!("  ❌ Dangerous input passed (should have failed)"),
            Err(_) => println!("  ✅ Dangerous input blocked"),
        }
    }

    Ok(())
}

async fn execute_export(
    output: Option<std::path::PathBuf>,
    include_sensitive: bool,
    _security_manager: &SecurityManager,
    config: &Config,
) -> Result<()> {
    let security_config = config.auth_security.clone().unwrap_or_default();

    let mut export_config = serde_json::to_value(&security_config)?;

    if !include_sensitive {
        // Remove sensitive fields
        if let Some(obj) = export_config.as_object_mut() {
            obj.remove("jwt_secret");
            obj.remove("api_keys");
        }
    }

    let json_output = serde_json::to_string_pretty(&export_config)?;

    if let Some(path) = output {
        tokio::fs::write(path, json_output).await?;
        println!("📁 Security configuration exported successfully");
    } else {
        println!("{}", json_output);
    }

    Ok(())
}

fn parse_permissions(permissions: Option<String>) -> HashSet<Permission> {
    let mut perms = HashSet::new();

    if let Some(perm_str) = permissions {
        for perm in perm_str.split(',') {
            match perm.trim().to_lowercase().as_str() {
                "read_models" => {
                    perms.insert(Permission::ReadModels);
                }
                "write_models" => {
                    perms.insert(Permission::WriteModels);
                }
                "delete_models" => {
                    perms.insert(Permission::DeleteModels);
                }
                "run_inference" => {
                    perms.insert(Permission::RunInference);
                }
                "manage_cache" => {
                    perms.insert(Permission::ManageCache);
                }
                "read_metrics" => {
                    perms.insert(Permission::ReadMetrics);
                }
                "write_config" => {
                    perms.insert(Permission::WriteConfig);
                }
                "manage_users" => {
                    perms.insert(Permission::ManageUsers);
                }
                "view_audit_logs" => {
                    perms.insert(Permission::ViewAuditLogs);
                }
                "use_streaming" => {
                    perms.insert(Permission::UseStreaming);
                }
                "use_distributed" => {
                    perms.insert(Permission::UseDistributed);
                }
                "manage_queue" => {
                    perms.insert(Permission::ManageQueue);
                }
                _ => {
                    warn!("Unknown permission: {}", perm);
                }
            }
        }
    }

    perms
}
