# TCP Fault Injection Proxy

A transparent TCP proxy implementation using Tokio for fault injection testing and network analysis.

## Features

- **Asynchronous TCP proxy** using Tokio
- **Concurrent connection handling**
- **Transparent bidirectional data forwarding**
- **Latency fault injection** with configurable parameters
- **Packet loss simulation** with burst mode support
- **Bandwidth throttling** with token bucket algorithm
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

**Bandwidth Throttling:**
- `--bandwidth-enabled`: Enable bandwidth throttling (default: `false`)
- `--bandwidth-limit`: Bandwidth limit with unit (e.g., "100kbps", "1mbps", "50000bps", "0" = unlimited) (default: `0`)
- `--bandwidth-burst-size`: Burst size in bytes for token bucket algorithm (default: `8192`)

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

Expected output should show significantly higher response times due to per-packet latency injection. With 500ms fixed + 100-300ms random per packet, a typical HTTPS request with multiple packets will show cumulative delays (e.g., 4-5 seconds for 8-10 packets).

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

### Bandwidth Throttling

The proxy supports bandwidth throttling to limit connection throughput and simulate slow network conditions.

#### Basic Bandwidth Throttling

NOTE: Must run as `proxy-injector` in order for `iptables` transparent proxy to work.

Limit bandwidth to 1MBps:
```bash
sudo -u proxy-injector ./target/release/fault-injection --port 8080 --dest-ip rlgncook.speedtest.sbcglobal.net --dest-port 8080 --bandwidth-enabled --bandwidth-limit 1mbps
```

#### Advanced Bandwidth Configuration

Configure bandwidth with custom burst size (allows temporary bursts above the limit):
```bash
cargo run -- --dest-ip httpbin.org --dest-port 443 --bandwidth-enabled --bandwidth-limit 50kbps --bandwidth-burst-size 16384
```

#### Combined Fault Injection

Combine bandwidth throttling with latency and packet loss:
```bash
cargo run -- --dest-ip httpbin.org --dest-port 443 \
  --bandwidth-enabled --bandwidth-limit 100kbps \
  --latency-enabled --latency-fixed-ms 100 \
  --packet-loss-enabled --packet-loss-probability 0.05
```

#### Testing Bandwidth Throttling

First, install speedtest:
```bash
snap install speedtest
```

Then, get the closest server ID:
```bash
speedtest -L
Closest servers:

    ID  Name                           Location             Country
==============================================================================
 **67937**  AT&T                           Raleigh, NC          United States
 29113  Duke University                Durham, NC           United States
 58326  Spectrum                       Durham, NC           United States
 14774  UNC Chapel Hill                Chapel Hill, NC      United States
  5401  Optimum Online                 Rocky Mount, NC      United States
 53507  Brightspeed                    Rocky Mount, NC      United States
 48128  Metronet                       Fayetteville, NC     United States
 69565  Brightspeed                    Fayetteville, NC     United States

```

Then, run the test:
```bash
speedtest -s 67937 -vv
```

Or test manually:
```bash
# Start the proxy with bandwidth throttling
cargo run -- --dest-ip httpbin.org --dest-port 443 --bandwidth-enabled --bandwidth-limit 10kbps

# In another terminal, test download speed
curl -o /dev/null -w "%{speed_download}\n" http://127.0.0.1:8080/bytes/1048576
```

Expected output should show download speeds limited to approximately the configured bandwidth limit.

#### How Bandwidth Throttling Works

The bandwidth throttling implementation uses a **token bucket algorithm**:

1. **Token Bucket**: A bucket holds tokens representing available bandwidth
2. **Token Refill**: Tokens are added to the bucket at the configured rate (bytes per second)
3. **Token Consumption**: Each data packet consumes tokens equal to its size
4. **Throttling**: If insufficient tokens are available, the connection waits until enough tokens are refilled
5. **Burst Handling**: The bucket size allows temporary bursts above the sustained rate

This provides smooth bandwidth limiting while allowing natural traffic bursts within the configured limits.

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
4. **Fault injection is applied** (if enabled) during connection handling
5. Each connection handler establishes a connection to the destination server
6. Data is forwarded bidirectionally with **per-packet fault injection** applied to each data packet
7. Connections are properly closed when either side disconnects
8. Errors are handled gracefully and logged with connection details

### Fault Injection Process

#### Latency Injection

When latency injection is enabled:

1. **Connection established**: Client connects to the proxy and destination connection is established
2. **Per-packet processing**: For each data packet received from either direction:
   - **Probability check**: If probability < 1.0, a random check determines if latency should be applied to this packet
   - **Latency calculation**:
     - Start with fixed latency (if configured)
     - Add random latency from specified range (if configured)
   - **Delay application**: The calculated delay is applied using `tokio::time::sleep`
   - **Packet forwarding**: After the delay, the packet is forwarded to its destination
3. **Cumulative effect**: Multiple packets result in cumulative latency, simulating realistic per-packet network delays

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

The key insight is **when** the latency is applied. Looking at [`copy_bidirectional_with_faults()`](src/main.rs:172), the latency is injected for each packet in both directions:

```rust
// For client->server packets
if fault_injector.should_drop_packet(connection_id) {
    continue; // Skip dropped packets
}
fault_injector.apply_latency(connection_id).await; // Apply latency per packet
match b.write_all(&buf_a[..n]).await { ... }

// For server->client packets
if fault_injector.should_drop_packet(connection_id) {
    continue; // Skip dropped packets
}
fault_injector.apply_latency(connection_id).await; // Apply latency per packet
match a.write_all(&buf_b[..n]).await { ... }
```

**Important**: The latency is applied **per packet** during data transfer, not just during connection establishment.

This simulates realistic network conditions where each packet experiences:
- Network routing delays
- Transmission delays
- Processing delays at network nodes
- Variable network conditions affecting individual packets

Each data chunk (up to 8KB buffer size) is treated as a packet and experiences the configured latency independently.

### Probability-Based Injection

Not every connection experiences latency. The system uses a probability check:

1. Generate a random number between 0.0 and 1.0
2. If the random number ≤ configured probability, apply latency
3. Otherwise, skip latency injection for this connection

This allows for realistic simulation where only some connections are affected by network delays.

### Real-World Example

From the test file, you can see an example configuration:
- **Fixed delay**: 500ms per packet
- **Random range**: 100-300ms per packet
- **Total expected latency**: 600-800ms × number of packets (typically 4-5 seconds for HTTPS requests with 8-10 packets)

The [`test_latency.py`](test_latency.py:1) script demonstrates this by:
1. Making HTTPS requests through the proxy
2. Measuring total response times
3. Showing the cumulative per-packet latency in action

For example, with 100ms per packet, a typical HTTPS request shows ~830ms total latency, indicating approximately 8-9 packets were processed.

### Implementation Details

- **Async/Await**: Uses [`tokio::time::sleep()`](src/fault_injection.rs:96) for non-blocking delays
- **Per-Connection**: Each connection gets its own [`FaultInjector`](src/fault_injection.rs:53) instance
- **Logging**: Comprehensive logging shows when latency is applied and skipped
- **Thread-Safe**: Uses [`StdRng`](src/fault_injection.rs:56) for random number generation

### Key Characteristics

- **Per-Packet Injection**: Latency is applied **to each individual packet** during data transfer, not just during connection establishment
- **Bidirectional**: Both client→server and server→client packets experience latency independently
- **Cumulative Effect**: Multiple packets result in cumulative delays, providing realistic network simulation
- **Configurable**: All parameters (delay, randomness, probability) are adjustable per packet
- **Realistic Network Simulation**: Simulates real network conditions where each packet experiences variable delays

This design is particularly useful for testing how applications handle:
- Slow data transfer rates
- Variable network conditions
- Per-packet network delays
- Cumulative latency effects
- Real-world network behavior where individual packets experience delays

The per-packet approach provides much more realistic network latency simulation compared to connection-level delays only.

## How Bandwidth Throttling Works in Detail

This section provides a comprehensive explanation of the bandwidth throttling implementation and mechanics.

### Configuration Structure

The bandwidth throttling is configured through the [`BandwidthConfig`](src/fault_injection.rs:24) struct:

```rust
pub struct BandwidthConfig {
    pub enabled: bool,      // Whether bandwidth throttling is active
    pub limit_bps: u64,     // Maximum bandwidth in bytes per second
    pub burst_size: u64,    // Burst size in bytes for token bucket
}
```

### Token Bucket Algorithm Implementation

The bandwidth throttling uses a **token bucket algorithm** implemented in the [`FaultInjector`](src/fault_injection.rs:74) struct with these key state variables:

```rust
pub struct FaultInjector {
    // ... other fields ...
    bandwidth_tokens: f64,    // Current available tokens (bytes)
    last_refill: Instant,     // Last time tokens were refilled
}
```

### Token Bucket Mechanics

The [`apply_bandwidth_throttling()`](src/fault_injection.rs:180) method implements the core token bucket logic:

#### 1. Token Refill Process

```rust
let now = Instant::now();
let elapsed = now.duration_since(self.last_refill).as_secs_f64();

// Refill tokens based on elapsed time
let tokens_to_add = elapsed * self.bandwidth_config.limit_bps as f64;
self.bandwidth_tokens = (self.bandwidth_tokens + tokens_to_add)
    .min(self.bandwidth_config.burst_size as f64);
```

**Key Points:**
- Tokens are refilled continuously based on elapsed time
- Refill rate = configured bandwidth limit (bytes per second)
- Token bucket capacity = configured burst size
- Tokens cannot exceed the burst size limit

#### 2. Token Consumption Process

For each data packet, the system checks token availability:

```rust
let bytes_needed = bytes as f64;

if self.bandwidth_tokens >= bytes_needed {
    // Sufficient tokens available - consume immediately
    self.bandwidth_tokens -= bytes_needed;
    // Packet forwarded without delay
} else {
    // Insufficient tokens - calculate required delay
    let tokens_deficit = bytes_needed - self.bandwidth_tokens;
    let delay_seconds = tokens_deficit / self.bandwidth_config.limit_bps as f64;
    let delay_ms = (delay_seconds * 1000.0) as u64;
    
    // Apply delay and consume all available tokens
    sleep(Duration::from_millis(delay_ms)).await;
    self.bandwidth_tokens = 0.0;
}
```

### When Bandwidth Throttling is Applied

Looking at [`copy_bidirectional_with_faults()`](src/main.rs:172), bandwidth throttling is applied **per packet** in both directions:

```rust
// For client->server packets
let n = a.read(&mut buf_a).await?;
fault_injector.apply_bandwidth_throttling(n, connection_id).await; // Throttle before forwarding
match b.write_all(&buf_a[..n]).await { ... }

// For server->client packets
let n = b.read(&mut buf_b).await?;
fault_injector.apply_bandwidth_throttling(n, connection_id).await; // Throttle before forwarding
match a.write_all(&buf_b[..n]).await { ... }
```

**Important**: Bandwidth throttling is applied **per packet** during data transfer, affecting each data chunk independently.

### Burst Behavior Explained

The token bucket algorithm allows for **burst traffic** within limits:

#### Normal Operation
- Steady-state traffic flows at the configured rate limit
- Small packets consume tokens and are forwarded immediately
- Token bucket refills continuously at the configured rate

#### Burst Scenarios
- **Burst Allowance**: If no traffic occurs for a period, tokens accumulate up to `burst_size`
- **Burst Consumption**: Large packets or rapid packet sequences can consume accumulated tokens
- **Burst Exhaustion**: Once burst tokens are consumed, traffic returns to steady-state rate limiting

#### Example Burst Calculation
With configuration: `--bandwidth-limit 100kbps --bandwidth-burst-size 16384`

- **Steady rate**: 100 KB/s (102,400 bytes/second)
- **Burst capacity**: 16,384 bytes
- **Burst duration**: ~0.16 seconds of full-rate traffic above the limit
- **Recovery time**: ~0.16 seconds to refill burst capacity

### Delay Calculation Details

When insufficient tokens are available, the delay is calculated as:

```rust
delay_seconds = tokens_deficit / bandwidth_limit_bps
```

**Example**:
- Packet size: 8,192 bytes
- Available tokens: 2,048 bytes
- Bandwidth limit: 10,240 bytes/second (10 KB/s)
- Tokens deficit: 8,192 - 2,048 = 6,144 bytes
- Required delay: 6,144 ÷ 10,240 = 0.6 seconds (600ms)

### Real-World Performance Characteristics

#### Traffic Shaping Effects
- **Smooth traffic flow**: Prevents sudden bandwidth spikes
- **Burst accommodation**: Allows temporary bursts within configured limits
- **Fair queuing**: Each connection gets proportional bandwidth access
- **Latency impact**: Large packets may experience delays when tokens are insufficient

#### Typical Behavior Patterns
1. **Small packets** (< burst_size): Usually forwarded immediately
2. **Large packets** (> available tokens): Experience calculated delays
3. **Sustained traffic**: Settles into steady-state rate limiting
4. **Bursty traffic**: Benefits from token accumulation during idle periods

### Configuration Examples and Effects

#### Conservative Throttling
```bash
--bandwidth-limit 50kbps --bandwidth-burst-size 4096
```
- **Effect**: Tight bandwidth control with small burst allowance
- **Use case**: Simulating slow network connections
- **Behavior**: Frequent delays for packets > 4KB

#### Generous Burst Allowance
```bash
--bandwidth-limit 1mbps --bandwidth-burst-size 65536
```
- **Effect**: 1 MB/s average with 64KB burst capacity
- **Use case**: Simulating networks with good burst tolerance
- **Behavior**: Accommodates larger packets and traffic bursts

#### Strict Rate Limiting
```bash
--bandwidth-limit 10240bps --bandwidth-burst-size 1024
```
- **Effect**: Very tight control with minimal burst allowance
- **Use case**: Testing application behavior under severe bandwidth constraints
- **Behavior**: Most packets experience some delay

### Integration with Other Fault Types

Bandwidth throttling works in combination with other fault injection types:

```rust
// Order of fault injection per packet:
1. Packet loss check (may drop packet entirely)
2. Latency injection (adds delay before throttling)
3. Bandwidth throttling (adds delay based on available tokens)
4. Packet forwarding (actual data transmission)
```

This layered approach provides realistic network simulation where packets can experience:
- **Packet loss**: Complete packet drops
- **Latency**: Fixed/random delays simulating network routing
- **Bandwidth limits**: Rate-based delays simulating capacity constraints

### Implementation Details

- **Async/Await**: Uses [`tokio::time::sleep()`](src/fault_injection.rs:213) for non-blocking delays
- **Per-Connection**: Each connection gets its own token bucket state
- **High Precision**: Uses `f64` for token calculations to handle fractional tokens accurately
- **Time-Based**: Token refill based on actual elapsed time for accurate rate limiting
- **Logging**: Comprehensive debug logging shows token consumption and delays

### Key Characteristics

- **Per-Packet Throttling**: Bandwidth limiting is applied **to each individual packet** during data transfer
- **Bidirectional**: Both client→server and server→client packets are throttled independently
- **Token Bucket Algorithm**: Provides smooth rate limiting with configurable burst allowance
- **Precise Rate Control**: Maintains accurate long-term bandwidth limits
- **Burst Tolerance**: Allows temporary traffic bursts within configured limits
- **Real-Time Adaptation**: Continuously adjusts based on actual traffic patterns

This implementation provides realistic bandwidth simulation that closely mimics real network behavior, including:
- **Network capacity limits**: Simulating connection bandwidth constraints
- **Traffic shaping**: Smooth bandwidth utilization over time
- **Burst handling**: Natural accommodation of bursty traffic patterns
- **Quality of Service**: Predictable bandwidth allocation per connection

The per-packet token bucket approach provides much more realistic bandwidth throttling compared to simple connection-level rate limiting.

## Transparent Proxy Setup with iptables

The [`setup_iptables_dedicated_user.sh`](setup_iptables_dedicated_user.sh:1) script provides transparent proxy functionality using iptables rules. This allows intercepting and redirecting network traffic to the fault injection proxy without requiring application configuration changes.

### How Transparent Proxy Works

**The Problem**: When setting up a transparent proxy, a common issue is the **infinite redirection loop**:
1. Client sends traffic to destination server
2. iptables redirects traffic to local proxy
3. Proxy forwards traffic to original destination
4. iptables redirects the proxy's own traffic back to the proxy
5. **Infinite loop occurs**

**The Solution**: The script solves this by creating a **dedicated system user** for the proxy and configuring iptables to exclude traffic from that user, breaking the redirection loop.

### Script Configuration

The script uses these key configuration variables:

```bash
readonly PROXY_USER="proxy-injector"           # Dedicated user for the proxy
readonly PROXY_PORT="8080"                     # Local port proxy listens on
readonly TARGET_HOST="rlgncook.speedtest.sbcglobal.net"  # Target to intercept
readonly TARGET_PORT="8080"                    # Target port to intercept
```

### Detailed Operation Flow

#### 1. Environment Setup and Validation

**Root Privilege Check**: The script requires root privileges for iptables manipulation and user management:
```bash
if [[ $EUID -ne 0 ]]; then
   echo "[!] This script must be run as root. Aborting." >&2
   exit 1
fi
```

**Cleanup Trap Setup**: Registers a cleanup function to remove all iptables rules on script exit:
```bash
trap cleanup INT TERM EXIT
```

#### 2. Dedicated User Creation

**User Existence Check**: Verifies if the proxy user already exists:
```bash
if ! id -u "$PROXY_USER" >/dev/null 2>&1; then
    useradd --system --shell /usr/sbin/nologin "$PROXY_USER"
fi
```

**User Properties**:
- **System user**: Created with `--system` flag (UID < 1000)
- **No login shell**: Uses `/usr/sbin/nologin` to prevent interactive login
- **Security**: Dedicated user isolates proxy process and enables iptables exclusion

#### 3. DNS Resolution for IPv4 and IPv6

**Dual-Stack Resolution**: Resolves the target hostname to both IPv4 and IPv6 addresses:
```bash
ipv4_ips=($(getent ahostsv4 "$TARGET_HOST" | awk '{ print $1 }' | sort -u))
ipv6_ips=($(getent ahostsv6 "$TARGET_HOST" | awk '{ print $1 }' | sort -u))
```

**Why Both Protocols**: Modern networks use dual-stack configurations, so the script handles both IPv4 and IPv6 traffic to ensure complete interception.

#### 4. IP Forwarding Configuration

**Enable IP Forwarding**: Required for the system to forward packets between interfaces:
```bash
ORIGINAL_IP_FORWARD=$(cat /proc/sys/net/ipv4/ip_forward)
if [[ "$ORIGINAL_IP_FORWARD" != "1" ]]; then
    sysctl -w net.ipv4.ip_forward=1 >/dev/null
fi
```

**State Preservation**: Saves the original forwarding state to restore it during cleanup.

#### 5. iptables Rule Creation

**The Core iptables Rule**: For each resolved IP address, the script creates rules like:
```bash
iptables -t nat -A OUTPUT -p tcp -d $ip --dport $TARGET_PORT -m owner ! --uid-owner $proxy_uid -j REDIRECT --to-port $PROXY_PORT
```

**Rule Breakdown**:
- **`-t nat`**: Uses the NAT table for address translation
- **`-A OUTPUT`**: Appends to OUTPUT chain (outgoing traffic)
- **`-p tcp`**: Matches TCP protocol only
- **`-d $ip --dport $TARGET_PORT`**: Matches traffic to specific destination IP and port
- **`-m owner ! --uid-owner $proxy_uid`**: **Critical**: Excludes traffic from the proxy user (prevents loops)
- **`-j REDIRECT --to-port $PROXY_PORT`**: Redirects matching traffic to local proxy port

**IPv6 Support**: Identical rules are created using `ip6tables` for IPv6 addresses.

#### 6. Loop Prevention Mechanism

**The Key Insight**: The `! --uid-owner $proxy_uid` condition is what prevents infinite loops:

1. **Normal traffic flow**:
   - User application → Target server
   - iptables matches (not from proxy user) → Redirects to proxy
   - Proxy receives traffic

2. **Proxy forwarding**:
   - Proxy (running as proxy-injector user) → Target server
   - iptables checks owner → Traffic is from proxy user → **No redirection**
   - Traffic flows directly to target server

**Visual Flow**:
```
[Client App] ---> [iptables] ---> [Proxy:8080] ---> [Target Server]
                     ↑                ↓
                  Redirects        Runs as proxy-injector
                  (not proxy       (bypasses iptables)
                   user traffic)
```

#### 7. Rule Management and Cleanup

**Rule Tracking**: The script maintains an array of all added rules:
```bash
declare -a ADDED_RULES
ADDED_RULES+=("$full_command")
```

**Cleanup Process**: On script exit, rules are removed in reverse order:
```bash
for ((i=${#ADDED_RULES[@]}-1; i>=0; i--)); do
    local delete_command="${full_rule_command/-A/-D}"  # Replace -A with -D
    eval "$delete_command" 2>/dev/null
done
```

**State Restoration**: IP forwarding is restored to its original value.

### Usage Instructions

#### 1. Configure the Script
Edit the configuration variables in [`setup_iptables_dedicated_user.sh`](setup_iptables_dedicated_user.sh:17):
```bash
readonly PROXY_USER="proxy-injector"
readonly PROXY_PORT="8080"
readonly TARGET_HOST="your-target-host.com"
readonly TARGET_PORT="80"
```

#### 2. Run the Setup Script
```bash
sudo ./setup_iptables_dedicated_user.sh
```

The script will:
- Create the `proxy-injector` user if needed
- Resolve target host to IP addresses
- Add iptables rules for transparent redirection
- Display success message and wait

#### 3. Run the Proxy as the Dedicated User
```bash
sudo -u proxy-injector ./target/release/fault-injection --port 8080 --dest-ip your-target-host.com --dest-port 80
```

#### 4. Test the Transparent Proxy
Applications can now connect directly to the target host, and traffic will be transparently intercepted:
```bash
curl http://your-target-host.com/  # Traffic automatically goes through proxy
```

#### 5. Cleanup
Press `Ctrl+C` in the setup script terminal to cleanly remove all iptables rules and restore the original system state.

### Security Considerations

**Dedicated User Benefits**:
- **Process Isolation**: Proxy runs with minimal privileges
- **Loop Prevention**: Enables iptables owner-based exclusion
- **Security**: Reduces attack surface by using non-login system user

**Root Privileges**: Required only for:
- iptables rule manipulation
- System user creation
- IP forwarding configuration

**Network Impact**: The transparent proxy intercepts **all** traffic to the specified target, affecting all applications on the system.

### Troubleshooting

**Common Issues**:

1. **Permission Denied**: Ensure script is run with `sudo`
2. **User Creation Failed**: Check if `useradd` is available and system supports user creation
3. **DNS Resolution Failed**: Verify target hostname is resolvable and network connectivity exists
4. **iptables Rules Failed**: Check if iptables/ip6tables are installed and kernel supports NAT
5. **Proxy Connection Failed**: Ensure proxy is running as the correct user (`proxy-injector`)

**Verification Commands**:
```bash
# Check if user was created
id proxy-injector

# View active iptables rules
sudo iptables -t nat -L OUTPUT -v

# Check IP forwarding status
cat /proc/sys/net/ipv4/ip_forward

# Test DNS resolution
getent ahosts your-target-host.com
```

### Integration with Fault Injection

The transparent proxy setup enables **system-wide fault injection** without application modifications:

1. **Setup Phase**: Run [`setup_iptables_dedicated_user.sh`](setup_iptables_dedicated_user.sh:1) to configure transparent redirection
2. **Proxy Phase**: Run fault injection proxy as `proxy-injector` user with desired fault parameters
3. **Testing Phase**: Applications connect normally to target hosts, experiencing injected faults transparently

**Example Complete Workflow**:
```bash
# Terminal 1: Setup transparent proxy
sudo ./setup_iptables_dedicated_user.sh

# Terminal 2: Run fault injection proxy
sudo -u proxy-injector ./target/release/fault-injection \
  --port 8080 --dest-ip httpbin.org --dest-port 80 \
  --latency-enabled --latency-fixed-ms 500 \
  --packet-loss-enabled --packet-loss-probability 0.1

# Terminal 3: Test applications (traffic automatically intercepted)
curl http://httpbin.org/get  # Experiences 500ms latency + 10% packet loss
```

This transparent approach is particularly valuable for:
- **Legacy application testing**: No code changes required
- **System-level fault injection**: Affects all network traffic to target
- **Realistic testing**: Applications behave exactly as in production
- **Network simulation**: Simulates real network conditions transparently

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

- **Connection drops**: Randomly terminate connections
- **Jitter injection**: Add variable delays to simulate network instability
- **Protocol-specific faults**: HTTP error injection, DNS failures, etc.
- **Configuration files**: YAML/JSON configuration support
- **Metrics and monitoring**: Export fault injection statistics
- **Advanced packet loss patterns**: Periodic loss, Gilbert-Elliott model
- **Advanced bandwidth patterns**: Variable bandwidth, time-based throttlingsudo -u proxy-injector ./target/