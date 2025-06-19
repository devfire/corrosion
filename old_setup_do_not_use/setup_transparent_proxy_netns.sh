#!/bin/bash

# Network Namespace Transparent Proxy - The Definitive Solution
# This uses network namespaces to completely isolate the proxy's outbound connections

set -e

# Configuration
PROXY_PORT=8080
TARGET_HOST="speedtest.net"
TARGET_PORT=443
NETNS_NAME="fault_injection_ns"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Network Namespace Transparent Proxy Setup ===${NC}"
echo

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}Error: This script must be run as root (use sudo)${NC}"
   echo "Usage: sudo ./setup_transparent_proxy_netns.sh"
   exit 1
fi

# Function to setup network namespace and iptables
setup_netns_proxy() {
    echo -e "${YELLOW}Setting up network namespace transparent proxy...${NC}"
    
    # Create network namespace
    ip netns add $NETNS_NAME 2>/dev/null || true
    echo -e "${GREEN}✓ Network namespace '$NETNS_NAME' created${NC}"
    
    # Setup loopback in namespace
    ip netns exec $NETNS_NAME ip link set lo up
    
    # Create veth pair
    ip link add veth0 type veth peer name veth1 2>/dev/null || true
    
    # Move one end to namespace
    ip link set veth1 netns $NETNS_NAME
    
    # Configure interfaces
    ip addr add 192.168.100.1/24 dev veth0 2>/dev/null || true
    ip link set veth0 up
    
    ip netns exec $NETNS_NAME ip addr add 192.168.100.2/24 dev veth1
    ip netns exec $NETNS_NAME ip link set veth1 up
    
    # Setup routing in namespace
    ip netns exec $NETNS_NAME ip route add default via 192.168.100.1
    
    # Enable forwarding
    echo 1 > /proc/sys/net/ipv4/ip_forward
    
    # Setup NAT for namespace
    iptables -t nat -A POSTROUTING -s 192.168.100.0/24 ! -d 192.168.100.0/24 -j MASQUERADE
    
    echo -e "${GREEN}✓ Network namespace configured${NC}"
    
    # Get speedtest.net IPs
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
    
    # Setup transparent proxy rules (these won't affect the namespace)
    iptables -t nat -N TRANSPARENT_PROXY 2>/dev/null || true
    
    for ip in $SPEEDTEST_IPS; do
        echo "  Adding rule for $ip:$TARGET_PORT -> localhost:$PROXY_PORT"
        iptables -t nat -A TRANSPARENT_PROXY -d $ip -p tcp --dport $TARGET_PORT -j REDIRECT --to-port $PROXY_PORT
    done
    
    # Apply rules only to main namespace traffic
    iptables -t nat -I OUTPUT -j TRANSPARENT_PROXY
    
    echo -e "${GREEN}✓ Transparent proxy rules configured${NC}"
}

# Function to start proxy in namespace
start_proxy_in_namespace() {
    echo -e "${YELLOW}Starting fault injection proxy in network namespace...${NC}"
    
    # Kill any existing proxy
    pkill -f "fault-injection.*--port $PROXY_PORT" 2>/dev/null || true
    sleep 1
    
    # Start proxy in namespace (this prevents it from being affected by iptables rules)
    cd /home/ig/Documents/rust/fault-injection
    ip netns exec $NETNS_NAME sudo -u ig \
        cargo run -- \
        --port $PROXY_PORT \
        --dest-ip $TARGET_HOST \
        --dest-port $TARGET_PORT \
        --bandwidth-enabled \
        --bandwidth-limit 1kbps &
    
    PROXY_PID=$!
    echo -e "${GREEN}✓ Proxy started in namespace (PID: $PROXY_PID)${NC}"
    
    # Wait for proxy to start
    sleep 3
    
    # Verify proxy is running
    if kill -0 $PROXY_PID 2>/dev/null; then
        echo -e "${GREEN}✓ Proxy is running successfully${NC}"
    else
        echo -e "${RED}✗ Proxy failed to start${NC}"
        exit 1
    fi
}

# Function to cleanup
cleanup() {
    echo -e "${YELLOW}Cleaning up network namespace setup...${NC}"
    
    # Kill proxy
    pkill -f "fault-injection.*--port $PROXY_PORT" 2>/dev/null || true
    
    # Remove iptables rules
    iptables -t nat -D OUTPUT -j TRANSPARENT_PROXY 2>/dev/null || true
    iptables -t nat -F TRANSPARENT_PROXY 2>/dev/null || true
    iptables -t nat -X TRANSPARENT_PROXY 2>/dev/null || true
    iptables -t nat -D POSTROUTING -s 192.168.100.0/24 ! -d 192.168.100.0/24 -j MASQUERADE 2>/dev/null || true
    
    # Remove network namespace
    ip netns del $NETNS_NAME 2>/dev/null || true
    
    # Remove veth interface
    ip link del veth0 2>/dev/null || true
    
    echo -e "${GREEN}✓ Cleanup completed${NC}"
}

# Handle cleanup on script exit
trap cleanup EXIT

# Setup everything
setup_netns_proxy
start_proxy_in_namespace

echo
echo -e "${GREEN}=== Network Namespace Transparent Proxy Active ===${NC}"
echo "The proxy is running in an isolated network namespace"
echo "This completely prevents redirect loops while maintaining transparent proxying"
echo
echo -e "${YELLOW}Testing Instructions:${NC}"
echo "1. Test with curl:"
echo "   curl -w 'Speed: %{speed_download} bytes/s\\n' -o /dev/null -s https://speedtest.net"
echo
echo "2. Test with browser:"
echo "   - Navigate to: https://speedtest.net"
echo "   - Should work normally with bandwidth throttling"
echo
echo -e "${YELLOW}Monitoring:${NC}"
echo "- Check proxy in namespace: ip netns exec $NETNS_NAME ss -tulpn | grep $PROXY_PORT"
echo "- Monitor iptables hits: iptables -t nat -L -n -v | grep REDIRECT"
echo

echo -e "${RED}Press Ctrl+C to stop and cleanup${NC}"

# Keep running
while true; do
    sleep 1
done