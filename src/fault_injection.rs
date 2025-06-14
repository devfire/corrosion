use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct LatencyConfig {
    pub enabled: bool,
    pub fixed_ms: u64,
    pub random_range: Option<(u64, u64)>,
    pub probability: f64,
}

impl LatencyConfig {
    pub fn new(enabled: bool, fixed_ms: u64, random_range: Option<(u64, u64)>, probability: f64) -> Self {
        Self {
            enabled,
            fixed_ms,
            random_range,
            probability: probability.clamp(0.0, 1.0),
        }
    }

    pub fn is_disabled(&self) -> bool {
        !self.enabled || (self.fixed_ms == 0 && self.random_range.is_none())
    }
}

pub struct FaultInjector {
    latency_config: LatencyConfig,
    rng: StdRng,
}

impl FaultInjector {
    pub fn new(latency_config: LatencyConfig) -> Self {
        Self {
            latency_config,
            rng: StdRng::from_entropy(),
        }
    }

    /// Apply latency fault injection if configured and probability check passes
    pub async fn apply_latency(&mut self, connection_id: &str) {
        if self.latency_config.is_disabled() {
            return;
        }

        // Check probability
        if self.latency_config.probability < 1.0 {
            let roll: f64 = self.rng.gen_range(0.0..1.0);
            if roll > self.latency_config.probability {
                debug!(
                    "Latency injection skipped for {} (probability check failed: {} > {})",
                    connection_id, roll, self.latency_config.probability
                );
                return;
            }
        }

        let delay_ms = self.calculate_delay();
        if delay_ms > 0 {
            info!(
                "Injecting {}ms latency for connection {}",
                delay_ms, connection_id
            );
            sleep(Duration::from_millis(delay_ms)).await;
        }
    }

    fn calculate_delay(&mut self) -> u64 {
        let mut total_delay = self.latency_config.fixed_ms;

        if let Some((min, max)) = self.latency_config.random_range {
            let random_delay = self.rng.gen_range(min..=max);
            total_delay += random_delay;
            debug!("Random latency component: {}ms (range: {}-{}ms)", random_delay, min, max);
        }

        total_delay
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latency_config_disabled() {
        let config = LatencyConfig::new(false, 100, None, 1.0);
        assert!(config.is_disabled());

        let config = LatencyConfig::new(true, 0, None, 1.0);
        assert!(config.is_disabled());

        let config = LatencyConfig::new(true, 100, None, 1.0);
        assert!(!config.is_disabled());
    }

    #[test]
    fn test_probability_clamping() {
        let config = LatencyConfig::new(true, 100, None, 1.5);
        assert_eq!(config.probability, 1.0);

        let config = LatencyConfig::new(true, 100, None, -0.5);
        assert_eq!(config.probability, 0.0);
    }

    #[tokio::test]
    async fn test_fault_injector_creation() {
        let config = LatencyConfig::new(true, 100, Some((50, 200)), 0.8);
        let injector = FaultInjector::new(config);
        assert_eq!(injector.latency_config.fixed_ms, 100);
        assert_eq!(injector.latency_config.random_range, Some((50, 200)));
        assert_eq!(injector.latency_config.probability, 0.8);
    }
}