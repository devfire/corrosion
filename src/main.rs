mod cli;
mod fault_injection;

use anyhow::{Context, Result};
use fault_injection::{FaultInjector, LatencyConfig, PacketLossConfig, BandwidthConfig};
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

    // Create bandwidth configuration from CLI args
    let bandwidth_config = BandwidthConfig::new(
        args.bandwidth_enabled,
        args.bandwidth_limit(),
        args.bandwidth_burst_size,
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
            info!(
                "  Burst probability: {:.3}",
                packet_loss_config.burst_probability
            );
        }
    } else {
        info!("Packet loss injection disabled");
    }

    if !bandwidth_config.is_disabled() {
        info!("Bandwidth throttling enabled:");
        info!("  Limit: {} bytes/sec ({:.2} MB/sec)",
              bandwidth_config.limit_bps,
              bandwidth_config.limit_bps as f64 / (1024.0 * 1024.0));
        info!("  Burst size: {} bytes", bandwidth_config.burst_size);
    } else {
        info!("Bandwidth throttling disabled");
    }

    // Bind the listener to the address
    let listener = TcpListener::bind(&bind_addr)
        .await
        .with_context(|| format!("Failed to bind TCP listener to {}", bind_addr))?;

    info!(
        "TCP proxy listening on {} -> forwarding to {}",
        bind_addr, dest_addr
    );
    info!("WARNING: When connecting via HTTPS, use the destination hostname in your browser, not localhost!");

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
        let bandwidth_config_clone = bandwidth_config.clone();

        // Spawn a new task to handle each connection
        tokio::spawn(async move {
            if let Err(e) = handle_connection(
                inbound,
                client_addr,
                dest_addr_clone,
                latency_config_clone,
                packet_loss_config_clone,
                bandwidth_config_clone,
            )
            .await
            {
                error!("Error handling connection from {}: {:?}", client_addr, e);
            }
        });
    }
}

// async fn resolve_original_destination(dest_addr: &str) -> String {
//     // For transparent proxying, we need to avoid redirect loops
//     // If the destination is our configured target, resolve it to actual IPs
//     if dest_addr.contains("speedtest.net") {
//         // Use DNS to get actual IP addresses to bypass iptables redirects
//         match tokio::net::lookup_host(dest_addr).await {
//             Ok(mut addrs) => {
//                 if let Some(addr) = addrs.next() {
//                     debug!("Resolved {} to {} to avoid redirect loop", dest_addr, addr);
//                     return addr.to_string();
//                 }
//             }
//             Err(e) => {
//                 debug!("Failed to resolve {}: {}, using original", dest_addr, e);
//             }
//         }
//     }
//     dest_addr.to_string()
// }

async fn handle_connection(
    mut inbound: TcpStream,
    client_addr: std::net::SocketAddr,
    dest_addr: String,
    latency_config: LatencyConfig,
    packet_loss_config: PacketLossConfig,
    bandwidth_config: BandwidthConfig,
) -> Result<()> {
    debug!("Attempting to connect to destination: {}", dest_addr);

    // Create fault injector for this connection
    let mut fault_injector = FaultInjector::new(latency_config, packet_loss_config, bandwidth_config);
    let connection_id = format!("{}->{}", client_addr, dest_addr);

    // Log TLS certificate warning for HTTPS connections
    if dest_addr.contains(":443") || dest_addr.ends_with(":443") {
        info!("HTTPS connection detected for {}", connection_id);
        info!("TLS certificate will be for destination hostname, not proxy hostname");
        info!("Browser should connect to destination hostname directly for proper certificate validation");
    }

    // For transparent proxying, we need to resolve the original destination
    // to avoid iptables redirect loops
    // let actual_dest = resolve_original_destination(&dest_addr).await;
    
    // Connect to the destination server
    let mut outbound = match TcpStream::connect(&dest_addr).await {
        Ok(stream) => {
            info!("Successfully connected to destination: {} ", dest_addr);
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

    // Use custom bidirectional copy with packet loss simulation
    match copy_bidirectional_with_faults(
        &mut inbound,
        &mut outbound,
        &mut fault_injector,
        &connection_id,
    )
    .await
    {
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

                        // Apply bandwidth throttling
                        fault_injector.apply_bandwidth_throttling(n, connection_id).await;

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

                        // Apply bandwidth throttling
                        fault_injector.apply_bandwidth_throttling(n, connection_id).await;

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
