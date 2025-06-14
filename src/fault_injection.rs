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

#[derive(Debug, Clone)]
pub struct PacketLossConfig {
    pub enabled: bool,
    pub probability: f64,
    pub burst_size: Option<u32>,
    pub burst_probability: f64,
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

impl PacketLossConfig {
    pub fn new(enabled: bool, probability: f64, burst_size: Option<u32>, burst_probability: f64) -> Self {
        Self {
            enabled,
            probability: probability.clamp(0.0, 1.0),
            burst_size,
            burst_probability: burst_probability.clamp(0.0, 1.0),
        }
    }

    pub fn is_disabled(&self) -> bool {
        !self.enabled || self.probability == 0.0
    }
}

pub struct FaultInjector {
    latency_config: LatencyConfig,
    packet_loss_config: PacketLossConfig,
    rng: StdRng,
    burst_counter: u32,
    in_burst_mode: bool,
}

impl FaultInjector {
    pub fn new(latency_config: LatencyConfig, packet_loss_config: PacketLossConfig) -> Self {
        Self {
            latency_config,
            packet_loss_config,
            rng: StdRng::from_entropy(),
            burst_counter: 0,
            in_burst_mode: false,
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
            debug!(
                "Injecting {}ms latency for packet on connection {}",
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

    /// Check if packet should be dropped due to packet loss simulation
    pub fn should_drop_packet(&mut self, connection_id: &str) -> bool {
        if self.packet_loss_config.is_disabled() {
            return false;
        }

        // Handle burst mode
        if let Some(burst_size) = self.packet_loss_config.burst_size {
            if self.in_burst_mode {
                self.burst_counter += 1;
                if self.burst_counter >= burst_size {
                    self.in_burst_mode = false;
                    self.burst_counter = 0;
                    debug!("Exiting burst packet loss mode for {}", connection_id);
                }
                info!("Dropping packet {} in burst mode for {}", self.burst_counter, connection_id);
                return true;
            } else {
                // Check if we should enter burst mode
                let burst_roll: f64 = self.rng.gen_range(0.0..1.0);
                if burst_roll <= self.packet_loss_config.burst_probability {
                    self.in_burst_mode = true;
                    self.burst_counter = 1;
                    info!("Entering burst packet loss mode for {}", connection_id);
                    return true;
                }
            }
        }

        // Regular packet loss check
        let roll: f64 = self.rng.gen_range(0.0..1.0);
        if roll <= self.packet_loss_config.probability {
            info!("Dropping packet for {} (probability: {:.3})", connection_id, roll);
            return true;
        }

        false
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
        let latency_config = LatencyConfig::new(true, 100, Some((50, 200)), 0.8);
        let packet_loss_config = PacketLossConfig::new(false, 0.0, None, 0.0);
        let injector = FaultInjector::new(latency_config, packet_loss_config);
        assert_eq!(injector.latency_config.fixed_ms, 100);
        assert_eq!(injector.latency_config.random_range, Some((50, 200)));
        assert_eq!(injector.latency_config.probability, 0.8);
        assert!(injector.packet_loss_config.is_disabled());
    }

    #[test]
    fn test_packet_loss_config_disabled() {
        let config = PacketLossConfig::new(false, 0.5, None, 0.0);
        assert!(config.is_disabled());

        let config = PacketLossConfig::new(true, 0.0, None, 0.0);
        assert!(config.is_disabled());

        let config = PacketLossConfig::new(true, 0.1, None, 0.0);
        assert!(!config.is_disabled());
    }

    #[test]
    fn test_packet_loss_probability_clamping() {
        let config = PacketLossConfig::new(true, 1.5, None, 0.0);
        assert_eq!(config.probability, 1.0);

        let config = PacketLossConfig::new(true, -0.5, None, 0.0);
        assert_eq!(config.probability, 0.0);

        let config = PacketLossConfig::new(true, 0.5, Some(5), 1.5);
        assert_eq!(config.burst_probability, 1.0);
    }
}