use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "fault-injection")]
#[command(about = "A TCP server for fault injection testing")]
pub struct Args {
    /// IP address to bind to
    #[arg(short, long, default_value = "127.0.0.1", env = "BIND_IP")]
    pub ip: String,

    /// Port to bind to
    #[arg(short, long, default_value = "8080", env = "BIND_PORT")]
    pub port: u16,
}

impl Args {
    pub fn parse_args() -> Self {
        Self::parse()
    }

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}