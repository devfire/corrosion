# TCP Fault Injection Proxy

A transparent TCP proxy implementation using Tokio for fault injection testing and network analysis.

## Features

- **Asynchronous TCP proxy** using Tokio
- **Concurrent connection handling**
- **Transparent bidirectional data forwarding**
- **Latency fault injection** with configurable parameters
- **Probability-based fault injection**
- **Fixed and random latency injection**
- **Proper error handling**
- **Clean connection lifecycle management**
- **Configurable bind and destination addresses** via command-line arguments or environment variables

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

**Latency Fault Injection:**
- `--latency-enabled`: Enable latency injection (default: `false`)
- `--latency-fixed-ms`: Fixed latency to add in milliseconds (default: `0`)
- `--latency-random-ms`: Random latency range in format `min-max` (e.g., `100-500`)
- `--latency-probability`: Probability of applying latency, 0.0-1.0 (default: `1.0`)

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

## Fault Injection

### Latency Injection

The proxy supports latency fault injection to simulate network delays and test application resilience.

#### Basic Latency Injection

Add a fixed 500ms delay to all connections:
```bash
cargo run -- --dest-ip httpbin.org --dest-port 443 --latency-enabled --latency-fixed-ms 500
```

#### Random Latency Injection

Add a random delay between 100-300ms to all connections:
```bash
cargo run -- --dest-ip httpbin.org --dest-port 443 --latency-enabled --latency-random-ms 100-300
```

#### Combined Fixed and Random Latency

Add 500ms fixed delay plus 100-300ms random delay:
```bash
cargo run -- --dest-ip httpbin.org --dest-port 443 --latency-enabled --latency-fixed-ms 500 --latency-random-ms 100-300
```

#### Probability-Based Latency Injection

Apply 1000ms delay to only 50% of connections:
```bash
cargo run -- --dest-ip httpbin.org --dest-port 443 --latency-enabled --latency-fixed-ms 1000 --latency-probability 0.5
```

#### Testing Latency Injection

Use the included test script to verify latency injection:
```bash
# Start the proxy with latency injection
cargo run -- --dest-ip httpbin.org --dest-port 443 --latency-enabled --latency-fixed-ms 500 --latency-random-ms 100-300

# In another terminal, run the test
python3 test_latency.py
```

Expected output should show response times in the 600-800ms range (500ms fixed + 100-300ms random + network overhead).
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
4. **Fault injection is applied** (if enabled) before establishing the destination connection
5. Each connection handler establishes a connection to the destination server
6. Data is transparently forwarded bidirectionally between client and destination
7. Connections are properly closed when either side disconnects
8. Errors are handled gracefully and logged with connection details

### Fault Injection Process

When latency injection is enabled:

1. **Connection established**: Client connects to the proxy
2. **Probability check**: If probability < 1.0, a random check determines if latency should be applied
3. **Latency calculation**:
   - Start with fixed latency (if configured)
   - Add random latency from specified range (if configured)
4. **Delay application**: The calculated delay is applied using `tokio::time::sleep`
5. **Destination connection**: After the delay, connection to the destination server is established
6. **Normal proxying**: Data flows transparently between client and destination

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
- `rand`: Random number generation for fault injection
- `tracing`: Structured logging
- `tracing-subscriber`: Log formatting and filtering

## Future Enhancements

The fault injection framework is designed to be extensible. Planned features include:

- **Packet loss injection**: Drop packets at configurable rates
- **Bandwidth throttling**: Limit connection throughput
- **Connection drops**: Randomly terminate connections
- **Jitter injection**: Add variable delays to simulate network instability
- **Protocol-specific faults**: HTTP error injection, DNS failures, etc.
- **Configuration files**: YAML/JSON configuration support
- **Metrics and monitoring**: Export fault injection statistics