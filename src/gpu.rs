use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    process::Command,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::{
    sync::RwLock,
    time::interval,
};
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub id: u32,
    pub name: String,
    pub vendor: GpuVendor,
    pub architecture: String,
    pub driver_version: String,
    pub cuda_version: Option<String>,
    pub memory_total_mb: u64,
    pub memory_free_mb: u64,
    pub memory_used_mb: u64,
    pub utilization_percent: f32,
    pub temperature_celsius: Option<f32>,
    pub power_usage_watts: Option<f32>,
    pub power_limit_watts: Option<f32>,
    pub clock_speed_mhz: Option<u32>,
    pub memory_clock_mhz: Option<u32>,
    pub compute_capability: Option<ComputeCapability>,
    pub supported_apis: Vec<GpuApi>,
    pub status: GpuStatus,
    pub last_updated: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Apple,
    Unknown(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeCapability {
    pub major: u32,
    pub minor: u32,
}

impl ComputeCapability {
    pub fn to_string(&self) -> String {
        format!("{}.{}", self.major, self.minor)
    }

    pub fn supports_feature(&self, required_major: u32, required_minor: u32) -> bool {
        self.major > required_major || (self.major == required_major && self.minor >= required_minor)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GpuApi {
    Cuda,
    OpenCL,
    Vulkan,
    DirectML,
    Metal,
    ROCm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GpuStatus {
    Available,
    InUse,
    Error(String),
    Overheated,
    LowMemory,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuConfiguration {
    pub enabled: bool,
    pub preferred_vendor: Option<GpuVendor>,
    pub memory_limit_mb: Option<u64>,
    pub max_utilization_percent: f32,
    pub temperature_limit_celsius: f32,
    pub power_limit_percent: Option<f32>,
    pub fallback_to_cpu: bool,
    pub auto_scaling: bool,
    pub monitoring_interval_seconds: u64,
}

impl Default for GpuConfiguration {
    fn default() -> Self {
        Self {
            enabled: true,
            preferred_vendor: None,
            memory_limit_mb: None,
            max_utilization_percent: 90.0,
            temperature_limit_celsius: 85.0,
            power_limit_percent: None,
            fallback_to_cpu: true,
            auto_scaling: false,
            monitoring_interval_seconds: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuAllocation {
    pub gpu_id: u32,
    pub allocated_memory_mb: u64,
    pub allocated_at: SystemTime,
    pub process_id: Option<u32>,
    pub model_name: String,
    pub estimated_duration: Option<Duration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuMetrics {
    pub gpu_id: u32,
    pub timestamp: SystemTime,
    pub memory_utilization_percent: f32,
    pub gpu_utilization_percent: f32,
    pub temperature_celsius: f32,
    pub power_usage_watts: f32,
    pub memory_throughput_gbps: Option<f32>,
    pub compute_throughput_tflops: Option<f32>,
}

pub struct GpuManager {
    config: GpuConfiguration,
    gpus: Arc<RwLock<HashMap<u32, GpuInfo>>>,
    allocations: Arc<RwLock<HashMap<u32, Vec<GpuAllocation>>>>,
    metrics_history: Arc<RwLock<Vec<GpuMetrics>>>,
    monitoring_active: Arc<std::sync::atomic::AtomicBool>,
}

impl GpuManager {
    pub fn new(config: GpuConfiguration) -> Self {
        Self {
            config,
            gpus: Arc::new(RwLock::new(HashMap::new())),
            allocations: Arc::new(RwLock::new(HashMap::new())),
            metrics_history: Arc::new(RwLock::new(Vec::new())),
            monitoring_active: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing GPU manager...");

        // Detect available GPUs
        self.detect_gpus().await?;

        // Start monitoring if enabled
        if self.config.enabled {
            self.start_monitoring().await?;
        }

        let gpu_count = self.gpus.read().await.len();
        info!("GPU manager initialized with {} GPUs", gpu_count);

        Ok(())
    }

    async fn detect_gpus(&self) -> Result<()> {
        let mut gpus = HashMap::new();

        // Try to detect NVIDIA GPUs
        if let Ok(nvidia_gpus) = self.detect_nvidia_gpus().await {
            for gpu in nvidia_gpus {
                gpus.insert(gpu.id, gpu);
            }
        }

        // Try to detect AMD GPUs
        if let Ok(amd_gpus) = self.detect_amd_gpus().await {
            for gpu in amd_gpus {
                gpus.insert(gpu.id, gpu);
            }
        }

        // Try to detect Intel GPUs
        if let Ok(intel_gpus) = self.detect_intel_gpus().await {
            for gpu in intel_gpus {
                gpus.insert(gpu.id, gpu);
            }
        }

        // Try to detect Apple Silicon GPUs
        if let Ok(apple_gpus) = self.detect_apple_gpus().await {
            for gpu in apple_gpus {
                gpus.insert(gpu.id, gpu);
            }
        }

        let mut gpu_store = self.gpus.write().await;
        *gpu_store = gpus;

        Ok(())
    }

    async fn detect_nvidia_gpus(&self) -> Result<Vec<GpuInfo>> {
        let mut gpus = Vec::new();

        // Try nvidia-smi command
        match Command::new("nvidia-smi")
            .args(&["--query-gpu=index,name,driver_version,memory.total,memory.free,memory.used,utilization.gpu,temperature.gpu,power.draw,power.limit,clocks.gr,clocks.mem"])
            .args(&["--format=csv,noheader,nounits"])
            .output()
        {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);

                for (i, line) in stdout.lines().enumerate() {
                    let fields: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

                    if fields.len() >= 12 {
                        let gpu = GpuInfo {
                            id: i as u32,
                            name: fields[1].to_string(),
                            vendor: GpuVendor::Nvidia,
                            architecture: "Unknown".to_string(), // Would need additional query
                            driver_version: fields[2].to_string(),
                            cuda_version: self.get_cuda_version().await,
                            memory_total_mb: fields[3].parse().unwrap_or(0),
                            memory_free_mb: fields[4].parse().unwrap_or(0),
                            memory_used_mb: fields[5].parse().unwrap_or(0),
                            utilization_percent: fields[6].parse().unwrap_or(0.0),
                            temperature_celsius: fields[7].parse().ok(),
                            power_usage_watts: fields[8].parse().ok(),
                            power_limit_watts: fields[9].parse().ok(),
                            clock_speed_mhz: fields[10].parse().ok(),
                            memory_clock_mhz: fields[11].parse().ok(),
                            compute_capability: self.get_compute_capability(i as u32).await,
                            supported_apis: vec![GpuApi::Cuda],
                            status: GpuStatus::Available,
                            last_updated: SystemTime::now(),
                        };
                        gpus.push(gpu);
                    }
                }
            }
            Ok(_) => {
                debug!("nvidia-smi command failed");
            }
            Err(e) => {
                debug!("nvidia-smi not found: {}", e);
            }
        }

        Ok(gpus)
    }

    async fn detect_amd_gpus(&self) -> Result<Vec<GpuInfo>> {
        let mut gpus = Vec::new();

        // Try rocm-smi command
        match Command::new("rocm-smi")
            .args(&["--showid", "--showproductname", "--showmeminfo", "--showuse", "--showtemp", "--showpower"])
            .output()
        {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                // Parse rocm-smi output (implementation would depend on exact format)
                debug!("AMD GPU detection: {}", stdout);
            }
            Ok(_) => {
                debug!("rocm-smi command failed");
            }
            Err(e) => {
                debug!("rocm-smi not found: {}", e);
            }
        }

        Ok(gpus)
    }

    async fn detect_intel_gpus(&self) -> Result<Vec<GpuInfo>> {
        let mut gpus = Vec::new();

        // Intel GPUs are typically detected through system info or OpenCL
        #[cfg(target_os = "linux")]
        {
            // Try to read from /sys/class/drm/
            if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with("card") && !name.contains('-') {
                            // This is a potential GPU card
                            if let Ok(vendor) = std::fs::read_to_string(path.join("device/vendor")) {
                                if vendor.trim() == "0x8086" { // Intel vendor ID
                                    // Detected Intel GPU
                                    let gpu = GpuInfo {
                                        id: gpus.len() as u32,
                                        name: "Intel GPU".to_string(),
                                        vendor: GpuVendor::Intel,
                                        architecture: "Unknown".to_string(),
                                        driver_version: "Unknown".to_string(),
                                        cuda_version: None,
                                        memory_total_mb: 0, // Would need more investigation
                                        memory_free_mb: 0,
                                        memory_used_mb: 0,
                                        utilization_percent: 0.0,
                                        temperature_celsius: None,
                                        power_usage_watts: None,
                                        power_limit_watts: None,
                                        clock_speed_mhz: None,
                                        memory_clock_mhz: None,
                                        compute_capability: None,
                                        supported_apis: vec![GpuApi::OpenCL],
                                        status: GpuStatus::Available,
                                        last_updated: SystemTime::now(),
                                    };
                                    gpus.push(gpu);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(gpus)
    }

    async fn detect_apple_gpus(&self) -> Result<Vec<GpuInfo>> {
        let mut gpus = Vec::new();

        #[cfg(target_os = "macos")]
        {
            // Try system_profiler for Apple Silicon detection
            match Command::new("system_profiler")
                .args(&["SPDisplaysDataType", "-json"])
                .output()
            {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                        if let Some(displays) = json["SPDisplaysDataType"].as_array() {
                            for (i, display) in displays.iter().enumerate() {
                                if let Some(gpu_name) = display["sppci_model"].as_str() {
                                    if gpu_name.contains("Apple") {
                                        let gpu = GpuInfo {
                                            id: i as u32,
                                            name: gpu_name.to_string(),
                                            vendor: GpuVendor::Apple,
                                            architecture: "Apple Silicon".to_string(),
                                            driver_version: "System".to_string(),
                                            cuda_version: None,
                                            memory_total_mb: 0, // Unified memory
                                            memory_free_mb: 0,
                                            memory_used_mb: 0,
                                            utilization_percent: 0.0,
                                            temperature_celsius: None,
                                            power_usage_watts: None,
                                            power_limit_watts: None,
                                            clock_speed_mhz: None,
                                            memory_clock_mhz: None,
                                            compute_capability: None,
                                            supported_apis: vec![GpuApi::Metal],
                                            status: GpuStatus::Available,
                                            last_updated: SystemTime::now(),
                                        };
                                        gpus.push(gpu);
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {
                    debug!("Could not detect Apple GPUs via system_profiler");
                }
            }
        }

        Ok(gpus)
    }

    async fn get_cuda_version(&self) -> Option<String> {
        match Command::new("nvcc").args(&["--version"]).output() {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                // Parse CUDA version from nvcc output
                for line in stdout.lines() {
                    if line.contains("release") {
                        if let Some(version_part) = line.split("release ").nth(1) {
                            if let Some(version) = version_part.split(',').next() {
                                return Some(version.trim().to_string());
                            }
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    async fn get_compute_capability(&self, gpu_id: u32) -> Option<ComputeCapability> {
        // This would require more sophisticated detection
        // For now, return a default for common GPUs
        Some(ComputeCapability { major: 7, minor: 5 })
    }

    async fn start_monitoring(&self) -> Result<()> {
        if self.monitoring_active.load(std::sync::atomic::Ordering::SeqCst) {
            return Ok(());
        }

        self.monitoring_active.store(true, std::sync::atomic::Ordering::SeqCst);

        let gpus = self.gpus.clone();
        let metrics_history = self.metrics_history.clone();
        let monitoring_active = self.monitoring_active.clone();
        let interval_duration = Duration::from_secs(self.config.monitoring_interval_seconds);

        tokio::spawn(async move {
            let mut interval_timer = interval(interval_duration);

            while monitoring_active.load(std::sync::atomic::Ordering::SeqCst) {
                interval_timer.tick().await;

                let gpu_store = gpus.read().await;
                for gpu_info in gpu_store.values() {
                    // Update GPU metrics
                    if let Ok(metrics) = Self::collect_gpu_metrics(gpu_info.id).await {
                        let mut history = metrics_history.write().await;
                        history.push(metrics);

                        // Keep only recent metrics (last hour)
                        let cutoff = SystemTime::now() - Duration::from_secs(3600);
                        history.retain(|m| m.timestamp > cutoff);
                    }
                }
            }

            info!("GPU monitoring stopped");
        });

        info!("GPU monitoring started");
        Ok(())
    }

    async fn collect_gpu_metrics(gpu_id: u32) -> Result<GpuMetrics> {
        // This would collect real-time metrics from the GPU
        // For now, return mock data
        Ok(GpuMetrics {
            gpu_id,
            timestamp: SystemTime::now(),
            memory_utilization_percent: 0.0,
            gpu_utilization_percent: 0.0,
            temperature_celsius: 65.0,
            power_usage_watts: 150.0,
            memory_throughput_gbps: Some(500.0),
            compute_throughput_tflops: Some(10.0),
        })
    }

    pub async fn get_available_gpus(&self) -> Vec<GpuInfo> {
        let gpus = self.gpus.read().await;
        gpus.values()
            .filter(|gpu| matches!(gpu.status, GpuStatus::Available))
            .cloned()
            .collect()
    }

    pub async fn get_gpu_info(&self, gpu_id: u32) -> Option<GpuInfo> {
        let gpus = self.gpus.read().await;
        gpus.get(&gpu_id).cloned()
    }

    pub async fn allocate_gpu(
        &self,
        memory_required_mb: u64,
        model_name: String,
        preferred_vendor: Option<GpuVendor>,
    ) -> Result<Option<u32>> {
        let gpus = self.gpus.read().await;
        let allocations = self.allocations.read().await;

        // Find the best available GPU
        let mut best_gpu = None;
        let mut best_score = f32::MIN;

        for gpu in gpus.values() {
            // Check if GPU is available
            if !matches!(gpu.status, GpuStatus::Available) {
                continue;
            }

            // Check memory requirement
            if gpu.memory_free_mb < memory_required_mb {
                continue;
            }

            // Check vendor preference
            if let Some(ref preferred) = preferred_vendor {
                if std::mem::discriminant(&gpu.vendor) != std::mem::discriminant(preferred) {
                    continue;
                }
            }

            // Calculate score based on available memory and utilization
            let memory_score = gpu.memory_free_mb as f32 / gpu.memory_total_mb as f32;
            let utilization_score = 1.0 - (gpu.utilization_percent / 100.0);
            let score = memory_score * 0.6 + utilization_score * 0.4;

            if score > best_score {
                best_score = score;
                best_gpu = Some(gpu.id);
            }
        }

        if let Some(gpu_id) = best_gpu {
            // Create allocation
            let allocation = GpuAllocation {
                gpu_id,
                allocated_memory_mb: memory_required_mb,
                allocated_at: SystemTime::now(),
                process_id: None, // Would be set by the actual process
                model_name,
                estimated_duration: None,
            };

            drop(gpus);
            drop(allocations);

            let mut allocations = self.allocations.write().await;
            allocations.entry(gpu_id).or_insert_with(Vec::new).push(allocation);

            info!("Allocated GPU {} with {}MB memory", gpu_id, memory_required_mb);
            Ok(Some(gpu_id))
        } else {
            warn!("No suitable GPU found for allocation ({}MB required)", memory_required_mb);
            Ok(None)
        }
    }

    pub async fn deallocate_gpu(&self, gpu_id: u32, model_name: &str) -> Result<()> {
        let mut allocations = self.allocations.write().await;

        if let Some(gpu_allocations) = allocations.get_mut(&gpu_id) {
            gpu_allocations.retain(|alloc| alloc.model_name != model_name);

            if gpu_allocations.is_empty() {
                allocations.remove(&gpu_id);
            }

            info!("Deallocated GPU {} for model {}", gpu_id, model_name);
        }

        Ok(())
    }

    pub async fn get_gpu_metrics(&self, gpu_id: Option<u32>) -> Vec<GpuMetrics> {
        let metrics = self.metrics_history.read().await;

        if let Some(id) = gpu_id {
            metrics.iter()
                .filter(|m| m.gpu_id == id)
                .cloned()
                .collect()
        } else {
            metrics.clone()
        }
    }

    pub async fn get_gpu_allocations(&self) -> HashMap<u32, Vec<GpuAllocation>> {
        let allocations = self.allocations.read().await;
        allocations.clone()
    }

    pub async fn check_gpu_health(&self) -> Result<HashMap<u32, GpuStatus>> {
        let mut health_status = HashMap::new();
        let gpus = self.gpus.read().await;

        for gpu in gpus.values() {
            let status = if let Some(temp) = gpu.temperature_celsius {
                if temp > self.config.temperature_limit_celsius {
                    GpuStatus::Overheated
                } else if gpu.memory_free_mb < (gpu.memory_total_mb * 10 / 100) { // Less than 10% free
                    GpuStatus::LowMemory
                } else if gpu.utilization_percent > self.config.max_utilization_percent {
                    GpuStatus::InUse
                } else {
                    GpuStatus::Available
                }
            } else {
                gpu.status.clone()
            };

            health_status.insert(gpu.id, status);
        }

        Ok(health_status)
    }

    pub async fn shutdown(&self) {
        self.monitoring_active.store(false, std::sync::atomic::Ordering::SeqCst);
        info!("GPU manager shutdown");
    }

    pub async fn refresh_gpu_info(&self) -> Result<()> {
        self.detect_gpus().await
    }

    pub fn get_configuration(&self) -> &GpuConfiguration {
        &self.config
    }

    pub async fn update_configuration(&mut self, new_config: GpuConfiguration) -> Result<()> {
        let restart_monitoring = self.config.monitoring_interval_seconds != new_config.monitoring_interval_seconds;

        self.config = new_config;

        if restart_monitoring && self.monitoring_active.load(std::sync::atomic::Ordering::SeqCst) {
            self.monitoring_active.store(false, std::sync::atomic::Ordering::SeqCst);
            tokio::time::sleep(Duration::from_millis(100)).await;
            self.start_monitoring().await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_capability() {
        let cc = ComputeCapability { major: 7, minor: 5 };
        assert_eq!(cc.to_string(), "7.5");
        assert!(cc.supports_feature(7, 0));
        assert!(cc.supports_feature(7, 5));
        assert!(!cc.supports_feature(8, 0));
    }

    #[tokio::test]
    async fn test_gpu_manager_creation() {
        let config = GpuConfiguration::default();
        let manager = GpuManager::new(config);
        assert!(!manager.monitoring_active.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_gpu_allocation() {
        let config = GpuConfiguration::default();
        let manager = GpuManager::new(config);

        // This would work with actual GPUs
        let result = manager.allocate_gpu(1024, "test-model".to_string(), None).await;
        assert!(result.is_ok());
    }
}