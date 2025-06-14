# TCP Transparent Proxy

A transparent TCP proxy implementation using Tokio for fault injection testing and network analysis.

## Features

- Asynchronous TCP proxy using Tokio
- Concurrent connection handling
- Transparent bidirectional data forwarding
- Proper error handling
- Clean connection lifecycle management
- Configurable bind and destination addresses via command-line arguments or environment variables

## Usage

### Running the Proxy

The proxy requires destination parameters to be specified:

#### Basic usage:
```bash
cargo run -- --dest-ip example.com --dest-port 80
```

#### Custom bind address and destination:
```bash
cargo run -- --ip 0.0.0.0 --port 9090 --dest-ip 192.168.1.100 --dest-port 3000
```

#### Using short flags:
```bash
cargo run -- -i 0.0.0.0 -p 8080 -d httpbin.org --dest-port 80
```

#### Using environment variables:
```bash
BIND_IP=0.0.0.0 BIND_PORT=9090 DEST_IP=example.com DEST_PORT=80 cargo run
```

#### View help:
```bash
cargo run -- --help
```

### Configuration Options

**Bind Configuration:**
- `--ip` / `-i`: IP address to bind to (default: `127.0.0.1`, env: `BIND_IP`)
- `--port` / `-p`: Port to bind to (default: `8080`, env: `BIND_PORT`)

**Destination Configuration (Required):**
- `--dest-ip` / `-d`: Destination IP address or hostname (env: `DEST_IP`)
- `--dest-port`: Destination port (env: `DEST_PORT`)

Command-line arguments take precedence over environment variables.

### Testing the Proxy

You can test the proxy using various tools. For example, if proxying to httpbin.org:

#### Start the proxy:
```bash
cargo run -- --dest-ip httpbin.org --dest-port 80
```

#### Test with curl:
```bash
curl -H "Host: httpbin.org" http://127.0.0.1:8080/get
```

#### Test with telnet:
```bash
telnet 127.0.0.1 8080
```

#### Test with netcat:
```bash
nc 127.0.0.1 8080
```

## How it Works

1. The proxy parses command-line arguments or environment variables for bind and destination configuration
2. The proxy binds to the specified local address and listens for incoming connections
3. For each new connection, a separate Tokio task is spawned to handle it concurrently
4. Each connection handler establishes a connection to the destination server
5. Data is transparently forwarded bidirectionally between client and destination
6. Connections are properly closed when either side disconnects
7. Errors are handled gracefully and logged with connection details

## Architecture

The implementation follows Rust best practices:

- **Async/await**: Uses Tokio's async runtime for non-blocking I/O
- **Error handling**: Proper `Result` types and error propagation
- **Concurrency**: Each proxy connection runs in its own task
- **Bidirectional forwarding**: Uses `io::copy_bidirectional` for efficient data transfer
- **Resource management**: Automatic cleanup of connections
- **Type safety**: Leverages Rust's type system for memory safety
- **Modular design**: CLI configuration separated into its own module

## Dependencies

- `tokio`: Async runtime with full features enabled
- `anyhow`: Error handling and context
- `clap`: Command-line argument parsing with derive and env features