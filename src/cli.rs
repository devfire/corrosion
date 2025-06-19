use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "fault-injection")]
#[command(about = "A transparent TCP proxy for fault injection testing")]
pub struct Args {
    /// IP address to bind to
    #[arg(short, long, default_value = "127.0.0.1", env = "BIND_IP")]
    pub ip: String,

    /// Port to bind to
    #[arg(short, long, default_value = "8080", env = "BIND_PORT")]
    pub port: u16,

    /// Destination IP address or hostname
    #[arg(short, long, env = "DEST_IP")]
    pub dest_ip: String,

    /// Destination port
    #[arg(long, env = "DEST_PORT")]
    pub dest_port: u16,

    /// Enable latency injection
    #[arg(long, default_value = "false")]
    pub latency_enabled: bool,

    /// Fixed latency to add in milliseconds
    #[arg(long, default_value = "0")]
    pub latency_fixed_ms: u64,

    /// Random latency range (min-max) in milliseconds
    #[arg(long, value_parser = parse_latency_range)]
    pub latency_random_ms: Option<(u64, u64)>,

    /// Probability of applying latency (0.0-1.0)
    #[arg(long, default_value = "1.0")]
    pub latency_probability: f64,

    /// Enable packet loss injection
    #[arg(long, default_value = "false")]
    pub packet_loss_enabled: bool,

    /// Probability of packet loss (0.0-1.0)
    #[arg(long, default_value = "0.0")]
    pub packet_loss_probability: f64,

    /// Burst packet loss size (number of consecutive packets to drop)
    #[arg(long)]
    pub packet_loss_burst_size: Option<u32>,

    /// Probability of entering burst packet loss mode (0.0-1.0)
    #[arg(long, default_value = "0.0")]
    pub packet_loss_burst_probability: f64,

    /// Enable bandwidth throttling
    #[arg(long, default_value = "false")]
    pub bandwidth_enabled: bool,

    /// Maximum bandwidth in bytes per second (0 = unlimited)
    #[arg(long, default_value = "0")]
    pub bandwidth_limit_bps: u64,

    /// Bandwidth limit in kilobytes per second (alternative to bps)
    #[arg(long, value_parser = parse_bandwidth_kbps)]
    pub bandwidth_limit_kbps: Option<u64>,

    /// Bandwidth limit in megabytes per second (alternative to bps/kbps)
    #[arg(long, value_parser = parse_bandwidth_mbps)]
    pub bandwidth_limit_mbps: Option<u64>,

    /// Bandwidth throttling burst size in bytes (allows temporary bursts)
    #[arg(long, default_value = "8192")]
    pub bandwidth_burst_size: u64,
}

fn parse_latency_range(s: &str) -> Result<(u64, u64), String> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 2 {
        return Err("Latency range must be in format 'min-max' (e.g., '100-500')".to_string());
    }
    
    let min = parts[0].parse::<u64>()
        .map_err(|_| "Invalid minimum latency value".to_string())?;
    let max = parts[1].parse::<u64>()
        .map_err(|_| "Invalid maximum latency value".to_string())?;
    
    if min > max {
        return Err("Minimum latency must be less than or equal to maximum latency".to_string());
    }
    
    Ok((min, max))
}

fn parse_bandwidth_kbps(s: &str) -> Result<u64, String> {
    let kbps = s.parse::<u64>()
        .map_err(|_| "Invalid bandwidth value in kbps".to_string())?;
    Ok(kbps * 1024) // Convert to bytes per second
}

fn parse_bandwidth_mbps(s: &str) -> Result<u64, String> {
    let mbps = s.parse::<u64>()
        .map_err(|_| "Invalid bandwidth value in mbps".to_string())?;
    Ok(mbps * 1024 * 1024) // Convert to bytes per second
}

impl Args {
    pub fn parse_args() -> Self {
        Self::parse()
    }

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }

    pub fn dest_address(&self) -> String {
        format!("{}:{}", self.dest_ip, self.dest_port)
    }

    /// Calculate the final bandwidth limit in bytes per second
    pub fn bandwidth_limit(&self) -> u64 {
        // Priority: mbps > kbps > bps
        if let Some(mbps_bytes) = self.bandwidth_limit_mbps {
            mbps_bytes
        } else if let Some(kbps_bytes) = self.bandwidth_limit_kbps {
            kbps_bytes
        } else {
            self.bandwidth_limit_bps
        }
    }
}