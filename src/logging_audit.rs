// Logging audit functionality - re-export from main audit module
// This module provides additional types for CLI compatibility

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

// Re-export main audit types
pub use crate::audit::{
    ActorType, AuditEvent, AuditLogger as LoggingAuditSystem, AuditQuery as AuditSearchQuery,
    AuditStatistics, EventType as AuditEventType, ExportFormat, Severity as EventSeverity,
    SortOrder,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LoggingAuditConfig {
    pub enabled: bool,
    pub audit_level: String,
    pub retention_days: u32,
    pub export_formats: Vec<String>,
    pub compliance_standards: Vec<String>,
    pub real_time_alerts: bool,
    pub settings: HashMap<String, String>,
}

// Additional types needed for CLI compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: SystemTime,
    pub level: String,
    pub message: String,
    pub module: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    pub format: ExportFormat,
    pub start_time: Option<SystemTime>,
    pub end_time: Option<SystemTime>,
    pub filters: HashMap<String, String>,
    pub destination: String,
    pub query: Option<String>,
    pub compression: Option<bool>,
    pub encryption: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStandard {
    pub name: String,
    pub description: String,
    pub requirements: Vec<String>,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: SystemTime,
    pub end: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionOutcome {
    Success,
    Failure,
    Partial,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorFilter {
    pub actor_type: Option<ActorType>,
    pub actor_ids: Vec<String>,
    pub exclude_system: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceFilter {
    pub resource_types: Vec<String>,
    pub resource_ids: Vec<String>,
    pub include_deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub id: String,
    pub standard: ComplianceStandard,
    pub compliance_score: f64,
    pub findings: Vec<ComplianceFinding>,
    pub recommendations: Vec<String>,
    pub generated_at: DateTime<Utc>,
    pub period_start: SystemTime,
    pub period_end: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFinding {
    pub id: String,
    pub severity: String,
    pub description: String,
    pub evidence: Vec<String>,
    pub remediation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityReport {
    pub id: String,
    pub status: IntegrityStatus,
    pub files_checked: usize,
    pub files_valid: usize,
    pub hash_mismatches: Vec<String>,
    pub missing_files: Vec<String>,
    pub errors: Vec<String>,
    pub generated_at: DateTime<Utc>,
    pub integrity_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntegrityStatus {
    Valid,
    Compromised,
    Unknown,
    ErrorsDetected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyAlert {
    pub id: String,
    pub alert_type: String,
    pub severity: String,
    pub description: String,
    pub detected_at: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportStatus {
    pub id: String,
    pub status: String,
    pub progress: f64,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}
