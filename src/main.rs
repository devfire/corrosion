mod cli;
mod fault_injection;

use anyhow::{Context, Result};
use fault_injection::{FaultInjector, LatencyConfig, PacketLossConfig};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
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

    // Create packet loss configuration from CLI args
    let packet_loss_config = PacketLossConfig::new(
        args.packet_loss_enabled,
        args.packet_loss_probability,
        args.packet_loss_burst_size,
        args.packet_loss_burst_probability,
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

    if !packet_loss_config.is_disabled() {
        info!("Packet loss injection enabled:");
        info!("  Drop probability: {:.3}", packet_loss_config.probability);
        if let Some(burst_size) = packet_loss_config.burst_size {
            info!("  Burst size: {} packets", burst_size);
            info!("  Burst probability: {:.3}", packet_loss_config.burst_probability);
        }
    } else {
        info!("Packet loss injection disabled");
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
        let packet_loss_config_clone = packet_loss_config.clone();

        // Spawn a new task to handle each connection
        tokio::spawn(async move {
            if let Err(e) = handle_connection(inbound, client_addr, dest_addr_clone, latency_config_clone, packet_loss_config_clone).await {
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
    packet_loss_config: PacketLossConfig,
) -> Result<()> {
    debug!("Attempting to connect to destination: {}", dest_addr);

    // Create fault injector for this connection
    let mut fault_injector = FaultInjector::new(latency_config, packet_loss_config);
    let connection_id = format!("{}->{}", client_addr, dest_addr);

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

    // Use custom bidirectional copy with packet loss simulation
    match copy_bidirectional_with_faults(&mut inbound, &mut outbound, &mut fault_injector, &connection_id).await {
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

async fn copy_bidirectional_with_faults(
    a: &mut TcpStream,
    b: &mut TcpStream,
    fault_injector: &mut FaultInjector,
    connection_id: &str,
) -> Result<(u64, u64), std::io::Error> {
    let mut buf_a = [0u8; 8192];
    let mut buf_b = [0u8; 8192];
    let mut total_a_to_b = 0u64;
    let mut total_b_to_a = 0u64;

    loop {
        tokio::select! {
            // Read from A, write to B
            result_a = a.read(&mut buf_a) => {
                match result_a {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        // Check if packet should be dropped
                        if fault_injector.should_drop_packet(connection_id) {
                            // Simulate packet loss by not forwarding the data
                            continue;
                        }
                        
                        // Apply latency per packet
                        fault_injector.apply_latency(connection_id).await;
                        
                        match b.write_all(&buf_a[..n]).await {
                            Ok(()) => {
                                total_a_to_b += n as u64;
                                b.flush().await?;
                            }
                            Err(e) => return Err(e),
                        }
                    }
                    Err(e) => return Err(e),
                }
            }
            
            // Read from B, write to A
            result_b = b.read(&mut buf_b) => {
                match result_b {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        // Check if packet should be dropped
                        if fault_injector.should_drop_packet(connection_id) {
                            // Simulate packet loss by not forwarding the data
                            continue;
                        }
                        
                        // Apply latency per packet
                        fault_injector.apply_latency(connection_id).await;
                        
                        match a.write_all(&buf_b[..n]).await {
                            Ok(()) => {
                                total_b_to_a += n as u64;
                                a.flush().await?;
                            }
                            Err(e) => return Err(e),
                        }
                    }
                    Err(e) => return Err(e),
                }
            }
        }
    }

    Ok((total_a_to_b, total_b_to_a))
}
