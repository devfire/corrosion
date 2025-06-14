mod cli;

use anyhow::{Context, Result};
use tokio::io;
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::Args::parse_args();
    let bind_addr = args.bind_address();
    let dest_addr = args.dest_address();

    // Bind the listener to the address
    let listener = TcpListener::bind(&bind_addr)
        .await
        .with_context(|| format!("Failed to bind TCP listener to {}", bind_addr))?;
    
    println!("TCP proxy listening on {} -> forwarding to {}", bind_addr, dest_addr);

    loop {
        // Accept new connections
        let (inbound, client_addr) = listener
            .accept()
            .await
            .context("Failed to accept incoming connection")?;
        
        println!("New connection from: {} -> {}", client_addr, dest_addr);

        let dest_addr_clone = dest_addr.clone();
        
        // Spawn a new task to handle each connection
        tokio::spawn(async move {
            if let Err(e) = handle_connection(inbound, client_addr, dest_addr_clone).await {
                eprintln!("Error handling connection from {}: {:?}", client_addr, e);
            }
        });
    }
}

async fn handle_connection(
    mut inbound: TcpStream,
    client_addr: std::net::SocketAddr,
    dest_addr: String
) -> Result<()> {
    // Connect to the destination server
    let mut outbound = TcpStream::connect(&dest_addr)
        .await
        .with_context(|| format!("Failed to connect to destination {}", dest_addr))?;

    println!("Established proxy connection: {} <-> {}", client_addr, dest_addr);

    // Use bidirectional copy to handle the proxy
    match io::copy_bidirectional(&mut inbound, &mut outbound).await {
        Ok((client_to_server, server_to_client)) => {
            println!(
                "Proxy connection completed: {} bytes client->server, {} bytes server->client",
                client_to_server, server_to_client
            );
        }
        Err(e) => {
            eprintln!("Proxy error for {}: {}", client_addr, e);
        }
    }

    println!("Proxy connection closed: {} <-> {}", client_addr, dest_addr);
    Ok(())
}
