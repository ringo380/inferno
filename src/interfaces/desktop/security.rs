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
}

impl SecurityManager {
    pub fn new<T>(_database_manager: T) -> Self
    where
        T: Send + Sync + 'static,
    {
        Self {
            api_keys: Arc::new(Mutex::new(Vec::new())),
            security_events: Arc::new(Mutex::new(Vec::new())),
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

        Ok(SecurityMetrics {
            total_api_keys,
            active_api_keys,
            expired_api_keys,
            security_events_24h,
            failed_auth_attempts_24h,
            suspicious_activities_24h,
            last_security_scan: Some(Utc::now()), // TODO: Implement actual security scanning
        })
    }

    pub async fn clear_security_events(&self) -> Result<(), String> {
        let mut events = self.security_events.lock().map_err(|e| e.to_string())?;
        events.clear();
        Ok(())
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
