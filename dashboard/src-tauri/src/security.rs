use crate::database::{DatabaseManager, DbApiKey, DbSecurityEvent};
use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ApiKey {
    pub id: String,
    pub name: String,
    pub key_hash: String,
    pub key_prefix: String,
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
    pub raw_key: String,
}

pub struct SecurityManager {
    database: Arc<DatabaseManager>,
}

impl SecurityManager {
    pub fn new(database: Arc<DatabaseManager>) -> Self {
        Self { database }
    }

    pub async fn generate_api_key(
        &self,
        request: CreateApiKeyRequest,
    ) -> Result<CreateApiKeyResponse, String> {
        let raw_key = self.generate_secure_key();
        let key_hash = self.hash_key(&raw_key);
        let key_prefix = raw_key.chars().take(8).collect::<String>();
        let created_at = Utc::now();
        let expires_at = request
            .expires_in_days
            .map(|days| created_at + chrono::Duration::days(days as i64));

        let api_key = ApiKey {
            id: Uuid::new_v4().to_string(),
            name: request.name.clone(),
            key_hash: key_hash.clone(),
            key_prefix,
            permissions: request.permissions.clone(),
            created_at,
            last_used: None,
            expires_at,
            is_active: true,
            usage_count: 0,
            created_by: "system".to_string(),
        };

        let db_api_key = DbApiKey {
            id: api_key.id.clone(),
            name: api_key.name.clone(),
            key_hash,
            key_prefix: api_key.key_prefix.clone(),
            permissions: serde_json::to_string(&api_key.permissions)
                .map_err(|e| e.to_string())?,
            is_active: api_key.is_active,
            usage_count: api_key.usage_count as i64,
            last_used: api_key.last_used,
            expires_at: api_key.expires_at,
            created_by: api_key.created_by.clone(),
            created_at: api_key.created_at,
        };

        self.database
            .create_api_key(&db_api_key)
            .await
            .map_err(|e| e.to_string())?;

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
        })
        .await?;

        Ok(CreateApiKeyResponse { api_key, raw_key })
    }

    pub async fn get_api_keys(&self) -> Result<Vec<ApiKey>, String> {
        let db_keys = self
            .database
            .get_api_keys()
            .await
            .map_err(|e| e.to_string())?;

        db_keys
            .into_iter()
            .map(ApiKey::try_from)
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn revoke_api_key(&self, key_id: String) -> Result<(), String> {
        self.database
            .deactivate_api_key(&key_id)
            .await
            .map_err(|e| e.to_string())?;

        self.log_security_event(SecurityEvent {
            id: Uuid::new_v4().to_string(),
            event_type: SecurityEventType::ApiKeyRevoked,
            severity: SecuritySeverity::Medium,
            timestamp: Utc::now(),
            source_ip: None,
            user_agent: None,
            api_key_id: Some(key_id.clone()),
            description: "API key revoked".to_string(),
            metadata: HashMap::new(),
        })
        .await
    }

    pub async fn delete_api_key(&self, key_id: String) -> Result<(), String> {
        self.database
            .delete_api_key(&key_id)
            .await
            .map_err(|e| e.to_string())?;

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
        })
        .await
    }

    pub async fn validate_api_key(&self, raw_key: String) -> Result<Option<ApiKey>, String> {
        let key_hash = self.hash_key(&raw_key);

        match self
            .database
            .get_api_key_by_hash(&key_hash)
            .await
            .map_err(|e| e.to_string())? {
            Some(db_key) => {
                self.database
                    .update_api_key_usage(&db_key.id)
                    .await
                    .map_err(|e| e.to_string())?;

                let api_key = ApiKey::try_from(db_key.clone())?;

                self.log_security_event(SecurityEvent {
                    id: Uuid::new_v4().to_string(),
                    event_type: SecurityEventType::ApiKeyUsed,
                    severity: SecuritySeverity::Low,
                    timestamp: Utc::now(),
                    source_ip: None,
                    user_agent: None,
                    api_key_id: Some(db_key.id),
                    description: "API key used for authentication".to_string(),
                    metadata: HashMap::new(),
                })
                .await?;

                Ok(Some(api_key))
            }
            None => Ok(None),
        }
    }

    pub async fn get_security_events(
        &self,
        limit: Option<usize>,
    ) -> Result<Vec<SecurityEvent>, String> {
        let db_events = self
            .database
            .get_security_events(limit)
            .await
            .map_err(|e| e.to_string())?;

        db_events
            .into_iter()
            .map(SecurityEvent::try_from)
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn get_security_metrics(&self) -> Result<SecurityMetrics, String> {
        let (
            total_api_keys,
            active_api_keys,
            expired_api_keys,
            security_events_24h,
            failed_auth_attempts_24h,
            suspicious_activities_24h,
        ) = self
            .database
            .get_security_metrics()
            .await
            .map_err(|e| e.to_string())?;

        Ok(SecurityMetrics {
            total_api_keys: to_u32(total_api_keys),
            active_api_keys: to_u32(active_api_keys),
            expired_api_keys: to_u32(expired_api_keys),
            security_events_24h: to_u32(security_events_24h),
            failed_auth_attempts_24h: to_u32(failed_auth_attempts_24h),
            suspicious_activities_24h: to_u32(suspicious_activities_24h),
            last_security_scan: None,
        })
    }

    pub async fn clear_security_events(&self) -> Result<(), String> {
        self.database
            .clear_security_events()
            .await
            .map_err(|e| e.to_string())
    }

    async fn log_security_event(&self, event: SecurityEvent) -> Result<(), String> {
        let db_event = DbSecurityEvent {
            id: event.id.clone(),
            event_type: event_type_to_db(&event.event_type).to_string(),
            severity: severity_to_db(&event.severity).to_string(),
            description: event.description,
            source_ip: event.source_ip,
            user_agent: event.user_agent,
            api_key_id: event.api_key_id,
            metadata: if event.metadata.is_empty() {
                None
            } else {
                Some(serde_json::to_string(&event.metadata).map_err(|e| e.to_string())?)
            },
            created_at: event.timestamp,
        };

        self.database
            .create_security_event(&db_event)
            .await
            .map_err(|e| e.to_string())
    }

    fn generate_secure_key(&self) -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::thread_rng();

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
}

impl TryFrom<DbApiKey> for ApiKey {
    type Error = String;

    fn try_from(db_key: DbApiKey) -> Result<Self, Self::Error> {
        let permissions: Vec<String> = serde_json::from_str(&db_key.permissions)
            .map_err(|e| e.to_string())?;

        Ok(ApiKey {
            id: db_key.id,
            name: db_key.name,
            key_hash: db_key.key_hash,
            key_prefix: db_key.key_prefix,
            permissions,
            created_at: db_key.created_at,
            last_used: db_key.last_used,
            expires_at: db_key.expires_at,
            is_active: db_key.is_active,
            usage_count: u64::try_from(db_key.usage_count)
                .map_err(|_| "usage_count out of range".to_string())?,
            created_by: db_key.created_by,
        })
    }
}

impl TryFrom<DbSecurityEvent> for SecurityEvent {
    type Error = String;

    fn try_from(db_event: DbSecurityEvent) -> Result<Self, Self::Error> {
        let metadata = db_event
            .metadata
            .as_ref()
            .map(|raw| serde_json::from_str(raw).map_err(|e| e.to_string()))
            .transpose()? 
            .unwrap_or_default();

        Ok(SecurityEvent {
            id: db_event.id,
            event_type: event_type_from_db(&db_event.event_type),
            severity: severity_from_db(&db_event.severity),
            timestamp: db_event.created_at,
            source_ip: db_event.source_ip,
            user_agent: db_event.user_agent,
            api_key_id: db_event.api_key_id,
            description: db_event.description,
            metadata,
        })
    }
}

fn event_type_to_db(event_type: &SecurityEventType) -> &'static str {
    match event_type {
        SecurityEventType::ApiKeyCreated => "apikeycreated",
        SecurityEventType::ApiKeyRevoked => "apikeyrevoked",
        SecurityEventType::ApiKeyUsed => "apikeyused",
        SecurityEventType::UnauthorizedAccess => "unauthorizedaccess",
        SecurityEventType::AuthenticationFailed => "authenticationfailed",
        SecurityEventType::PermissionDenied => "permissiondenied",
        SecurityEventType::SuspiciousActivity => "suspiciousactivity",
        SecurityEventType::ConfigurationChanged => "configurationchanged",
    }
}

fn event_type_from_db(value: &str) -> SecurityEventType {
    match value {
        "apikeycreated" => SecurityEventType::ApiKeyCreated,
        "apikeyrevoked" => SecurityEventType::ApiKeyRevoked,
        "apikeyused" => SecurityEventType::ApiKeyUsed,
        "unauthorizedaccess" => SecurityEventType::UnauthorizedAccess,
        "authenticationfailed" => SecurityEventType::AuthenticationFailed,
        "permissiondenied" => SecurityEventType::PermissionDenied,
        "suspiciousactivity" => SecurityEventType::SuspiciousActivity,
        "configurationchanged" => SecurityEventType::ConfigurationChanged,
        _ => SecurityEventType::SuspiciousActivity,
    }
}

fn severity_to_db(severity: &SecuritySeverity) -> &'static str {
    match severity {
        SecuritySeverity::Low => "low",
        SecuritySeverity::Medium => "medium",
        SecuritySeverity::High => "high",
        SecuritySeverity::Critical => "critical",
    }
}

fn severity_from_db(value: &str) -> SecuritySeverity {
    match value {
        "low" => SecuritySeverity::Low,
        "medium" => SecuritySeverity::Medium,
        "high" => SecuritySeverity::High,
        "critical" => SecuritySeverity::Critical,
        _ => SecuritySeverity::Medium,
    }
}

fn to_u32(value: i64) -> u32 {
    u32::try_from(value).unwrap_or(0)
}
