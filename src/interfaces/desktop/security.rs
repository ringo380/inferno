#![allow(clippy::clone_on_copy)]

use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ApiKey {
    pub id: String,
    pub name: String,
    pub key_hash: String,
    pub key_prefix: String, // First 8 characters for display
    pub permissions: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub usage_count: u64,
    pub created_by: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SecurityEvent {
    pub id: String,
    pub event_type: SecurityEventType,
    pub severity: SecuritySeverity,
    pub timestamp: DateTime<Utc>,
    pub source_ip: Option<String>,
    pub user_agent: Option<String>,
    pub api_key_id: Option<String>,
    pub description: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum SecurityEventType {
    ApiKeyCreated,
    ApiKeyRevoked,
    ApiKeyUsed,
    UnauthorizedAccess,
    AuthenticationFailed,
    PermissionDenied,
    SuspiciousActivity,
    ConfigurationChanged,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SecurityMetrics {
    pub total_api_keys: u32,
    pub active_api_keys: u32,
    pub expired_api_keys: u32,
    pub security_events_24h: u32,
    pub failed_auth_attempts_24h: u32,
    pub suspicious_activities_24h: u32,
    pub last_security_scan: Option<DateTime<Utc>>,
}

/// Result of a security scan
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SecurityScanResult {
    pub scan_id: String,
    pub timestamp: DateTime<Utc>,
    pub status: String, // "passed" | "warnings" | "failed"
    pub checks_passed: u32,
    pub checks_failed: u32,
    pub checks_warning: u32,
    pub findings: Vec<SecurityFinding>,
    pub recommendations: Vec<String>,
    pub scan_duration_ms: u64,
}

/// Individual security finding from a scan
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SecurityFinding {
    pub id: String,
    pub category: String,
    pub severity: SecuritySeverity,
    pub title: String,
    pub description: String,
    pub remediation: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub permissions: Vec<String>,
    pub expires_in_days: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateApiKeyResponse {
    pub api_key: ApiKey,
    pub raw_key: String, // Only returned once during creation
}

pub struct SecurityManager {
    api_keys: Arc<Mutex<Vec<ApiKey>>>,
    security_events: Arc<Mutex<Vec<SecurityEvent>>>,
    last_security_scan: Arc<Mutex<Option<DateTime<Utc>>>>,
}

impl SecurityManager {
    pub fn new<T>(_database_manager: T) -> Self
    where
        T: Send + Sync + 'static,
    {
        Self {
            api_keys: Arc::new(Mutex::new(Vec::new())),
            security_events: Arc::new(Mutex::new(Vec::new())),
            last_security_scan: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn generate_api_key(
        &self,
        request: CreateApiKeyRequest,
    ) -> Result<CreateApiKeyResponse, String> {
        // Generate a secure random API key
        let raw_key = self.generate_secure_key();
        let key_hash = self.hash_key(&raw_key);
        let key_prefix = raw_key.chars().take(8).collect::<String>();

        let expires_at = request
            .expires_in_days
            .map(|days| Utc::now() + chrono::Duration::days(days as i64));

        let api_key = ApiKey {
            id: Uuid::new_v4().to_string(),
            name: request.name.clone(),
            key_hash,
            key_prefix,
            permissions: request.permissions,
            created_at: Utc::now(),
            last_used: None,
            expires_at,
            is_active: true,
            usage_count: 0,
            created_by: "system".to_string(), // TODO: Replace with actual user when auth is implemented
        };

        // Store the API key
        let mut keys = self.api_keys.lock().map_err(|e| e.to_string())?;
        keys.push(api_key.clone());

        // Log security event
        self.log_security_event(SecurityEvent {
            id: Uuid::new_v4().to_string(),
            event_type: SecurityEventType::ApiKeyCreated,
            severity: SecuritySeverity::Medium,
            timestamp: Utc::now(),
            source_ip: None,
            user_agent: None,
            api_key_id: Some(api_key.id.clone()),
            description: format!("API key '{}' created", request.name),
            metadata: HashMap::new(),
        });

        Ok(CreateApiKeyResponse { api_key, raw_key })
    }

    pub async fn get_api_keys(&self) -> Result<Vec<ApiKey>, String> {
        let keys = self.api_keys.lock().map_err(|e| e.to_string())?;
        Ok(keys.clone())
    }

    pub async fn revoke_api_key(&self, key_id: String) -> Result<(), String> {
        let mut keys = self.api_keys.lock().map_err(|e| e.to_string())?;

        if let Some(key) = keys.iter_mut().find(|k| k.id == key_id) {
            key.is_active = false;

            // Log security event
            self.log_security_event(SecurityEvent {
                id: Uuid::new_v4().to_string(),
                event_type: SecurityEventType::ApiKeyRevoked,
                severity: SecuritySeverity::Medium,
                timestamp: Utc::now(),
                source_ip: None,
                user_agent: None,
                api_key_id: Some(key_id),
                description: format!("API key '{}' revoked", key.name),
                metadata: HashMap::new(),
            });

            Ok(())
        } else {
            Err("API key not found".to_string())
        }
    }

    pub async fn delete_api_key(&self, key_id: String) -> Result<(), String> {
        let mut keys = self.api_keys.lock().map_err(|e| e.to_string())?;
        let initial_len = keys.len();
        keys.retain(|k| k.id != key_id);

        if keys.len() < initial_len {
            // Log security event
            self.log_security_event(SecurityEvent {
                id: Uuid::new_v4().to_string(),
                event_type: SecurityEventType::ApiKeyRevoked,
                severity: SecuritySeverity::High,
                timestamp: Utc::now(),
                source_ip: None,
                user_agent: None,
                api_key_id: Some(key_id),
                description: "API key permanently deleted".to_string(),
                metadata: HashMap::new(),
            });

            Ok(())
        } else {
            Err("API key not found".to_string())
        }
    }

    pub async fn validate_api_key(&self, raw_key: String) -> Result<Option<ApiKey>, String> {
        let key_hash = self.hash_key(&raw_key);
        let mut keys = self.api_keys.lock().map_err(|e| e.to_string())?;

        if let Some(key) = keys
            .iter_mut()
            .find(|k| k.key_hash == key_hash && k.is_active)
        {
            // Check if key is expired
            if let Some(expires_at) = key.expires_at {
                if Utc::now() > expires_at {
                    key.is_active = false;
                    return Ok(None);
                }
            }

            // Update usage statistics
            key.last_used = Some(Utc::now());
            key.usage_count += 1;

            // Log usage event
            self.log_security_event(SecurityEvent {
                id: Uuid::new_v4().to_string(),
                event_type: SecurityEventType::ApiKeyUsed,
                severity: SecuritySeverity::Low,
                timestamp: Utc::now(),
                source_ip: None,
                user_agent: None,
                api_key_id: Some(key.id.clone()),
                description: "API key used successfully".to_string(),
                metadata: HashMap::new(),
            });

            Ok(Some(key.clone()))
        } else {
            // Log failed authentication
            self.log_security_event(SecurityEvent {
                id: Uuid::new_v4().to_string(),
                event_type: SecurityEventType::AuthenticationFailed,
                severity: SecuritySeverity::High,
                timestamp: Utc::now(),
                source_ip: None,
                user_agent: None,
                api_key_id: None,
                description: "Invalid API key used".to_string(),
                metadata: HashMap::new(),
            });

            Ok(None)
        }
    }

    pub async fn get_security_events(
        &self,
        limit: Option<usize>,
    ) -> Result<Vec<SecurityEvent>, String> {
        let events = self.security_events.lock().map_err(|e| e.to_string())?;
        let limit = limit.unwrap_or(100);

        // Return most recent events first
        let mut sorted_events = events.clone();
        sorted_events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        sorted_events.truncate(limit);

        Ok(sorted_events)
    }

    pub async fn get_security_metrics(&self) -> Result<SecurityMetrics, String> {
        let keys = self.api_keys.lock().map_err(|e| e.to_string())?;
        let events = self.security_events.lock().map_err(|e| e.to_string())?;

        let total_api_keys = keys.len() as u32;
        let active_api_keys = keys.iter().filter(|k| k.is_active).count() as u32;
        let expired_api_keys = keys
            .iter()
            .filter(|k| {
                if let Some(expires_at) = k.expires_at {
                    Utc::now() > expires_at
                } else {
                    false
                }
            })
            .count() as u32;

        let twenty_four_hours_ago = Utc::now() - chrono::Duration::hours(24);
        let security_events_24h = events
            .iter()
            .filter(|e| e.timestamp > twenty_four_hours_ago)
            .count() as u32;

        let failed_auth_attempts_24h = events
            .iter()
            .filter(|e| {
                e.timestamp > twenty_four_hours_ago
                    && matches!(e.event_type, SecurityEventType::AuthenticationFailed)
            })
            .count() as u32;

        let suspicious_activities_24h = events
            .iter()
            .filter(|e| {
                e.timestamp > twenty_four_hours_ago
                    && matches!(e.event_type, SecurityEventType::SuspiciousActivity)
            })
            .count() as u32;

        // Get actual last scan time
        let last_scan = self
            .last_security_scan
            .lock()
            .map_err(|e| e.to_string())?
            .clone();

        Ok(SecurityMetrics {
            total_api_keys,
            active_api_keys,
            expired_api_keys,
            security_events_24h,
            failed_auth_attempts_24h,
            suspicious_activities_24h,
            last_security_scan: last_scan,
        })
    }

    pub async fn clear_security_events(&self) -> Result<(), String> {
        let mut events = self.security_events.lock().map_err(|e| e.to_string())?;
        events.clear();
        Ok(())
    }

    /// Run a comprehensive security scan
    ///
    /// This performs multiple security checks:
    /// - API key expiration warnings
    /// - Failed authentication attempt analysis
    /// - Suspicious activity detection
    /// - Configuration security assessment
    pub async fn run_security_scan(&self) -> Result<SecurityScanResult, String> {
        let start = std::time::Instant::now();
        let mut findings = Vec::new();
        let mut recommendations = Vec::new();
        let mut checks_passed = 0u32;
        let mut checks_failed = 0u32;
        let mut checks_warning = 0u32;

        let now = Utc::now();
        let seven_days = chrono::Duration::days(7);
        let twenty_four_hours = chrono::Duration::hours(24);

        // Check 1: API Key Expiration
        {
            let keys = self.api_keys.lock().map_err(|e| e.to_string())?;
            let mut expiring_soon = 0;
            let mut expired = 0;

            for key in keys.iter() {
                if let Some(expires_at) = key.expires_at {
                    if expires_at < now {
                        expired += 1;
                        findings.push(SecurityFinding {
                            id: Uuid::new_v4().to_string(),
                            category: "API Keys".to_string(),
                            severity: SecuritySeverity::Medium,
                            title: format!("API key '{}' has expired", key.name),
                            description: format!(
                                "The API key '{}' expired on {}",
                                key.name,
                                expires_at.format("%Y-%m-%d")
                            ),
                            remediation: Some(
                                "Revoke the expired key and create a new one if still needed."
                                    .to_string(),
                            ),
                        });
                    } else if expires_at < now + seven_days {
                        expiring_soon += 1;
                        findings.push(SecurityFinding {
                            id: Uuid::new_v4().to_string(),
                            category: "API Keys".to_string(),
                            severity: SecuritySeverity::Low,
                            title: format!("API key '{}' expires soon", key.name),
                            description: format!(
                                "The API key '{}' will expire on {}",
                                key.name,
                                expires_at.format("%Y-%m-%d")
                            ),
                            remediation: Some(
                                "Consider renewing the API key before it expires.".to_string(),
                            ),
                        });
                    }
                }
            }

            if expired > 0 {
                checks_warning += 1;
                recommendations.push(format!("Revoke {} expired API key(s)", expired));
            } else {
                checks_passed += 1;
            }

            if expiring_soon > 0 {
                recommendations.push(format!(
                    "Renew {} API key(s) expiring within 7 days",
                    expiring_soon
                ));
            }
        }

        // Check 2: Failed Authentication Attempts
        {
            let events = self.security_events.lock().map_err(|e| e.to_string())?;
            let failed_auth_count = events
                .iter()
                .filter(|e| e.timestamp > now - twenty_four_hours)
                .filter(|e| matches!(e.event_type, SecurityEventType::AuthenticationFailed))
                .count();

            if failed_auth_count > 10 {
                checks_failed += 1;
                findings.push(SecurityFinding {
                    id: Uuid::new_v4().to_string(),
                    category: "Authentication".to_string(),
                    severity: SecuritySeverity::High,
                    title: "High number of failed authentication attempts".to_string(),
                    description: format!("{} failed authentication attempts in the last 24 hours", failed_auth_count),
                    remediation: Some("Review failed authentication logs and consider implementing rate limiting.".to_string()),
                });
                recommendations
                    .push("Investigate the source of failed authentication attempts".to_string());
            } else if failed_auth_count > 5 {
                checks_warning += 1;
                findings.push(SecurityFinding {
                    id: Uuid::new_v4().to_string(),
                    category: "Authentication".to_string(),
                    severity: SecuritySeverity::Medium,
                    title: "Elevated failed authentication attempts".to_string(),
                    description: format!(
                        "{} failed authentication attempts in the last 24 hours",
                        failed_auth_count
                    ),
                    remediation: Some("Monitor authentication logs for patterns.".to_string()),
                });
            } else {
                checks_passed += 1;
            }
        }

        // Check 3: Suspicious Activity
        {
            let events = self.security_events.lock().map_err(|e| e.to_string())?;
            let suspicious_count = events
                .iter()
                .filter(|e| e.timestamp > now - twenty_four_hours)
                .filter(|e| matches!(e.event_type, SecurityEventType::SuspiciousActivity))
                .count();

            if suspicious_count > 0 {
                checks_warning += 1;
                findings.push(SecurityFinding {
                    id: Uuid::new_v4().to_string(),
                    category: "Activity".to_string(),
                    severity: SecuritySeverity::High,
                    title: "Suspicious activity detected".to_string(),
                    description: format!(
                        "{} suspicious activity event(s) in the last 24 hours",
                        suspicious_count
                    ),
                    remediation: Some(
                        "Review the suspicious activity events and take appropriate action."
                            .to_string(),
                    ),
                });
                recommendations.push("Review and address suspicious activity alerts".to_string());
            } else {
                checks_passed += 1;
            }
        }

        // Check 4: Unauthorized Access Attempts
        {
            let events = self.security_events.lock().map_err(|e| e.to_string())?;
            let unauthorized_count = events
                .iter()
                .filter(|e| e.timestamp > now - twenty_four_hours)
                .filter(|e| matches!(e.event_type, SecurityEventType::UnauthorizedAccess))
                .count();

            if unauthorized_count > 0 {
                checks_failed += 1;
                findings.push(SecurityFinding {
                    id: Uuid::new_v4().to_string(),
                    category: "Access Control".to_string(),
                    severity: SecuritySeverity::Critical,
                    title: "Unauthorized access attempts detected".to_string(),
                    description: format!(
                        "{} unauthorized access attempt(s) in the last 24 hours",
                        unauthorized_count
                    ),
                    remediation: Some(
                        "Investigate the source of unauthorized access attempts immediately."
                            .to_string(),
                    ),
                });
                recommendations
                    .push("Investigate unauthorized access attempts immediately".to_string());
            } else {
                checks_passed += 1;
            }
        }

        // Check 5: API Key Usage
        {
            let keys = self.api_keys.lock().map_err(|e| e.to_string())?;
            let unused_keys: Vec<_> = keys
                .iter()
                .filter(|k| k.is_active && k.last_used.is_none())
                .filter(|k| (now - k.created_at) > chrono::Duration::days(30))
                .collect();

            if !unused_keys.is_empty() {
                checks_warning += 1;
                for key in unused_keys {
                    findings.push(SecurityFinding {
                        id: Uuid::new_v4().to_string(),
                        category: "API Keys".to_string(),
                        severity: SecuritySeverity::Low,
                        title: format!("API key '{}' has never been used", key.name),
                        description: format!(
                            "The API key '{}' was created over 30 days ago but has never been used",
                            key.name
                        ),
                        remediation: Some(
                            "Consider revoking unused API keys to reduce security risk."
                                .to_string(),
                        ),
                    });
                }
                recommendations.push("Review and revoke unused API keys".to_string());
            } else {
                checks_passed += 1;
            }
        }

        let duration_ms = start.elapsed().as_millis() as u64;
        let status = if checks_failed > 0 {
            "failed"
        } else if checks_warning > 0 {
            "warnings"
        } else {
            "passed"
        };

        // Update last scan time
        if let Ok(mut last_scan) = self.last_security_scan.lock() {
            *last_scan = Some(now);
        }

        // Log the scan as a security event
        self.log_security_event(SecurityEvent {
            id: Uuid::new_v4().to_string(),
            event_type: SecurityEventType::ConfigurationChanged,
            severity: if checks_failed > 0 {
                SecuritySeverity::High
            } else {
                SecuritySeverity::Low
            },
            timestamp: now,
            source_ip: None,
            user_agent: None,
            api_key_id: None,
            description: format!(
                "Security scan completed: {} passed, {} warnings, {} failed",
                checks_passed, checks_warning, checks_failed
            ),
            metadata: HashMap::new(),
        });

        Ok(SecurityScanResult {
            scan_id: Uuid::new_v4().to_string(),
            timestamp: now,
            status: status.to_string(),
            checks_passed,
            checks_failed,
            checks_warning,
            findings,
            recommendations,
            scan_duration_ms: duration_ms,
        })
    }

    fn generate_secure_key(&self) -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::thread_rng();

        // Generate a 32-character key with prefix
        let prefix = "inf_";
        let key_part: String = (0..28)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        format!("{}{}", prefix, key_part)
    }

    fn hash_key(&self, key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn log_security_event(&self, event: SecurityEvent) {
        if let Ok(mut events) = self.security_events.lock() {
            events.push(event);

            // Keep only last 1000 events to prevent memory issues
            if events.len() > 1000 {
                let len = events.len();
                events.drain(0..len - 1000);
            }
        }
    }

    // Initialize with some sample data for testing
    pub async fn initialize_with_sample_data(&self) -> Result<(), String> {
        // Create a sample API key
        let sample_request = CreateApiKeyRequest {
            name: "Dashboard Access".to_string(),
            permissions: vec!["read".to_string(), "write".to_string()],
            expires_in_days: Some(30),
        };

        self.generate_api_key(sample_request).await?;

        // Add some sample security events
        let sample_events = vec![
            SecurityEvent {
                id: Uuid::new_v4().to_string(),
                event_type: SecurityEventType::UnauthorizedAccess,
                severity: SecuritySeverity::High,
                timestamp: Utc::now() - chrono::Duration::hours(2),
                source_ip: Some("192.168.1.100".to_string()),
                user_agent: Some("Mozilla/5.0".to_string()),
                api_key_id: None,
                description: "Unauthorized access attempt detected".to_string(),
                metadata: HashMap::new(),
            },
            SecurityEvent {
                id: Uuid::new_v4().to_string(),
                event_type: SecurityEventType::SuspiciousActivity,
                severity: SecuritySeverity::Medium,
                timestamp: Utc::now() - chrono::Duration::hours(1),
                source_ip: Some("10.0.0.5".to_string()),
                user_agent: None,
                api_key_id: None,
                description: "Multiple failed authentication attempts".to_string(),
                metadata: HashMap::new(),
            },
        ];

        if let Ok(mut events) = self.security_events.lock() {
            events.extend(sample_events);
        }

        Ok(())
    }
}

impl Default for SecurityManager {
    fn default() -> Self {
        Self::new(())
    }
}
