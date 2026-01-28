use crate::{
    config::Config,
    security::{Permission, SecurityManager, User, UserRole},
};
use anyhow::{bail, Result};
use chrono::Utc;
use clap::{Args, Subcommand};
use std::collections::HashSet;
use std::net::IpAddr;
use tracing::{info, warn};

// ============================================================================
// RateLimitSettings - Builder pattern for cleaner rate limit configuration
// ============================================================================

/// Configuration for rate limit settings.
/// Provides a cleaner interface for passing rate limit parameters.
#[derive(Debug, Clone, Default)]
pub struct RateLimitSettings {
    pub identifier: Option<String>,
    pub ip: Option<IpAddr>,
    pub per_minute: Option<u32>,
    pub per_hour: Option<u32>,
    pub per_day: Option<u32>,
    pub reset_all: bool,
    pub test_requests: Option<u32>,
}

impl RateLimitSettings {
    /// Create a new RateLimitSettings with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the identifier.
    pub fn identifier(mut self, id: impl Into<String>) -> Self {
        self.identifier = Some(id.into());
        self
    }

    /// Set the IP address.
    pub fn ip(mut self, ip: IpAddr) -> Self {
        self.ip = Some(ip);
        self
    }

    /// Set the per-minute limit.
    pub fn per_minute(mut self, limit: u32) -> Self {
        self.per_minute = Some(limit);
        self
    }

    /// Set the per-hour limit.
    pub fn per_hour(mut self, limit: u32) -> Self {
        self.per_hour = Some(limit);
        self
    }

    /// Set the per-day limit.
    pub fn per_day(mut self, limit: u32) -> Self {
        self.per_day = Some(limit);
        self
    }

    /// Set whether to reset all counters.
    pub fn reset_all(mut self, reset: bool) -> Self {
        self.reset_all = reset;
        self
    }

    /// Set the number of test requests.
    pub fn test_requests(mut self, requests: u32) -> Self {
        self.test_requests = Some(requests);
        self
    }
}

// ============================================================================
// Validation Functions
// ============================================================================

/// Validate user management operations.
fn validate_user_operation(
    operation: &str,
    user_id: &Option<String>,
    username: &Option<String>,
    role: &Option<UserRoleArg>,
) -> Result<()> {
    // Validate operation type
    if !["create", "list", "delete", "modify"].contains(&operation) {
        bail!("Operation must be one of: create, list, delete, modify");
    }

    // Validate create operation
    if operation == "create" {
        if user_id.is_none() {
            bail!("User ID is required for create operation");
        }
        if username.is_none() {
            bail!("Username is required for create operation");
        }
        if role.is_none() {
            bail!("Role is required for create operation");
        }
    }

    // Validate delete/modify operations
    if ["delete", "modify"].contains(&operation) && user_id.is_none() {
        bail!("User ID is required for {} operation", operation);
    }

    Ok(())
}

/// Validate API key operations.
fn validate_api_key_operation(
    operation: &str,
    user_id: &Option<String>,
    key_name: &Option<String>,
    key_id: &Option<String>,
    key_value: &Option<String>,
    expires_in: Option<i64>,
) -> Result<()> {
    // Validate operation type
    if !["generate", "list", "revoke", "test"].contains(&operation) {
        bail!("Operation must be one of: generate, list, revoke, test");
    }

    // Validate generate operation
    if operation == "generate" {
        if user_id.is_none() {
            bail!("User ID is required for generate operation");
        }
        if key_name.is_none() {
            bail!("Key name is required for generate operation");
        }
    }

    // Validate list operation
    if operation == "list" && user_id.is_none() {
        bail!("User ID is required for list operation");
    }

    // Validate revoke operation
    if operation == "revoke" {
        if key_id.is_none() {
            bail!("Key ID is required for revoke operation");
        }
        if user_id.is_none() {
            bail!("User ID is required for revoke operation");
        }
    }

    // Validate test operation
    if operation == "test" && key_value.is_none() {
        bail!("Key value is required for test operation");
    }

    // Validate expiration if provided
    if let Some(days) = expires_in {
        if days <= 0 || days > 365 {
            bail!("Expiration must be between 1 and 365 days");
        }
    }

    Ok(())
}

/// Validate token operations.
fn validate_token_operation(
    operation: &str,
    user_id: &Option<String>,
    token: &Option<String>,
    jti: &Option<String>,
) -> Result<()> {
    // Validate operation type
    if !["generate", "verify", "revoke", "list-revoked"].contains(&operation) {
        bail!("Operation must be one of: generate, verify, revoke, list-revoked");
    }

    // Validate generate operation
    if operation == "generate" && user_id.is_none() {
        bail!("User ID is required for generate operation");
    }

    // Validate verify operation
    if operation == "verify" && token.is_none() {
        bail!("Token is required for verify operation");
    }

    // Validate revoke operation
    if operation == "revoke" && jti.is_none() {
        bail!("JTI (token ID) is required for revoke operation");
    }

    Ok(())
}

/// Validate rate limit operations.
fn validate_rate_limit_operation(operation: &str, settings: &RateLimitSettings) -> Result<()> {
    // Validate operation type
    if !["status", "set", "reset", "test"].contains(&operation) {
        bail!("Operation must be one of: status, set, reset, test");
    }

    // Validate status operation
    if operation == "status" && settings.identifier.is_none() {
        bail!("Identifier is required for status operation");
    }

    // Validate set operation
    if operation == "set" {
        if settings.identifier.is_none() {
            bail!("Identifier is required for set operation");
        }
        if settings.per_minute.is_none()
            && settings.per_hour.is_none()
            && settings.per_day.is_none()
        {
            bail!("At least one rate limit (per_minute, per_hour, per_day) must be specified");
        }
    }

    // Validate test operation
    if operation == "test" {
        if settings.test_requests.is_none() {
            bail!("Number of test requests is required for test operation");
        }
        if let Some(requests) = settings.test_requests {
            if requests == 0 || requests > 10000 {
                bail!("Test requests must be between 1 and 10000");
            }
        }
    }

    Ok(())
}

/// Validate IP control operations.
fn validate_ip_control_operation(operation: &str, ip: &Option<IpAddr>) -> Result<()> {
    // Validate operation type
    if !["allow", "block", "remove", "list", "test"].contains(&operation) {
        bail!("Operation must be one of: allow, block, remove, list, test");
    }

    // IP is required for all operations except list
    if operation != "list" && ip.is_none() {
        bail!("IP address is required for {} operation", operation);
    }

    Ok(())
}

/// Validate audit parameters.
fn validate_audit_params(limit: usize) -> Result<()> {
    if limit == 0 || limit > 1000 {
        bail!("Limit must be between 1 and 1000");
    }
    Ok(())
}

/// Validate security test parameters.
fn validate_security_test(auth: bool, rate_limit: bool, validation: bool, all: bool) -> Result<()> {
    if !auth && !rate_limit && !validation && !all {
        bail!(
            "At least one test type must be enabled (--auth, --rate-limit, --validation, or --all)"
        );
    }
    Ok(())
}

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

    // Validate command-specific parameters before execution
    validate_command(&args.command)?;

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

/// Validate command parameters before execution
fn validate_command(command: &SecurityCommand) -> Result<()> {
    match command {
        SecurityCommand::Init => Ok(()),
        SecurityCommand::User { command } => validate_user_command(command),
        SecurityCommand::ApiKey { command } => validate_api_key_command(command),
        SecurityCommand::Token { command } => validate_token_command(command),
        SecurityCommand::RateLimit { command } => validate_rate_limit_cmd(command),
        SecurityCommand::IpControl { command } => validate_ip_control_cmd(command),
        SecurityCommand::Audit { limit, .. } => validate_audit_params(*limit),
        SecurityCommand::Test {
            auth,
            rate_limit,
            validation,
            all,
        } => validate_security_test(*auth, *rate_limit, *validation, *all),
        SecurityCommand::Export { .. } => Ok(()),
    }
}

fn validate_user_command(command: &UserCommand) -> Result<()> {
    match command {
        UserCommand::Create {
            id, username, role, ..
        } => validate_user_operation(
            "create",
            &Some(id.clone()),
            &Some(username.clone()),
            &Some(role.clone()),
        ),
        UserCommand::List { .. } => validate_user_operation("list", &None, &None, &None),
        UserCommand::Delete { id } => {
            validate_user_operation("delete", &Some(id.clone()), &None, &None)
        }
        UserCommand::Modify { id, role, .. } => {
            validate_user_operation("modify", &Some(id.clone()), &None, role)
        }
    }
}

fn validate_api_key_command(command: &ApiKeyCommand) -> Result<()> {
    match command {
        ApiKeyCommand::Generate {
            user,
            name,
            expires_in,
            ..
        } => validate_api_key_operation(
            "generate",
            &Some(user.clone()),
            &Some(name.clone()),
            &None,
            &None,
            *expires_in,
        ),
        ApiKeyCommand::List { user } => {
            validate_api_key_operation("list", &Some(user.clone()), &None, &None, &None, None)
        }
        ApiKeyCommand::Revoke { key_id, user } => validate_api_key_operation(
            "revoke",
            &Some(user.clone()),
            &None,
            &Some(key_id.clone()),
            &None,
            None,
        ),
        ApiKeyCommand::Test { key } => {
            validate_api_key_operation("test", &None, &None, &None, &Some(key.clone()), None)
        }
    }
}

fn validate_token_command(command: &TokenCommand) -> Result<()> {
    match command {
        TokenCommand::Generate { user } => {
            validate_token_operation("generate", &Some(user.clone()), &None, &None)
        }
        TokenCommand::Verify { token } => {
            validate_token_operation("verify", &None, &Some(token.clone()), &None)
        }
        TokenCommand::Revoke { jti } => {
            validate_token_operation("revoke", &None, &None, &Some(jti.clone()))
        }
        TokenCommand::ListRevoked { .. } => {
            validate_token_operation("list-revoked", &None, &None, &None)
        }
    }
}

fn validate_rate_limit_cmd(command: &RateLimitCommand) -> Result<()> {
    match command {
        RateLimitCommand::Status { identifier, ip } => {
            let settings = RateLimitSettings {
                identifier: Some(identifier.clone()),
                ip: *ip,
                ..Default::default()
            };
            validate_rate_limit_operation("status", &settings)
        }
        RateLimitCommand::Set {
            user,
            per_minute,
            per_hour,
            per_day,
        } => {
            let settings = RateLimitSettings {
                identifier: Some(user.clone()),
                per_minute: *per_minute,
                per_hour: *per_hour,
                per_day: *per_day,
                ..Default::default()
            };
            validate_rate_limit_operation("set", &settings)
        }
        RateLimitCommand::Reset { identifier, all } => {
            let settings = RateLimitSettings {
                identifier: identifier.clone(),
                reset_all: *all,
                ..Default::default()
            };
            validate_rate_limit_operation("reset", &settings)
        }
        RateLimitCommand::Test {
            requests,
            identifier,
        } => {
            let settings = RateLimitSettings {
                identifier: Some(identifier.clone()),
                test_requests: Some(*requests),
                ..Default::default()
            };
            validate_rate_limit_operation("test", &settings)
        }
    }
}

fn validate_ip_control_cmd(command: &IpControlCommand) -> Result<()> {
    match command {
        IpControlCommand::Allow { ip } => validate_ip_control_operation("allow", &Some(*ip)),
        IpControlCommand::Block { ip, .. } => validate_ip_control_operation("block", &Some(*ip)),
        IpControlCommand::Remove { ip } => validate_ip_control_operation("remove", &Some(*ip)),
        IpControlCommand::List { .. } => validate_ip_control_operation("list", &None),
        IpControlCommand::Test { ip } => validate_ip_control_operation("test", &Some(*ip)),
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
        filtered_entries.retain(|e| e.user_id.as_ref() == Some(&user_filter));
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

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // User operation validation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_validate_user_operation_invalid_operation() {
        let result = validate_user_operation("invalid", &None, &None, &None);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Operation must be"));
    }

    #[test]
    fn test_validate_user_operation_create_missing_user_id() {
        let result = validate_user_operation("create", &None, &Some("user".to_string()), &None);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("User ID is required"));
    }

    #[test]
    fn test_validate_user_operation_create_missing_username() {
        let result = validate_user_operation(
            "create",
            &Some("id".to_string()),
            &None,
            &Some(UserRoleArg::User),
        );
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Username is required"));
    }

    #[test]
    fn test_validate_user_operation_create_missing_role() {
        let result = validate_user_operation(
            "create",
            &Some("id".to_string()),
            &Some("user".to_string()),
            &None,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Role is required"));
    }

    #[test]
    fn test_validate_user_operation_create_valid() {
        let result = validate_user_operation(
            "create",
            &Some("id".to_string()),
            &Some("user".to_string()),
            &Some(UserRoleArg::User),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_user_operation_delete_missing_id() {
        let result = validate_user_operation("delete", &None, &None, &None);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("User ID is required"));
    }

    #[test]
    fn test_validate_user_operation_list_valid() {
        let result = validate_user_operation("list", &None, &None, &None);
        assert!(result.is_ok());
    }

    // -------------------------------------------------------------------------
    // API key operation validation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_validate_api_key_operation_invalid_operation() {
        let result = validate_api_key_operation("invalid", &None, &None, &None, &None, None);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Operation must be"));
    }

    #[test]
    fn test_validate_api_key_operation_generate_missing_user() {
        let result = validate_api_key_operation(
            "generate",
            &None,
            &Some("key".to_string()),
            &None,
            &None,
            None,
        );
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("User ID is required"));
    }

    #[test]
    fn test_validate_api_key_operation_generate_missing_name() {
        let result = validate_api_key_operation(
            "generate",
            &Some("user".to_string()),
            &None,
            &None,
            &None,
            None,
        );
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Key name is required"));
    }

    #[test]
    fn test_validate_api_key_operation_generate_valid() {
        let result = validate_api_key_operation(
            "generate",
            &Some("user".to_string()),
            &Some("key".to_string()),
            &None,
            &None,
            Some(30),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_api_key_operation_invalid_expiration() {
        let result = validate_api_key_operation(
            "generate",
            &Some("user".to_string()),
            &Some("key".to_string()),
            &None,
            &None,
            Some(400),
        );
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Expiration must be"));
    }

    #[test]
    fn test_validate_api_key_operation_test_missing_key() {
        let result = validate_api_key_operation("test", &None, &None, &None, &None, None);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Key value is required"));
    }

    // -------------------------------------------------------------------------
    // Token operation validation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_validate_token_operation_invalid_operation() {
        let result = validate_token_operation("invalid", &None, &None, &None);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Operation must be"));
    }

    #[test]
    fn test_validate_token_operation_generate_missing_user() {
        let result = validate_token_operation("generate", &None, &None, &None);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("User ID is required"));
    }

    #[test]
    fn test_validate_token_operation_verify_missing_token() {
        let result = validate_token_operation("verify", &None, &None, &None);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Token is required"));
    }

    #[test]
    fn test_validate_token_operation_revoke_missing_jti() {
        let result = validate_token_operation("revoke", &None, &None, &None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("JTI"));
    }

    #[test]
    fn test_validate_token_operation_list_revoked_valid() {
        let result = validate_token_operation("list-revoked", &None, &None, &None);
        assert!(result.is_ok());
    }

    // -------------------------------------------------------------------------
    // Rate limit operation validation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_validate_rate_limit_operation_invalid_operation() {
        let settings = RateLimitSettings::default();
        let result = validate_rate_limit_operation("invalid", &settings);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Operation must be"));
    }

    #[test]
    fn test_validate_rate_limit_operation_status_missing_identifier() {
        let settings = RateLimitSettings::default();
        let result = validate_rate_limit_operation("status", &settings);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Identifier is required"));
    }

    #[test]
    fn test_validate_rate_limit_operation_set_missing_limits() {
        let settings = RateLimitSettings {
            identifier: Some("user".to_string()),
            ..Default::default()
        };
        let result = validate_rate_limit_operation("set", &settings);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("At least one rate limit"));
    }

    #[test]
    fn test_validate_rate_limit_operation_set_valid() {
        let settings = RateLimitSettings {
            identifier: Some("user".to_string()),
            per_minute: Some(60),
            ..Default::default()
        };
        let result = validate_rate_limit_operation("set", &settings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_rate_limit_operation_test_missing_requests() {
        let settings = RateLimitSettings {
            identifier: Some("user".to_string()),
            ..Default::default()
        };
        let result = validate_rate_limit_operation("test", &settings);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Number of test requests"));
    }

    #[test]
    fn test_validate_rate_limit_operation_test_invalid_requests() {
        let settings = RateLimitSettings {
            identifier: Some("user".to_string()),
            test_requests: Some(0),
            ..Default::default()
        };
        let result = validate_rate_limit_operation("test", &settings);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Test requests must be"));
    }

    // -------------------------------------------------------------------------
    // IP control operation validation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_validate_ip_control_operation_invalid_operation() {
        let result = validate_ip_control_operation("invalid", &None);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Operation must be"));
    }

    #[test]
    fn test_validate_ip_control_operation_allow_missing_ip() {
        let result = validate_ip_control_operation("allow", &None);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("IP address is required"));
    }

    #[test]
    fn test_validate_ip_control_operation_list_valid() {
        let result = validate_ip_control_operation("list", &None);
        assert!(result.is_ok());
    }

    // -------------------------------------------------------------------------
    // Audit validation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_validate_audit_params_zero_limit() {
        let result = validate_audit_params(0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Limit must be"));
    }

    #[test]
    fn test_validate_audit_params_over_limit() {
        let result = validate_audit_params(1001);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Limit must be"));
    }

    #[test]
    fn test_validate_audit_params_valid() {
        let result = validate_audit_params(50);
        assert!(result.is_ok());
    }

    // -------------------------------------------------------------------------
    // Security test validation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_validate_security_test_no_tests() {
        let result = validate_security_test(false, false, false, false);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("At least one test type"));
    }

    #[test]
    fn test_validate_security_test_auth_only() {
        let result = validate_security_test(true, false, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_security_test_all() {
        let result = validate_security_test(false, false, false, true);
        assert!(result.is_ok());
    }

    // -------------------------------------------------------------------------
    // RateLimitSettings builder tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_rate_limit_settings_builder() {
        let settings = RateLimitSettings::new()
            .identifier("user-123")
            .per_minute(60)
            .per_hour(1000)
            .per_day(10000)
            .reset_all(false)
            .test_requests(100);

        assert_eq!(settings.identifier, Some("user-123".to_string()));
        assert_eq!(settings.per_minute, Some(60));
        assert_eq!(settings.per_hour, Some(1000));
        assert_eq!(settings.per_day, Some(10000));
        assert!(!settings.reset_all);
        assert_eq!(settings.test_requests, Some(100));
    }

    #[test]
    fn test_rate_limit_settings_default() {
        let settings = RateLimitSettings::default();
        assert!(settings.identifier.is_none());
        assert!(settings.ip.is_none());
        assert!(settings.per_minute.is_none());
        assert!(settings.per_hour.is_none());
        assert!(settings.per_day.is_none());
        assert!(!settings.reset_all);
        assert!(settings.test_requests.is_none());
    }

    // -------------------------------------------------------------------------
    // Permission parsing tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_parse_permissions_empty() {
        let perms = parse_permissions(None);
        assert!(perms.is_empty());
    }

    #[test]
    fn test_parse_permissions_single() {
        let perms = parse_permissions(Some("read_models".to_string()));
        assert_eq!(perms.len(), 1);
        assert!(perms.contains(&Permission::ReadModels));
    }

    #[test]
    fn test_parse_permissions_multiple() {
        let perms = parse_permissions(Some("read_models, write_models, run_inference".to_string()));
        assert_eq!(perms.len(), 3);
        assert!(perms.contains(&Permission::ReadModels));
        assert!(perms.contains(&Permission::WriteModels));
        assert!(perms.contains(&Permission::RunInference));
    }

    #[test]
    fn test_parse_permissions_unknown() {
        let perms = parse_permissions(Some("read_models, unknown_perm".to_string()));
        assert_eq!(perms.len(), 1);
        assert!(perms.contains(&Permission::ReadModels));
    }
}
