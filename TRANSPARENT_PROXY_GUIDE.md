# Transparent Proxy Setup Guide

This guide explains how to set up transparent proxying to automatically intercept traffic to speedtest.net and route it through your fault injection proxy, eliminating TLS certificate issues.

## Overview

Your fault injection proxy currently runs as a TCP proxy on `localhost:8080` forwarding to `speedtest.net:443`. To test bandwidth throttling without certificate errors, you need to intercept traffic transparently.

## ⚠️ IMPORTANT: "Too Many Open Files" Fix

If you encountered the "Too many open files" error with the original iptables method, this was caused by a redirect loop. Use the **Fixed iptables method** or **DNS redirect method** below.

## Method 1: DNS Redirect (Recommended - Simplest & Most Reliable)

This method redirects `speedtest.net` to `localhost` in your system's DNS resolution.

### Setup:
```bash
sudo ./setup_dns_redirect.sh
```

### How it works:
1. Modifies `/etc/hosts` to redirect `speedtest.net` to `127.0.0.1`
2. When you visit `https://speedtest.net`, it connects to `localhost:8080`
3. Your proxy forwards the traffic to the real speedtest.net
4. Automatically cleans up when you stop the script

### Pros:
- Simple to implement
- No iptables rules required
- Easy to understand and debug
- Automatic cleanup

### Cons:
- Still shows certificate warning (but you can proceed safely)
- Affects all applications system-wide

## Method 2: Fixed iptables REDIRECT (Advanced - Loop Prevention)

This method uses iptables with loop prevention to intercept packets destined for speedtest.net IPs and redirect them to your proxy.

### Setup:
```bash
sudo ./setup_transparent_proxy_fixed.sh
```

### How it works:
1. Resolves speedtest.net to get current IP addresses
2. Creates iptables NAT rules with packet marking to prevent loops
3. Redirects port 443 traffic to your proxy on port 8080
4. Uses mangle table to mark proxy-originated packets and skip redirecting them
5. Automatically cleans up iptables rules when stopped

### Pros:
- Prevents "too many open files" errors with loop detection
- More sophisticated approach
- Can be selective about which traffic to intercept
- No DNS modification required

### Cons:
- Requires iptables knowledge
- More complex to debug
- May interfere with other network tools

## Method 3: Original iptables (Deprecated - Has Loop Issue)

⚠️ **DO NOT USE** - The original `setup_transparent_proxy.sh` causes redirect loops and "too many open files" errors.
Use Method 2 (Fixed iptables) instead.

## Method 4: Manual iptables (Expert Level)

For complete control, you can set up iptables rules manually with loop prevention:

### Get speedtest.net IPs:
```bash
dig +short speedtest.net
```

### Create redirect rules with loop prevention:
```bash
# Create custom chains
sudo iptables -t nat -N SPEEDTEST_PROXY
sudo iptables -t mangle -N PROXY_MARK

# Mark packets from proxy to avoid loops
sudo iptables -t mangle -A OUTPUT -p tcp --sport 8080 -j MARK --set-mark 1
sudo iptables -t mangle -A OUTPUT -p tcp --dport 443 -m owner --uid-owner $(id -u) -j MARK --set-mark 1

# Add redirect rules for each IP (with loop prevention)
sudo iptables -t nat -A SPEEDTEST_PROXY -d 151.101.194.219 -p tcp --dport 443 -m mark ! --mark 1 -j REDIRECT --to-port 8080
sudo iptables -t nat -A SPEEDTEST_PROXY -d 151.101.66.219 -p tcp --dport 443 -m mark ! --mark 1 -j REDIRECT --to-port 8080
# ... repeat for all IPs

# Apply the chain
sudo iptables -t nat -I OUTPUT -j SPEEDTEST_PROXY
```

### Cleanup:
```bash
sudo iptables -t nat -D OUTPUT -j SPEEDTEST_PROXY
sudo iptables -t nat -F SPEEDTEST_PROXY
sudo iptables -t nat -X SPEEDTEST_PROXY
sudo iptables -t mangle -F PROXY_MARK
sudo iptables -t mangle -X PROXY_MARK
sudo iptables -t mangle -D OUTPUT -p tcp --sport 8080 -j MARK --set-mark 1
sudo iptables -t mangle -D OUTPUT -p tcp --dport 443 -m owner --uid-owner $(id -u) -j MARK --set-mark 1
```

## Testing Your Setup

### 1. Verify proxy is running:
```bash
ps aux | grep fault-injection
ss -tlnp | grep 8080
```

### 2. Test with curl:
```bash
# With DNS redirect method:
curl -k -w "Speed: %{speed_download} bytes/s\n" -o /dev/null -s https://speedtest.net

# Direct test:
curl -k -w "Speed: %{speed_download} bytes/s\n" -o /dev/null -s https://localhost:8080
```

### 3. Browser testing:
1. Navigate to `https://speedtest.net`
2. If you see certificate warning, click "Advanced" → "Proceed to speedtest.net (unsafe)"
3. The page should load slowly due to bandwidth throttling
4. Run a speed test to verify throttling is working

## Troubleshooting

### Certificate Errors:
- Expected with transparent proxying
- Click "Advanced" → "Proceed" in browser
- Use `curl -k` flag to ignore certificates

### DNS Issues:
```bash
# Flush DNS cache
sudo systemd-resolve --flush-caches
# or
sudo resolvectl flush-caches

# Verify DNS resolution
nslookup speedtest.net
```

### iptables Issues:
```bash
# List current NAT rules
sudo iptables -t nat -L -n -v

# Check if rules are being hit
sudo iptables -t nat -L -n -v | grep REDIRECT
```

### Proxy Not Working:
```bash
# Check if proxy is listening
netstat -tlnp | grep 8080
# or
ss -tlnp | grep 8080

# Check proxy logs
tail -f /var/log/syslog | grep fault-injection
```

## Security Considerations

- These methods modify system networking behavior
- Only use on development/testing systems
- Always clean up when finished
- Be aware that certificate warnings are expected and safe in this context

## Cleanup

Both automated scripts handle cleanup automatically when you press Ctrl+C. For manual cleanup:

### DNS Method:
```bash
sudo cp /etc/hosts.backup.fault-injection /etc/hosts
sudo systemd-resolve --flush-caches
```

### iptables Method:
```bash
sudo iptables -t nat -D OUTPUT -j TRANSPARENT_PROXY
sudo iptables -t nat -F TRANSPARENT_PROXY
sudo iptables -t nat -X TRANSPARENT_PROXY