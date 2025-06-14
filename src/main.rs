mod cli;
mod fault_injection;

use anyhow::{Context, Result};
use fault_injection::{FaultInjector, LatencyConfig};
use tokio::io::{self};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, error, info};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fault_injection=info".parse().unwrap()),
        )
        .init();

    let args = cli::Args::parse_args();
    let bind_addr = args.bind_address();
    let dest_addr = args.dest_address();

    // Create latency configuration from CLI args
    let latency_config = LatencyConfig::new(
        args.latency_enabled,
        args.latency_fixed_ms,
        args.latency_random_ms,
        args.latency_probability,
    );

    // Log fault injection configuration
    if !latency_config.is_disabled() {
        info!("Latency injection enabled:");
        info!("  Fixed delay: {}ms", latency_config.fixed_ms);
        if let Some((min, max)) = latency_config.random_range {
            info!("  Random delay range: {}-{}ms", min, max);
        }
        info!("  Probability: {:.2}", latency_config.probability);
    } else {
        info!("Latency injection disabled");
    }

    // Bind the listener to the address
    let listener = TcpListener::bind(&bind_addr)
        .await
        .with_context(|| format!("Failed to bind TCP listener to {}", bind_addr))?;

    info!(
        "TCP proxy listening on {} -> forwarding to {}",
        bind_addr, dest_addr
    );

    loop {
        // Accept new connections
        let (inbound, client_addr) = listener
            .accept()
            .await
            .context("Failed to accept incoming connection")?;

        info!("New connection from: {} -> {}", client_addr, dest_addr);

        let dest_addr_clone = dest_addr.clone();
        let latency_config_clone = latency_config.clone();

        // Spawn a new task to handle each connection
        tokio::spawn(async move {
            if let Err(e) = handle_connection(inbound, client_addr, dest_addr_clone, latency_config_clone).await {
                error!("Error handling connection from {}: {:?}", client_addr, e);
            }
        });
    }
}

async fn handle_connection(
    mut inbound: TcpStream,
    client_addr: std::net::SocketAddr,
    dest_addr: String,
    latency_config: LatencyConfig,
) -> Result<()> {
    debug!("Attempting to connect to destination: {}", dest_addr);

    // Create fault injector for this connection
    let mut fault_injector = FaultInjector::new(latency_config);
    let connection_id = format!("{}->{}", client_addr, dest_addr);

    // Apply latency before connecting to destination
    fault_injector.apply_latency(&connection_id).await;

    // Connect to the destination server
    let mut outbound = match TcpStream::connect(&dest_addr).await {
        Ok(stream) => {
            info!("Successfully connected to destination: {}", dest_addr);
            stream
        }
        Err(e) => {
            error!("Failed to connect to destination {}: {}", dest_addr, e);
            return Err(e)
                .with_context(|| format!("Failed to connect to destination {}", dest_addr));
        }
    };

    debug!(
        "Established proxy connection: {} <-> {}",
        client_addr, dest_addr
    );

    // Peek at the first few bytes to detect protocol
    // let mut peek_buf = [0u8; 16];
    // match inbound.peek(&mut peek_buf).await {
    //     Ok(n) if n > 0 => {
    //         let protocol_hint = if peek_buf[0] == 0x16 {
    //             "TLS/SSL handshake"
    //         } else if peek_buf.starts_with(b"GET ")
    //             || peek_buf.starts_with(b"POST ")
    //             || peek_buf.starts_with(b"PUT ")
    //             || peek_buf.starts_with(b"HEAD ")
    //         {
    //             "HTTP request"
    //         } else {
    //             "Unknown protocol"
    //         };
    //         debug!(
    //             "Detected incoming protocol: {} (first {} bytes: {:02x?})",
    //             protocol_hint,
    //             n,
    //             &peek_buf[..n]
    //         );
    //     }
    //     Ok(_) => debug!("No data available to peek"),
    //     Err(e) => warn!("Failed to peek at incoming data: {}", e),
    // }

    // Use bidirectional copy to handle the proxy
    match io::copy_bidirectional(&mut inbound, &mut outbound).await {
        Ok((client_to_server, server_to_client)) => {
            info!(
                "Proxy connection completed: {} bytes client->server, {} bytes server->client",
                client_to_server, server_to_client
            );
        }
        Err(e) => {
            error!("Proxy error for {}: {}", client_addr, e);
        }
    }

    info!("Proxy connection closed: {} <-> {}", client_addr, dest_addr);
    Ok(())
}
