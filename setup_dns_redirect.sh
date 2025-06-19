#!/bin/bash

# DNS Redirect Setup for Transparent Proxying
# This script modifies /etc/hosts to redirect speedtest.net to localhost
# where your fault injection proxy is running

set -e

# Configuration
PROXY_PORT=8080
TARGET_HOST="speedtest.net"
HOSTS_FILE="/etc/hosts"
BACKUP_FILE="/etc/hosts.backup.fault-injection"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== DNS Redirect Setup for Fault Injection ===${NC}"
echo

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}Error: This script must be run as root (use sudo)${NC}"
   echo "Usage: sudo ./setup_dns_redirect.sh"
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

# Function to setup DNS redirect
setup_dns_redirect() {
    echo -e "${YELLOW}Setting up DNS redirect...${NC}"
    
    # Create backup of hosts file
    cp $HOSTS_FILE $BACKUP_FILE
    echo -e "${GREEN}✓ Backup created: $BACKUP_FILE${NC}"
    
    # Check if entry already exists
    if grep -q "127.0.0.1.*$TARGET_HOST" $HOSTS_FILE; then
        echo -e "${YELLOW}Entry for $TARGET_HOST already exists in $HOSTS_FILE${NC}"
    else
        # Add entry to redirect speedtest.net to localhost
        echo "127.0.0.1    $TARGET_HOST" >> $HOSTS_FILE
        echo -e "${GREEN}✓ Added DNS redirect: $TARGET_HOST -> 127.0.0.1${NC}"
    fi
    
    # Flush DNS cache
    if command -v systemd-resolve &> /dev/null; then
        systemd-resolve --flush-caches
        echo -e "${GREEN}✓ DNS cache flushed (systemd-resolve)${NC}"
    elif command -v resolvectl &> /dev/null; then
        resolvectl flush-caches
        echo -e "${GREEN}✓ DNS cache flushed (resolvectl)${NC}"
    else
        echo -e "${YELLOW}⚠ Could not flush DNS cache automatically${NC}"
    fi
}

# Function to cleanup DNS redirect
cleanup_dns_redirect() {
    echo -e "${YELLOW}Cleaning up DNS redirect...${NC}"
    
    if [ -f $BACKUP_FILE ]; then
        mv $BACKUP_FILE $HOSTS_FILE
        echo -e "${GREEN}✓ Hosts file restored from backup${NC}"
        
        # Flush DNS cache again
        if command -v systemd-resolve &> /dev/null; then
            systemd-resolve --flush-caches
        elif command -v resolvectl &> /dev/null; then
            resolvectl flush-caches
        fi
        echo -e "${GREEN}✓ DNS cache flushed${NC}"
    else
        echo -e "${RED}⚠ Backup file not found, manual cleanup may be required${NC}"
    fi
}

# Handle cleanup on script exit
trap cleanup_dns_redirect EXIT

# Setup the DNS redirect
setup_dns_redirect

echo
echo -e "${GREEN}=== DNS Redirect Active ===${NC}"
echo "$TARGET_HOST now resolves to 127.0.0.1 (localhost)"
echo "Your fault injection proxy on port $PROXY_PORT will handle the traffic"
echo
echo -e "${YELLOW}Testing Instructions:${NC}"
echo "1. Open your browser and navigate to https://speedtest.net"
echo "2. You may see a certificate warning (click 'Advanced' -> 'Proceed')"
echo "3. The traffic will be throttled through your proxy"
echo
echo -e "${YELLOW}Alternative Testing:${NC}"
echo "- Test with curl: curl -k -w 'Speed: %{speed_download} bytes/s\\n' -o /dev/null -s https://speedtest.net"
echo "- Monitor proxy: ps aux | grep fault-injection"
echo
echo -e "${RED}Press Ctrl+C to restore original DNS settings${NC}"

# Keep the script running until interrupted
while true; do
    sleep 1
done