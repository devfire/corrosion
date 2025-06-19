# Fix for "Too Many Open Files" Error

## Problem Description

When using the original `setup_transparent_proxy.sh` script, you encountered this error:

```
Error: Failed to accept incoming connection
Caused by:
    Too many open files (os error 24)
```

## Root Cause Analysis

The error was caused by a **redirect loop** in the iptables configuration:

1. Browser connects to `https://speedtest.net`
2. iptables redirects this to `localhost:8080` (your proxy)
3. Your proxy tries to connect to `speedtest.net:443`
4. iptables redirects this connection AGAIN to `localhost:8080`
5. This creates an infinite loop, rapidly exhausting file descriptors
6. System hits the "too many open files" limit

## The Fix

The solution uses **packet marking** to prevent the redirect loop:

### Method 1: Use the Fixed Script (Recommended)

```bash
sudo ./setup_transparent_proxy_fixed.sh
```

This script:
- Marks packets originating from your proxy process
- Only redirects unmarked packets (from browser/other apps)
- Skips redirecting packets from the proxy itself
- Prevents the infinite loop

### Method 2: Use DNS Redirect (Simplest Alternative)

```bash
sudo ./setup_dns_redirect.sh
```

This method:
- Modifies `/etc/hosts` to redirect `speedtest.net` to `127.0.0.1`
- No iptables complexity
- No loop possibility
- You'll see a certificate warning (click "Proceed" - it's safe)

## Technical Details of the Fix

The fixed iptables script uses these techniques:

### 1. Packet Marking in Mangle Table
```bash
# Mark packets from our proxy to identify them
iptables -t mangle -A OUTPUT -p tcp --sport 8080 -j MARK --set-mark 1
iptables -t mangle -A OUTPUT -p tcp --dport 443 -m owner --uid-owner $(id -u) -j MARK --set-mark 1
```

### 2. Conditional Redirect in NAT Table
```bash
# Only redirect unmarked packets (skip proxy's own connections)
iptables -t nat -A TRANSPARENT_PROXY -d $ip -p tcp --dport 443 -m mark ! --mark 1 -j REDIRECT --to-port 8080
```

### 3. Automatic Cleanup
Both scripts automatically clean up all rules when you press Ctrl+C.

## Testing the Fix

After running the fixed script:

1. **Browser Test**: Navigate to `https://speedtest.net`
   - Should load without "too many open files" error
   - May show certificate warning (click "Proceed")
   - Speed test should be throttled to your configured limit

2. **Command Line Test**:
   ```bash
   curl -k -w "Speed: %{speed_download} bytes/s\n" -o /dev/null -s https://speedtest.net
   ```

3. **Monitor Connections**:
   ```bash
   ss -tulpn | grep 8080
   ```

## Prevention for Future Use

- Always use the **fixed** version: `setup_transparent_proxy_fixed.sh`
- Or use the simpler DNS method: `setup_dns_redirect.sh`
- Avoid the original `setup_transparent_proxy.sh` (it has the loop bug)

## If You Still Have Issues

1. **Clean up any existing rules**:
   ```bash
   sudo iptables -t nat -F
   sudo iptables -t mangle -F
   ```

2. **Restart your proxy**:
   ```bash
   # Kill existing proxy
   pkill -f fault-injection
   
   # Start fresh
   cargo run -- --port 8080 --dest-ip speedtest.net --dest-port 443 --bandwidth-enabled --bandwidth-limit 1kbps
   ```

3. **Use the DNS method instead** (most reliable):
   ```bash
   sudo ./setup_dns_redirect.sh
   ```

The DNS redirect method is the most foolproof approach and doesn't have any possibility of creating loops.