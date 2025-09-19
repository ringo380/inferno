use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrafficRampStrategy {
    Immediate,   // All traffic goes to new version immediately
    Gradual,     // Traffic increases gradually over time
    Canary,      // Small percentage of traffic for extended period
    BlueGreen,   // Switch all traffic at once after validation
}

/// A/B testing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTestingConfig {
    pub enabled: bool,
    pub default_test_duration_hours: u64,
    pub min_sample_size: usize,
    pub confidence_level: f64,
    pub auto_promote_threshold: f64,
    pub auto_rollback_threshold: f64,
    pub max_concurrent_tests: usize,
    pub traffic_ramp_strategy: TrafficRampStrategy,
    pub monitoring_interval_ms: u64,
}

impl Default for ABTestingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_test_duration_hours: 24,
            min_sample_size: 1000,
            confidence_level: 0.95,
            auto_promote_threshold: 0.05, // 5% improvement
            auto_rollback_threshold: -0.10, // 10% degradation
            max_concurrent_tests: 5,
            traffic_ramp_strategy: TrafficRampStrategy::Gradual,
            monitoring_interval_ms: 5000,
        }
    }
}