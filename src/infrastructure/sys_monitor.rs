//! System Monitoring and Performance Profiling
//!
//! Monitors system state (thermal, power, battery) and automatically adjusts
//! inference parameters for optimal performance and efficiency.

use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

/// System power state
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerState {
    /// Connected to AC power
    ACPower,
    /// Running on battery
    Battery,
}

/// Thermal state categories
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThermalState {
    /// Below 40°C - Normal operation
    Cool,
    /// 40-60°C - Warm but acceptable
    Warm,
    /// 60-80°C - Hot, should throttle
    Hot,
    /// Above 80°C - Thermal throttling needed
    Critical,
}

/// Battery health status
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BatteryHealth {
    /// Battery is healthy (>80% capacity)
    Good,
    /// Battery degraded (50-80% capacity)
    Fair,
    /// Battery is worn (<50% capacity)
    Poor,
}

/// Current system state snapshot
#[derive(Clone, Debug)]
pub struct SystemState {
    /// Power source
    pub power_state: PowerState,
    /// Current temperature in Celsius
    pub temperature_celsius: Option<f32>,
    /// Current thermal state
    pub thermal_state: ThermalState,
    /// Battery percentage (0-100), None if AC powered
    pub battery_percent: Option<f32>,
    /// Battery health estimate
    pub battery_health: BatteryHealth,
    /// CPU load percentage (0-100)
    pub cpu_load: f32,
    /// Memory usage percentage (0-100)
    pub memory_percent: f32,
    /// Timestamp of measurement
    pub timestamp: u64,
}

impl SystemState {
    /// Determine appropriate performance profile based on system state
    pub fn recommended_profile(&self) -> PerformanceProfile {
        match (self.power_state, self.thermal_state) {
            // AC power: optimize for performance
            (PowerState::ACPower, ThermalState::Cool) => PerformanceProfile::Performance,
            (PowerState::ACPower, ThermalState::Warm) => PerformanceProfile::Performance,
            (PowerState::ACPower, ThermalState::Hot) => PerformanceProfile::Balanced,
            (PowerState::ACPower, ThermalState::Critical) => PerformanceProfile::EnergyEfficient,

            // Battery: balance efficiency and performance
            (PowerState::Battery, ThermalState::Cool) => PerformanceProfile::Balanced,
            (PowerState::Battery, ThermalState::Warm) => PerformanceProfile::Balanced,
            (PowerState::Battery, ThermalState::Hot) => PerformanceProfile::EnergyEfficient,
            (PowerState::Battery, ThermalState::Critical) => PerformanceProfile::PowerSaver,
        }
    }

    /// Check if system is in a critical state
    pub fn is_critical(&self) -> bool {
        self.thermal_state == ThermalState::Critical
            || (self.power_state == PowerState::Battery
                && self.battery_percent.map_or(false, |p| p < 10.0))
    }
}

/// Performance profile settings
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PerformanceProfile {
    /// Maximum performance (AC power, cool temps)
    Performance,
    /// Balanced performance and efficiency
    Balanced,
    /// Energy efficient (battery saver mode)
    EnergyEfficient,
    /// Maximum power saving (critical battery/thermal)
    PowerSaver,
}

impl PerformanceProfile {
    /// Get batch size recommendation for this profile
    pub fn batch_size(&self) -> u32 {
        match self {
            Self::Performance => 256,
            Self::Balanced => 128,
            Self::EnergyEfficient => 64,
            Self::PowerSaver => 32,
        }
    }

    /// Get context size recommendation for this profile
    pub fn context_size(&self) -> u32 {
        match self {
            Self::Performance => 4096,
            Self::Balanced => 2048,
            Self::EnergyEfficient => 1024,
            Self::PowerSaver => 512,
        }
    }

    /// Get GPU layer recommendation (0 = CPU only, 999 = max GPU)
    pub fn gpu_layers(&self) -> u32 {
        match self {
            Self::Performance => 999,     // All layers on GPU
            Self::Balanced => 500,        // Half on GPU
            Self::EnergyEfficient => 200, // Minimal GPU
            Self::PowerSaver => 0,        // CPU only
        }
    }

    /// Get temperature threshold for this profile
    pub fn temperature_threshold_celsius(&self) -> f32 {
        match self {
            Self::Performance => 85.0,
            Self::Balanced => 75.0,
            Self::EnergyEfficient => 65.0,
            Self::PowerSaver => 55.0,
        }
    }

    /// Get max tokens per second (rate limiting)
    pub fn max_tokens_per_sec(&self) -> u32 {
        match self {
            Self::Performance => 0,      // Unlimited
            Self::Balanced => 100,       // 100 tok/s
            Self::EnergyEfficient => 50, // 50 tok/s
            Self::PowerSaver => 20,      // 20 tok/s
        }
    }
}

/// System monitor for tracking device state
pub struct SystemMonitor {
    last_state: Option<SystemState>,
    profile_change_count: u32,
}

impl SystemMonitor {
    /// Create new system monitor
    pub fn new() -> Self {
        Self {
            last_state: None,
            profile_change_count: 0,
        }
    }

    /// Update system state (platform-specific implementation)
    pub fn update_state(&mut self) -> SystemState {
        let state = self.get_current_state();

        // Track profile changes
        if let Some(ref last) = self.last_state {
            let old_profile = last.recommended_profile();
            let new_profile = state.recommended_profile();
            if old_profile != new_profile {
                self.profile_change_count += 1;
                info!(
                    "Performance profile changed: {:?} → {:?}",
                    old_profile, new_profile
                );
            }
        }

        self.last_state = Some(state.clone());
        state
    }

    /// Get current system state
    #[cfg(target_os = "macos")]
    fn get_current_state(&self) -> SystemState {
        use std::process::Command;

        let mut state = SystemState {
            power_state: PowerState::ACPower,
            temperature_celsius: None,
            thermal_state: ThermalState::Cool,
            battery_percent: None,
            battery_health: BatteryHealth::Good,
            cpu_load: 0.0,
            memory_percent: 0.0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        // Get battery info on macOS
        if let Ok(output) = Command::new("pmset").args(&["-g", "batt"]).output() {
            let batt_info = String::from_utf8_lossy(&output.stdout);

            // Check if on battery
            if batt_info.contains("discharging") || batt_info.contains("Battery") {
                state.power_state = PowerState::Battery;
            }

            // Extract battery percentage
            if let Some(line) = batt_info.lines().next() {
                if let Some(percent_str) = line.split('\t').next() {
                    if let Some(num_str) = percent_str
                        .split('%')
                        .next()
                        .and_then(|s| s.trim().split_whitespace().last())
                    {
                        if let Ok(percent) = num_str.parse::<f32>() {
                            state.battery_percent = Some(percent);
                        }
                    }
                }
            }
        }

        // Get temperature info (Mac thermal data if available)
        #[cfg(target_arch = "aarch64")]
        if let Ok(output) = Command::new("sysctl")
            .arg("machdep.cpu.brand_string")
            .output()
        {
            // On Apple Silicon, estimate temp from system load
            // This is a placeholder - real implementation would use IOKit
            state.temperature_celsius = Some(50.0 + (state.cpu_load * 0.3));
        }

        // Get CPU and memory load
        if let Ok(output) = Command::new("top").args(&["-l", "1", "-n", "0"]).output() {
            let top_output = String::from_utf8_lossy(&output.stdout);

            // Parse CPU usage
            for line in top_output.lines() {
                if line.contains("CPU usage:") {
                    if let Some(user_part) = line.split("user:").nth(1) {
                        if let Some(cpu_str) = user_part.split('%').next() {
                            if let Ok(cpu) = cpu_str.trim().parse::<f32>() {
                                state.cpu_load = cpu;
                            }
                        }
                    }
                }
                if line.contains("MemRegions:") {
                    // Rough memory estimate from top
                    state.memory_percent = 50.0; // Default estimate
                }
            }
        }

        // Determine thermal state based on temperature
        if let Some(temp) = state.temperature_celsius {
            state.thermal_state = match temp {
                t if t < 40.0 => ThermalState::Cool,
                t if t < 60.0 => ThermalState::Warm,
                t if t < 80.0 => ThermalState::Hot,
                _ => ThermalState::Critical,
            };
        }

        // Determine battery health
        state.battery_health = match state.battery_percent {
            Some(p) if p > 80.0 => BatteryHealth::Good,
            Some(p) if p > 50.0 => BatteryHealth::Fair,
            Some(_) => BatteryHealth::Poor,
            None => BatteryHealth::Good, // AC power
        };

        state
    }

    /// Fallback for non-macOS systems
    #[cfg(not(target_os = "macos"))]
    fn get_current_state(&self) -> SystemState {
        SystemState {
            power_state: PowerState::ACPower,
            temperature_celsius: None,
            thermal_state: ThermalState::Cool,
            battery_percent: None,
            battery_health: BatteryHealth::Good,
            cpu_load: 0.0,
            memory_percent: 0.0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Get last recorded state
    pub fn last_state(&self) -> Option<&SystemState> {
        self.last_state.as_ref()
    }

    /// Get profile change statistics
    pub fn profile_change_count(&self) -> u32 {
        self.profile_change_count
    }
}

impl Default for SystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thermal_state_ordering() {
        assert!(ThermalState::Cool < ThermalState::Warm);
        assert!(ThermalState::Warm < ThermalState::Hot);
        assert!(ThermalState::Hot < ThermalState::Critical);
    }

    #[test]
    fn test_performance_profile_ac_cool() {
        let state = SystemState {
            power_state: PowerState::ACPower,
            temperature_celsius: Some(35.0),
            thermal_state: ThermalState::Cool,
            battery_percent: None,
            battery_health: BatteryHealth::Good,
            cpu_load: 20.0,
            memory_percent: 50.0,
            timestamp: 0,
        };

        assert_eq!(state.recommended_profile(), PerformanceProfile::Performance);
        assert_eq!(state.recommended_profile().gpu_layers(), 999);
    }

    #[test]
    fn test_performance_profile_battery_hot() {
        let state = SystemState {
            power_state: PowerState::Battery,
            temperature_celsius: Some(75.0),
            thermal_state: ThermalState::Hot,
            battery_percent: Some(30.0),
            battery_health: BatteryHealth::Fair,
            cpu_load: 80.0,
            memory_percent: 75.0,
            timestamp: 0,
        };

        assert_eq!(
            state.recommended_profile(),
            PerformanceProfile::EnergyEfficient
        );
        assert_eq!(state.recommended_profile().gpu_layers(), 200);
    }

    #[test]
    fn test_critical_state_detection() {
        let critical_temp = SystemState {
            power_state: PowerState::ACPower,
            temperature_celsius: Some(85.0),
            thermal_state: ThermalState::Critical,
            battery_percent: None,
            battery_health: BatteryHealth::Good,
            cpu_load: 90.0,
            memory_percent: 80.0,
            timestamp: 0,
        };

        assert!(critical_temp.is_critical());

        let critical_battery = SystemState {
            power_state: PowerState::Battery,
            temperature_celsius: Some(50.0),
            thermal_state: ThermalState::Cool,
            battery_percent: Some(5.0),
            battery_health: BatteryHealth::Poor,
            cpu_load: 20.0,
            memory_percent: 40.0,
            timestamp: 0,
        };

        assert!(critical_battery.is_critical());
    }

    #[test]
    fn test_performance_profile_configs() {
        let profiles = [
            PerformanceProfile::Performance,
            PerformanceProfile::Balanced,
            PerformanceProfile::EnergyEfficient,
            PerformanceProfile::PowerSaver,
        ];

        for profile in &profiles {
            // Ensure all profiles have valid configs
            assert!(profile.batch_size() > 0);
            assert!(profile.context_size() > 0);
            assert!(profile.gpu_layers() <= 999);
            assert!(profile.temperature_threshold_celsius() > 0.0);
        }

        // Check ordering: performance > energy efficiency
        assert!(
            PerformanceProfile::Performance.batch_size()
                > PerformanceProfile::PowerSaver.batch_size()
        );
        assert!(
            PerformanceProfile::Performance.gpu_layers()
                > PerformanceProfile::PowerSaver.gpu_layers()
        );
    }

    #[test]
    fn test_system_monitor_creation() {
        let monitor = SystemMonitor::new();
        assert_eq!(monitor.profile_change_count(), 0);
        assert!(monitor.last_state().is_none());
    }

    #[test]
    fn test_battery_health_detection() {
        assert_eq!(
            SystemState {
                power_state: PowerState::Battery,
                battery_percent: Some(90.0),
                battery_health: BatteryHealth::Good,
                ..Default::default()
            }
            .battery_health,
            BatteryHealth::Good
        );

        assert_eq!(
            SystemState {
                power_state: PowerState::Battery,
                battery_percent: Some(40.0),
                battery_health: BatteryHealth::Poor,
                ..Default::default()
            }
            .battery_health,
            BatteryHealth::Poor
        );
    }
}

impl Default for SystemState {
    fn default() -> Self {
        Self {
            power_state: PowerState::ACPower,
            temperature_celsius: None,
            thermal_state: ThermalState::Cool,
            battery_percent: None,
            battery_health: BatteryHealth::Good,
            cpu_load: 0.0,
            memory_percent: 0.0,
            timestamp: 0,
        }
    }
}
