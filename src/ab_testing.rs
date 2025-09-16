use crate::{
    config::Config,
    metrics::MetricsCollector,
    models::{ModelInfo, ModelManager},
    monitoring::{PerformanceMetric, Alert},
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::{
    sync::{Mutex, RwLock},
    time::interval,
};
use tracing::{debug, error, info, warn};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrafficRampStrategy {
    Immediate,   // All traffic goes to new version immediately
    Gradual,     // Traffic increases gradually over time
    Canary,      // Small percentage of traffic for extended period
    BlueGreen,   // Switch all traffic at once after validation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTest {
    pub id: String,
    pub name: String,
    pub description: String,
    pub control_model: ModelVariant,
    pub treatment_model: ModelVariant,
    pub status: TestStatus,
    pub config: ABTestConfig,
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub traffic_allocation: TrafficAllocation,
    pub metrics: TestMetrics,
    pub statistical_results: Option<StatisticalResults>,
    pub created_by: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVariant {
    pub model_id: String,
    pub model_version: String,
    pub model_path: String,
    pub configuration: HashMap<String, String>,
    pub load_time: Option<SystemTime>,
    pub health_status: VariantHealth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariantHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Loading,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TestStatus {
    Draft,
    Starting,
    Running,
    Paused,
    Completed,
    Failed,
    RolledBack,
    Promoted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTestConfig {
    pub duration_hours: u64,
    pub target_sample_size: usize,
    pub significance_level: f64,
    pub minimum_effect_size: f64,
    pub auto_promote: bool,
    pub auto_rollback: bool,
    pub traffic_ramp_schedule: Vec<TrafficRampStep>,
    pub success_metrics: Vec<String>,
    pub guard_metrics: Vec<GuardMetric>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficRampStep {
    pub time_offset_hours: f64,
    pub control_percentage: f64,
    pub treatment_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardMetric {
    pub metric_name: String,
    pub threshold_type: ThresholdType,
    pub threshold_value: f64,
    pub action: GuardAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThresholdType {
    Maximum,
    Minimum,
    Percentage,
    Absolute,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GuardAction {
    Pause,
    Rollback,
    Alert,
    ReduceTraffic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficAllocation {
    pub control_percentage: f64,
    pub treatment_percentage: f64,
    pub current_ramp_step: usize,
    pub last_updated: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestMetrics {
    pub control_metrics: VariantMetrics,
    pub treatment_metrics: VariantMetrics,
    pub samples_collected: usize,
    pub last_updated: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantMetrics {
    pub request_count: u64,
    pub success_count: u64,
    pub error_count: u64,
    pub avg_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub p99_response_time_ms: f64,
    pub throughput_rps: f64,
    pub error_rate: f64,
    pub custom_metrics: HashMap<String, f64>,
}

impl Default for VariantMetrics {
    fn default() -> Self {
        Self {
            request_count: 0,
            success_count: 0,
            error_count: 0,
            avg_response_time_ms: 0.0,
            p95_response_time_ms: 0.0,
            p99_response_time_ms: 0.0,
            throughput_rps: 0.0,
            error_rate: 0.0,
            custom_metrics: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalResults {
    pub control_mean: f64,
    pub treatment_mean: f64,
    pub effect_size: f64,
    pub confidence_interval: (f64, f64),
    pub p_value: f64,
    pub is_significant: bool,
    pub statistical_power: f64,
    pub recommendation: TestRecommendation,
    pub analysis_timestamp: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestRecommendation {
    Promote,
    Rollback,
    Continue,
    ExtendTest,
    IncreaseTraffic,
    DecreaseTraffic,
}

#[derive(Debug, Clone)]
pub struct CanaryDeployment {
    pub id: String,
    pub name: String,
    pub model_id: String,
    pub canary_version: String,
    pub stable_version: String,
    pub status: CanaryStatus,
    pub config: CanaryConfig,
    pub start_time: SystemTime,
    pub metrics: CanaryMetrics,
    pub health_checks: Vec<HealthCheck>,
    pub rollback_trigger: Option<RollbackTrigger>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CanaryStatus {
    Preparing,
    Deploying,
    Monitoring,
    Promoting,
    RollingBack,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryConfig {
    pub traffic_percentage: f64,
    pub duration_minutes: u64,
    pub success_threshold: f64,
    pub error_threshold: f64,
    pub auto_promote: bool,
    pub auto_rollback: bool,
    pub health_check_interval_ms: u64,
    pub promotion_criteria: Vec<PromotionCriterion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionCriterion {
    pub metric_name: String,
    pub comparison: ComparisonOperator,
    pub threshold_value: f64,
    pub required_duration_minutes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOperator {
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Equal,
    NotEqual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryMetrics {
    pub canary_requests: u64,
    pub stable_requests: u64,
    pub canary_success_rate: f64,
    pub stable_success_rate: f64,
    pub canary_avg_latency: f64,
    pub stable_avg_latency: f64,
    pub last_updated: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub check_type: HealthCheckType,
    pub status: HealthCheckStatus,
    pub last_check: SystemTime,
    pub message: String,
    pub check_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthCheckType {
    ModelLoad,
    InferenceLatency,
    ErrorRate,
    MemoryUsage,
    CustomCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthCheckStatus {
    Pass,
    Fail,
    Warning,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackTrigger {
    pub trigger_type: RollbackTriggerType,
    pub threshold: f64,
    pub evaluation_window_minutes: u64,
    pub triggered_at: Option<SystemTime>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RollbackTriggerType {
    ErrorRate,
    LatencyIncrease,
    SuccessRateDecrease,
    CustomMetric,
    ManualTrigger,
}

pub struct ABTestingManager {
    config: ABTestingConfig,
    active_tests: Arc<RwLock<HashMap<String, ABTest>>>,
    test_history: Arc<RwLock<Vec<ABTest>>>,
    canary_deployments: Arc<RwLock<HashMap<String, CanaryDeployment>>>,
    model_manager: Arc<ModelManager>,
    metrics_collector: Option<Arc<MetricsCollector>>,
    background_tasks: Vec<tokio::task::JoinHandle<()>>,
}

impl ABTestingManager {
    pub async fn new(
        config: ABTestingConfig,
        model_manager: Arc<ModelManager>,
        metrics_collector: Option<Arc<MetricsCollector>>,
    ) -> Result<Self> {
        let active_tests = Arc::new(RwLock::new(HashMap::new()));
        let test_history = Arc::new(RwLock::new(Vec::new()));
        let canary_deployments = Arc::new(RwLock::new(HashMap::new()));

        let mut manager = Self {
            config,
            active_tests,
            test_history,
            canary_deployments,
            model_manager,
            metrics_collector,
            background_tasks: Vec::new(),
        };

        if manager.config.enabled {
            manager.start_background_monitoring().await?;
        }

        Ok(manager)
    }

    pub async fn create_ab_test(&self, mut test: ABTest) -> Result<String> {
        let test_id = test.id.clone();

        // Validate test configuration
        self.validate_test_config(&test).await?;

        // Check if we can accommodate another test
        let active_tests = self.active_tests.read().await;
        if active_tests.len() >= self.config.max_concurrent_tests {
            return Err(anyhow::anyhow!("Maximum number of concurrent tests ({}) reached", self.config.max_concurrent_tests));
        }
        drop(active_tests);

        // Initialize test status
        test.status = TestStatus::Draft;
        test.start_time = SystemTime::now();

        // Add to active tests
        let mut active_tests = self.active_tests.write().await;
        active_tests.insert(test_id.clone(), test);

        info!("Created A/B test: {}", test_id);
        Ok(test_id)
    }

    pub async fn start_ab_test(&self, test_id: &str) -> Result<()> {
        let mut active_tests = self.active_tests.write().await;

        if let Some(test) = active_tests.get_mut(test_id) {
            // Load models for both variants
            test.status = TestStatus::Starting;
            test.start_time = SystemTime::now();

            // Initialize traffic allocation
            if let Some(first_step) = test.config.traffic_ramp_schedule.first() {
                test.traffic_allocation = TrafficAllocation {
                    control_percentage: first_step.control_percentage,
                    treatment_percentage: first_step.treatment_percentage,
                    current_ramp_step: 0,
                    last_updated: SystemTime::now(),
                };
            }

            test.status = TestStatus::Running;

            info!("Started A/B test: {}", test_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Test not found: {}", test_id))
        }
    }

    pub async fn pause_ab_test(&self, test_id: &str) -> Result<()> {
        let mut active_tests = self.active_tests.write().await;

        if let Some(test) = active_tests.get_mut(test_id) {
            match test.status {
                TestStatus::Running => {
                    test.status = TestStatus::Paused;
                    info!("Paused A/B test: {}", test_id);
                    Ok(())
                }
                _ => Err(anyhow::anyhow!("Test {} is not in running state", test_id)),
            }
        } else {
            Err(anyhow::anyhow!("Test not found: {}", test_id))
        }
    }

    pub async fn stop_ab_test(&self, test_id: &str, reason: Option<String>) -> Result<()> {
        let mut active_tests = self.active_tests.write().await;

        if let Some(mut test) = active_tests.remove(test_id) {
            test.end_time = Some(SystemTime::now());
            test.status = TestStatus::Completed;

            // Perform final statistical analysis
            if let Ok(results) = self.analyze_test_results(&test).await {
                test.statistical_results = Some(results);
            }

            // Move to history
            let mut history = self.test_history.write().await;
            history.push(test);

            info!("Stopped A/B test: {} (reason: {:?})", test_id, reason);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Test not found: {}", test_id))
        }
    }

    pub async fn create_canary_deployment(&self, deployment: CanaryDeployment) -> Result<String> {
        let deployment_id = deployment.id.clone();

        // Validate deployment configuration
        self.validate_canary_config(&deployment).await?;

        // Add to canary deployments
        let mut deployments = self.canary_deployments.write().await;
        deployments.insert(deployment_id.clone(), deployment);

        info!("Created canary deployment: {}", deployment_id);
        Ok(deployment_id)
    }

    pub async fn promote_canary(&self, deployment_id: &str) -> Result<()> {
        let mut deployments = self.canary_deployments.write().await;

        if let Some(deployment) = deployments.get_mut(deployment_id) {
            deployment.status = CanaryStatus::Promoting;

            // Logic to promote canary to stable
            info!("Promoting canary deployment: {}", deployment_id);

            deployment.status = CanaryStatus::Completed;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Canary deployment not found: {}", deployment_id))
        }
    }

    pub async fn rollback_canary(&self, deployment_id: &str, reason: String) -> Result<()> {
        let mut deployments = self.canary_deployments.write().await;

        if let Some(deployment) = deployments.get_mut(deployment_id) {
            deployment.status = CanaryStatus::RollingBack;
            deployment.rollback_trigger = Some(RollbackTrigger {
                trigger_type: RollbackTriggerType::ManualTrigger,
                threshold: 0.0,
                evaluation_window_minutes: 0,
                triggered_at: Some(SystemTime::now()),
                reason: Some(reason.clone()),
            });

            info!("Rolling back canary deployment: {} (reason: {})", deployment_id, reason);

            deployment.status = CanaryStatus::Failed;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Canary deployment not found: {}", deployment_id))
        }
    }

    pub async fn get_active_tests(&self) -> Vec<ABTest> {
        let active_tests = self.active_tests.read().await;
        active_tests.values().cloned().collect()
    }

    pub async fn get_test_history(&self, limit: Option<usize>) -> Vec<ABTest> {
        let history = self.test_history.read().await;
        match limit {
            Some(n) => history.iter().rev().take(n).cloned().collect(),
            None => history.iter().rev().cloned().collect(),
        }
    }

    pub async fn get_canary_deployments(&self) -> Vec<CanaryDeployment> {
        let deployments = self.canary_deployments.read().await;
        deployments.values().cloned().collect()
    }

    pub async fn record_request_result(
        &self,
        test_id: &str,
        variant: &str,
        success: bool,
        response_time_ms: u64,
    ) -> Result<()> {
        let mut active_tests = self.active_tests.write().await;

        if let Some(test) = active_tests.get_mut(test_id) {
            let variant_metrics = match variant {
                "control" => &mut test.metrics.control_metrics,
                "treatment" => &mut test.metrics.treatment_metrics,
                _ => return Err(anyhow::anyhow!("Invalid variant: {}", variant)),
            };

            variant_metrics.request_count += 1;
            if success {
                variant_metrics.success_count += 1;
            } else {
                variant_metrics.error_count += 1;
            }

            // Update response time metrics (simplified)
            let new_avg = (variant_metrics.avg_response_time_ms * (variant_metrics.request_count - 1) as f64
                          + response_time_ms as f64) / variant_metrics.request_count as f64;
            variant_metrics.avg_response_time_ms = new_avg;

            // Update error rate
            variant_metrics.error_rate = variant_metrics.error_count as f64 / variant_metrics.request_count as f64;

            // Update test metrics
            test.metrics.samples_collected += 1;
            test.metrics.last_updated = SystemTime::now();

            debug!("Recorded request result for test {} variant {}: success={}, response_time={}ms",
                   test_id, variant, success, response_time_ms);
        }

        Ok(())
    }

    async fn validate_test_config(&self, test: &ABTest) -> Result<()> {
        // Validate that models exist
        if let Err(e) = self.model_manager.resolve_model(&test.control_model.model_id).await {
            return Err(anyhow::anyhow!("Control model not found: {}", e));
        }

        if let Err(e) = self.model_manager.resolve_model(&test.treatment_model.model_id).await {
            return Err(anyhow::anyhow!("Treatment model not found: {}", e));
        }

        // Validate traffic allocation
        for step in &test.config.traffic_ramp_schedule {
            if step.control_percentage + step.treatment_percentage > 100.0 {
                return Err(anyhow::anyhow!("Traffic allocation exceeds 100%"));
            }
        }

        Ok(())
    }

    async fn validate_canary_config(&self, deployment: &CanaryDeployment) -> Result<()> {
        // Validate that model exists
        if let Err(e) = self.model_manager.resolve_model(&deployment.model_id).await {
            return Err(anyhow::anyhow!("Model not found: {}", e));
        }

        // Validate traffic percentage
        if deployment.config.traffic_percentage > 100.0 || deployment.config.traffic_percentage < 0.0 {
            return Err(anyhow::anyhow!("Invalid traffic percentage: {}", deployment.config.traffic_percentage));
        }

        Ok(())
    }

    async fn analyze_test_results(&self, test: &ABTest) -> Result<StatisticalResults> {
        let control = &test.metrics.control_metrics;
        let treatment = &test.metrics.treatment_metrics;

        // Simple statistical analysis (in real implementation, use proper statistical tests)
        let control_mean = control.avg_response_time_ms;
        let treatment_mean = treatment.avg_response_time_ms;
        let effect_size = (treatment_mean - control_mean) / control_mean;

        // Simplified confidence interval and p-value calculation
        let confidence_interval = (effect_size - 0.05, effect_size + 0.05);
        let p_value = if effect_size.abs() > 0.05 { 0.01 } else { 0.5 };
        let is_significant = p_value < (1.0 - test.config.significance_level);

        let recommendation = if is_significant {
            if effect_size > test.config.minimum_effect_size {
                TestRecommendation::Promote
            } else if effect_size < -test.config.minimum_effect_size {
                TestRecommendation::Rollback
            } else {
                TestRecommendation::Continue
            }
        } else {
            if test.metrics.samples_collected < test.config.target_sample_size {
                TestRecommendation::Continue
            } else {
                TestRecommendation::ExtendTest
            }
        };

        Ok(StatisticalResults {
            control_mean,
            treatment_mean,
            effect_size,
            confidence_interval,
            p_value,
            is_significant,
            statistical_power: 0.8, // Placeholder
            recommendation,
            analysis_timestamp: SystemTime::now(),
        })
    }

    async fn start_background_monitoring(&mut self) -> Result<()> {
        let monitoring_handle = self.start_monitoring_task().await;
        self.background_tasks.push(monitoring_handle);

        let evaluation_handle = self.start_evaluation_task().await;
        self.background_tasks.push(evaluation_handle);

        info!("Started A/B testing background monitoring");
        Ok(())
    }

    async fn start_monitoring_task(&self) -> tokio::task::JoinHandle<()> {
        let config = self.config.clone();
        let active_tests = Arc::clone(&self.active_tests);
        let canary_deployments = Arc::clone(&self.canary_deployments);

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(config.monitoring_interval_ms));

            loop {
                interval.tick().await;

                // Monitor active tests
                let tests = active_tests.read().await;
                for (test_id, test) in tests.iter() {
                    if test.status == TestStatus::Running {
                        debug!("Monitoring A/B test: {}", test_id);
                        // Check guard metrics, traffic ramp schedule, etc.
                    }
                }
                drop(tests);

                // Monitor canary deployments
                let deployments = canary_deployments.read().await;
                for (deployment_id, deployment) in deployments.iter() {
                    if deployment.status == CanaryStatus::Monitoring {
                        debug!("Monitoring canary deployment: {}", deployment_id);
                        // Check health checks, success rates, etc.
                    }
                }
            }
        })
    }

    async fn start_evaluation_task(&self) -> tokio::task::JoinHandle<()> {
        let config = self.config.clone();
        let active_tests = Arc::clone(&self.active_tests);

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60)); // Evaluate every minute

            loop {
                interval.tick().await;

                let tests_guard = active_tests.read().await;
                let running_tests: Vec<_> = tests_guard
                    .iter()
                    .filter(|(_, test)| test.status == TestStatus::Running)
                    .map(|(id, test)| (id.clone(), test.clone()))
                    .collect();
                drop(tests_guard);

                for (test_id, test) in running_tests {
                    // Check if test should be automatically promoted or rolled back
                    if config.auto_promote_threshold > 0.0 || config.auto_rollback_threshold < 0.0 {
                        debug!("Evaluating A/B test for auto-actions: {}", test_id);
                        // Perform automatic evaluation
                    }
                }
            }
        })
    }

    pub async fn get_test_status(&self, test_id: &str) -> Option<ABTest> {
        let active_tests = self.active_tests.read().await;
        active_tests.get(test_id).cloned()
    }

    pub async fn shutdown(&mut self) {
        info!("Shutting down A/B testing manager");

        for handle in &self.background_tasks {
            handle.abort();
        }

        self.background_tasks.clear();
    }
}

impl Drop for ABTestingManager {
    fn drop(&mut self) {
        for handle in &self.background_tasks {
            handle.abort();
        }
    }
}

// Traffic routing logic for A/B tests
pub struct TrafficRouter {
    ab_testing_manager: Arc<ABTestingManager>,
    routing_strategy: RoutingStrategy,
}

#[derive(Debug, Clone)]
pub enum RoutingStrategy {
    Random,
    Hash,
    Sticky,
    WeightedRoundRobin,
}

impl TrafficRouter {
    pub fn new(ab_testing_manager: Arc<ABTestingManager>, strategy: RoutingStrategy) -> Self {
        Self {
            ab_testing_manager,
            routing_strategy: strategy,
        }
    }

    pub async fn route_request(&self, request_id: &str, user_id: Option<&str>) -> Result<String> {
        let active_tests = self.ab_testing_manager.get_active_tests().await;

        for test in &active_tests {
            if test.status == TestStatus::Running {
                let variant = self.select_variant(&test, request_id, user_id)?;
                return Ok(variant);
            }
        }

        // Default to control if no tests are running
        Ok("control".to_string())
    }

    fn select_variant(&self, test: &ABTest, request_id: &str, user_id: Option<&str>) -> Result<String> {
        match self.routing_strategy {
            RoutingStrategy::Random => {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let random_value: f64 = rng.gen();

                if random_value < test.traffic_allocation.treatment_percentage / 100.0 {
                    Ok("treatment".to_string())
                } else {
                    Ok("control".to_string())
                }
            }
            RoutingStrategy::Hash => {
                let hash_input = user_id.unwrap_or(request_id);
                let hash = self.simple_hash(hash_input);
                let normalized = (hash % 100) as f64;

                if normalized < test.traffic_allocation.treatment_percentage {
                    Ok("treatment".to_string())
                } else {
                    Ok("control".to_string())
                }
            }
            _ => Ok("control".to_string()), // Simplified for other strategies
        }
    }

    fn simple_hash(&self, input: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        hasher.finish()
    }
}

// Utility functions for creating test configurations
pub fn create_gradual_ramp_schedule(steps: usize, max_treatment_percentage: f64) -> Vec<TrafficRampStep> {
    let mut schedule = Vec::new();
    let step_size = max_treatment_percentage / steps as f64;
    let time_step = 24.0 / steps as f64; // Spread over 24 hours

    for i in 0..steps {
        let treatment_percentage = step_size * (i + 1) as f64;
        let control_percentage = 100.0 - treatment_percentage;

        schedule.push(TrafficRampStep {
            time_offset_hours: time_step * i as f64,
            control_percentage,
            treatment_percentage,
        });
    }

    schedule
}

pub fn create_canary_config(traffic_percentage: f64, duration_minutes: u64) -> CanaryConfig {
    CanaryConfig {
        traffic_percentage,
        duration_minutes,
        success_threshold: 0.99,
        error_threshold: 0.01,
        auto_promote: false,
        auto_rollback: true,
        health_check_interval_ms: 30000,
        promotion_criteria: vec![
            PromotionCriterion {
                metric_name: "success_rate".to_string(),
                comparison: ComparisonOperator::GreaterThanOrEqual,
                threshold_value: 0.99,
                required_duration_minutes: 30,
            },
            PromotionCriterion {
                metric_name: "avg_latency".to_string(),
                comparison: ComparisonOperator::LessThan,
                threshold_value: 1000.0, // 1 second
                required_duration_minutes: 30,
            },
        ],
    }
}