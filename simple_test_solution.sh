#!/bin/bash

# Simple Test Solution - No iptables, No loops, Just works
# This script provides a foolproof way to test bandwidth throttling

set -e

# Configuration
PROXY_PORT=8080
TARGET_HOST="speedtest.net"
TARGET_PORT=443

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Simple Bandwidth Throttling Test ===${NC}"
echo

# Check if fault injection proxy is running
if ! pgrep -f "fault-injection.*--port $PROXY_PORT" > /dev/null; then
    echo -e "${RED}Error: Fault injection proxy not running on port $PROXY_PORT${NC}"
    echo "Please start your proxy first:"
    echo "  cargo run -- --port $PROXY_PORT --dest-ip $TARGET_HOST --dest-port $TARGET_PORT --bandwidth-enabled --bandwidth-limit 1kbps"
    exit 1
fi

echo -e "${GREEN}✓ Fault injection proxy detected on port $PROXY_PORT${NC}"
echo

echo -e "${YELLOW}=== Testing Bandwidth Throttling ===${NC}"
echo

# Method 1: Direct connection to proxy (will show certificate error but works)
echo -e "${GREEN}Method 1: Direct proxy connection${NC}"
echo "Testing: curl -k https://localhost:$PROXY_PORT"
echo "Expected: Slow download due to 1kbps throttling"
echo

# Test with a small file to avoid timeout
echo "Downloading small test file through proxy..."
START_TIME=$(date +%s)
curl -k -w "Time: %{time_total}s, Speed: %{speed_download} bytes/s, Size: %{size_download} bytes\n" \
     -o /dev/null -s --connect-timeout 30 --max-time 60 \
     https://localhost:$PROXY_PORT 2>/dev/null || echo "Connection completed (may have timed out due to throttling)"
END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

echo "Test completed in ${DURATION} seconds"
echo

# Method 2: Test without proxy for comparison
echo -e "${GREEN}Method 2: Direct connection (no proxy)${NC}"
echo "Testing: curl https://speedtest.net (for comparison)"
echo

START_TIME=$(date +%s)
curl -w "Time: %{time_total}s, Speed: %{speed_download} bytes/s, Size: %{size_download} bytes\n" \
     -o /dev/null -s --connect-timeout 10 --max-time 30 \
     https://speedtest.net 2>/dev/null || echo "Direct connection completed"
END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

echo "Direct test completed in ${DURATION} seconds"
echo

echo -e "${YELLOW}=== Analysis ===${NC}"
echo "If bandwidth throttling is working:"
echo "- Method 1 (through proxy) should be MUCH slower"
echo "- Method 2 (direct) should be normal speed"
echo "- The proxy connection should show very low bytes/s (around 1000 bytes/s = 1kbps)"
echo

echo -e "${GREEN}=== Alternative Testing Methods ===${NC}"
echo
echo "1. Browser test with certificate override:"
echo "   - Navigate to: https://localhost:$PROXY_PORT"
echo "   - Click 'Advanced' -> 'Proceed to localhost (unsafe)'"
echo "   - Page should load very slowly due to throttling"
echo
echo "2. Use the DNS redirect method (recommended):"
echo "   - Run: sudo ./setup_dns_redirect.sh"
echo "   - Then navigate to: https://speedtest.net"
echo "   - This avoids iptables complexity entirely"
echo

echo -e "${YELLOW}=== Monitoring Your Proxy ===${NC}"
echo "Watch proxy activity:"
echo "  ss -tulpn | grep $PROXY_PORT"
echo "  ps aux | grep fault-injection"
echo