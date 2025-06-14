# TCP Tokio Listener

A simple TCP server implementation using Tokio that demonstrates Rust best practices for asynchronous networking.

## Features

- Asynchronous TCP server using Tokio
- Concurrent connection handling
- Echo server functionality
- Proper error handling
- Clean connection lifecycle management
- Configurable IP address and port via command-line arguments or environment variables

## Usage

### Running the Server

#### Default configuration (127.0.0.1:8080):
```bash
cargo run
```

#### Custom IP and port via command-line arguments:
```bash
cargo run -- --ip 0.0.0.0 --port 9090
```

#### Using short flags:
```bash
cargo run -- -i 192.168.1.100 -p 3000
```

#### Using environment variables:
```bash
BIND_IP=0.0.0.0 BIND_PORT=9090 cargo run
```

#### View help:
```bash
cargo run -- --help
```

### Configuration Options

- `--ip` / `-i`: IP address to bind to (default: `127.0.0.1`, env: `BIND_IP`)
- `--port` / `-p`: Port to bind to (default: `8080`, env: `BIND_PORT`)

Command-line arguments take precedence over environment variables.

### Testing the Server

You can test the server using various tools:

#### Using telnet:
```bash
telnet 127.0.0.1 8080
```

#### Using netcat:
```bash
nc 127.0.0.1 8080
```

#### Using curl:
```bash
curl telnet://127.0.0.1:8080
```

## How it Works

1. The server parses command-line arguments or environment variables for IP and port configuration
2. The server binds to the specified address and listens for incoming connections
3. For each new connection, a separate Tokio task is spawned to handle it concurrently
4. Each connection handler reads data from the client and echoes it back
5. Connections are properly closed when the client disconnects
6. Errors are handled gracefully and logged

## Architecture

The implementation follows Rust best practices:

- **Async/await**: Uses Tokio's async runtime for non-blocking I/O
- **Error handling**: Proper `Result` types and error propagation
- **Concurrency**: Each connection runs in its own task
- **Resource management**: Automatic cleanup of connections
- **Type safety**: Leverages Rust's type system for memory safety

## Dependencies

- `tokio`: Async runtime with full features enabled
- `anyhow`: Error handling and context
- `clap`: Command-line argument parsing with derive and env features