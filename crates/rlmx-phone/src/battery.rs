use serde::{Deserialize, Serialize};

/// GPU type available on the mobile device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GpuType {
    WebGPU,
    Metal,
    None,
}

/// Network connectivity type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkType {
    WiFi,
    Cellular,
    Offline,
}

/// Device thermal state (normalised across iOS/Android).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ThermalState {
    Nominal,
    Fair,
    Serious,
    Critical,
}

/// Mobile operating system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MobileOS {
    #[serde(rename = "iOS")]
    IOS,
    Android,
}

/// OS-level background execution policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackgroundPolicy {
    Unrestricted,
    Adaptive,
    Restricted,
    Suspended,
}

/// Battery-derived execution policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatteryPolicy {
    /// 80-100 %: full inference, all agents.
    Full,
    /// 30-79 %: balanced, reduced concurrency.
    Balanced,
    /// 10-29 %: low-power, essential agents only.
    LowPower,
    /// <10 %: critical, suspend non-essential agents.
    Critical,
}

/// Runtime-detected device capabilities (value object).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    pub gpu: GpuType,
    pub memory_mb: u32,
    pub battery_pct: f32,
    pub network: NetworkType,
    pub thermal_state: ThermalState,
    pub os: MobileOS,
    pub background_policy: BackgroundPolicy,
}

impl DeviceCapabilities {
    /// Create capabilities with sensible defaults.
    pub fn default_ios() -> Self {
        Self {
            gpu: GpuType::Metal,
            memory_mb: 6144,
            battery_pct: 1.0,
            network: NetworkType::WiFi,
            thermal_state: ThermalState::Nominal,
            os: MobileOS::IOS,
            background_policy: BackgroundPolicy::Adaptive,
        }
    }

    /// Create capabilities for Android device.
    pub fn default_android() -> Self {
        Self {
            gpu: GpuType::WebGPU,
            memory_mb: 8192,
            battery_pct: 1.0,
            network: NetworkType::WiFi,
            thermal_state: ThermalState::Nominal,
            os: MobileOS::Android,
            background_policy: BackgroundPolicy::Unrestricted,
        }
    }
}

/// Scheduler that adjusts agent execution based on battery and thermal state.
#[derive(Debug)]
pub struct BatteryAwareScheduler {
    pub policy: BatteryPolicy,
    pub capabilities: DeviceCapabilities,
}

impl BatteryAwareScheduler {
    pub fn new(capabilities: DeviceCapabilities) -> Self {
        let policy = Self::derive_policy(capabilities.battery_pct, capabilities.thermal_state);
        Self {
            policy,
            capabilities,
        }
    }

    /// Derive battery policy from percentage and thermal state.
    pub fn derive_policy(battery_pct: f32, thermal_state: ThermalState) -> BatteryPolicy {
        // Thermal override: serious/critical forces low-power or critical.
        if thermal_state >= ThermalState::Critical {
            return BatteryPolicy::Critical;
        }
        if thermal_state >= ThermalState::Serious {
            if battery_pct < 0.30 {
                return BatteryPolicy::Critical;
            }
            return BatteryPolicy::LowPower;
        }

        if battery_pct < 0.10 {
            BatteryPolicy::Critical
        } else if battery_pct < 0.30 {
            BatteryPolicy::LowPower
        } else if battery_pct < 0.80 {
            BatteryPolicy::Balanced
        } else {
            BatteryPolicy::Full
        }
    }

    /// Update capabilities and recompute the battery policy.
    /// Returns the new policy (and whether it changed).
    pub fn update(
        &mut self,
        battery_pct: f32,
        thermal_state: ThermalState,
    ) -> (BatteryPolicy, bool) {
        self.capabilities.battery_pct = battery_pct;
        self.capabilities.thermal_state = thermal_state;
        let new_policy = Self::derive_policy(battery_pct, thermal_state);
        let changed = new_policy != self.policy;
        self.policy = new_policy;
        (new_policy, changed)
    }

    /// Maximum concurrent agents allowed under the current policy.
    pub fn max_concurrent_agents(&self) -> u8 {
        match self.policy {
            BatteryPolicy::Full => 8,
            BatteryPolicy::Balanced => 5,
            BatteryPolicy::LowPower => 3,
            BatteryPolicy::Critical => 0,
        }
    }

    /// Whether inference workloads are allowed.
    pub fn inference_allowed(&self) -> bool {
        !matches!(self.policy, BatteryPolicy::Critical)
    }

    /// Recommended widget update interval in seconds.
    pub fn widget_update_interval_secs(&self) -> u64 {
        match self.policy {
            BatteryPolicy::Full => 60,
            BatteryPolicy::Balanced => 120,
            BatteryPolicy::LowPower => 300,
            BatteryPolicy::Critical => 600,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_policy_full() {
        assert_eq!(
            BatteryAwareScheduler::derive_policy(0.95, ThermalState::Nominal),
            BatteryPolicy::Full
        );
    }

    #[test]
    fn test_derive_policy_balanced() {
        assert_eq!(
            BatteryAwareScheduler::derive_policy(0.50, ThermalState::Nominal),
            BatteryPolicy::Balanced
        );
    }

    #[test]
    fn test_derive_policy_low_power() {
        assert_eq!(
            BatteryAwareScheduler::derive_policy(0.15, ThermalState::Nominal),
            BatteryPolicy::LowPower
        );
    }

    #[test]
    fn test_derive_policy_critical_battery() {
        assert_eq!(
            BatteryAwareScheduler::derive_policy(0.05, ThermalState::Nominal),
            BatteryPolicy::Critical
        );
    }

    #[test]
    fn test_thermal_override_critical() {
        assert_eq!(
            BatteryAwareScheduler::derive_policy(0.90, ThermalState::Critical),
            BatteryPolicy::Critical
        );
    }

    #[test]
    fn test_thermal_override_serious() {
        assert_eq!(
            BatteryAwareScheduler::derive_policy(0.90, ThermalState::Serious),
            BatteryPolicy::LowPower
        );
    }

    #[test]
    fn test_max_concurrent_agents() {
        let caps = DeviceCapabilities::default_ios();
        let scheduler = BatteryAwareScheduler::new(caps);
        assert_eq!(scheduler.max_concurrent_agents(), 8);
    }

    #[test]
    fn test_update_changes_policy() {
        let caps = DeviceCapabilities::default_ios();
        let mut scheduler = BatteryAwareScheduler::new(caps);
        assert_eq!(scheduler.policy, BatteryPolicy::Full);

        let (new_policy, changed) = scheduler.update(0.05, ThermalState::Nominal);
        assert!(changed);
        assert_eq!(new_policy, BatteryPolicy::Critical);
    }
}
