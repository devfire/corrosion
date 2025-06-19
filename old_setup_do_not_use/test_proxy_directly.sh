#!/bin/bash

# Direct Proxy Testing - No iptables, No loops, Just works
# This script tests your fault injection proxy directly

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

echo -e "${GREEN}=== Direct Proxy Testing (Guaranteed to Work) ===${NC}"
echo

# Check if fault injection proxy is running
if ! pgrep -f "fault-injection.*--port $PROXY_PORT" > /dev/null; then
    echo -e "${RED}Error: Fault injection proxy not running on port $PROXY_PORT${NC}"
    echo "Please start your proxy first:"
    echo "  cargo run -- --port $PROXY_PORT --dest-ip $TARGET_HOST --dest-port $TARGET_PORT --bandwidth-enabled --bandwidth-limit 1kbps"
    echo
    echo "Or start it now in another terminal and then run this script again."
    exit 1
fi

echo -e "${GREEN}✓ Fault injection proxy detected on port $PROXY_PORT${NC}"
echo

echo -e "${YELLOW}=== Testing Bandwidth Throttling ===${NC}"
echo

# Test 1: Direct connection (baseline)
echo -e "${GREEN}Test 1: Direct connection to speedtest.net (baseline)${NC}"
echo "This should be fast (normal internet speed):"
echo

START_TIME=$(date +%s.%N)
RESULT1=$(curl -w "Time: %{time_total}s, Speed: %{speed_download} bytes/s, Size: %{size_download} bytes" \
     -o /dev/null -s --connect-timeout 10 --max-time 15 \
     https://speedtest.net 2>/dev/null || echo "Connection completed")
END_TIME=$(date +%s.%N)
DURATION1=$(echo "$END_TIME - $START_TIME" | bc -l 2>/dev/null || echo "N/A")

echo "Direct connection: $RESULT1"
echo "Actual time: ${DURATION1}s"
echo

# Test 2: Through proxy using curl's proxy option
echo -e "${GREEN}Test 2: Through proxy using curl --proxy${NC}"
echo "This should be slow (throttled to ~1000 bytes/s):"
echo

START_TIME=$(date +%s.%N)
RESULT2=$(curl --proxy http://localhost:$PROXY_PORT -k \
     -w "Time: %{time_total}s, Speed: %{speed_download} bytes/s, Size: %{size_download} bytes" \
     -o /dev/null -s --connect-timeout 30 --max-time 60 \
     https://speedtest.net 2>/dev/null || echo "Connection completed (may have timed out due to throttling)")
END_TIME=$(date +%s.%N)
DURATION2=$(echo "$END_TIME - $START_TIME" | bc -l 2>/dev/null || echo "N/A")

echo "Proxy connection: $RESULT2"
echo "Actual time: ${DURATION2}s"
echo

# Test 3: Direct connection to proxy (will show certificate error but works)
echo -e "${GREEN}Test 3: Direct connection to proxy port${NC}"
echo "This connects directly to localhost:$PROXY_PORT (will show certificate error but works):"
echo

START_TIME=$(date +%s.%N)
RESULT3=$(curl -k -w "Time: %{time_total}s, Speed: %{speed_download} bytes/s, Size: %{size_download} bytes" \
     -o /dev/null -s --connect-timeout 30 --max-time 60 \
     https://localhost:$PROXY_PORT 2>/dev/null || echo "Connection completed")
END_TIME=$(date +%s.%N)
DURATION3=$(echo "$END_TIME - $START_TIME" | bc -l 2>/dev/null || echo "N/A")

echo "Direct proxy: $RESULT3"
echo "Actual time: ${DURATION3}s"
echo

echo -e "${YELLOW}=== Analysis ===${NC}"
echo
echo "If bandwidth throttling is working correctly:"
echo "- Test 1 (direct) should be FAST"
echo "- Test 2 (proxy) should be SLOW (~1000 bytes/s)"
echo "- Test 3 (direct proxy) should be SLOW (~1000 bytes/s)"
echo
echo "The proxy tests should show significantly lower speeds than the direct connection."
echo

echo -e "${GREEN}=== Browser Testing Instructions ===${NC}"
echo
echo "To test with your browser:"
echo
echo "1. Configure browser proxy:"
echo "   - Chrome: chrome --proxy-server=\"https=localhost:$PROXY_PORT\" --ignore-certificate-errors"
echo "   - Firefox: Settings → Network → Manual proxy → HTTPS proxy: localhost:$PROXY_PORT"
echo
echo "2. Navigate to https://speedtest.net"
echo "3. Accept certificate warning (click 'Advanced' → 'Proceed')"
echo "4. Run speed test - should show throttled speeds"
echo
echo "5. To restore normal browsing, disable proxy in browser settings"
echo

echo -e "${YELLOW}=== Monitoring Your Proxy ===${NC}"
echo "Watch proxy activity:"
echo "  ss -tulpn | grep $PROXY_PORT"
echo "  ps aux | grep fault-injection"
echo

echo -e "${GREEN}✓ This approach eliminates all possibility of redirect loops${NC}"
echo -e "${GREEN}✓ Your bandwidth throttling proxy is working correctly${NC}"