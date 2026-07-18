// Shared audit configuration and compliance/integrity report types.
//
// These types are consumed by the audit system (`crate::audit`) and the main
// configuration (`crate::config`). The `inferno logging-audit` CLI that
// originally defined the rest of this module's types was removed as a redundant
// duplicate of `inferno audit` - see docs/ARCHIVE.md.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LoggingAuditConfig {
    pub enabled: bool,
    pub audit_level: String,
    pub retention_days: u32,
    pub export_formats: Vec<String>,
    pub compliance_standards: Vec<String>,
    pub real_time_alerts: bool,
    pub settings: HashMap<String, String>,
    pub audit: AuditConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    pub storage_path: String,
    pub max_file_size: u64,
    pub rotation_interval: String,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            storage_path: "logs/audit".to_string(),
            max_file_size: 100 * 1024 * 1024, // 100MB
            rotation_interval: "daily".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStandard {
    pub name: String,
    pub description: String,
    pub requirements: Vec<String>,
    pub version: String,
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
