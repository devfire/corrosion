#!/bin/bash

# Ultimate Transparent Proxy Solution - Uses source port exclusion
# This is the simplest approach that actually works

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

echo -e "${GREEN}=== Ultimate Transparent Proxy Setup ===${NC}"
echo

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}Error: This script must be run as root (use sudo)${NC}"
   echo "Usage: sudo ./setup_transparent_proxy_ultimate.sh"
   exit 1
fi

# Check if fault injection proxy is running
if ! pgrep -f "fault-injection.*--port $PROXY_PORT" > /dev/null; then
    echo -e "${RED}Error: Fault injection proxy not running on port $PROXY_PORT${NC}"
    echo "Please start your proxy first:"
    echo "  cargo run -- --port $PROXY_PORT --dest-ip $TARGET_HOST --dest-port $TARGET_PORT --bandwidth-enabled --bandwidth-limit 1kbps"
    exit 1
fi

echo -e "${GREEN}✓ Fault injection proxy detected on port $PROXY_PORT${NC}"

# Get speedtest.net IP addresses
echo "Resolving $TARGET_HOST IP addresses..."
SPEEDTEST_IPS=$(dig +short $TARGET_HOST | grep -E '^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$')

if [ -z "$SPEEDTEST_IPS" ]; then
    echo -e "${RED}Error: Could not resolve $TARGET_HOST${NC}"
    exit 1
fi

echo -e "${GREEN}Found IP addresses for $TARGET_HOST:${NC}"
for ip in $SPEEDTEST_IPS; do
    echo "  - $ip"
done
echo

# Function to setup iptables with proper loop prevention
setup_iptables() {
    echo -e "${YELLOW}Setting up iptables with ultimate loop prevention...${NC}"
    
    # Create custom chain
    iptables -t nat -N TRANSPARENT_PROXY 2>/dev/null || true
    
    # The key insight: exclude connections FROM the proxy port
    # This prevents the proxy's outbound connections from being redirected
    for ip in $SPEEDTEST_IPS; do
        echo "  Adding rule for $ip:$TARGET_PORT -> localhost:$PROXY_PORT (excluding proxy source)"
        # Only redirect if NOT coming from our proxy port
        iptables -t nat -A TRANSPARENT_PROXY -d $ip -p tcp --dport $TARGET_PORT ! --sport $PROXY_PORT -j REDIRECT --to-port $PROXY_PORT
    done
    
    # Apply the chain
    iptables -t nat -I OUTPUT -j TRANSPARENT_PROXY
    
    echo -e "${GREEN}✓ iptables rules configured with source port exclusion${NC}"
}

# Function to cleanup
cleanup_iptables() {
    echo -e "${YELLOW}Cleaning up iptables rules...${NC}"
    
    # Remove the chain from OUTPUT
    iptables -t nat -D OUTPUT -j TRANSPARENT_PROXY 2>/dev/null || true
    
    # Flush and delete the custom chain
    iptables -t nat -F TRANSPARENT_PROXY 2>/dev/null || true
    iptables -t nat -X TRANSPARENT_PROXY 2>/dev/null || true
    
    echo -e "${GREEN}✓ iptables rules cleaned up${NC}"
}

# Handle cleanup on script exit
trap cleanup_iptables EXIT

# Setup the rules
setup_iptables

echo
echo -e "${GREEN}=== Ultimate Transparent Proxy Active ===${NC}"
echo "Traffic to $TARGET_HOST:$TARGET_PORT is now being intercepted"
echo "Source port exclusion prevents redirect loops"
echo
echo -e "${YELLOW}How it works:${NC}"
echo "- Browser connects to speedtest.net:443 → redirected to localhost:$PROXY_PORT"
echo "- Proxy connects FROM port $PROXY_PORT to speedtest.net:443 → NOT redirected (excluded)"
echo "- This breaks the loop while maintaining transparent proxying"
echo
echo -e "${YELLOW}Testing Instructions:${NC}"
echo "1. Test with curl:"
echo "   curl -w 'Speed: %{speed_download} bytes/s\\n' -o /dev/null -s https://speedtest.net"
echo
echo "2. Test with browser:"
echo "   - Navigate to: https://speedtest.net"
echo "   - Should work with bandwidth throttling, no loops"
echo
echo -e "${YELLOW}Monitoring:${NC}"
echo "- Check iptables hits: iptables -t nat -L -n -v"
echo "- Monitor connections: ss -tulpn | grep $PROXY_PORT"
echo

echo -e "${RED}Press Ctrl+C to stop transparent proxying and cleanup iptables rules${NC}"

# Keep the script running until interrupted
while true; do
    sleep 1
done