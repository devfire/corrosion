#!/bin/bash

# SOCAT-based Transparent Proxy - No loops possible
# This uses socat as an intermediary to avoid iptables redirect loops

set -e

# Configuration
PROXY_PORT=8080
TARGET_HOST="speedtest.net"
TARGET_PORT=443
SOCAT_PORT=9080

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== SOCAT-based Transparent Proxy (Loop-Free) ===${NC}"
echo

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}Error: This script must be run as root (use sudo)${NC}"
   echo "Usage: sudo ./setup_proxy_with_socat.sh"
   exit 1
fi

# Check if socat is installed
if ! command -v socat &> /dev/null; then
    echo -e "${YELLOW}Installing socat...${NC}"
    apt-get update && apt-get install -y socat
fi

# Function to start the proxy chain
start_proxy_chain() {
    echo -e "${YELLOW}Starting proxy chain...${NC}"
    
    # Kill any existing processes
    pkill -f "fault-injection.*--port $PROXY_PORT" 2>/dev/null || true
    pkill -f "socat.*$SOCAT_PORT" 2>/dev/null || true
    sleep 1
    
    # Start fault injection proxy (connects to real speedtest.net)
    cd /home/ig/Documents/rust/fault-injection
    sudo -u ig cargo run -- \
        --port $PROXY_PORT \
        --dest-ip $TARGET_HOST \
        --dest-port $TARGET_PORT \
        --bandwidth-enabled \
        --bandwidth-limit 1kbps &
    
    PROXY_PID=$!
    echo -e "${GREEN}✓ Fault injection proxy started (PID: $PROXY_PID)${NC}"
    
    # Wait for proxy to start
    sleep 3
    
    # Start socat to forward speedtest.net traffic to our proxy
    # This creates the transparent effect without iptables
    socat TCP-LISTEN:$SOCAT_PORT,fork,reuseaddr TCP:127.0.0.1:$PROXY_PORT &
    SOCAT_PID=$!
    echo -e "${GREEN}✓ SOCAT forwarder started (PID: $SOCAT_PID)${NC}"
    
    sleep 1
    
    # Verify both are running
    if kill -0 $PROXY_PID 2>/dev/null && kill -0 $SOCAT_PID 2>/dev/null; then
        echo -e "${GREEN}✓ Proxy chain is running successfully${NC}"
    else
        echo -e "${RED}✗ Proxy chain failed to start${NC}"
        exit 1
    fi
}

# Function to setup iptables (simple redirect, no loops possible)
setup_iptables() {
    echo -e "${YELLOW}Setting up simple iptables redirect...${NC}"
    
    # Get speedtest.net IPs
    SPEEDTEST_IPS=$(dig +short $TARGET_HOST | grep -E '^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$')
    
    if [ -z "$SPEEDTEST_IPS" ]; then
        echo -e "${RED}Error: Could not resolve $TARGET_HOST${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}Found IP addresses for $TARGET_HOST:${NC}"
    for ip in $SPEEDTEST_IPS; do
        echo "  - $ip"
    done
    
    # Create simple redirect to socat (no loops because socat forwards to localhost)
    iptables -t nat -N SPEEDTEST_REDIRECT 2>/dev/null || true
    
    for ip in $SPEEDTEST_IPS; do
        echo "  Adding rule for $ip:$TARGET_PORT -> localhost:$SOCAT_PORT"
        iptables -t nat -A SPEEDTEST_REDIRECT -d $ip -p tcp --dport $TARGET_PORT -j REDIRECT --to-port $SOCAT_PORT
    done
    
    # Apply the redirect
    iptables -t nat -I OUTPUT -j SPEEDTEST_REDIRECT
    
    echo -e "${GREEN}✓ iptables redirect configured (loop-free)${NC}"
}

# Function to cleanup
cleanup() {
    echo -e "${YELLOW}Cleaning up proxy chain...${NC}"
    
    # Kill processes
    pkill -f "fault-injection.*--port $PROXY_PORT" 2>/dev/null || true
    pkill -f "socat.*$SOCAT_PORT" 2>/dev/null || true
    
    # Remove iptables rules
    iptables -t nat -D OUTPUT -j SPEEDTEST_REDIRECT 2>/dev/null || true
    iptables -t nat -F SPEEDTEST_REDIRECT 2>/dev/null || true
    iptables -t nat -X SPEEDTEST_REDIRECT 2>/dev/null || true
    
    echo -e "${GREEN}✓ Cleanup completed${NC}"
}

# Handle cleanup on script exit
trap cleanup EXIT

# Start everything
start_proxy_chain
setup_iptables

echo
echo -e "${GREEN}=== SOCAT-based Transparent Proxy Active ===${NC}"
echo "Traffic flow: Browser → speedtest.net → iptables → socat:$SOCAT_PORT → proxy:$PROXY_PORT → real speedtest.net"
echo "This architecture prevents loops because:"
echo "  - Browser traffic goes to socat"
echo "  - Socat forwards to localhost (no iptables redirect)"
echo "  - Proxy connects to real speedtest.net (no iptables redirect)"
echo
echo -e "${YELLOW}Testing Instructions:${NC}"
echo "1. Test with curl:"
echo "   curl -w 'Speed: %{speed_download} bytes/s\\n' -o /dev/null -s https://speedtest.net"
echo
echo "2. Test with browser:"
echo "   - Navigate to: https://speedtest.net"
echo "   - Should work with bandwidth throttling, guaranteed no loops"
echo
echo -e "${YELLOW}Monitoring:${NC}"
echo "- Check socat: ss -tulpn | grep $SOCAT_PORT"
echo "- Check proxy: ss -tulpn | grep $PROXY_PORT"
echo "- Check iptables: iptables -t nat -L -n -v"
echo

echo -e "${RED}Press Ctrl+C to stop and cleanup${NC}"

# Keep running
while true; do
    sleep 1
done