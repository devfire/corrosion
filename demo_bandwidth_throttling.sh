#!/bin/bash

# Demo script for bandwidth throttling functionality

echo "=== Bandwidth Throttling Demo ==="
echo

# Start a simple HTTP server in the background for testing
echo "Starting test HTTP server on port 8000..."
python3 -m http.server 8000 &
SERVER_PID=$!

# Give the server time to start
sleep 2

echo "Starting fault injection proxy with bandwidth throttling..."
echo "  - Proxy listening on: 127.0.0.1:8080"
echo "  - Forwarding to: 127.0.0.1:8000"
echo "  - Bandwidth limit: 10 KB/s"
echo "  - Burst size: 4096 bytes"
echo

# Start the fault injection proxy with bandwidth throttling
cargo run -- \
    --dest-ip 127.0.0.1 \
    --dest-port 8000 \
    --bandwidth-enabled \
    --bandwidth-limit-kbps 10 \
    --bandwidth-burst-size 4096 &
PROXY_PID=$!

# Give the proxy time to start
sleep 2

echo "Testing bandwidth throttling..."
echo "You can now test the bandwidth throttling by:"
echo "1. Opening http://127.0.0.1:8080 in your browser"
echo "2. Or using curl: curl -o /dev/null http://127.0.0.1:8080"
echo "3. Or using wget: wget -O /dev/null http://127.0.0.1:8080"
echo
echo "The download should be throttled to approximately 10 KB/s"
echo
echo "Press Ctrl+C to stop the demo"

# Wait for user to interrupt
trap "echo; echo 'Stopping demo...'; kill $PROXY_PID $SERVER_PID 2>/dev/null; exit 0" INT
wait