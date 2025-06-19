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

    /// Bandwidth limit with unit (e.g., "100kbps", "1mbps", "50000bps", "0" = unlimited)
    #[arg(long, value_parser = parse_bandwidth_limit, default_value = "0")]
    pub bandwidth_limit: u64,

    /// Bandwidth throttling burst size in bytes (allows temporary bursts)
    #[arg(long, default_value = "8192")]
    pub bandwidth_burst_size: u64,
}

fn parse_latency_range(s: &str) -> Result<(u64, u64), String> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 2 {
        return Err("Latency range must be in format 'min-max' (e.g., '100-500')".to_string());
    }

    let min = parts[0]
        .parse::<u64>()
        .map_err(|_| "Invalid minimum latency value".to_string())?;
    let max = parts[1]
        .parse::<u64>()
        .map_err(|_| "Invalid maximum latency value".to_string())?;

    if min > max {
        return Err("Minimum latency must be less than or equal to maximum latency".to_string());
    }

    Ok((min, max))
}

fn parse_bandwidth_limit(s: &str) -> Result<u64, String> {
    if s == "0" {
        return Ok(0); // Unlimited
    }

    let s = s.to_lowercase();

    if let Some(stripped) = s.strip_suffix("mbps") {
        // Handle "mbps" suffix (megabytes per second) - check first
        let mbps = stripped
            .parse::<u64>()
            .map_err(|_| "Invalid bandwidth value for mbps".to_string())?;
        Ok(mbps * 1024 * 1024)
    } else if let Some(stripped) = s.strip_suffix("kbps") {
        // Handle "kbps" suffix (kilobytes per second) - check second
        let kbps = stripped
            .parse::<u64>()
            .map_err(|_| "Invalid bandwidth value for kbps".to_string())?;
        Ok(kbps * 1024)
    } else if let Some(stripped) = s.strip_suffix("bps") {
        // Handle "bps" suffix (bytes per second) - check last
        stripped
            .parse::<u64>()
            .map_err(|_| "Invalid bandwidth value for bps".to_string())
    } else {
        // No suffix, assume bytes per second
        s.parse::<u64>().map_err(|_| {
            "Invalid bandwidth value (use format like '100kbps', '1mbps', or '50000bps')"
                .to_string()
        })
    }
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

    /// Get the bandwidth limit in bytes per second
    pub fn bandwidth_limit(&self) -> u64 {
        self.bandwidth_limit
    }
}
