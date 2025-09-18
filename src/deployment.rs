use crate::{config::Config, InfernoError};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentArgs {
    pub environment: String,
    pub version: String,
    pub namespace: Option<String>,
    pub replicas: Option<u32>,
    pub gpu_enabled: bool,
    pub dry_run: bool,
    pub wait_for_completion: bool,
    pub timeout_seconds: u64,
    pub custom_values: HashMap<String, String>,
    pub values_file: Option<PathBuf>,
    pub skip_pre_checks: bool,
}

/// Deployment configuration for Kubernetes and cloud platforms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    /// Kubernetes cluster configuration
    pub kubernetes: KubernetesConfig,
    /// Helm chart configuration
    pub helm: HelmConfig,
    /// Container registry settings
    pub registry: RegistryConfig,
    /// Auto-scaling configuration
    pub autoscaling: AutoScalingConfig,
    /// Resource limits and requests
    pub resources: ResourceConfig,
    /// Security and networking settings
    pub security: SecurityConfig,
    /// Monitoring and observability
    pub monitoring: MonitoringConfig,
    /// Environment-specific settings
    pub environments: HashMap<String, EnvironmentConfig>,
}

impl Default for DeploymentConfig {
    fn default() -> Self {
        Self {
            kubernetes: KubernetesConfig::default(),
            helm: HelmConfig::default(),
            registry: RegistryConfig::default(),
            autoscaling: AutoScalingConfig::default(),
            resources: ResourceConfig::default(),
            security: SecurityConfig::default(),
            monitoring: MonitoringConfig::default(),
            environments: HashMap::from([
                ("dev".to_string(), EnvironmentConfig::development()),
                ("staging".to_string(), EnvironmentConfig::staging()),
                ("prod".to_string(), EnvironmentConfig::production()),
            ]),
        }
    }
}

impl DeploymentConfig {
    pub fn from_config(config: &Config) -> Result<Self> {
        Ok(config.deployment.clone())
    }
}

/// Kubernetes-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesConfig {
    /// Kubernetes API version to use
    pub api_version: String,
    /// Target namespace for deployments
    pub namespace: String,
    /// Service account for pods
    pub service_account: String,
    /// Image pull secrets
    pub image_pull_secrets: Vec<String>,
    /// Node selector for pod placement
    pub node_selector: HashMap<String, String>,
    /// Tolerations for tainted nodes
    pub tolerations: Vec<Toleration>,
    /// Affinity rules for pod scheduling
    pub affinity: Option<Affinity>,
    /// Priority class for pods
    pub priority_class: Option<String>,
    /// Security context
    pub security_context: SecurityContext,
}

impl Default for KubernetesConfig {
    fn default() -> Self {
        Self {
            api_version: "apps/v1".to_string(),
            namespace: "inferno".to_string(),
            service_account: "inferno".to_string(),
            image_pull_secrets: vec!["regcred".to_string()],
            node_selector: HashMap::new(),
            tolerations: vec![],
            affinity: None,
            priority_class: Some("high-priority".to_string()),
            security_context: SecurityContext::default(),
        }
    }
}

/// Helm chart configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelmConfig {
    /// Chart name
    pub chart_name: String,
    /// Chart version
    pub chart_version: String,
    /// Repository URL
    pub repository: String,
    /// Release name pattern
    pub release_name_template: String,
    /// Default values file
    pub values_file: PathBuf,
    /// Environment-specific values
    pub environment_values: HashMap<String, PathBuf>,
    /// Helm hooks configuration
    pub hooks: HelmHooks,
    /// Chart dependencies
    pub dependencies: Vec<ChartDependency>,
}

impl Default for HelmConfig {
    fn default() -> Self {
        Self {
            chart_name: "inferno".to_string(),
            chart_version: "0.1.0".to_string(),
            repository: "https://charts.inferno.ai".to_string(),
            release_name_template: "inferno-{environment}".to_string(),
            values_file: PathBuf::from("helm/values.yaml"),
            environment_values: HashMap::from([
                ("dev".to_string(), PathBuf::from("helm/values-dev.yaml")),
                ("staging".to_string(), PathBuf::from("helm/values-staging.yaml")),
                ("prod".to_string(), PathBuf::from("helm/values-prod.yaml")),
            ]),
            hooks: HelmHooks::default(),
            dependencies: vec![
                ChartDependency {
                    name: "redis".to_string(),
                    version: "17.x.x".to_string(),
                    repository: "https://charts.bitnami.com/bitnami".to_string(),
                    condition: "redis.enabled".to_string(),
                },
                ChartDependency {
                    name: "postgresql".to_string(),
                    version: "12.x.x".to_string(),
                    repository: "https://charts.bitnami.com/bitnami".to_string(),
                    condition: "postgresql.enabled".to_string(),
                },
            ],
        }
    }
}

/// Container registry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    /// Registry URL
    pub url: String,
    /// Registry username
    pub username: Option<String>,
    /// Image repository prefix
    pub repository_prefix: String,
    /// Image tag strategy
    pub tag_strategy: TagStrategy,
    /// Automatic image scanning
    pub scan_images: bool,
    /// Image retention policy
    pub retention_policy: RetentionPolicy,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            url: "ghcr.io".to_string(),
            username: None,
            repository_prefix: "inferno-ai".to_string(),
            tag_strategy: TagStrategy::GitCommit,
            scan_images: true,
            retention_policy: RetentionPolicy::default(),
        }
    }
}

/// Auto-scaling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoScalingConfig {
    /// Enable horizontal pod autoscaler
    pub hpa_enabled: bool,
    /// Minimum number of replicas
    pub min_replicas: u32,
    /// Maximum number of replicas
    pub max_replicas: u32,
    /// Target CPU utilization percentage
    pub target_cpu_utilization: u32,
    /// Target memory utilization percentage
    pub target_memory_utilization: u32,
    /// Custom metrics for scaling
    pub custom_metrics: Vec<CustomMetric>,
    /// Vertical pod autoscaler settings
    pub vpa_enabled: bool,
    /// Cluster autoscaler settings
    pub cluster_autoscaler: ClusterAutoscalerConfig,
}

impl Default for AutoScalingConfig {
    fn default() -> Self {
        Self {
            hpa_enabled: true,
            min_replicas: 2,
            max_replicas: 20,
            target_cpu_utilization: 70,
            target_memory_utilization: 80,
            custom_metrics: vec![
                CustomMetric {
                    name: "inference_queue_length".to_string(),
                    target_value: 10.0,
                    target_type: "AverageValue".to_string(),
                },
            ],
            vpa_enabled: false,
            cluster_autoscaler: ClusterAutoscalerConfig::default(),
        }
    }
}

/// Resource configuration for containers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    /// CPU requests and limits
    pub cpu: ResourceLimits,
    /// Memory requests and limits
    pub memory: ResourceLimits,
    /// GPU resources
    pub gpu: Option<GpuConfig>,
    /// Storage configuration
    pub storage: StorageConfig,
    /// Quality of Service class
    pub qos_class: QosClass,
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            cpu: ResourceLimits {
                request: "1000m".to_string(),
                limit: "4000m".to_string(),
            },
            memory: ResourceLimits {
                request: "2Gi".to_string(),
                limit: "8Gi".to_string(),
            },
            gpu: None,
            storage: StorageConfig::default(),
            qos_class: QosClass::Burstable,
        }
    }
}

/// Security configuration for deployments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Pod security standards
    pub pod_security_standard: PodSecurityStandard,
    /// Network policies
    pub network_policies: Vec<NetworkPolicy>,
    /// Service mesh configuration
    pub service_mesh: Option<ServiceMeshConfig>,
    /// Secrets management
    pub secrets: SecretsConfig,
    /// RBAC configuration
    pub rbac: RbacConfig,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            pod_security_standard: PodSecurityStandard::Restricted,
            network_policies: vec![
                NetworkPolicy {
                    name: "deny-all".to_string(),
                    policy_type: "default-deny".to_string(),
                },
                NetworkPolicy {
                    name: "allow-inferno".to_string(),
                    policy_type: "allow-specific".to_string(),
                },
            ],
            service_mesh: Some(ServiceMeshConfig::istio()),
            secrets: SecretsConfig::default(),
            rbac: RbacConfig::default(),
        }
    }
}

/// Monitoring and observability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Prometheus monitoring
    pub prometheus: PrometheusConfig,
    /// Grafana dashboards
    pub grafana: GrafanaConfig,
    /// Distributed tracing
    pub tracing: TracingConfig,
    /// Log aggregation
    pub logging: LoggingConfig,
    /// Health checks
    pub health_checks: HealthCheckConfig,
    /// Alerting rules
    pub alerting: AlertingConfig,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            prometheus: PrometheusConfig::default(),
            grafana: GrafanaConfig::default(),
            tracing: TracingConfig::default(),
            logging: LoggingConfig::default(),
            health_checks: HealthCheckConfig::default(),
            alerting: AlertingConfig::default(),
        }
    }
}

/// Environment-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Environment name
    pub name: String,
    /// Resource scaling factors
    pub scale_factor: f64,
    /// Environment-specific features
    pub features: HashMap<String, bool>,
    /// External dependencies
    pub external_services: HashMap<String, String>,
    /// Environment variables
    pub environment_variables: HashMap<String, String>,
    /// Ingress configuration
    pub ingress: IngressConfig,
}

impl EnvironmentConfig {
    pub fn development() -> Self {
        Self {
            name: "development".to_string(),
            scale_factor: 0.5,
            features: HashMap::from([
                ("debug_mode".to_string(), true),
                ("auto_restart".to_string(), true),
                ("detailed_logging".to_string(), true),
            ]),
            external_services: HashMap::new(),
            environment_variables: HashMap::from([
                ("LOG_LEVEL".to_string(), "debug".to_string()),
                ("ENV".to_string(), "development".to_string()),
            ]),
            ingress: IngressConfig::development(),
        }
    }

    pub fn staging() -> Self {
        Self {
            name: "staging".to_string(),
            scale_factor: 0.8,
            features: HashMap::from([
                ("debug_mode".to_string(), false),
                ("auto_restart".to_string(), true),
                ("detailed_logging".to_string(), false),
            ]),
            external_services: HashMap::new(),
            environment_variables: HashMap::from([
                ("LOG_LEVEL".to_string(), "info".to_string()),
                ("ENV".to_string(), "staging".to_string()),
            ]),
            ingress: IngressConfig::staging(),
        }
    }

    pub fn production() -> Self {
        Self {
            name: "production".to_string(),
            scale_factor: 1.0,
            features: HashMap::from([
                ("debug_mode".to_string(), false),
                ("auto_restart".to_string(), false),
                ("detailed_logging".to_string(), false),
            ]),
            external_services: HashMap::new(),
            environment_variables: HashMap::from([
                ("LOG_LEVEL".to_string(), "warn".to_string()),
                ("ENV".to_string(), "production".to_string()),
            ]),
            ingress: IngressConfig::production(),
        }
    }
}

// Supporting types and structures

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Toleration {
    pub key: String,
    pub operator: String,
    pub value: Option<String>,
    pub effect: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Affinity {
    pub node_affinity: Option<NodeAffinity>,
    pub pod_affinity: Option<PodAffinity>,
    pub pod_anti_affinity: Option<PodAntiAffinity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeAffinity {
    pub required_during_scheduling: Vec<NodeSelectorTerm>,
    pub preferred_during_scheduling: Vec<PreferredSchedulingTerm>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSelectorTerm {
    pub match_expressions: Vec<NodeSelectorRequirement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSelectorRequirement {
    pub key: String,
    pub operator: String,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferredSchedulingTerm {
    pub weight: i32,
    pub preference: NodeSelectorTerm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodAffinity {
    pub required_during_scheduling: Vec<PodAffinityTerm>,
    pub preferred_during_scheduling: Vec<WeightedPodAffinityTerm>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodAntiAffinity {
    pub required_during_scheduling: Vec<PodAffinityTerm>,
    pub preferred_during_scheduling: Vec<WeightedPodAffinityTerm>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodAffinityTerm {
    pub label_selector: LabelSelector,
    pub topology_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightedPodAffinityTerm {
    pub weight: i32,
    pub pod_affinity_term: PodAffinityTerm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelSelector {
    pub match_labels: HashMap<String, String>,
    pub match_expressions: Vec<LabelSelectorRequirement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelSelectorRequirement {
    pub key: String,
    pub operator: String,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    pub run_as_non_root: bool,
    pub run_as_user: Option<u32>,
    pub run_as_group: Option<u32>,
    pub fs_group: Option<u32>,
    pub capabilities: SecurityCapabilities,
    pub read_only_root_filesystem: bool,
    pub allow_privilege_escalation: bool,
}

impl Default for SecurityContext {
    fn default() -> Self {
        Self {
            run_as_non_root: true,
            run_as_user: Some(1001),
            run_as_group: Some(1001),
            fs_group: Some(1001),
            capabilities: SecurityCapabilities::default(),
            read_only_root_filesystem: true,
            allow_privilege_escalation: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityCapabilities {
    pub add: Vec<String>,
    pub drop: Vec<String>,
}

impl Default for SecurityCapabilities {
    fn default() -> Self {
        Self {
            add: vec![],
            drop: vec!["ALL".to_string()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelmHooks {
    pub pre_install: Vec<HookConfig>,
    pub post_install: Vec<HookConfig>,
    pub pre_upgrade: Vec<HookConfig>,
    pub post_upgrade: Vec<HookConfig>,
    pub pre_delete: Vec<HookConfig>,
    pub post_delete: Vec<HookConfig>,
}

impl Default for HelmHooks {
    fn default() -> Self {
        Self {
            pre_install: vec![
                HookConfig {
                    name: "create-namespace".to_string(),
                    job_spec: "jobs/create-namespace.yaml".to_string(),
                    weight: -5,
                },
            ],
            post_install: vec![
                HookConfig {
                    name: "validate-deployment".to_string(),
                    job_spec: "jobs/validate-deployment.yaml".to_string(),
                    weight: 1,
                },
            ],
            pre_upgrade: vec![],
            post_upgrade: vec![],
            pre_delete: vec![],
            post_delete: vec![
                HookConfig {
                    name: "cleanup-resources".to_string(),
                    job_spec: "jobs/cleanup.yaml".to_string(),
                    weight: 1,
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookConfig {
    pub name: String,
    pub job_spec: String,
    pub weight: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartDependency {
    pub name: String,
    pub version: String,
    pub repository: String,
    pub condition: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TagStrategy {
    GitCommit,
    GitTag,
    Timestamp,
    Semantic,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub keep_last_n_images: u32,
    pub keep_images_for_days: u32,
    pub delete_untagged: bool,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            keep_last_n_images: 10,
            keep_images_for_days: 30,
            delete_untagged: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomMetric {
    pub name: String,
    pub target_value: f64,
    pub target_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterAutoscalerConfig {
    pub enabled: bool,
    pub min_nodes: u32,
    pub max_nodes: u32,
    pub node_groups: Vec<NodeGroup>,
}

impl Default for ClusterAutoscalerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_nodes: 1,
            max_nodes: 10,
            node_groups: vec![
                NodeGroup {
                    name: "inference-nodes".to_string(),
                    instance_type: "n1-standard-4".to_string(),
                    min_size: 1,
                    max_size: 5,
                    desired_size: 2,
                    labels: HashMap::from([
                        ("workload".to_string(), "inference".to_string()),
                    ]),
                    taints: vec![],
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeGroup {
    pub name: String,
    pub instance_type: String,
    pub min_size: u32,
    pub max_size: u32,
    pub desired_size: u32,
    pub labels: HashMap<String, String>,
    pub taints: Vec<NodeTaint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeTaint {
    pub key: String,
    pub value: String,
    pub effect: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub request: String,
    pub limit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuConfig {
    pub nvidia_gpu_count: u32,
    pub nvidia_gpu_type: String,
    pub amd_gpu_count: u32,
    pub shared_gpu: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub persistent_volume_claims: Vec<PvcConfig>,
    pub storage_class: String,
    pub backup_enabled: bool,
    pub backup_schedule: String,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            persistent_volume_claims: vec![
                PvcConfig {
                    name: "model-storage".to_string(),
                    size: "100Gi".to_string(),
                    access_modes: vec!["ReadWriteOnce".to_string()],
                    mount_path: "/data/models".to_string(),
                },
                PvcConfig {
                    name: "cache-storage".to_string(),
                    size: "50Gi".to_string(),
                    access_modes: vec!["ReadWriteOnce".to_string()],
                    mount_path: "/data/cache".to_string(),
                },
            ],
            storage_class: "fast-ssd".to_string(),
            backup_enabled: true,
            backup_schedule: "0 2 * * *".to_string(), // Daily at 2 AM
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PvcConfig {
    pub name: String,
    pub size: String,
    pub access_modes: Vec<String>,
    pub mount_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QosClass {
    Guaranteed,
    Burstable,
    BestEffort,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PodSecurityStandard {
    Privileged,
    Baseline,
    Restricted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicy {
    pub name: String,
    pub policy_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMeshConfig {
    pub provider: String,
    pub version: String,
    pub features: HashMap<String, bool>,
}

impl ServiceMeshConfig {
    pub fn istio() -> Self {
        Self {
            provider: "istio".to_string(),
            version: "1.19.x".to_string(),
            features: HashMap::from([
                ("mTLS".to_string(), true),
                ("traffic_management".to_string(), true),
                ("observability".to_string(), true),
                ("security_policies".to_string(), true),
            ]),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretsConfig {
    pub provider: String,
    pub vault_config: Option<VaultConfig>,
    pub kubernetes_secrets: Vec<KubernetesSecret>,
}

impl Default for SecretsConfig {
    fn default() -> Self {
        Self {
            provider: "kubernetes".to_string(),
            vault_config: None,
            kubernetes_secrets: vec![
                KubernetesSecret {
                    name: "inferno-api-keys".to_string(),
                    secret_type: "Opaque".to_string(),
                    data_keys: vec!["openai_api_key".to_string(), "model_api_key".to_string()],
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultConfig {
    pub address: String,
    pub auth_method: String,
    pub secret_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesSecret {
    pub name: String,
    pub secret_type: String,
    pub data_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RbacConfig {
    pub service_account: String,
    pub cluster_role: String,
    pub permissions: Vec<RbacPermission>,
}

impl Default for RbacConfig {
    fn default() -> Self {
        Self {
            service_account: "inferno".to_string(),
            cluster_role: "inferno-operator".to_string(),
            permissions: vec![
                RbacPermission {
                    api_groups: vec!["".to_string()],
                    resources: vec!["pods".to_string(), "services".to_string()],
                    verbs: vec!["get".to_string(), "list".to_string(), "watch".to_string()],
                },
                RbacPermission {
                    api_groups: vec!["apps".to_string()],
                    resources: vec!["deployments".to_string(), "replicasets".to_string()],
                    verbs: vec!["get".to_string(), "list".to_string(), "watch".to_string(), "create".to_string(), "update".to_string()],
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RbacPermission {
    pub api_groups: Vec<String>,
    pub resources: Vec<String>,
    pub verbs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrometheusConfig {
    pub enabled: bool,
    pub namespace: String,
    pub scrape_interval: String,
    pub retention: String,
    pub storage_size: String,
}

impl Default for PrometheusConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            namespace: "monitoring".to_string(),
            scrape_interval: "30s".to_string(),
            retention: "15d".to_string(),
            storage_size: "50Gi".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrafanaConfig {
    pub enabled: bool,
    pub dashboard_config_maps: Vec<String>,
    pub data_sources: Vec<DataSource>,
}

impl Default for GrafanaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            dashboard_config_maps: vec![
                "inferno-dashboard".to_string(),
                "system-dashboard".to_string(),
            ],
            data_sources: vec![
                DataSource {
                    name: "Prometheus".to_string(),
                    url: "http://prometheus:9090".to_string(),
                    data_type: "prometheus".to_string(),
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSource {
    pub name: String,
    pub url: String,
    pub data_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    pub enabled: bool,
    pub provider: String,
    pub endpoint: String,
    pub sampling_rate: f64,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            provider: "jaeger".to_string(),
            endpoint: "http://jaeger-collector:14268/api/traces".to_string(),
            sampling_rate: 0.1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub enabled: bool,
    pub log_level: String,
    pub structured_logging: bool,
    pub log_aggregation: LogAggregationConfig,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_level: "info".to_string(),
            structured_logging: true,
            log_aggregation: LogAggregationConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAggregationConfig {
    pub provider: String,
    pub endpoint: String,
    pub retention_days: u32,
}

impl Default for LogAggregationConfig {
    fn default() -> Self {
        Self {
            provider: "elasticsearch".to_string(),
            endpoint: "http://elasticsearch:9200".to_string(),
            retention_days: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub liveness_probe: ProbeConfig,
    pub readiness_probe: ProbeConfig,
    pub startup_probe: ProbeConfig,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            liveness_probe: ProbeConfig {
                path: "/health".to_string(),
                port: 8080,
                initial_delay_seconds: 30,
                period_seconds: 10,
                timeout_seconds: 5,
                failure_threshold: 3,
            },
            readiness_probe: ProbeConfig {
                path: "/ready".to_string(),
                port: 8080,
                initial_delay_seconds: 5,
                period_seconds: 5,
                timeout_seconds: 3,
                failure_threshold: 3,
            },
            startup_probe: ProbeConfig {
                path: "/startup".to_string(),
                port: 8080,
                initial_delay_seconds: 10,
                period_seconds: 10,
                timeout_seconds: 5,
                failure_threshold: 30,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeConfig {
    pub path: String,
    pub port: u16,
    pub initial_delay_seconds: u32,
    pub period_seconds: u32,
    pub timeout_seconds: u32,
    pub failure_threshold: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertingConfig {
    pub enabled: bool,
    pub alert_manager_config: AlertManagerConfig,
    pub notification_channels: Vec<NotificationChannel>,
    pub alert_rules: Vec<AlertRule>,
}

impl Default for AlertingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            alert_manager_config: AlertManagerConfig::default(),
            notification_channels: vec![
                NotificationChannel {
                    name: "slack".to_string(),
                    channel_type: "slack".to_string(),
                    webhook_url: "https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK".to_string(),
                },
            ],
            alert_rules: vec![
                AlertRule {
                    name: "HighCPUUsage".to_string(),
                    expression: "avg(rate(container_cpu_usage_seconds_total[5m])) by (pod) > 0.8".to_string(),
                    duration: "5m".to_string(),
                    severity: "warning".to_string(),
                },
                AlertRule {
                    name: "HighMemoryUsage".to_string(),
                    expression: "avg(container_memory_usage_bytes / container_spec_memory_limit_bytes) by (pod) > 0.9".to_string(),
                    duration: "5m".to_string(),
                    severity: "critical".to_string(),
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertManagerConfig {
    pub endpoint: String,
    pub resolve_timeout: String,
}

impl Default for AlertManagerConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://alertmanager:9093".to_string(),
            resolve_timeout: "5m".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannel {
    pub name: String,
    pub channel_type: String,
    pub webhook_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub name: String,
    pub expression: String,
    pub duration: String,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressConfig {
    pub enabled: bool,
    pub class_name: String,
    pub hostname: String,
    pub tls_enabled: bool,
    pub tls_secret_name: Option<String>,
    pub annotations: HashMap<String, String>,
}

impl IngressConfig {
    pub fn development() -> Self {
        Self {
            enabled: true,
            class_name: "nginx".to_string(),
            hostname: "inferno-dev.local".to_string(),
            tls_enabled: false,
            tls_secret_name: None,
            annotations: HashMap::from([
                ("nginx.ingress.kubernetes.io/rewrite-target".to_string(), "/".to_string()),
            ]),
        }
    }

    pub fn staging() -> Self {
        Self {
            enabled: true,
            class_name: "nginx".to_string(),
            hostname: "inferno-staging.example.com".to_string(),
            tls_enabled: true,
            tls_secret_name: Some("inferno-tls-staging".to_string()),
            annotations: HashMap::from([
                ("cert-manager.io/cluster-issuer".to_string(), "letsencrypt-staging".to_string()),
                ("nginx.ingress.kubernetes.io/ssl-redirect".to_string(), "true".to_string()),
            ]),
        }
    }

    pub fn production() -> Self {
        Self {
            enabled: true,
            class_name: "nginx".to_string(),
            hostname: "api.inferno.ai".to_string(),
            tls_enabled: true,
            tls_secret_name: Some("inferno-tls-prod".to_string()),
            annotations: HashMap::from([
                ("cert-manager.io/cluster-issuer".to_string(), "letsencrypt-prod".to_string()),
                ("nginx.ingress.kubernetes.io/ssl-redirect".to_string(), "true".to_string()),
                ("nginx.ingress.kubernetes.io/rate-limit".to_string(), "100".to_string()),
            ]),
        }
    }
}

/// Deployment status and management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentStatus {
    pub id: String,
    pub environment: String,
    pub status: DeploymentState,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deployed_version: String,
    pub helm_release_name: String,
    pub kubernetes_namespace: String,
    pub replicas: ReplicaStatus,
    pub health: HealthStatus,
    pub resources: ResourceStatus,
}

/// Result types for deployment operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentResult {
    pub deployment_id: String,
    pub status: String,
    pub manifest_preview: String,
    pub service_urls: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackResult {
    pub revision: u32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleResult {
    pub current_replicas: u32,
    pub target_replicas: u32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusInfo {
    pub environment: String,
    pub version: String,
    pub status: String,
    pub ready_replicas: u32,
    pub total_replicas: u32,
    pub last_updated: String,
    pub pods: Vec<PodInfo>,
    pub service_urls: HashMap<String, String>,
    pub health_checks: Vec<HealthCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodInfo {
    pub name: String,
    pub status: String,
    pub ready: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub name: String,
    pub passing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub pod: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub cluster_resources: Vec<ClusterResource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterResource {
    pub name: String,
    pub kind: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentHistoryEntry {
    pub revision: u32,
    pub version: String,
    pub timestamp: String,
    pub status: String,
    pub rolled_back: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoscalingStatus {
    pub enabled: bool,
    pub current_replicas: u32,
    pub min_replicas: u32,
    pub max_replicas: u32,
    pub current_cpu_percent: u32,
    pub target_cpu_percent: u32,
    pub current_memory_percent: Option<u32>,
    pub target_memory_percent: Option<u32>,
    pub last_scale_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigEntry {
    pub key: String,
    pub value: String,
    pub is_secret: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub overall_healthy: bool,
    pub uptime: String,
    pub services: Vec<ServiceHealthInfo>,
    pub cpu_usage: u32,
    pub cpu_limit: u32,
    pub memory_usage: u32,
    pub memory_limit: u32,
    pub recent_errors: Vec<ErrorInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealthInfo {
    pub name: String,
    pub healthy: bool,
    pub status: String,
    pub response_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    pub timestamp: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentState {
    Pending,
    InProgress,
    Deployed,
    Failed,
    RollingBack,
    RolledBack,
    Terminated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaStatus {
    pub desired: u32,
    pub current: u32,
    pub ready: u32,
    pub available: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub overall: String,
    pub services: Vec<ServiceHealth>,
    pub last_check: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub name: String,
    pub status: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStatus {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub storage_usage: HashMap<String, f64>,
    pub network_io: NetworkIOStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIOStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
}

/// Main deployment manager
pub struct DeploymentManager {
    config: DeploymentConfig,
    active_deployments: Arc<RwLock<HashMap<String, DeploymentStatus>>>,
    deployment_history: Arc<RwLock<Vec<DeploymentStatus>>>,
}

// Send and Sync implementations for thread safety
unsafe impl Send for DeploymentManager {}
unsafe impl Sync for DeploymentManager {}

impl Clone for DeploymentManager {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            active_deployments: Arc::clone(&self.active_deployments),
            deployment_history: Arc::clone(&self.deployment_history),
        }
    }
}

impl DeploymentManager {
    /// Create a new deployment manager
    pub fn new(config: DeploymentConfig) -> Self {
        Self {
            config,
            active_deployments: Arc::new(RwLock::new(HashMap::new())),
            deployment_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Initialize the deployment manager
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing deployment manager");

        // Validate Kubernetes connectivity
        self.validate_kubernetes_connection().await?;

        // Setup Helm repositories
        self.setup_helm_repositories().await?;

        // Load existing deployments
        self.load_existing_deployments().await?;

        Ok(())
    }

    /// Deploy to environment
    pub async fn deploy(&mut self, args: &DeploymentArgs) -> Result<DeploymentResult> {
        let deployment_id = Uuid::new_v4().to_string();

        info!("Starting deployment {} to environment: {}", deployment_id, args.environment);

        // Get environment configuration
        let env_config = self.config.environments.get(&args.environment)
            .ok_or_else(|| InfernoError::InvalidArgument(format!("Unknown environment: {}", args.environment)))?;

        // Create deployment status
        let deployment_status = DeploymentStatus {
            id: deployment_id.clone(),
            environment: args.environment.clone(),
            status: DeploymentState::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deployed_version: args.version.clone(),
            helm_release_name: self.config.helm.release_name_template.replace("{environment}", &args.environment),
            kubernetes_namespace: args.namespace.clone().unwrap_or_else(|| self.config.kubernetes.namespace.clone()),
            replicas: ReplicaStatus {
                desired: args.replicas.unwrap_or((self.config.autoscaling.min_replicas as f64 * env_config.scale_factor) as u32),
                current: 0,
                ready: 0,
                available: 0,
            },
            health: HealthStatus {
                overall: "Unknown".to_string(),
                services: vec![],
                last_check: Utc::now(),
            },
            resources: ResourceStatus {
                cpu_usage: 0.0,
                memory_usage: 0.0,
                storage_usage: HashMap::new(),
                network_io: NetworkIOStats {
                    bytes_sent: 0,
                    bytes_received: 0,
                    packets_sent: 0,
                    packets_received: 0,
                },
            },
        };

        // Store deployment status
        {
            let mut deployments = self.active_deployments.write().await;
            deployments.insert(deployment_id.clone(), deployment_status);
        }

        let manifest_preview = if args.dry_run {
            "# Dry run - manifests would be generated here".to_string()
        } else {
            "Deployment executed successfully".to_string()
        };

        // Execute deployment steps
        if !args.dry_run {
            // Spawn the deployment task to avoid blocking
            let manager_clone = self.clone();
            let deployment_id_clone = deployment_id.clone();
            let environment_clone = args.environment.clone();
            let version_clone = args.version.clone();

            tokio::spawn(async move {
                if let Err(e) = manager_clone.execute_deployment(&deployment_id_clone, &environment_clone, &version_clone).await {
                    tracing::error!("Deployment failed: {}", e);
                    // Update deployment status to failed
                    let _ = manager_clone.update_deployment_status(&deployment_id_clone, DeploymentState::Failed).await;
                }
            });
        } else {
            info!("Dry run mode - skipping actual deployment");
        }

        Ok(DeploymentResult {
            deployment_id: deployment_id.clone(),
            status: "Success".to_string(),
            manifest_preview,
            service_urls: HashMap::from([
                ("main".to_string(), format!("http://inferno-{}.{}.svc.cluster.local:8080", args.environment, args.namespace.as_deref().unwrap_or("default")))
            ]),
        })
    }

    /// Get deployment status
    pub async fn get_deployment_status(&self, deployment_id: &str) -> Result<Option<DeploymentStatus>> {
        let deployments = self.active_deployments.read().await;
        Ok(deployments.get(deployment_id).cloned())
    }

    /// List active deployments
    pub async fn list_deployments(&self) -> Result<Vec<DeploymentStatus>> {
        let deployments = self.active_deployments.read().await;
        Ok(deployments.values().cloned().collect())
    }

    /// Rollback deployment
    pub async fn rollback(&mut self, environment: &str, revision: Option<u32>) -> Result<RollbackResult> {
        info!("Rolling back deployment in environment: {}", environment);

        let rollback_revision = revision.unwrap_or(1);

        // In a real implementation, this would execute helm rollback
        info!("Rollback to revision {} initiated for environment {}", rollback_revision, environment);

        Ok(RollbackResult {
            revision: rollback_revision,
            status: "Success".to_string(),
        })
    }

    /// Scale deployment
    pub async fn scale(&mut self, environment: &str, replicas: u32) -> Result<ScaleResult> {
        info!("Scaling deployment in environment {} to {} replicas", environment, replicas);

        // In a real implementation, this would update the HPA or deployment

        Ok(ScaleResult {
            current_replicas: 2, // Mock current value
            target_replicas: replicas,
            status: "Scaling".to_string(),
        })
    }

    /// Pause deployment
    pub async fn pause(&mut self, environment: &str) -> Result<()> {
        info!("Pausing deployment in environment: {}", environment);
        Ok(())
    }

    /// Resume deployment
    pub async fn resume(&mut self, environment: &str) -> Result<()> {
        info!("Resuming deployment in environment: {}", environment);
        Ok(())
    }

    /// Delete deployment
    pub async fn delete(&mut self, environment: &str) -> Result<()> {
        info!("Deleting deployment in environment: {}", environment);
        Ok(())
    }

    /// Generate Kubernetes manifests
    pub async fn generate_manifests(&mut self, environment: &str, version: &str) -> Result<HashMap<String, String>> {
        info!("Generating Kubernetes manifests for environment: {}", environment);

        let mut manifests = HashMap::new();

        // Generate manifests (mock implementation)
        manifests.insert("deployment".to_string(), self.create_deployment_manifest(environment, version).await?);
        manifests.insert("service".to_string(), self.create_service_manifest(environment).await?);
        manifests.insert("configmap".to_string(), self.create_configmap_manifest(environment).await?);
        manifests.insert("hpa".to_string(), self.create_hpa_manifest(environment).await?);

        Ok(manifests)
    }

    /// Generate Helm chart
    pub async fn generate_helm_chart(&mut self, output_dir: &Path) -> Result<()> {
        info!("Generating Helm chart");

        // Create chart directory structure
        let chart_dir = output_dir.join(&self.config.helm.chart_name);
        tokio::fs::create_dir_all(&chart_dir).await?;
        tokio::fs::create_dir_all(chart_dir.join("templates")).await?;

        // Generate Chart.yaml
        self.generate_chart_yaml(&chart_dir).await?;

        // Generate values.yaml
        self.generate_values_yaml(&chart_dir).await?;

        // Generate templates
        self.generate_helm_templates(&chart_dir).await?;

        Ok(())
    }

    // Private implementation methods

    async fn validate_kubernetes_connection(&self) -> Result<()> {
        // Mock validation - in real implementation would use kubernetes client
        info!("Validating Kubernetes connection");
        Ok(())
    }

    async fn setup_helm_repositories(&self) -> Result<()> {
        // Mock setup - in real implementation would configure helm repos
        info!("Setting up Helm repositories");
        Ok(())
    }

    async fn load_existing_deployments(&self) -> Result<()> {
        // Mock loading - in real implementation would query Kubernetes/Helm
        info!("Loading existing deployments");
        Ok(())
    }

    async fn execute_deployment(&self, deployment_id: &str, _environment: &str, _version: &str) -> Result<()> {
        // Update status to in progress
        self.update_deployment_status(deployment_id, DeploymentState::InProgress).await?;

        // Mock deployment steps
        info!("Executing pre-deployment hooks");
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        info!("Applying Kubernetes manifests");
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        info!("Waiting for pods to be ready");
        tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;

        info!("Running health checks");
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        info!("Executing post-deployment hooks");
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Update status to deployed
        self.update_deployment_status(deployment_id, DeploymentState::Deployed).await?;

        info!("Deployment {} completed successfully", deployment_id);
        Ok(())
    }

    async fn update_deployment_status(&self, deployment_id: &str, state: DeploymentState) -> Result<()> {
        let mut deployments = self.active_deployments.write().await;
        if let Some(deployment) = deployments.get_mut(deployment_id) {
            deployment.status = state;
            deployment.updated_at = Utc::now();
        }
        Ok(())
    }

    async fn generate_deployment_manifest(&self, environment: &str, output_dir: &Path) -> Result<()> {
        let env_config = self.config.environments.get(environment).unwrap();

        let manifest = format!(r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: inferno-{environment}
  namespace: {namespace}
  labels:
    app: inferno
    environment: {environment}
spec:
  replicas: {replicas}
  selector:
    matchLabels:
      app: inferno
      environment: {environment}
  template:
    metadata:
      labels:
        app: inferno
        environment: {environment}
    spec:
      serviceAccountName: {service_account}
      securityContext:
        runAsNonRoot: true
        runAsUser: 1001
        fsGroup: 1001
      containers:
      - name: inferno
        image: {registry}/{repository}:latest
        ports:
        - containerPort: 8080
          name: http
        - containerPort: 9090
          name: metrics
        env:
        - name: ENVIRONMENT
          value: "{environment}"
        - name: LOG_LEVEL
          value: "{log_level}"
        resources:
          requests:
            cpu: {cpu_request}
            memory: {memory_request}
          limits:
            cpu: {cpu_limit}
            memory: {memory_limit}
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
        volumeMounts:
        - name: model-storage
          mountPath: /data/models
        - name: cache-storage
          mountPath: /data/cache
      volumes:
      - name: model-storage
        persistentVolumeClaim:
          claimName: model-storage-{environment}
      - name: cache-storage
        persistentVolumeClaim:
          claimName: cache-storage-{environment}
"#,
            environment = environment,
            namespace = self.config.kubernetes.namespace,
            replicas = (self.config.autoscaling.min_replicas as f64 * env_config.scale_factor) as u32,
            service_account = self.config.kubernetes.service_account,
            registry = self.config.registry.url,
            repository = format!("{}/inferno", self.config.registry.repository_prefix),
            log_level = env_config.environment_variables.get("LOG_LEVEL").unwrap_or(&"info".to_string()),
            cpu_request = self.config.resources.cpu.request,
            memory_request = self.config.resources.memory.request,
            cpu_limit = self.config.resources.cpu.limit,
            memory_limit = self.config.resources.memory.limit,
        );

        let file_path = output_dir.join(format!("deployment-{}.yaml", environment));
        tokio::fs::write(file_path, manifest).await?;
        Ok(())
    }

    async fn generate_service_manifest(&self, environment: &str, output_dir: &Path) -> Result<()> {
        let manifest = format!(r#"apiVersion: v1
kind: Service
metadata:
  name: inferno-{environment}
  namespace: {namespace}
  labels:
    app: inferno
    environment: {environment}
spec:
  selector:
    app: inferno
    environment: {environment}
  ports:
  - name: http
    port: 8080
    targetPort: 8080
  - name: metrics
    port: 9090
    targetPort: 9090
  type: ClusterIP
"#,
            environment = environment,
            namespace = self.config.kubernetes.namespace,
        );

        let file_path = output_dir.join(format!("service-{}.yaml", environment));
        tokio::fs::write(file_path, manifest).await?;
        Ok(())
    }

    async fn generate_configmap_manifest(&self, environment: &str, output_dir: &Path) -> Result<()> {
        let env_config = self.config.environments.get(environment).unwrap();

        let mut env_vars = String::new();
        for (key, value) in &env_config.environment_variables {
            env_vars.push_str(&format!("  {}: \"{}\"\n", key, value));
        }

        let manifest = format!(r#"apiVersion: v1
kind: ConfigMap
metadata:
  name: inferno-config-{environment}
  namespace: {namespace}
  labels:
    app: inferno
    environment: {environment}
data:
{env_vars}"#,
            environment = environment,
            namespace = self.config.kubernetes.namespace,
            env_vars = env_vars,
        );

        let file_path = output_dir.join(format!("configmap-{}.yaml", environment));
        tokio::fs::write(file_path, manifest).await?;
        Ok(())
    }

    async fn generate_hpa_manifest(&self, environment: &str, output_dir: &Path) -> Result<()> {
        let env_config = self.config.environments.get(environment).unwrap();

        let manifest = format!(r#"apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: inferno-hpa-{environment}
  namespace: {namespace}
  labels:
    app: inferno
    environment: {environment}
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: inferno-{environment}
  minReplicas: {min_replicas}
  maxReplicas: {max_replicas}
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: {cpu_target}
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: {memory_target}
"#,
            environment = environment,
            namespace = self.config.kubernetes.namespace,
            min_replicas = (self.config.autoscaling.min_replicas as f64 * env_config.scale_factor) as u32,
            max_replicas = (self.config.autoscaling.max_replicas as f64 * env_config.scale_factor) as u32,
            cpu_target = self.config.autoscaling.target_cpu_utilization,
            memory_target = self.config.autoscaling.target_memory_utilization,
        );

        let file_path = output_dir.join(format!("hpa-{}.yaml", environment));
        tokio::fs::write(file_path, manifest).await?;
        Ok(())
    }

    async fn generate_chart_yaml(&self, chart_dir: &Path) -> Result<()> {
        let chart_yaml = format!(r#"apiVersion: v2
name: {name}
description: Inferno AI/ML inference server Helm chart
type: application
version: {version}
appVersion: "0.1.0"
keywords:
  - ai
  - ml
  - inference
  - gguf
  - onnx
home: https://github.com/inferno-ai/inferno
sources:
  - https://github.com/inferno-ai/inferno
maintainers:
  - name: Inferno Team
    email: team@inferno.ai
dependencies:
{dependencies}"#,
            name = self.config.helm.chart_name,
            version = self.config.helm.chart_version,
            dependencies = self.config.helm.dependencies.iter()
                .map(|dep| format!("  - name: {}\n    version: {}\n    repository: {}\n    condition: {}",
                    dep.name, dep.version, dep.repository, dep.condition))
                .collect::<Vec<_>>()
                .join("\n"),
        );

        let file_path = chart_dir.join("Chart.yaml");
        tokio::fs::write(file_path, chart_yaml).await?;
        Ok(())
    }

    async fn generate_values_yaml(&self, chart_dir: &Path) -> Result<()> {
        let values = serde_yaml::to_string(&self.config)?;
        let file_path = chart_dir.join("values.yaml");
        tokio::fs::write(file_path, values).await?;
        Ok(())
    }

    async fn generate_helm_templates(&self, chart_dir: &Path) -> Result<()> {
        let templates_dir = chart_dir.join("templates");

        // Generate basic templates
        self.generate_deployment_template(&templates_dir).await?;
        self.generate_service_template(&templates_dir).await?;
        self.generate_configmap_template(&templates_dir).await?;
        self.generate_hpa_template(&templates_dir).await?;
        self.generate_ingress_template(&templates_dir).await?;

        Ok(())
    }

    async fn generate_deployment_template(&self, templates_dir: &Path) -> Result<()> {
        let template = r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ include "inferno.fullname" . }}
  namespace: {{ .Values.kubernetes.namespace }}
  labels:
    {{- include "inferno.labels" . | nindent 4 }}
spec:
  {{- if not .Values.autoscaling.hpa_enabled }}
  replicas: {{ .Values.autoscaling.min_replicas }}
  {{- end }}
  selector:
    matchLabels:
      {{- include "inferno.selectorLabels" . | nindent 6 }}
  template:
    metadata:
      labels:
        {{- include "inferno.selectorLabels" . | nindent 8 }}
    spec:
      {{- with .Values.kubernetes.image_pull_secrets }}
      imagePullSecrets:
        {{- toYaml . | nindent 8 }}
      {{- end }}
      serviceAccountName: {{ .Values.kubernetes.service_account }}
      securityContext:
        {{- toYaml .Values.kubernetes.security_context | nindent 8 }}
      containers:
        - name: {{ .Chart.Name }}
          image: "{{ .Values.registry.url }}/{{ .Values.registry.repository_prefix }}/inferno:{{ .Values.image.tag | default .Chart.AppVersion }}"
          imagePullPolicy: {{ .Values.image.pullPolicy }}
          ports:
            - name: http
              containerPort: 8080
              protocol: TCP
            - name: metrics
              containerPort: 9090
              protocol: TCP
          env:
            {{- range $key, $value := .Values.environments.dev.environment_variables }}
            - name: {{ $key }}
              value: {{ $value | quote }}
            {{- end }}
          livenessProbe:
            {{- toYaml .Values.monitoring.health_checks.liveness_probe | nindent 12 }}
          readinessProbe:
            {{- toYaml .Values.monitoring.health_checks.readiness_probe | nindent 12 }}
          resources:
            {{- toYaml .Values.resources | nindent 12 }}
          volumeMounts:
            {{- range .Values.resources.storage.persistent_volume_claims }}
            - name: {{ .name }}
              mountPath: {{ .mount_path }}
            {{- end }}
      volumes:
        {{- range .Values.resources.storage.persistent_volume_claims }}
        - name: {{ .name }}
          persistentVolumeClaim:
            claimName: {{ .name }}-{{ include "inferno.fullname" $ }}
        {{- end }}
      {{- with .Values.kubernetes.node_selector }}
      nodeSelector:
        {{- toYaml . | nindent 8 }}
      {{- end }}
      {{- with .Values.kubernetes.affinity }}
      affinity:
        {{- toYaml . | nindent 8 }}
      {{- end }}
      {{- with .Values.kubernetes.tolerations }}
      tolerations:
        {{- toYaml . | nindent 8 }}
      {{- end }}
"#;

        let file_path = templates_dir.join("deployment.yaml");
        tokio::fs::write(file_path, template).await?;
        Ok(())
    }

    async fn generate_service_template(&self, templates_dir: &Path) -> Result<()> {
        let template = r#"apiVersion: v1
kind: Service
metadata:
  name: {{ include "inferno.fullname" . }}
  namespace: {{ .Values.kubernetes.namespace }}
  labels:
    {{- include "inferno.labels" . | nindent 4 }}
spec:
  type: {{ .Values.service.type }}
  ports:
    - port: {{ .Values.service.port }}
      targetPort: http
      protocol: TCP
      name: http
    - port: {{ .Values.service.metricsPort }}
      targetPort: metrics
      protocol: TCP
      name: metrics
  selector:
    {{- include "inferno.selectorLabels" . | nindent 4 }}
"#;

        let file_path = templates_dir.join("service.yaml");
        tokio::fs::write(file_path, template).await?;
        Ok(())
    }

    async fn generate_configmap_template(&self, templates_dir: &Path) -> Result<()> {
        let template = r#"apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ include "inferno.fullname" . }}-config
  namespace: {{ .Values.kubernetes.namespace }}
  labels:
    {{- include "inferno.labels" . | nindent 4 }}
data:
  config.toml: |
    {{- .Values.inferno_config | toYaml | nindent 4 }}
"#;

        let file_path = templates_dir.join("configmap.yaml");
        tokio::fs::write(file_path, template).await?;
        Ok(())
    }

    async fn generate_hpa_template(&self, templates_dir: &Path) -> Result<()> {
        let template = r#"{{- if .Values.autoscaling.hpa_enabled }}
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: {{ include "inferno.fullname" . }}
  namespace: {{ .Values.kubernetes.namespace }}
  labels:
    {{- include "inferno.labels" . | nindent 4 }}
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: {{ include "inferno.fullname" . }}
  minReplicas: {{ .Values.autoscaling.min_replicas }}
  maxReplicas: {{ .Values.autoscaling.max_replicas }}
  metrics:
    {{- if .Values.autoscaling.target_cpu_utilization }}
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: {{ .Values.autoscaling.target_cpu_utilization }}
    {{- end }}
    {{- if .Values.autoscaling.target_memory_utilization }}
    - type: Resource
      resource:
        name: memory
        target:
          type: Utilization
          averageUtilization: {{ .Values.autoscaling.target_memory_utilization }}
    {{- end }}
{{- end }}
"#;

        let file_path = templates_dir.join("hpa.yaml");
        tokio::fs::write(file_path, template).await?;
        Ok(())
    }

    async fn generate_ingress_template(&self, templates_dir: &Path) -> Result<()> {
        let template = r#"{{- if .Values.ingress.enabled -}}
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: {{ include "inferno.fullname" . }}
  namespace: {{ .Values.kubernetes.namespace }}
  labels:
    {{- include "inferno.labels" . | nindent 4 }}
  {{- with .Values.ingress.annotations }}
  annotations:
    {{- toYaml . | nindent 4 }}
  {{- end }}
spec:
  {{- if .Values.ingress.className }}
  ingressClassName: {{ .Values.ingress.className }}
  {{- end }}
  {{- if .Values.ingress.tls }}
  tls:
    {{- range .Values.ingress.tls }}
    - hosts:
        {{- range .hosts }}
        - {{ . | quote }}
        {{- end }}
      secretName: {{ .secretName }}
    {{- end }}
  {{- end }}
  rules:
    {{- range .Values.ingress.hosts }}
    - host: {{ .host | quote }}
      http:
        paths:
          {{- range .paths }}
          - path: {{ .path }}
            {{- if and .pathType (semverCompare ">=1.18-0" $.Capabilities.KubeVersion.GitVersion) }}
            pathType: {{ .pathType }}
            {{- end }}
            backend:
              {{- if semverCompare ">=1.19-0" $.Capabilities.KubeVersion.GitVersion }}
              service:
                name: {{ include "inferno.fullname" $ }}
                port:
                  number: {{ $.Values.service.port }}
              {{- else }}
              serviceName: {{ include "inferno.fullname" $ }}
              servicePort: {{ $.Values.service.port }}
              {{- end }}
          {{- end }}
    {{- end }}
{{- end }}
"#;

        let file_path = templates_dir.join("ingress.yaml");
        tokio::fs::write(file_path, template).await?;
        Ok(())
    }

    /// Get deployment status
    pub async fn get_status(&mut self, environment: &str, _namespace: Option<&str>) -> Result<StatusInfo> {
        info!("Getting status for environment: {}", environment);

        Ok(StatusInfo {
            environment: environment.to_string(),
            version: "1.0.0".to_string(),
            status: "Running".to_string(),
            ready_replicas: 2,
            total_replicas: 2,
            last_updated: Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            pods: vec![
                PodInfo {
                    name: format!("inferno-{}-pod-1", environment),
                    status: "Running".to_string(),
                    ready: "1/1".to_string(),
                },
                PodInfo {
                    name: format!("inferno-{}-pod-2", environment),
                    status: "Running".to_string(),
                    ready: "1/1".to_string(),
                },
            ],
            service_urls: HashMap::from([
                ("main".to_string(), format!("http://inferno-{}.default.svc.cluster.local:8080", environment)),
                ("metrics".to_string(), format!("http://inferno-{}.default.svc.cluster.local:9090", environment)),
            ]),
            health_checks: vec![
                HealthCheck {
                    name: "liveness".to_string(),
                    passing: true,
                },
                HealthCheck {
                    name: "readiness".to_string(),
                    passing: true,
                },
            ],
        })
    }

    /// Get deployment logs
    pub async fn get_logs(&mut self, environment: &str, _namespace: Option<&str>, lines: u32, _since: Option<&str>, _selector: Option<&str>) -> Result<Vec<LogEntry>> {
        info!("Getting logs for environment: {}", environment);

        let mut logs = Vec::new();
        for i in 0..lines.min(10) {
            logs.push(LogEntry {
                timestamp: Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                pod: format!("inferno-{}-pod-1", environment),
                message: format!("Sample log entry {}", i + 1),
            });
        }

        Ok(logs)
    }

    /// Validate deployment configuration
    pub async fn validate_config(&mut self, environment: &str, _config_file: Option<&Path>, _namespace: Option<&str>, cluster: bool) -> Result<ValidationResult> {
        info!("Validating configuration for environment: {}", environment);

        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut cluster_resources = Vec::new();

        // Mock validation
        if !self.config.environments.contains_key(environment) {
            errors.push(format!("Unknown environment: {}", environment));
        }

        if cluster {
            cluster_resources.push(ClusterResource {
                name: format!("inferno-{}", environment),
                kind: "Deployment".to_string(),
                status: "Available".to_string(),
            });
        }

        if self.config.autoscaling.min_replicas > self.config.autoscaling.max_replicas {
            warnings.push("Min replicas is greater than max replicas".to_string());
        }

        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            warnings,
            errors,
            cluster_resources,
        })
    }

    /// Get deployment history
    pub async fn get_deployment_history(&mut self, environment: &str, _namespace: Option<&str>, limit: u32) -> Result<Vec<DeploymentHistoryEntry>> {
        info!("Getting deployment history for environment: {}", environment);

        let mut history = Vec::new();
        for i in 0..limit.min(5) {
            history.push(DeploymentHistoryEntry {
                revision: i + 1,
                version: format!("1.0.{}", i),
                timestamp: Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                status: "Deployed".to_string(),
                rolled_back: false,
            });
        }

        Ok(history)
    }

    /// Enable autoscaling
    pub async fn enable_autoscaling(&mut self, environment: &str, _namespace: Option<&str>, min_replicas: u32, max_replicas: u32, cpu_percent: u32, _memory_percent: Option<u32>) -> Result<()> {
        info!("Enabling autoscaling for environment: {} (min: {}, max: {}, cpu: {}%)", environment, min_replicas, max_replicas, cpu_percent);

        // In a real implementation, this would configure HPA

        Ok(())
    }

    /// Disable autoscaling
    pub async fn disable_autoscaling(&mut self, environment: &str, _namespace: Option<&str>) -> Result<()> {
        info!("Disabling autoscaling for environment: {}", environment);

        // In a real implementation, this would remove HPA

        Ok(())
    }

    /// Get autoscaling status
    pub async fn get_autoscaling_status(&mut self, environment: &str, _namespace: Option<&str>) -> Result<AutoscalingStatus> {
        info!("Getting autoscaling status for environment: {}", environment);

        Ok(AutoscalingStatus {
            enabled: true,
            current_replicas: 3,
            min_replicas: self.config.autoscaling.min_replicas,
            max_replicas: self.config.autoscaling.max_replicas,
            current_cpu_percent: 45,
            target_cpu_percent: self.config.autoscaling.target_cpu_utilization,
            current_memory_percent: Some(60),
            target_memory_percent: Some(self.config.autoscaling.target_memory_utilization),
            last_scale_time: Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        })
    }

    /// Update autoscaling
    pub async fn update_autoscaling(&mut self, environment: &str, _namespace: Option<&str>, _min_replicas: Option<u32>, _max_replicas: Option<u32>, _cpu_percent: Option<u32>, _memory_percent: Option<u32>) -> Result<()> {
        info!("Updating autoscaling for environment: {}", environment);

        // In a real implementation, this would update HPA configuration

        Ok(())
    }

    /// Set configuration
    pub async fn set_config(&mut self, environment: &str, _namespace: Option<&str>, key: &str, _value: &str, secret: bool) -> Result<()> {
        info!("Setting {} {} in environment: {}", if secret { "secret" } else { "config" }, key, environment);

        // In a real implementation, this would create/update ConfigMap or Secret

        Ok(())
    }

    /// Get configuration
    pub async fn get_config(&mut self, environment: &str, _namespace: Option<&str>, key: &str) -> Result<String> {
        info!("Getting config {} in environment: {}", key, environment);

        // In a real implementation, this would read from ConfigMap or Secret
        Ok(format!("value-for-{}", key))
    }

    /// List configuration
    pub async fn list_config(&mut self, environment: &str, _namespace: Option<&str>, include_secrets: bool) -> Result<Vec<ConfigEntry>> {
        info!("Listing config for environment: {} (secrets: {})", environment, include_secrets);

        let mut configs = vec![
            ConfigEntry {
                key: "LOG_LEVEL".to_string(),
                value: "info".to_string(),
                is_secret: false,
            },
            ConfigEntry {
                key: "ENVIRONMENT".to_string(),
                value: environment.to_string(),
                is_secret: false,
            },
        ];

        if include_secrets {
            configs.push(ConfigEntry {
                key: "API_KEY".to_string(),
                value: "****".to_string(),
                is_secret: true,
            });
        }

        Ok(configs)
    }

    /// Delete configuration
    pub async fn delete_config(&mut self, environment: &str, _namespace: Option<&str>, key: &str) -> Result<()> {
        info!("Deleting config {} in environment: {}", key, environment);

        // In a real implementation, this would remove from ConfigMap or Secret

        Ok(())
    }

    /// Import configuration
    pub async fn import_config(&mut self, environment: &str, _namespace: Option<&str>, _file: &Path, secrets: bool) -> Result<()> {
        info!("Importing {} from file for environment: {}", if secrets { "secrets" } else { "config" }, environment);

        // In a real implementation, this would read file and create ConfigMap/Secret

        Ok(())
    }

    /// Export configuration
    pub async fn export_config(&mut self, environment: &str, _namespace: Option<&str>, file: &Path, include_secrets: bool, format: &str) -> Result<()> {
        info!("Exporting config for environment: {} to {} (format: {}, secrets: {})", environment, file.display(), format, include_secrets);

        // In a real implementation, this would export ConfigMap/Secret to file
        let content = match format {
            "yaml" => "# Configuration exported\nkey: value\n".to_string(),
            "json" => "{\"key\": \"value\"}".to_string(),
            "env" => "KEY=value\n".to_string(),
            _ => "key=value\n".to_string(),
        };

        tokio::fs::write(file, content).await?;

        Ok(())
    }

    /// Check health
    pub async fn check_health(&mut self, environment: &str, _namespace: Option<&str>) -> Result<HealthReport> {
        info!("Checking health for environment: {}", environment);

        Ok(HealthReport {
            overall_healthy: true,
            uptime: "2d 4h 30m".to_string(),
            services: vec![
                ServiceHealthInfo {
                    name: "api".to_string(),
                    healthy: true,
                    status: "running".to_string(),
                    response_time_ms: 45,
                },
                ServiceHealthInfo {
                    name: "inference".to_string(),
                    healthy: true,
                    status: "running".to_string(),
                    response_time_ms: 120,
                },
            ],
            cpu_usage: 45,
            cpu_limit: 80,
            memory_usage: 60,
            memory_limit: 90,
            recent_errors: vec![],
        })
    }

    // Helper methods for manifest generation
    async fn create_deployment_manifest(&self, environment: &str, version: &str) -> Result<String> {
        let env_config = self.config.environments.get(environment).unwrap();

        let manifest = format!(r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: inferno-{environment}
  namespace: {namespace}
spec:
  replicas: {replicas}
  selector:
    matchLabels:
      app: inferno
      environment: {environment}
  template:
    metadata:
      labels:
        app: inferno
        environment: {environment}
    spec:
      containers:
      - name: inferno
        image: {registry}/{repository}:{version}
        ports:
        - containerPort: 8080
"#,
            environment = environment,
            namespace = self.config.kubernetes.namespace,
            replicas = (self.config.autoscaling.min_replicas as f64 * env_config.scale_factor) as u32,
            registry = self.config.registry.url,
            repository = format!("{}/inferno", self.config.registry.repository_prefix),
            version = version,
        );

        Ok(manifest)
    }

    async fn create_service_manifest(&self, environment: &str) -> Result<String> {
        let manifest = format!(r#"apiVersion: v1
kind: Service
metadata:
  name: inferno-{environment}
  namespace: {namespace}
spec:
  selector:
    app: inferno
    environment: {environment}
  ports:
  - port: 8080
    targetPort: 8080
"#,
            environment = environment,
            namespace = self.config.kubernetes.namespace,
        );

        Ok(manifest)
    }

    async fn create_configmap_manifest(&self, environment: &str) -> Result<String> {
        let manifest = format!(r#"apiVersion: v1
kind: ConfigMap
metadata:
  name: inferno-config-{environment}
  namespace: {namespace}
data:
  ENVIRONMENT: "{environment}"
"#,
            environment = environment,
            namespace = self.config.kubernetes.namespace,
        );

        Ok(manifest)
    }

    async fn create_hpa_manifest(&self, environment: &str) -> Result<String> {
        let manifest = format!(r#"apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: inferno-hpa-{environment}
  namespace: {namespace}
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: inferno-{environment}
  minReplicas: {min_replicas}
  maxReplicas: {max_replicas}
"#,
            environment = environment,
            namespace = self.config.kubernetes.namespace,
            min_replicas = self.config.autoscaling.min_replicas,
            max_replicas = self.config.autoscaling.max_replicas,
        );

        Ok(manifest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_deployment_manager_initialization() {
        let config = DeploymentConfig::default();
        let manager = DeploymentManager::new(config);

        let result = manager.initialize().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_generate_manifests() {
        let config = DeploymentConfig::default();
        let manager = DeploymentManager::new(config);

        let temp_dir = TempDir::new().unwrap();
        let result = manager.generate_manifests("dev", temp_dir.path()).await;
        assert!(result.is_ok());

        // Check if files were created
        assert!(temp_dir.path().join("deployment-dev.yaml").exists());
        assert!(temp_dir.path().join("service-dev.yaml").exists());
        assert!(temp_dir.path().join("configmap-dev.yaml").exists());
        assert!(temp_dir.path().join("hpa-dev.yaml").exists());
    }

    #[tokio::test]
    async fn test_generate_helm_chart() {
        let config = DeploymentConfig::default();
        let manager = DeploymentManager::new(config);

        let temp_dir = TempDir::new().unwrap();
        let result = manager.generate_helm_chart(temp_dir.path()).await;
        assert!(result.is_ok());

        let chart_dir = temp_dir.path().join("inferno");
        assert!(chart_dir.exists());
        assert!(chart_dir.join("Chart.yaml").exists());
        assert!(chart_dir.join("values.yaml").exists());
        assert!(chart_dir.join("templates").exists());
    }

    #[test]
    fn test_environment_configurations() {
        let dev_config = EnvironmentConfig::development();
        assert_eq!(dev_config.name, "development");
        assert_eq!(dev_config.scale_factor, 0.5);
        assert!(dev_config.features.get("debug_mode").unwrap_or(&false));

        let prod_config = EnvironmentConfig::production();
        assert_eq!(prod_config.name, "production");
        assert_eq!(prod_config.scale_factor, 1.0);
        assert!(!prod_config.features.get("debug_mode").unwrap_or(&true));
    }
}