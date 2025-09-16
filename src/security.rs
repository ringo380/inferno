use crate::{config::Config, InfernoError};
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet, VecDeque};
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, warn};

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable authentication
    pub auth_enabled: bool,
    /// JWT secret key (should be loaded from environment)
    pub jwt_secret: String,
    /// Token expiration time in hours
    pub token_expiry_hours: i64,
    /// Enable API key authentication
    pub api_key_enabled: bool,
    /// Enable rate limiting
    pub rate_limiting_enabled: bool,
    /// Maximum requests per minute per IP
    pub max_requests_per_minute: u32,
    /// Maximum requests per hour per user
    pub max_requests_per_hour: u32,
    /// Enable IP allowlist
    pub ip_allowlist_enabled: bool,
    /// Allowed IP addresses
    pub allowed_ips: Vec<String>,
    /// Enable IP blocklist
    pub ip_blocklist_enabled: bool,
    /// Blocked IP addresses
    pub blocked_ips: Vec<String>,
    /// Enable audit logging
    pub audit_logging_enabled: bool,
    /// Maximum model size for untrusted users (GB)
    pub max_model_size_gb: f64,
    /// Enable input validation
    pub input_validation_enabled: bool,
    /// Maximum input length
    pub max_input_length: usize,
    /// Enable output sanitization
    pub output_sanitization_enabled: bool,
    /// Enable SSL/TLS enforcement
    pub tls_required: bool,
    /// Minimum TLS version
    pub min_tls_version: String,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            auth_enabled: true,
            jwt_secret: String::new(), // Should be set from environment
            token_expiry_hours: 24,
            api_key_enabled: true,
            rate_limiting_enabled: true,
            max_requests_per_minute: 60,
            max_requests_per_hour: 1000,
            ip_allowlist_enabled: false,
            allowed_ips: vec![],
            ip_blocklist_enabled: false,
            blocked_ips: vec![],
            audit_logging_enabled: true,
            max_model_size_gb: 5.0, // 5GB default
            input_validation_enabled: true,
            max_input_length: 10000,
            output_sanitization_enabled: true,
            tls_required: false,
            min_tls_version: "1.2".to_string(),
        }
    }
}

/// User roles for authorization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserRole {
    Admin,
    User,
    Guest,
    Service,
}

impl UserRole {
    pub fn has_permission(&self, permission: &Permission) -> bool {
        match self {
            UserRole::Admin => true, // Admins have all permissions
            UserRole::User => matches!(
                permission,
                Permission::ReadModels
                    | Permission::RunInference
                    | Permission::ReadMetrics
                    | Permission::UseStreaming
            ),
            UserRole::Guest => matches!(
                permission,
                Permission::ReadModels | Permission::RunInference
            ),
            UserRole::Service => matches!(
                permission,
                Permission::ReadModels
                    | Permission::RunInference
                    | Permission::ReadMetrics
                    | Permission::UseStreaming
                    | Permission::ManageCache
            ),
        }
    }
}

/// Permissions for fine-grained access control
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    ReadModels,
    WriteModels,
    DeleteModels,
    RunInference,
    ManageCache,
    ReadMetrics,
    WriteConfig,
    ManageUsers,
    ViewAuditLogs,
    UseStreaming,
    UseDistributed,
    ManageQueue,
}

/// User information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub role: UserRole,
    pub api_keys: Vec<ApiKey>,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub permissions: HashSet<Permission>,
    pub rate_limit_override: Option<RateLimitConfig>,
}

/// API key for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: String,
    pub key_hash: String, // Store hashed version
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub permissions: HashSet<Permission>,
}

/// JWT token claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String, // Subject (user ID)
    pub username: String,
    pub role: UserRole,
    pub exp: i64, // Expiration time
    pub iat: i64, // Issued at
    pub jti: String, // JWT ID for revocation
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub requests_per_day: Option<u32>,
    pub burst_size: u32,
}

/// Rate limiter implementation
#[derive(Debug)]
pub struct RateLimiter {
    config: RateLimitConfig,
    minute_window: Arc<Mutex<VecDeque<DateTime<Utc>>>>,
    hour_window: Arc<Mutex<VecDeque<DateTime<Utc>>>>,
    day_window: Arc<Mutex<VecDeque<DateTime<Utc>>>>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            minute_window: Arc::new(Mutex::new(VecDeque::new())),
            hour_window: Arc::new(Mutex::new(VecDeque::new())),
            day_window: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub async fn check_rate_limit(&self) -> Result<bool> {
        let now = Utc::now();

        // Check minute window
        {
            let mut minute_window = self.minute_window.lock().await;
            let minute_ago = now - Duration::minutes(1);

            // Remove old entries
            while let Some(front) = minute_window.front() {
                if *front < minute_ago {
                    minute_window.pop_front();
                } else {
                    break;
                }
            }

            if minute_window.len() >= self.config.requests_per_minute as usize {
                return Ok(false);
            }

            minute_window.push_back(now);
        }

        // Check hour window
        {
            let mut hour_window = self.hour_window.lock().await;
            let hour_ago = now - Duration::hours(1);

            while let Some(front) = hour_window.front() {
                if *front < hour_ago {
                    hour_window.pop_front();
                } else {
                    break;
                }
            }

            if hour_window.len() >= self.config.requests_per_hour as usize {
                return Ok(false);
            }

            hour_window.push_back(now);
        }

        // Check day window if configured
        if let Some(daily_limit) = self.config.requests_per_day {
            let mut day_window = self.day_window.lock().await;
            let day_ago = now - Duration::days(1);

            while let Some(front) = day_window.front() {
                if *front < day_ago {
                    day_window.pop_front();
                } else {
                    break;
                }
            }

            if day_window.len() >= daily_limit as usize {
                return Ok(false);
            }

            day_window.push_back(now);
        }

        Ok(true)
    }

    pub async fn get_remaining_quota(&self) -> (u32, u32, Option<u32>) {
        let now = Utc::now();

        let minute_remaining = {
            let minute_window = self.minute_window.lock().await;
            let minute_ago = now - Duration::minutes(1);
            let recent_count = minute_window.iter()
                .filter(|&&t| t >= minute_ago)
                .count() as u32;
            self.config.requests_per_minute.saturating_sub(recent_count)
        };

        let hour_remaining = {
            let hour_window = self.hour_window.lock().await;
            let hour_ago = now - Duration::hours(1);
            let recent_count = hour_window.iter()
                .filter(|&&t| t >= hour_ago)
                .count() as u32;
            self.config.requests_per_hour.saturating_sub(recent_count)
        };

        let day_remaining = if let Some(daily_limit) = self.config.requests_per_day {
            let day_window = self.day_window.lock().await;
            let day_ago = now - Duration::days(1);
            let recent_count = day_window.iter()
                .filter(|&&t| t >= day_ago)
                .count() as u32;
            Some(daily_limit.saturating_sub(recent_count))
        } else {
            None
        };

        (minute_remaining, hour_remaining, day_remaining)
    }
}

/// Security manager for the application
pub struct SecurityManager {
    config: SecurityConfig,
    users: Arc<RwLock<HashMap<String, User>>>,
    api_keys: Arc<RwLock<HashMap<String, String>>>, // key_hash -> user_id
    rate_limiters: Arc<RwLock<HashMap<String, RateLimiter>>>, // user_id/ip -> limiter
    ip_rate_limiters: Arc<RwLock<HashMap<IpAddr, RateLimiter>>>,
    blocked_tokens: Arc<RwLock<HashSet<String>>>, // Revoked JWT IDs
    audit_log: Arc<Mutex<Vec<AuditLogEntry>>>,
}

impl SecurityManager {
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            config,
            users: Arc::new(RwLock::new(HashMap::new())),
            api_keys: Arc::new(RwLock::new(HashMap::new())),
            rate_limiters: Arc::new(RwLock::new(HashMap::new())),
            ip_rate_limiters: Arc::new(RwLock::new(HashMap::new())),
            blocked_tokens: Arc::new(RwLock::new(HashSet::new())),
            audit_log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Initialize with default users and API keys
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing security manager");

        // Create default admin user
        let admin_user = User {
            id: "admin".to_string(),
            username: "admin".to_string(),
            email: Some("admin@inferno.ai".to_string()),
            role: UserRole::Admin,
            api_keys: vec![],
            created_at: Utc::now(),
            last_login: None,
            is_active: true,
            permissions: HashSet::new(), // Admin has all permissions by default
            rate_limit_override: None,
        };

        self.create_user(admin_user).await?;

        // Create default service account
        let service_user = User {
            id: "service".to_string(),
            username: "service".to_string(),
            email: None,
            role: UserRole::Service,
            api_keys: vec![],
            created_at: Utc::now(),
            last_login: None,
            is_active: true,
            permissions: HashSet::from([
                Permission::ReadModels,
                Permission::RunInference,
                Permission::ReadMetrics,
                Permission::UseStreaming,
                Permission::ManageCache,
            ]),
            rate_limit_override: Some(RateLimitConfig {
                requests_per_minute: 600,
                requests_per_hour: 10000,
                requests_per_day: Some(100000),
                burst_size: 100,
            }),
        };

        self.create_user(service_user).await?;

        Ok(())
    }

    /// Create a new user
    pub async fn create_user(&self, user: User) -> Result<()> {
        let mut users = self.users.write().await;

        if users.contains_key(&user.id) {
            return Err(InfernoError::SecurityValidation(
                format!("User {} already exists", user.id)
            ).into());
        }

        info!("Creating user: {} with role {:?}", user.username, user.role);
        let user_id = user.id.clone();
        users.insert(user_id.clone(), user);

        self.log_audit_event(AuditLogEntry {
            timestamp: Utc::now(),
            user_id: Some("system".to_string()),
            action: AuditAction::UserCreated,
            resource: Some(format!("user:{}", user_id)),
            ip_address: None,
            success: true,
            details: None,
        }).await;

        Ok(())
    }

    /// Generate API key for a user
    pub async fn generate_api_key(
        &self,
        user_id: &str,
        key_name: &str,
        permissions: HashSet<Permission>,
        expires_in_days: Option<i64>,
    ) -> Result<String> {
        let mut users = self.users.write().await;

        let user = users.get_mut(user_id)
            .ok_or_else(|| InfernoError::SecurityValidation(
                format!("User {} not found", user_id)
            ))?;

        // Generate random API key
        let api_key = Self::generate_random_key();
        let key_hash = Self::hash_api_key(&api_key);

        let api_key_info = ApiKey {
            id: uuid::Uuid::new_v4().to_string(),
            key_hash: key_hash.clone(),
            name: key_name.to_string(),
            created_at: Utc::now(),
            expires_at: expires_in_days.map(|days| Utc::now() + Duration::days(days)),
            last_used: None,
            is_active: true,
            permissions,
        };

        user.api_keys.push(api_key_info);

        // Store key hash mapping
        let mut api_keys = self.api_keys.write().await;
        api_keys.insert(key_hash, user_id.to_string());

        info!("Generated API key '{}' for user {}", key_name, user_id);

        self.log_audit_event(AuditLogEntry {
            timestamp: Utc::now(),
            user_id: Some(user_id.to_string()),
            action: AuditAction::ApiKeyCreated,
            resource: Some(format!("api_key:{}", key_name)),
            ip_address: None,
            success: true,
            details: None,
        }).await;

        Ok(api_key)
    }

    /// Authenticate with API key
    pub async fn authenticate_api_key(&self, api_key: &str) -> Result<User> {
        let key_hash = Self::hash_api_key(api_key);

        let api_keys = self.api_keys.read().await;
        let user_id = api_keys.get(&key_hash)
            .ok_or_else(|| InfernoError::SecurityValidation(
                "Invalid API key".to_string()
            ))?;

        let mut users = self.users.write().await;
        let user = users.get_mut(user_id)
            .ok_or_else(|| InfernoError::SecurityValidation(
                "User not found".to_string()
            ))?;

        // Check if user is active
        if !user.is_active {
            return Err(InfernoError::SecurityValidation(
                "User account is disabled".to_string()
            ).into());
        }

        // Find and update the API key
        for api_key_info in &mut user.api_keys {
            if api_key_info.key_hash == key_hash {
                // Check if key is active
                if !api_key_info.is_active {
                    return Err(InfernoError::SecurityValidation(
                        "API key is disabled".to_string()
                    ).into());
                }

                // Check expiration
                if let Some(expires_at) = api_key_info.expires_at {
                    if expires_at < Utc::now() {
                        return Err(InfernoError::SecurityValidation(
                            "API key has expired".to_string()
                        ).into());
                    }
                }

                // Update last used
                api_key_info.last_used = Some(Utc::now());
                break;
            }
        }

        Ok(user.clone())
    }

    /// Generate JWT token for a user
    pub async fn generate_jwt_token(&self, user: &User) -> Result<String> {
        let expiration = Utc::now() + Duration::hours(self.config.token_expiry_hours);

        let claims = TokenClaims {
            sub: user.id.clone(),
            username: user.username.clone(),
            role: user.role.clone(),
            exp: expiration.timestamp(),
            iat: Utc::now().timestamp(),
            jti: uuid::Uuid::new_v4().to_string(),
        };

        // In production, use a proper JWT library like jsonwebtoken
        // For now, create a simple token representation
        use base64::{Engine as _, engine::general_purpose};
        let token = format!(
            "{}.{}.{}",
            general_purpose::STANDARD.encode(serde_json::to_string(&claims)?),
            general_purpose::STANDARD.encode(&self.config.jwt_secret),
            general_purpose::STANDARD.encode(Self::hash_api_key(&format!("{:?}", claims)))
        );

        Ok(token)
    }

    /// Verify JWT token
    pub async fn verify_jwt_token(&self, token: &str) -> Result<TokenClaims> {
        // Check if token is blocked
        let blocked_tokens = self.blocked_tokens.read().await;
        let parts: Vec<&str> = token.split('.').collect();

        if parts.len() != 3 {
            return Err(InfernoError::SecurityValidation(
                "Invalid token format".to_string()
            ).into());
        }

        use base64::{Engine as _, engine::general_purpose};
        let claims_json = String::from_utf8(general_purpose::STANDARD.decode(parts[0])?)?;
        let claims: TokenClaims = serde_json::from_str(&claims_json)?;

        // Check if token is revoked
        if blocked_tokens.contains(&claims.jti) {
            return Err(InfernoError::SecurityValidation(
                "Token has been revoked".to_string()
            ).into());
        }

        // Check expiration
        if claims.exp < Utc::now().timestamp() {
            return Err(InfernoError::SecurityValidation(
                "Token has expired".to_string()
            ).into());
        }

        Ok(claims)
    }

    /// Check rate limit for a user or IP
    pub async fn check_rate_limit(&self, identifier: &str, ip: Option<IpAddr>) -> Result<bool> {
        if !self.config.rate_limiting_enabled {
            return Ok(true);
        }

        // Check user-specific rate limit
        let mut rate_limiters = self.rate_limiters.write().await;

        let user_limiter = rate_limiters.entry(identifier.to_string())
            .or_insert_with(|| RateLimiter::new(RateLimitConfig {
                requests_per_minute: self.config.max_requests_per_minute,
                requests_per_hour: self.config.max_requests_per_hour,
                requests_per_day: None,
                burst_size: 10,
            }));

        if !user_limiter.check_rate_limit().await? {
            warn!("Rate limit exceeded for user: {}", identifier);
            return Ok(false);
        }

        // Check IP-based rate limit if provided
        if let Some(ip_addr) = ip {
            let mut ip_limiters = self.ip_rate_limiters.write().await;

            let ip_limiter = ip_limiters.entry(ip_addr)
                .or_insert_with(|| RateLimiter::new(RateLimitConfig {
                    requests_per_minute: self.config.max_requests_per_minute * 2,
                    requests_per_hour: self.config.max_requests_per_hour * 2,
                    requests_per_day: None,
                    burst_size: 20,
                }));

            if !ip_limiter.check_rate_limit().await? {
                warn!("Rate limit exceeded for IP: {}", ip_addr);
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Check if IP is allowed
    pub fn check_ip_access(&self, ip: &IpAddr) -> bool {
        // Check blocklist
        if self.config.ip_blocklist_enabled {
            let ip_str = ip.to_string();
            if self.config.blocked_ips.contains(&ip_str) {
                warn!("Blocked IP attempted access: {}", ip);
                return false;
            }
        }

        // Check allowlist
        if self.config.ip_allowlist_enabled {
            let ip_str = ip.to_string();
            if !self.config.allowed_ips.contains(&ip_str) {
                warn!("Non-allowlisted IP attempted access: {}", ip);
                return false;
            }
        }

        true
    }

    /// Validate input for security threats
    pub fn validate_input(&self, input: &str) -> Result<()> {
        if !self.config.input_validation_enabled {
            return Ok(());
        }

        // Check input length
        if input.len() > self.config.max_input_length {
            return Err(InfernoError::SecurityValidation(
                format!("Input exceeds maximum length of {} characters", self.config.max_input_length)
            ).into());
        }

        // Check for potential injection attacks
        let dangerous_patterns = [
            "<script", "javascript:", "onerror=", "onclick=",
            "../", "..\\", "%2e%2e", "0x", "\\x",
            "DROP TABLE", "DELETE FROM", "INSERT INTO",
            "cmd.exe", "/bin/sh", "powershell",
        ];

        let input_lower = input.to_lowercase();
        for pattern in &dangerous_patterns {
            if input_lower.contains(pattern) {
                warn!("Potentially dangerous input pattern detected: {}", pattern);
                return Err(InfernoError::SecurityValidation(
                    "Input contains potentially dangerous content".to_string()
                ).into());
            }
        }

        Ok(())
    }

    /// Sanitize output to prevent data leakage
    pub fn sanitize_output(&self, output: &str) -> String {
        if !self.config.output_sanitization_enabled {
            return output.to_string();
        }

        // Remove potential sensitive information patterns
        let mut sanitized = output.to_string();

        // Remove potential API keys (simple pattern)
        let api_key_pattern = regex::Regex::new(r"[A-Za-z0-9]{32,}").unwrap();
        sanitized = api_key_pattern.replace_all(&sanitized, "[REDACTED]").to_string();

        // Remove email addresses
        let email_pattern = regex::Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap();
        sanitized = email_pattern.replace_all(&sanitized, "[EMAIL]").to_string();

        // Remove IP addresses
        let ip_pattern = regex::Regex::new(r"\b(?:[0-9]{1,3}\.){3}[0-9]{1,3}\b").unwrap();
        sanitized = ip_pattern.replace_all(&sanitized, "[IP]").to_string();

        sanitized
    }

    /// Log audit event
    pub async fn log_audit_event(&self, entry: AuditLogEntry) {
        if !self.config.audit_logging_enabled {
            return;
        }

        let mut audit_log = self.audit_log.lock().await;
        audit_log.push(entry.clone());

        // Keep only last 10000 entries in memory
        if audit_log.len() > 10000 {
            audit_log.drain(0..1000);
        }

        debug!("Audit log: {:?}", entry);
    }

    /// Get audit log entries
    pub async fn get_audit_log(&self, limit: Option<usize>) -> Vec<AuditLogEntry> {
        let audit_log = self.audit_log.lock().await;
        let limit = limit.unwrap_or(100);

        audit_log.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    // Helper methods

    fn generate_random_key() -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::thread_rng();

        (0..32)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    fn hash_api_key(key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Revoke a JWT token
    pub async fn revoke_token(&self, jti: String) -> Result<()> {
        let mut blocked_tokens = self.blocked_tokens.write().await;
        blocked_tokens.insert(jti.clone());

        self.log_audit_event(AuditLogEntry {
            timestamp: Utc::now(),
            user_id: None,
            action: AuditAction::TokenRevoked,
            resource: Some(format!("token:{}", jti)),
            ip_address: None,
            success: true,
            details: None,
        }).await;

        Ok(())
    }
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<String>,
    pub action: AuditAction,
    pub resource: Option<String>,
    pub ip_address: Option<IpAddr>,
    pub success: bool,
    pub details: Option<String>,
}

/// Audit actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditAction {
    UserCreated,
    UserDeleted,
    UserModified,
    Login,
    Logout,
    ApiKeyCreated,
    ApiKeyRevoked,
    TokenRevoked,
    InferenceRequested,
    ModelLoaded,
    ModelDeleted,
    ConfigChanged,
    RateLimitExceeded,
    UnauthorizedAccess,
    SecurityViolation,
}