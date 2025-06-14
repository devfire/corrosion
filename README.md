# TCP Fault Injection Proxy

A transparent TCP proxy implementation using Tokio for fault injection testing and network analysis.

## Features

- **Asynchronous TCP proxy** using Tokio
- **Concurrent connection handling**
- **Transparent bidirectional data forwarding**
- **Latency fault injection** with configurable parameters
- **Packet loss simulation** with burst mode support
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

**Packet Loss Simulation:**
- `--packet-loss-enabled`: Enable packet loss simulation (default: `false`)
- `--packet-loss-probability`: Probability of dropping packets, 0.0-1.0 (default: `0.0`)
- `--packet-loss-burst-size`: Number of consecutive packets to drop in burst mode
- `--packet-loss-burst-probability`: Probability of entering burst mode, 0.0-1.0 (default: `0.0`)

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

### Packet Loss Simulation

The proxy supports packet loss simulation to test application resilience to network packet drops.

#### Basic Packet Loss

Drop 10% of packets randomly:
```bash
cargo run -- --dest-ip httpbin.org --dest-port 443 --packet-loss-enabled --packet-loss-probability 0.1
```

#### Burst Packet Loss

Drop packets in bursts of 3 consecutive packets, with 5% chance of entering burst mode:
```bash
cargo run -- --dest-ip httpbin.org --dest-port 443 --packet-loss-enabled --packet-loss-probability 0.05 --packet-loss-burst-size 3 --packet-loss-burst-probability 0.05
```

#### Combined Latency and Packet Loss

Add 200ms latency and 15% packet loss:
```bash
cargo run -- --dest-ip httpbin.org --dest-port 443 --latency-enabled --latency-fixed-ms 200 --packet-loss-enabled --packet-loss-probability 0.15
```

#### Testing Packet Loss Simulation

Use the included test script to verify packet loss simulation:
```bash
# Start the proxy with packet loss simulation
cargo run -- --dest-ip httpbin.org --dest-port 443 --packet-loss-enabled --packet-loss-probability 0.2

# In another terminal, run the test
python3 test_packet_loss.py
```

Expected output should show approximately 20% of requests failing due to packet loss.

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
4. **Fault injection is applied** (if enabled) before and during connection handling
5. Each connection handler establishes a connection to the destination server
6. Data is forwarded bidirectionally with fault injection applied to each data packet
7. Connections are properly closed when either side disconnects
8. Errors are handled gracefully and logged with connection details

### Fault Injection Process

#### Latency Injection

When latency injection is enabled:

1. **Connection established**: Client connects to the proxy
2. **Probability check**: If probability < 1.0, a random check determines if latency should be applied
3. **Latency calculation**:
   - Start with fixed latency (if configured)
   - Add random latency from specified range (if configured)
4. **Delay application**: The calculated delay is applied using `tokio::time::sleep`
5. **Destination connection**: After the delay, connection to the destination server is established
6. **Normal proxying**: Data flows transparently between client and destination

#### Packet Loss Simulation

When packet loss simulation is enabled:

1. **Data packet received**: For each chunk of data received from either direction
2. **Packet loss check**: Random probability check determines if packet should be dropped
3. **Burst mode handling**:
   - If burst mode is configured, check for entering burst mode
   - In burst mode, drop consecutive packets until burst size is reached
4. **Packet forwarding**: If packet is not dropped, forward it to the destination
5. **Logging**: All packet drops are logged with connection details

The packet loss simulation operates at the application data level, simulating the effect of network packet loss on the data stream.

## How Latency Injection Works in Detail

This section provides a comprehensive explanation of the latency injection implementation and mechanics.

### Configuration Structure

The latency injection is configured through the [`LatencyConfig`](src/fault_injection.rs:8) struct:

```rust
pub struct LatencyConfig {
    pub enabled: bool,           // Whether latency injection is active
    pub fixed_ms: u64,          // Fixed delay in milliseconds
    pub random_range: Option<(u64, u64)>, // Optional random delay range
    pub probability: f64,        // Probability of applying latency (0.0-1.0)
}
```

### Latency Application Process

The latency injection happens in the [`apply_latency()`](src/fault_injection.rs:73) method following this flow:

1. **Check if enabled**: If latency injection is disabled, skip entirely
2. **Probability check**: Generate random number (0.0-1.0) and compare to configured probability
3. **Calculate delay**: Combine fixed delay + random delay (if configured)
4. **Apply delay**: Use `tokio::time::sleep()` for non-blocking delay
5. **Continue**: Proceed with normal connection handling

### Delay Calculation

The [`calculate_delay()`](src/fault_injection.rs:100) method combines two types of delays:

- **Fixed Delay**: A constant delay specified in `fixed_ms`
- **Random Delay**: An optional random component within a specified range

```rust
fn calculate_delay(&mut self) -> u64 {
    let mut total_delay = self.latency_config.fixed_ms;
    
    if let Some((min, max)) = self.latency_config.random_range {
        let random_delay = self.rng.gen_range(min..=max);
        total_delay += random_delay;
    }
    
    total_delay
}
```

### When Latency is Applied

The key insight is **when** the latency is applied. Looking at [`handle_connection()`](src/main.rs:95), the latency is injected at line 109:

```rust
// Apply latency before connecting to destination
fault_injector.apply_latency(&connection_id).await;

// Connect to the destination server
let mut outbound = match TcpStream::connect(&dest_addr).await {
    // ... connection logic
}
```

This means the latency is applied **before establishing the connection to the destination server**, simulating network delays that would occur during connection establishment.

### Probability-Based Injection

Not every connection experiences latency. The system uses a probability check:

1. Generate a random number between 0.0 and 1.0
2. If the random number ≤ configured probability, apply latency
3. Otherwise, skip latency injection for this connection

This allows for realistic simulation where only some connections are affected by network delays.

### Real-World Example

From the test file, you can see an example configuration:
- **Fixed delay**: 500ms
- **Random range**: 100-300ms
- **Total expected latency**: 600-800ms per connection

The [`test_latency.py`](test_latency.py:1) script demonstrates this by:
1. Making HTTP requests through the proxy
2. Measuring response times
3. Showing the added latency in action

### Implementation Details

- **Async/Await**: Uses [`tokio::time::sleep()`](src/fault_injection.rs:96) for non-blocking delays
- **Per-Connection**: Each connection gets its own [`FaultInjector`](src/fault_injection.rs:53) instance
- **Logging**: Comprehensive logging shows when latency is applied and skipped
- **Thread-Safe**: Uses [`StdRng`](src/fault_injection.rs:56) for random number generation

### Key Characteristics

- **Connection-Level**: Latency is applied once per connection, not per packet
- **Transparent**: The client sees normal TCP behavior, just slower
- **Configurable**: All parameters (delay, randomness, probability) are adjustable
- **Realistic**: Simulates real network conditions where some connections are slower than others

This design effectively simulates network latency conditions that applications might encounter in production, making it useful for testing how systems handle slow or unreliable network connections.

## Architecture

The implementation follows Rust best practices:

- **Async/await**: Uses Tokio's async runtime for non-blocking I/O
- **Error handling**: Proper `Result` types and error propagation
- **Concurrency**: Each proxy connection runs in its own task
- **Bidirectional forwarding**: Uses custom bidirectional copy with fault injection support
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

- **Bandwidth throttling**: Limit connection throughput
- **Connection drops**: Randomly terminate connections
- **Jitter injection**: Add variable delays to simulate network instability
- **Protocol-specific faults**: HTTP error injection, DNS failures, etc.
- **Configuration files**: YAML/JSON configuration support
- **Metrics and monitoring**: Export fault injection statistics
- **Advanced packet loss patterns**: Periodic loss, Gilbert-Elliott model