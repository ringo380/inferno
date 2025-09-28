use crate::InfernoError;
use anyhow::Result;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet, VecDeque};
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info, warn};

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
    /// Data directory for persistent storage
    pub data_dir: PathBuf,
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
            data_dir: dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("inferno")
                .join("security"),
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
    pub password_hash: Option<String>, // Argon2 hash
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
    pub exp: i64,    // Expiration time
    pub iat: i64,    // Issued at
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
            let recent_count = minute_window.iter().filter(|&&t| t >= minute_ago).count() as u32;
            self.config.requests_per_minute.saturating_sub(recent_count)
        };

        let hour_remaining = {
            let hour_window = self.hour_window.lock().await;
            let hour_ago = now - Duration::hours(1);
            let recent_count = hour_window.iter().filter(|&&t| t >= hour_ago).count() as u32;
            self.config.requests_per_hour.saturating_sub(recent_count)
        };

        let day_remaining = if let Some(daily_limit) = self.config.requests_per_day {
            let day_window = self.day_window.lock().await;
            let day_ago = now - Duration::days(1);
            let recent_count = day_window.iter().filter(|&&t| t >= day_ago).count() as u32;
            Some(daily_limit.saturating_sub(recent_count))
        } else {
            None
        };

        (minute_remaining, hour_remaining, day_remaining)
    }
}

/// Security manager for the application
#[derive(Debug)]
pub struct SecurityManager {
    config: SecurityConfig,
    pub users: Arc<RwLock<HashMap<String, User>>>,
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

    /// Initialize with default users and API keys (legacy method)
    pub async fn initialize_default_users(&self) -> Result<()> {
        info!("Initializing security manager");

        // Create default admin user
        let admin_password_hash = self.hash_password("admin123")?; // Default password
        let admin_user = User {
            id: "admin".to_string(),
            username: "admin".to_string(),
            email: Some("admin@inferno.ai".to_string()),
            password_hash: Some(admin_password_hash),
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
            password_hash: None, // Service accounts use API keys instead
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
            return Err(InfernoError::Security(format!(
                "User {} already exists",
                user.id
            ))
            .into());
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
        })
        .await;

        // Release the write lock before saving
        drop(users);

        // Save users to persistent storage
        self.save_users().await?;

        Ok(())
    }

    /// Delete a user
    pub async fn delete_user(&self, user_id: &str) -> Result<()> {
        let mut users = self.users.write().await;

        let user = users.remove(user_id).ok_or_else(|| {
            InfernoError::Security(format!("User {} not found", user_id))
        })?;

        info!("Deleting user: {} ({})", user.username, user_id);

        // Remove associated API keys
        let mut api_keys = self.api_keys.write().await;
        let keys_to_remove: Vec<String> = api_keys
            .iter()
            .filter(|(_, uid)| *uid == user_id)
            .map(|(key_hash, _)| key_hash.clone())
            .collect();

        for key_hash in keys_to_remove {
            api_keys.remove(&key_hash);
        }

        // Remove rate limiters
        let mut rate_limiters = self.rate_limiters.write().await;
        rate_limiters.remove(user_id);

        self.log_audit_event(AuditLogEntry {
            timestamp: Utc::now(),
            user_id: Some("system".to_string()),
            action: AuditAction::UserDeleted,
            resource: Some(format!("user:{}", user_id)),
            ip_address: None,
            success: true,
            details: Some(format!("Deleted user: {}", user.username)),
        })
        .await;

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

        let user = users.get_mut(user_id).ok_or_else(|| {
            InfernoError::Security(format!("User {} not found", user_id))
        })?;

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
        })
        .await;

        Ok(api_key)
    }

    /// Authenticate with API key
    pub async fn authenticate_api_key(&self, api_key: &str) -> Result<User> {
        let key_hash = Self::hash_api_key(api_key);

        let api_keys = self.api_keys.read().await;
        let user_id = api_keys
            .get(&key_hash)
            .ok_or_else(|| InfernoError::Security("Invalid API key".to_string()))?;

        let mut users = self.users.write().await;
        let user = users
            .get_mut(user_id)
            .ok_or_else(|| InfernoError::Security("User not found".to_string()))?;

        // Check if user is active
        if !user.is_active {
            return Err(
                InfernoError::Security("User account is disabled".to_string()).into(),
            );
        }

        // Find and update the API key
        for api_key_info in &mut user.api_keys {
            if api_key_info.key_hash == key_hash {
                // Check if key is active
                if !api_key_info.is_active {
                    return Err(InfernoError::Security(
                        "API key is disabled".to_string(),
                    )
                    .into());
                }

                // Check expiration
                if let Some(expires_at) = api_key_info.expires_at {
                    if expires_at < Utc::now() {
                        return Err(InfernoError::Security(
                            "API key has expired".to_string(),
                        )
                        .into());
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

        let header = Header::new(Algorithm::HS256);
        let encoding_key = EncodingKey::from_secret(self.config.jwt_secret.as_ref());

        let token = encode(&header, &claims, &encoding_key)
            .map_err(|e| InfernoError::Security(format!("JWT encoding failed: {}", e)))?;

        Ok(token)
    }

    /// Verify JWT token
    pub async fn verify_jwt_token(&self, token: &str) -> Result<TokenClaims> {
        let decoding_key = DecodingKey::from_secret(self.config.jwt_secret.as_ref());
        let validation = Validation::new(Algorithm::HS256);

        let token_data = decode::<TokenClaims>(token, &decoding_key, &validation)
            .map_err(|e| InfernoError::Security(format!("JWT verification failed: {}", e)))?;

        let claims = token_data.claims;

        // Check if token is revoked
        let blocked_tokens = self.blocked_tokens.read().await;
        if blocked_tokens.contains(&claims.jti) {
            return Err(
                InfernoError::Security("Token has been revoked".to_string()).into(),
            );
        }

        // Check expiration
        if claims.exp < Utc::now().timestamp() {
            return Err(InfernoError::Security("Token has expired".to_string()).into());
        }

        Ok(claims)
    }

    /// Hash a password using Argon2
    pub fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| InfernoError::Security(format!("Password hashing failed: {}", e)))?;

        Ok(password_hash.to_string())
    }

    /// Verify a password against its hash
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| InfernoError::Security(format!("Invalid password hash: {}", e)))?;

        let argon2 = Argon2::default();

        match argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Authenticate user with username and password
    pub async fn authenticate_user(&self, username: &str, password: &str) -> Result<Option<User>> {
        let users = self.users.read().await;

        // Find user by username
        let user = users.values().find(|u| u.username == username && u.is_active);

        if let Some(user) = user {
            if let Some(ref stored_hash) = user.password_hash {
                if self.verify_password(password, stored_hash)? {
                    // Update last login time
                    let mut user_copy = user.clone();
                    user_copy.last_login = Some(Utc::now());
                    let user_id = user_copy.id.clone();

                    // Drop read lock to acquire write lock
                    drop(users);

                    // Update user in storage
                    {
                        let mut users_write = self.users.write().await;
                        users_write.insert(user_id, user_copy.clone());
                    }

                    // Save to persistent storage
                    if let Err(e) = self.save_users().await {
                        warn!("Failed to save user update after login: {}", e);
                    }

                    return Ok(Some(user_copy));
                }
            }
        }

        Ok(None)
    }

    /// Check rate limit for a user or IP
    pub async fn check_rate_limit(&self, identifier: &str, ip: Option<IpAddr>) -> Result<bool> {
        if !self.config.rate_limiting_enabled {
            return Ok(true);
        }

        // Check user-specific rate limit
        let mut rate_limiters = self.rate_limiters.write().await;

        let user_limiter = rate_limiters
            .entry(identifier.to_string())
            .or_insert_with(|| {
                RateLimiter::new(RateLimitConfig {
                    requests_per_minute: self.config.max_requests_per_minute,
                    requests_per_hour: self.config.max_requests_per_hour,
                    requests_per_day: None,
                    burst_size: 10,
                })
            });

        if !user_limiter.check_rate_limit().await? {
            warn!("Rate limit exceeded for user: {}", identifier);
            return Ok(false);
        }

        // Check IP-based rate limit if provided
        if let Some(ip_addr) = ip {
            let mut ip_limiters = self.ip_rate_limiters.write().await;

            let ip_limiter = ip_limiters.entry(ip_addr).or_insert_with(|| {
                RateLimiter::new(RateLimitConfig {
                    requests_per_minute: self.config.max_requests_per_minute * 2,
                    requests_per_hour: self.config.max_requests_per_hour * 2,
                    requests_per_day: None,
                    burst_size: 20,
                })
            });

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
            return Err(InfernoError::Security(format!(
                "Input exceeds maximum length of {} characters",
                self.config.max_input_length
            ))
            .into());
        }

        // Check for potential injection attacks
        let dangerous_patterns = [
            "<script",
            "javascript:",
            "onerror=",
            "onclick=",
            "../",
            "..\\",
            "%2e%2e",
            "0x",
            "\\x",
            "DROP TABLE",
            "DELETE FROM",
            "INSERT INTO",
            "cmd.exe",
            "/bin/sh",
            "powershell",
        ];

        let input_lower = input.to_lowercase();
        for pattern in &dangerous_patterns {
            if input_lower.contains(pattern) {
                warn!("Potentially dangerous input pattern detected: {}", pattern);
                return Err(InfernoError::Security(
                    "Input contains potentially dangerous content".to_string(),
                )
                .into());
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
        sanitized = api_key_pattern
            .replace_all(&sanitized, "[REDACTED]")
            .to_string();

        // Remove email addresses
        let email_pattern =
            regex::Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap();
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

        audit_log.iter().rev().take(limit).cloned().collect()
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
        })
        .await;

        Ok(())
    }

    /// Save users to persistent storage
    async fn save_users(&self) -> Result<()> {
        let users_file = self.config.data_dir.join("users.json");

        // Create directory if it doesn't exist
        if let Some(parent) = users_file.parent() {
            fs::create_dir_all(parent).await?;
        }

        let users = self.users.read().await;
        let serialized = serde_json::to_string_pretty(&*users)?;
        fs::write(&users_file, serialized).await?;

        debug!("Saved {} users to {}", users.len(), users_file.display());
        Ok(())
    }

    /// Load users from persistent storage
    async fn load_users(&self) -> Result<()> {
        let users_file = self.config.data_dir.join("users.json");

        if !users_file.exists() {
            debug!("Users file does not exist, starting with empty user store");
            return Ok(());
        }

        let content = fs::read_to_string(&users_file).await?;
        let loaded_users: HashMap<String, User> = serde_json::from_str(&content)?;

        let mut users = self.users.write().await;
        *users = loaded_users;

        info!("Loaded {} users from {}", users.len(), users_file.display());
        Ok(())
    }

    /// Initialize the security manager with persistence
    pub async fn initialize(&self) -> Result<()> {
        // Load existing users
        if let Err(e) = self.load_users().await {
            warn!("Failed to load users from storage: {}. Creating default admin user.", e);
        }

        // Create default admin user if no users exist
        let users_count = {
            let users = self.users.read().await;
            users.len()
        };

        if users_count == 0 {
            info!("No users found, creating default admin user");
            let default_user = User {
                id: "admin".to_string(),
                username: "admin".to_string(),
                email: Some("admin@localhost".to_string()),
                password_hash: Some("admin123".to_string()), // Simplified for now
                role: UserRole::Admin,
                api_keys: vec![],
                created_at: chrono::Utc::now(),
                last_login: None,
                is_active: true,
                permissions: [
                    Permission::ReadModels,
                    Permission::WriteModels,
                    Permission::DeleteModels,
                    Permission::RunInference,
                    Permission::ManageCache,
                    Permission::ReadMetrics,
                    Permission::WriteConfig,
                    Permission::ManageUsers,
                    Permission::ViewAuditLogs,
                    Permission::UseStreaming,
                    Permission::UseDistributed,
                    Permission::ManageQueue,
                ].into_iter().collect(),
                rate_limit_override: None,
            };
            self.create_user(default_user).await?;
            self.save_users().await?;
        }

        Ok(())
    }

    /// Get all users for admin purposes
    pub async fn get_all_users(&self) -> Vec<User> {
        let users = self.users.read().await;
        users.values().cloned().collect()
    }

    /// Get a specific user by ID
    pub async fn get_user_by_id(&self, user_id: &str) -> Option<User> {
        let users = self.users.read().await;
        users.get(user_id).cloned()
    }

    /// Update a user
    pub async fn update_user(&self, user_id: &str, updated_user: User) -> Result<()> {
        let mut users = self.users.write().await;
        if users.contains_key(user_id) {
            users.insert(user_id.to_string(), updated_user);
            Ok(())
        } else {
            Err(anyhow::anyhow!("User not found"))
        }
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
    ModelVerificationStarted,
    ModelVerificationCompleted,
    ModelVerificationFailed,
    SecurityScanStarted,
    SecurityScanCompleted,
    SecurityScanFailed,
}

/// Comprehensive security scanner for models and system components
#[derive(Debug)]
pub struct SecurityScanner {
    config: SecurityScanConfig,
    threat_signatures: ThreatSignatureDatabase,
    audit_logger: Arc<SecurityManager>,
}

/// Security scan configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScanConfig {
    /// Enable file structure validation
    pub validate_file_structure: bool,
    /// Enable content scanning for embedded threats
    pub scan_embedded_content: bool,
    /// Enable metadata threat scanning
    pub scan_metadata_threats: bool,
    /// Enable signature verification
    pub verify_signatures: bool,
    /// Enable checksum verification
    pub verify_checksums: bool,
    /// Maximum file size to scan (bytes)
    pub max_scan_size: u64,
    /// Quarantine suspicious files
    pub quarantine_enabled: bool,
    /// Quarantine directory path
    pub quarantine_dir: PathBuf,
}

impl Default for SecurityScanConfig {
    fn default() -> Self {
        Self {
            validate_file_structure: true,
            scan_embedded_content: true,
            scan_metadata_threats: true,
            verify_signatures: false, // Requires signature infrastructure
            verify_checksums: true,
            max_scan_size: 50_000_000_000, // 50GB
            quarantine_enabled: true,
            quarantine_dir: PathBuf::from("./quarantine"),
        }
    }
}

/// Threat signature database for pattern matching
#[derive(Debug)]
struct ThreatSignatureDatabase {
    executable_patterns: Vec<Vec<u8>>,
    script_patterns: Vec<Vec<u8>>,
    suspicious_strings: Vec<String>,
    metadata_threats: Vec<String>,
}

impl Default for ThreatSignatureDatabase {
    fn default() -> Self {
        Self {
            executable_patterns: vec![
                b"\x4d\x5a".to_vec(),           // PE header (Windows executable)
                b"\x7f\x45\x4c\x46".to_vec(),  // ELF header (Linux executable)
                b"\xfe\xed\xfa\xce".to_vec(),  // Mach-O header (macOS executable)
                b"\xfe\xed\xfa\xcf".to_vec(),  // Mach-O header (macOS executable)
                b"\xca\xfe\xba\xbe".to_vec(),  // Java class file
                b"\x50\x4b\x03\x04".to_vec(),  // ZIP file header
            ],
            script_patterns: vec![
                b"#!/bin/sh".to_vec(),
                b"#!/bin/bash".to_vec(),
                b"#!/usr/bin/env".to_vec(),
                b"<script".to_vec(),
                b"javascript:".to_vec(),
                b"data:text/html".to_vec(),
                b"eval(".to_vec(),
                b"exec(".to_vec(),
            ],
            suspicious_strings: vec![
                "password".to_string(),
                "api_key".to_string(),
                "secret".to_string(),
                "token".to_string(),
                "private_key".to_string(),
                "ssh_key".to_string(),
                "credential".to_string(),
                "backdoor".to_string(),
                "exploit".to_string(),
                "payload".to_string(),
            ],
            metadata_threats: vec![
                "exec".to_string(),
                "execute".to_string(),
                "script".to_string(),
                "command".to_string(),
                "shell".to_string(),
                "eval".to_string(),
                "import".to_string(),
                "require".to_string(),
                "load".to_string(),
                "include".to_string(),
                "__import__".to_string(),
                "subprocess".to_string(),
                "os.system".to_string(),
            ],
        }
    }
}

/// Security scan result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScanResult {
    pub file_path: PathBuf,
    pub scan_timestamp: DateTime<Utc>,
    pub scan_duration_ms: u64,
    pub threats_detected: Vec<ThreatDetection>,
    pub overall_risk_level: RiskLevel,
    pub file_quarantined: bool,
    pub scan_success: bool,
    pub error_message: Option<String>,
}

/// Individual threat detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatDetection {
    pub threat_type: ThreatType,
    pub severity: ThreatSeverity,
    pub description: String,
    pub location: Option<String>,
    pub mitigation_advice: String,
}

/// Types of threats that can be detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatType {
    EmbeddedExecutable,
    SuspiciousScript,
    DataExfiltration,
    MetadataThreats,
    InvalidFileStructure,
    SuspiciousSize,
    UnknownFormat,
    ChecksumMismatch,
    SignatureInvalid,
    PolicyViolation,
}

/// Threat severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Overall risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Critical,
    High,
    Medium,
    Low,
    Safe,
}

impl SecurityScanner {
    /// Create new security scanner
    pub fn new(config: SecurityScanConfig, audit_logger: Arc<SecurityManager>) -> Self {
        Self {
            config,
            threat_signatures: ThreatSignatureDatabase::default(),
            audit_logger,
        }
    }

    /// Perform comprehensive security scan on a file
    pub async fn scan_file(&self, file_path: &Path) -> Result<SecurityScanResult> {
        let start_time = std::time::Instant::now();
        let scan_timestamp = Utc::now();

        info!("Starting security scan for: {}", file_path.display());

        // Log audit event
        self.audit_logger.log_audit_event(AuditLogEntry {
            timestamp: scan_timestamp,
            user_id: None,
            action: AuditAction::SecurityScanStarted,
            resource: Some(file_path.to_string_lossy().to_string()),
            ip_address: None,
            success: true,
            details: None,
        }).await;

        let mut threats = Vec::new();
        let mut scan_success = true;
        let mut error_message = None;

        // Check if file exists and is accessible
        if !file_path.exists() {
            let error = "File does not exist".to_string();
            error_message = Some(error.clone());
            scan_success = false;

            return Ok(SecurityScanResult {
                file_path: file_path.to_path_buf(),
                scan_timestamp,
                scan_duration_ms: start_time.elapsed().as_millis() as u64,
                threats_detected: threats,
                overall_risk_level: RiskLevel::High,
                file_quarantined: false,
                scan_success,
                error_message,
            });
        }

        // Check file size
        let metadata = match tokio::fs::metadata(file_path).await {
            Ok(meta) => meta,
            Err(e) => {
                error_message = Some(format!("Failed to read file metadata: {}", e));
                scan_success = false;

                return Ok(SecurityScanResult {
                    file_path: file_path.to_path_buf(),
                    scan_timestamp,
                    scan_duration_ms: start_time.elapsed().as_millis() as u64,
                    threats_detected: threats,
                    overall_risk_level: RiskLevel::High,
                    file_quarantined: false,
                    scan_success,
                    error_message,
                });
            }
        };

        let file_size = metadata.len();

        // Check if file is too large to scan
        if file_size > self.config.max_scan_size {
            threats.push(ThreatDetection {
                threat_type: ThreatType::SuspiciousSize,
                severity: ThreatSeverity::Medium,
                description: format!("File size ({} bytes) exceeds maximum scan size", file_size),
                location: None,
                mitigation_advice: "Consider scanning with specialized tools for large files".to_string(),
            });
        } else {
            // Perform detailed scans
            if self.config.validate_file_structure {
                if let Err(e) = self.scan_file_structure(file_path, &mut threats).await {
                    warn!("File structure scan failed: {}", e);
                }
            }

            if self.config.scan_embedded_content {
                if let Err(e) = self.scan_embedded_content(file_path, &mut threats).await {
                    warn!("Embedded content scan failed: {}", e);
                }
            }

            if self.config.scan_metadata_threats {
                if let Err(e) = self.scan_metadata_threats(file_path, &mut threats).await {
                    warn!("Metadata threat scan failed: {}", e);
                }
            }
        }

        // Assess overall risk level
        let overall_risk_level = self.assess_risk_level(&threats);

        // Quarantine file if necessary
        let mut file_quarantined = false;
        if self.config.quarantine_enabled && matches!(overall_risk_level, RiskLevel::Critical | RiskLevel::High) {
            if let Err(e) = self.quarantine_file(file_path).await {
                warn!("Failed to quarantine file: {}", e);
            } else {
                file_quarantined = true;
                info!("File quarantined due to security threats: {}", file_path.display());
            }
        }

        let scan_duration_ms = start_time.elapsed().as_millis() as u64;

        // Log completion
        self.audit_logger.log_audit_event(AuditLogEntry {
            timestamp: Utc::now(),
            user_id: None,
            action: if scan_success { AuditAction::SecurityScanCompleted } else { AuditAction::SecurityScanFailed },
            resource: Some(file_path.to_string_lossy().to_string()),
            ip_address: None,
            success: scan_success,
            details: Some(format!("Threats: {}, Risk: {:?}, Duration: {}ms",
                threats.len(), overall_risk_level, scan_duration_ms)),
        }).await;

        Ok(SecurityScanResult {
            file_path: file_path.to_path_buf(),
            scan_timestamp,
            scan_duration_ms,
            threats_detected: threats,
            overall_risk_level,
            file_quarantined,
            scan_success,
            error_message,
        })
    }

    async fn scan_file_structure(&self, file_path: &Path, threats: &mut Vec<ThreatDetection>) -> Result<()> {
        let extension = file_path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        match extension.to_lowercase().as_str() {
            "gguf" => self.validate_gguf_file(file_path, threats).await?,
            "onnx" => self.validate_onnx_file(file_path, threats).await?,
            "safetensors" => self.validate_safetensors_file(file_path, threats).await?,
            "bin" | "pt" | "pth" => self.validate_pytorch_file(file_path, threats).await?,
            _ => {
                threats.push(ThreatDetection {
                    threat_type: ThreatType::UnknownFormat,
                    severity: ThreatSeverity::Low,
                    description: format!("Unknown file format: {}", extension),
                    location: None,
                    mitigation_advice: "Verify file type and ensure it's a valid model format".to_string(),
                });
            }
        }

        Ok(())
    }

    async fn scan_embedded_content(&self, file_path: &Path, threats: &mut Vec<ThreatDetection>) -> Result<()> {
        let file_content = tokio::fs::read(file_path).await?;

        // Scan for executable patterns
        for pattern in &self.threat_signatures.executable_patterns {
            if let Some(position) = self.find_pattern(&file_content, pattern) {
                threats.push(ThreatDetection {
                    threat_type: ThreatType::EmbeddedExecutable,
                    severity: ThreatSeverity::Critical,
                    description: "Embedded executable code detected".to_string(),
                    location: Some(format!("Byte offset: {}", position)),
                    mitigation_advice: "Remove or verify embedded executable content".to_string(),
                });
            }
        }

        // Scan for script patterns
        for pattern in &self.threat_signatures.script_patterns {
            if let Some(position) = self.find_pattern(&file_content, pattern) {
                threats.push(ThreatDetection {
                    threat_type: ThreatType::SuspiciousScript,
                    severity: ThreatSeverity::High,
                    description: "Embedded script code detected".to_string(),
                    location: Some(format!("Byte offset: {}", position)),
                    mitigation_advice: "Review and validate embedded script content".to_string(),
                });
            }
        }

        // Check for excessive printable characters (potential data exfiltration)
        let printable_count = file_content.iter()
            .filter(|&b| *b >= 32 && *b <= 126)
            .count();
        let printable_ratio = printable_count as f64 / file_content.len() as f64;

        if printable_ratio > 0.7 {
            threats.push(ThreatDetection {
                threat_type: ThreatType::DataExfiltration,
                severity: ThreatSeverity::Medium,
                description: format!("High ratio of printable characters: {:.1}%", printable_ratio * 100.0),
                location: None,
                mitigation_advice: "Review file content for hidden data or text".to_string(),
            });
        }

        // Scan for suspicious strings
        let content_str = String::from_utf8_lossy(&file_content).to_lowercase();
        for suspicious_string in &self.threat_signatures.suspicious_strings {
            if content_str.contains(suspicious_string) {
                threats.push(ThreatDetection {
                    threat_type: ThreatType::DataExfiltration,
                    severity: ThreatSeverity::Medium,
                    description: format!("Suspicious string found: {}", suspicious_string),
                    location: None,
                    mitigation_advice: "Review file for embedded credentials or sensitive data".to_string(),
                });
            }
        }

        Ok(())
    }

    async fn scan_metadata_threats(&self, file_path: &Path, threats: &mut Vec<ThreatDetection>) -> Result<()> {
        let file_content = tokio::fs::read(file_path).await?;
        let content_str = String::from_utf8_lossy(&file_content).to_lowercase();

        for threat_pattern in &self.threat_signatures.metadata_threats {
            if content_str.contains(threat_pattern) {
                threats.push(ThreatDetection {
                    threat_type: ThreatType::MetadataThreats,
                    severity: ThreatSeverity::High,
                    description: format!("Suspicious metadata pattern: {}", threat_pattern),
                    location: None,
                    mitigation_advice: "Review model metadata for malicious content".to_string(),
                });
            }
        }

        Ok(())
    }

    async fn validate_gguf_file(&self, file_path: &Path, threats: &mut Vec<ThreatDetection>) -> Result<()> {
        let file_content = tokio::fs::read(file_path).await?;

        if file_content.len() < 8 {
            threats.push(ThreatDetection {
                threat_type: ThreatType::InvalidFileStructure,
                severity: ThreatSeverity::High,
                description: "GGUF file too small".to_string(),
                location: None,
                mitigation_advice: "Verify file integrity and re-download if necessary".to_string(),
            });
            return Ok(());
        }

        if &file_content[0..4] != b"GGUF" {
            threats.push(ThreatDetection {
                threat_type: ThreatType::InvalidFileStructure,
                severity: ThreatSeverity::High,
                description: "Invalid GGUF magic bytes".to_string(),
                location: Some("File header".to_string()),
                mitigation_advice: "File may be corrupted or not a valid GGUF file".to_string(),
            });
        }

        let version = u32::from_le_bytes([
            file_content[4], file_content[5], file_content[6], file_content[7]
        ]);

        if version < 1 || version > 3 {
            threats.push(ThreatDetection {
                threat_type: ThreatType::InvalidFileStructure,
                severity: ThreatSeverity::Medium,
                description: format!("Unsupported GGUF version: {}", version),
                location: Some("Version header".to_string()),
                mitigation_advice: "Update to a supported GGUF version".to_string(),
            });
        }

        Ok(())
    }

    async fn validate_onnx_file(&self, file_path: &Path, _threats: &mut Vec<ThreatDetection>) -> Result<()> {
        let file_content = tokio::fs::read(file_path).await?;

        if file_content.len() < 16 {
            return Err(anyhow::anyhow!("ONNX file too small"));
        }

        // Basic ONNX validation would go here
        // This is a placeholder for more comprehensive ONNX validation

        Ok(())
    }

    async fn validate_safetensors_file(&self, file_path: &Path, threats: &mut Vec<ThreatDetection>) -> Result<()> {
        let file_content = tokio::fs::read(file_path).await?;

        if file_content.len() < 8 {
            threats.push(ThreatDetection {
                threat_type: ThreatType::InvalidFileStructure,
                severity: ThreatSeverity::High,
                description: "SafeTensors file too small".to_string(),
                location: None,
                mitigation_advice: "Verify file integrity".to_string(),
            });
            return Ok(());
        }

        let header_length = u64::from_le_bytes([
            file_content[0], file_content[1], file_content[2], file_content[3],
            file_content[4], file_content[5], file_content[6], file_content[7]
        ]);

        if header_length > file_content.len() as u64 - 8 {
            threats.push(ThreatDetection {
                threat_type: ThreatType::InvalidFileStructure,
                severity: ThreatSeverity::High,
                description: "Invalid SafeTensors header length".to_string(),
                location: Some("File header".to_string()),
                mitigation_advice: "File may be corrupted".to_string(),
            });
        }

        Ok(())
    }

    async fn validate_pytorch_file(&self, file_path: &Path, threats: &mut Vec<ThreatDetection>) -> Result<()> {
        let file_content = tokio::fs::read(file_path).await?;

        // PyTorch files often start with a ZIP magic number or pickle protocol
        if file_content.len() >= 4 {
            if &file_content[0..4] == b"PK\x03\x04" {
                // ZIP-based PyTorch file
                debug!("Detected ZIP-based PyTorch file");
            } else if file_content[0] == 0x80 {
                // Pickle protocol
                debug!("Detected pickle-based PyTorch file");

                // Pickle files can be dangerous as they can execute arbitrary code
                threats.push(ThreatDetection {
                    threat_type: ThreatType::SuspiciousScript,
                    severity: ThreatSeverity::High,
                    description: "PyTorch pickle file detected - can execute arbitrary code".to_string(),
                    location: None,
                    mitigation_advice: "Use SafeTensors format instead of pickle for security".to_string(),
                });
            }
        }

        Ok(())
    }

    fn find_pattern(&self, haystack: &[u8], needle: &[u8]) -> Option<usize> {
        haystack.windows(needle.len()).position(|window| window == needle)
    }

    fn assess_risk_level(&self, threats: &[ThreatDetection]) -> RiskLevel {
        if threats.is_empty() {
            return RiskLevel::Safe;
        }

        let has_critical = threats.iter().any(|t| matches!(t.severity, ThreatSeverity::Critical));
        let has_high = threats.iter().any(|t| matches!(t.severity, ThreatSeverity::High));
        let medium_count = threats.iter().filter(|t| matches!(t.severity, ThreatSeverity::Medium)).count();

        if has_critical {
            RiskLevel::Critical
        } else if has_high || medium_count >= 3 {
            RiskLevel::High
        } else if medium_count >= 1 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        }
    }

    async fn quarantine_file(&self, file_path: &Path) -> Result<()> {
        // Create quarantine directory if it doesn't exist
        tokio::fs::create_dir_all(&self.config.quarantine_dir).await?;

        // Generate unique quarantine filename
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let original_name = file_path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");

        let quarantine_filename = format!("{}_{}", timestamp, original_name);
        let quarantine_path = self.config.quarantine_dir.join(quarantine_filename);

        // Move file to quarantine
        tokio::fs::rename(file_path, &quarantine_path).await?;

        // Create metadata file
        let metadata_path = quarantine_path.with_extension("quarantine_metadata.json");
        let metadata = serde_json::json!({
            "original_path": file_path,
            "quarantined_at": Utc::now(),
            "reason": "Security scan detected threats"
        });

        tokio::fs::write(&metadata_path, serde_json::to_string_pretty(&metadata)?).await?;

        info!("File quarantined: {} -> {}", file_path.display(), quarantine_path.display());

        Ok(())
    }
}
