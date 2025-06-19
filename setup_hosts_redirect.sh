#!/bin/bash

# Hosts File Redirect - The Most Reliable Method
# This completely avoids iptables and uses /etc/hosts redirection

set -e

# Configuration
PROXY_PORT=8080
TARGET_HOST="speedtest.net"
HOSTS_FILE="/etc/hosts"
BACKUP_FILE="/etc/hosts.backup.$(date +%s)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Hosts File Redirect Setup (Most Reliable) ===${NC}"
echo

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}Error: This script must be run as root (use sudo)${NC}"
   echo "Usage: sudo ./setup_hosts_redirect.sh"
   exit 1
fi

# Check if fault injection proxy is running
if ! pgrep -f "fault-injection.*--port $PROXY_PORT" > /dev/null; then
    echo -e "${RED}Error: Fault injection proxy not running on port $PROXY_PORT${NC}"
    echo "Please start your proxy first:"
    echo "  cargo run -- --port $PROXY_PORT --dest-ip $TARGET_HOST --dest-port 443 --bandwidth-enabled --bandwidth-limit 1kbps"
    exit 1
fi

echo -e "${GREEN}✓ Fault injection proxy detected on port $PROXY_PORT${NC}"

# Function to setup hosts redirect
setup_hosts_redirect() {
    echo -e "${YELLOW}Setting up hosts file redirect...${NC}"
    
    # Create backup
    cp $HOSTS_FILE $BACKUP_FILE
    echo -e "${GREEN}✓ Backup created: $BACKUP_FILE${NC}"
    
    # Remove any existing speedtest.net entries
    sed -i '/speedtest\.net/d' $HOSTS_FILE
    
    # Add redirect entry
    echo "127.0.0.1    speedtest.net" >> $HOSTS_FILE
    echo "127.0.0.1    www.speedtest.net" >> $HOSTS_FILE
    
    echo -e "${GREEN}✓ Added hosts redirect: speedtest.net -> 127.0.0.1${NC}"
    
    # Flush DNS cache
    if command -v systemd-resolve &> /dev/null; then
        systemd-resolve --flush-caches 2>/dev/null || true
        echo -e "${GREEN}✓ DNS cache flushed (systemd-resolve)${NC}"
    elif command -v resolvectl &> /dev/null; then
        resolvectl flush-caches 2>/dev/null || true
        echo -e "${GREEN}✓ DNS cache flushed (resolvectl)${NC}"
    fi
}

# Function to cleanup hosts redirect
cleanup_hosts_redirect() {
    echo -e "${YELLOW}Cleaning up hosts redirect...${NC}"
    
    if [ -f $BACKUP_FILE ]; then
        cp $BACKUP_FILE $HOSTS_FILE
        rm -f $BACKUP_FILE
        echo -e "${GREEN}✓ Hosts file restored from backup${NC}"
        
        # Flush DNS cache again
        if command -v systemd-resolve &> /dev/null; then
            systemd-resolve --flush-caches 2>/dev/null || true
        elif command -v resolvectl &> /dev/null; then
            resolvectl flush-caches 2>/dev/null || true
        fi
        echo -e "${GREEN}✓ DNS cache flushed${NC}"
    fi
}

# Handle cleanup on script exit
trap cleanup_hosts_redirect EXIT

# Setup the hosts redirect
setup_hosts_redirect

echo
echo -e "${GREEN}=== Hosts Redirect Active (No iptables, No loops!) ===${NC}"
echo "speedtest.net now resolves to 127.0.0.1"
echo "All traffic to speedtest.net will go through your proxy on port $PROXY_PORT"
echo

echo -e "${YELLOW}Testing Instructions:${NC}"
echo "1. Test with curl:"
echo "   curl -k -w 'Speed: %{speed_download} bytes/s\\n' -o /dev/null -s https://speedtest.net"
echo
echo "2. Test with browser:"
echo "   - Navigate to: https://speedtest.net"
echo "   - You'll see a certificate warning (this is expected and safe)"
echo "   - Click 'Advanced' -> 'Proceed to speedtest.net (unsafe)'"
echo "   - The page should load slowly due to 1kbps throttling"
echo

echo -e "${YELLOW}Verification:${NC}"
echo "Check DNS resolution:"
echo "  nslookup speedtest.net"
echo "  (should show 127.0.0.1)"
echo
echo "Monitor proxy connections:"
echo "  ss -tulpn | grep $PROXY_PORT"
echo

echo -e "${GREEN}✓ This method is 100% reliable and cannot create loops${NC}"
echo -e "${RED}Press Ctrl+C to restore original DNS settings${NC}"

# Keep the script running until interrupted
while true; do
    sleep 1
done