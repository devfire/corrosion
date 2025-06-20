use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::time::{Duration, Instant};
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

#[derive(Debug, Clone)]
pub struct BandwidthConfig {
    pub enabled: bool,
    pub limit_bps: u64,
    pub burst_size: u64,
}

impl LatencyConfig {
    pub fn new(
        enabled: bool,
        fixed_ms: u64,
        random_range: Option<(u64, u64)>,
        probability: f64,
    ) -> Self {
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
    pub fn new(
        enabled: bool,
        probability: f64,
        burst_size: Option<u32>,
        burst_probability: f64,
    ) -> Self {
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

impl BandwidthConfig {
    pub fn new(enabled: bool, limit_bps: u64, burst_size: u64) -> Self {
        Self {
            enabled,
            limit_bps,
            burst_size,
        }
    }

    pub fn is_disabled(&self) -> bool {
        !self.enabled || self.limit_bps == 0
    }
}

pub struct FaultInjector {
    latency_config: LatencyConfig,
    packet_loss_config: PacketLossConfig,
    bandwidth_config: BandwidthConfig,
    rng: StdRng,
    burst_counter: u32,
    in_burst_mode: bool,
    // Bandwidth throttling state
    bandwidth_tokens: f64,
    last_refill: Instant,
}

impl FaultInjector {
    pub fn new(
        latency_config: LatencyConfig,
        packet_loss_config: PacketLossConfig,
        bandwidth_config: BandwidthConfig,
    ) -> Self {
        Self {
            latency_config,
            packet_loss_config,
            bandwidth_config: bandwidth_config.clone(),
            rng: StdRng::from_entropy(),
            burst_counter: 0,
            in_burst_mode: false,
            bandwidth_tokens: bandwidth_config.burst_size as f64,
            last_refill: Instant::now(),
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
            debug!(
                "Random latency component: {}ms (range: {}-{}ms)",
                random_delay, min, max
            );
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
                info!(
                    "Dropping packet {} in burst mode for {}",
                    self.burst_counter, connection_id
                );
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
            info!(
                "Dropping packet for {} (probability: {:.3})",
                connection_id, roll
            );
            return true;
        }

        false
    }

    /// Apply bandwidth throttling by delaying if necessary
    pub async fn apply_bandwidth_throttling(&mut self, bytes: usize, connection_id: &str) {
        if self.bandwidth_config.is_disabled() {
            return;
        }

        // Convert limit from bits-per-second to bytes-per-second
        let rate_bytes_per_sec = self.bandwidth_config.limit_bps as f64 / 8.0;
        if rate_bytes_per_sec <= 0.0 {
            return;
        }

        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();

        // Refill tokens based on elapsed time
        let tokens_to_add = elapsed * rate_bytes_per_sec;
        self.bandwidth_tokens =
            (self.bandwidth_tokens + tokens_to_add).min(self.bandwidth_config.burst_size as f64);
        self.last_refill = now;

        let bytes_needed = bytes as f64;
        let tokens_after_consumption = self.bandwidth_tokens - bytes_needed;

        if tokens_after_consumption < 0.0 {
            // Not enough tokens, calculate delay needed based on the deficit
            let tokens_deficit = -tokens_after_consumption;
            let delay_seconds = tokens_deficit / rate_bytes_per_sec;
            let delay_ms = (delay_seconds * 1000.0) as u64;

            if delay_ms > 0 {
                info!(
                    "Bandwidth throttling: delaying {}ms for {} bytes on {}",
                    delay_ms, bytes, connection_id
                );
                sleep(Duration::from_millis(delay_ms)).await;

                // After sleeping, update the last_refill time to account for the delay
                self.last_refill = Instant::now();
            }
        }

        // Always consume the tokens, allowing the balance to go negative (into "debt")
        self.bandwidth_tokens -= bytes_needed;
        info!(
            "Bandwidth throttling: consumed {} tokens, {} remaining for {}",
            bytes_needed, self.bandwidth_tokens, connection_id
        );
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
        let bandwidth_config = BandwidthConfig::new(false, 0, 8192);
        let injector = FaultInjector::new(latency_config, packet_loss_config, bandwidth_config);
        assert_eq!(injector.latency_config.fixed_ms, 100);
        assert_eq!(injector.latency_config.random_range, Some((50, 200)));
        assert_eq!(injector.latency_config.probability, 0.8);
        assert!(injector.packet_loss_config.is_disabled());
        assert!(injector.bandwidth_config.is_disabled());
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

    #[test]
    fn test_bandwidth_config_disabled() {
        let config = BandwidthConfig::new(false, 1000, 8192);
        assert!(config.is_disabled());

        let config = BandwidthConfig::new(true, 0, 8192);
        assert!(config.is_disabled());

        let config = BandwidthConfig::new(true, 1000, 8192);
        assert!(!config.is_disabled());
    }
}
