mod cli;

use anyhow::{Context, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::Args::parse_args();
    let bind_addr = args.bind_address();

    // Bind the listener to the address
    let listener = TcpListener::bind(&bind_addr)
        .await
        .with_context(|| format!("Failed to bind TCP listener to {}", bind_addr))?;
    
    println!("TCP server listening on {}", bind_addr);

    loop {
        // Accept new connections
        let (socket, addr) = listener
            .accept()
            .await
            .context("Failed to accept incoming connection")?;
        
        println!("New connection from: {}", addr);

        // Spawn a new task to handle each connection
        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket, addr).await {
                eprintln!("Error handling connection from {}: {:?}", addr, e);
            }
        });
    }
}

async fn handle_connection(mut socket: TcpStream, addr: std::net::SocketAddr) -> Result<()> {
    let mut buffer = [0; 1024];

    loop {
        // Read data from the socket
        let bytes_read = socket
            .read(&mut buffer)
            .await
            .with_context(|| format!("Failed to read data from client {}", addr))?;
        
        // If no bytes were read, the connection is closed
        if bytes_read == 0 {
            println!("Connection closed by client {}", addr);
            break;
        }

        // Convert bytes to string for display
        let received = String::from_utf8_lossy(&buffer[..bytes_read]);
        println!("Received from {}: {}", addr, received.trim());

        // Echo the data back to the client
        socket
            .write_all(&buffer[..bytes_read])
            .await
            .with_context(|| format!("Failed to write data to client {}", addr))?;
        
        socket
            .flush()
            .await
            .with_context(|| format!("Failed to flush data to client {}", addr))?;
    }

    println!("Connection with {} closed successfully", addr);
    Ok(())
}
