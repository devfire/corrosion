#!/bin/bash

echo "=== TCP Fault Injection Proxy - Packet Loss Demo ==="
echo

# Check if httpbin.org is reachable
echo "Testing connectivity to httpbin.org..."
if ! curl -s --max-time 5 https://httpbin.org/get > /dev/null; then
    echo "Warning: httpbin.org may not be reachable. Using localhost examples instead."
    DEST_HOST="127.0.0.1"
    DEST_PORT="80"
else
    DEST_HOST="httpbin.org"
    DEST_PORT="443"
fi

echo "Using destination: $DEST_HOST:$DEST_PORT"
echo

# Build the project
echo "Building the fault injection proxy..."
cargo build --release
if [ $? -ne 0 ]; then
    echo "Build failed!"
    exit 1
fi

echo
echo "=== Demo 1: Basic Packet Loss (10%) ==="
echo "Starting proxy with 10% packet loss on port 8080..."
echo "Command: ./target/release/fault-injection --dest-ip $DEST_HOST --dest-port $DEST_PORT --packet-loss-enabled --packet-loss-probability 0.1"
echo
echo "Test this with:"
echo "  curl -H 'Host: $DEST_HOST' http://127.0.0.1:8080/get"
echo "  python3 test_packet_loss.py --url http://127.0.0.1:8080 --requests 20"
echo
echo "Press Ctrl+C to stop and continue to next demo..."
./target/release/fault-injection --dest-ip $DEST_HOST --dest-port $DEST_PORT --packet-loss-enabled --packet-loss-probability 0.1 &
PROXY_PID=$!

# Wait for user to stop
wait $PROXY_PID 2>/dev/null || true

echo
echo "=== Demo 2: Burst Packet Loss ==="
echo "Starting proxy with burst packet loss (5 packets, 10% burst probability) on port 8081..."
echo "Command: ./target/release/fault-injection --ip 127.0.0.1 --port 8081 --dest-ip $DEST_HOST --dest-port $DEST_PORT --packet-loss-enabled --packet-loss-probability 0.05 --packet-loss-burst-size 5 --packet-loss-burst-probability 0.1"
echo
echo "Test this with:"
echo "  python3 test_packet_loss.py --url http://127.0.0.1:8081 --requests 30"
echo
echo "Press Ctrl+C to stop and continue to next demo..."
./target/release/fault-injection --ip 127.0.0.1 --port 8081 --dest-ip $DEST_HOST --dest-port $DEST_PORT --packet-loss-enabled --packet-loss-probability 0.05 --packet-loss-burst-size 5 --packet-loss-burst-probability 0.1 &
PROXY_PID=$!

# Wait for user to stop
wait $PROXY_PID 2>/dev/null || true

echo
echo "=== Demo 3: Combined Latency and Packet Loss ==="
echo "Starting proxy with 200ms latency + 15% packet loss on port 8082..."
echo "Command: ./target/release/fault-injection --ip 127.0.0.1 --port 8082 --dest-ip $DEST_HOST --dest-port $DEST_PORT --latency-enabled --latency-fixed-ms 200 --packet-loss-enabled --packet-loss-probability 0.15"
echo
echo "Test this with:"
echo "  python3 test_packet_loss.py --url http://127.0.0.1:8082 --requests 25"
echo "  python3 test_latency.py --url http://127.0.0.1:8082 --requests 25"
echo
echo "Press Ctrl+C to stop..."
./target/release/fault-injection --ip 127.0.0.1 --port 8082 --dest-ip $DEST_HOST --dest-port $DEST_PORT --latency-enabled --latency-fixed-ms 200 --packet-loss-enabled --packet-loss-probability 0.15 &
PROXY_PID=$!

# Wait for user to stop
wait $PROXY_PID 2>/dev/null || true

echo
echo "=== Demo Complete ==="
echo "The fault injection proxy now supports:"
echo "  ✓ Latency injection (fixed, random, probability-based)"
echo "  ✓ Packet loss simulation (basic and burst modes)"
echo "  ✓ Combined fault injection scenarios"
echo
echo "Use --help to see all available options:"
echo "./target/release/fault-injection --help"