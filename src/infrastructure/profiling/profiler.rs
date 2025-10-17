//! Per-Operation Performance Profiler
//!
//! Tracks timing and resource usage for each inference operation phase:
//! - Tokenization: converting prompt to token IDs
//! - Inference: token generation
//! - Detokenization: converting tokens back to text

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Individual operation profile for a phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationProfile {
    /// Phase name: "tokenization", "inference", "detokenization"
    pub phase: String,
    /// Duration in milliseconds
    pub duration_ms: f32,
    /// GPU memory used in MB (if applicable)
    pub gpu_memory_used_mb: Option<f32>,
    /// CPU memory used in MB
    pub cpu_memory_used_mb: f32,
    /// GPU utilization percentage (0-100)
    pub gpu_utilization_percent: Option<f32>,
}

impl OperationProfile {
    /// Create a new operation profile
    pub fn new(phase: String, duration: Duration) -> Self {
        Self {
            phase,
            duration_ms: duration.as_secs_f32() * 1000.0,
            gpu_memory_used_mb: None,
            cpu_memory_used_mb: 0.0,
            gpu_utilization_percent: None,
        }
    }

    /// Set GPU memory usage
    pub fn with_gpu_memory(mut self, mb: f32) -> Self {
        self.gpu_memory_used_mb = Some(mb);
        self
    }

    /// Set CPU memory usage
    pub fn with_cpu_memory(mut self, mb: f32) -> Self {
        self.cpu_memory_used_mb = mb;
        self
    }

    /// Set GPU utilization
    pub fn with_gpu_utilization(mut self, percent: f32) -> Self {
        self.gpu_utilization_percent = Some(percent.max(0.0).min(100.0));
        self
    }
}

/// Complete inference profile with all phases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceProfile {
    /// Unique request identifier
    pub request_id: String,
    /// Model identifier
    pub model_id: String,
    /// Input token count
    pub input_tokens: u32,
    /// Output token count
    pub output_tokens: u32,
    /// Total inference time in milliseconds
    pub total_time_ms: f32,
    /// Individual phase profiles
    pub phases: Vec<OperationProfile>,
    /// Unix timestamp in milliseconds
    pub timestamp: u64,
}

impl InferenceProfile {
    /// Create a new inference profile
    pub fn new(request_id: String, model_id: String, input_tokens: u32, output_tokens: u32) -> Self {
        Self {
            request_id,
            model_id,
            input_tokens,
            output_tokens,
            total_time_ms: 0.0,
            phases: Vec::new(),
            timestamp: Self::current_timestamp(),
        }
    }

    /// Add a phase profile
    pub fn add_phase(&mut self, phase: OperationProfile) {
        self.phases.push(phase);
    }

    /// Set total inference time
    pub fn set_total_time(&mut self, duration: Duration) {
        self.total_time_ms = duration.as_secs_f32() * 1000.0;
    }

    /// Calculate throughput in tokens per second
    pub fn throughput_tokens_per_sec(&self) -> f32 {
        if self.total_time_ms > 0.0 {
            self.output_tokens as f32 / (self.total_time_ms / 1000.0)
        } else {
            0.0
        }
    }

    /// Get current Unix timestamp in milliseconds
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

/// Phase timer for measuring operation duration
pub struct PhaseTimer {
    phase_name: String,
    start: Instant,
}

impl PhaseTimer {
    /// Create a new phase timer
    pub fn new(phase_name: String) -> Self {
        Self {
            phase_name,
            start: Instant::now(),
        }
    }

    /// Complete timing and return profile
    pub fn finish(self) -> OperationProfile {
        let duration = self.start.elapsed();
        OperationProfile::new(self.phase_name, duration)
    }
}

/// Thread-safe profile collector
pub struct ProfileCollector {
    profiles: Arc<Mutex<Vec<InferenceProfile>>>,
    max_profiles: usize,
}

impl ProfileCollector {
    /// Create a new profile collector
    pub fn new(max_profiles: usize) -> Self {
        Self {
            profiles: Arc::new(Mutex::new(Vec::with_capacity(max_profiles))),
            max_profiles,
        }
    }

    /// Record an inference profile
    pub fn record_profile(&self, profile: InferenceProfile) -> anyhow::Result<()> {
        let mut profiles = self.profiles.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;

        profiles.push(profile);

        // Keep only most recent profiles
        if profiles.len() > self.max_profiles {
            profiles.remove(0);
        }

        Ok(())
    }

    /// Get recent profiles
    pub fn get_recent(&self, count: usize) -> anyhow::Result<Vec<InferenceProfile>> {
        let profiles = self.profiles.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;

        let start_idx = profiles.len().saturating_sub(count);
        Ok(profiles[start_idx..].to_vec())
    }

    /// Get all profiles
    pub fn get_all(&self) -> anyhow::Result<Vec<InferenceProfile>> {
        let profiles = self.profiles.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        Ok(profiles.clone())
    }

    /// Clear all profiles
    pub fn clear(&self) -> anyhow::Result<()> {
        let mut profiles = self.profiles.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        profiles.clear();
        Ok(())
    }

    /// Get profile count
    pub fn len(&self) -> anyhow::Result<usize> {
        let profiles = self.profiles.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        Ok(profiles.len())
    }

    /// Get average metrics across recent profiles
    pub fn get_average_metrics(&self, count: usize) -> anyhow::Result<AverageMetrics> {
        let recent = self.get_recent(count)?;

        if recent.is_empty() {
            return Ok(AverageMetrics::default());
        }

        let avg_total_time = recent.iter().map(|p| p.total_time_ms).sum::<f32>() / recent.len() as f32;
        let avg_input_tokens = recent.iter().map(|p| p.input_tokens as f32).sum::<f32>() / recent.len() as f32;
        let avg_output_tokens = recent.iter().map(|p| p.output_tokens as f32).sum::<f32>() / recent.len() as f32;
        let avg_throughput = recent.iter().map(|p| p.throughput_tokens_per_sec()).sum::<f32>() / recent.len() as f32;

        Ok(AverageMetrics {
            avg_total_time_ms: avg_total_time,
            avg_input_tokens,
            avg_output_tokens,
            avg_throughput_tokens_per_sec: avg_throughput,
        })
    }
}

impl Clone for ProfileCollector {
    fn clone(&self) -> Self {
        Self {
            profiles: Arc::clone(&self.profiles),
            max_profiles: self.max_profiles,
        }
    }
}

/// Average metrics across multiple profiles
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AverageMetrics {
    pub avg_total_time_ms: f32,
    pub avg_input_tokens: f32,
    pub avg_output_tokens: f32,
    pub avg_throughput_tokens_per_sec: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_profile() {
        let duration = Duration::from_millis(100);
        let profile = OperationProfile::new("tokenization".to_string(), duration)
            .with_cpu_memory(256.0)
            .with_gpu_memory(1024.0)
            .with_gpu_utilization(75.5);

        assert_eq!(profile.phase, "tokenization");
        assert!((profile.duration_ms - 100.0).abs() < 1.0);
        assert_eq!(profile.cpu_memory_used_mb, 256.0);
        assert_eq!(profile.gpu_memory_used_mb, Some(1024.0));
        assert_eq!(profile.gpu_utilization_percent, Some(75.5));
    }

    #[test]
    fn test_inference_profile() {
        let mut profile = InferenceProfile::new(
            "req_123".to_string(),
            "llama-2-7b".to_string(),
            256,
            128,
        );

        let tokenize = OperationProfile::new("tokenization".to_string(), Duration::from_millis(10));
        let inference = OperationProfile::new("inference".to_string(), Duration::from_millis(800));
        let detokenize = OperationProfile::new("detokenization".to_string(), Duration::from_millis(5));

        profile.add_phase(tokenize);
        profile.add_phase(inference);
        profile.add_phase(detokenize);

        profile.set_total_time(Duration::from_millis(815));

        assert_eq!(profile.phases.len(), 3);
        assert!((profile.total_time_ms - 815.0).abs() < 1.0);

        let throughput = profile.throughput_tokens_per_sec();
        assert!(throughput > 0.0); // 128 tokens in 0.815 seconds
    }

    #[test]
    fn test_phase_timer() {
        let timer = PhaseTimer::new("inference".to_string());
        std::thread::sleep(Duration::from_millis(10));
        let profile = timer.finish();

        assert_eq!(profile.phase, "inference");
        assert!(profile.duration_ms >= 10.0);
    }

    #[test]
    fn test_profile_collector() {
        let collector = ProfileCollector::new(100);

        let mut profile1 = InferenceProfile::new(
            "req_1".to_string(),
            "model1".to_string(),
            100,
            50,
        );
        profile1.set_total_time(Duration::from_millis(500));

        let mut profile2 = InferenceProfile::new(
            "req_2".to_string(),
            "model1".to_string(),
            200,
            100,
        );
        profile2.set_total_time(Duration::from_millis(1000));

        collector.record_profile(profile1).unwrap();
        collector.record_profile(profile2).unwrap();

        assert_eq!(collector.len().unwrap(), 2);

        let recent = collector.get_recent(2).unwrap();
        assert_eq!(recent.len(), 2);

        let avg = collector.get_average_metrics(2).unwrap();
        assert!((avg.avg_total_time_ms - 750.0).abs() < 1.0); // (500 + 1000) / 2
    }

    #[test]
    fn test_profile_collector_max_size() {
        let collector = ProfileCollector::new(5);

        for i in 0..10 {
            let mut profile = InferenceProfile::new(
                format!("req_{}", i),
                "model".to_string(),
                100,
                50,
            );
            profile.set_total_time(Duration::from_millis(500));
            collector.record_profile(profile).unwrap();
        }

        // Should only keep last 5
        assert_eq!(collector.len().unwrap(), 5);
    }

    #[test]
    fn test_throughput_calculation() {
        let mut profile = InferenceProfile::new(
            "req_123".to_string(),
            "model".to_string(),
            100,
            200, // Generated 200 tokens
        );

        profile.set_total_time(Duration::from_secs(2)); // In 2 seconds

        let throughput = profile.throughput_tokens_per_sec();
        assert!((throughput - 100.0).abs() < 0.1); // 200 / 2 = 100
    }
}
